//! MCP tool implementations wrapping Stage B KG queries.

use crate::errors::{ToolError, ToolErrorCode};
use anyhow::Result;
use prism_obs::{emit_index_event, IndexEvent};
use prism_store::{parse_edge_kinds, EdgeDirection, RepoMap, SqliteKgStore, SqliteMetaStore};
use serde::Serialize;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::info;

/// Allowlisted MCP tool names (no write/rename in P1).
pub const ALLOWED_TOOLS: &[&str] = &[
    "index_status",
    "resolve_symbol",
    "neighbors",
    "impact",
    "repo_map",
];

#[derive(Debug, Clone)]
pub struct ToolContext {
    pub workspace: PathBuf,
}

impl ToolContext {
    pub fn open(workspace: impl Into<PathBuf>) -> Result<Self, ToolError> {
        let workspace = workspace.into();
        let prism = workspace.join(".prism");
        if !prism.join("graph.sqlite").exists() {
            return Err(ToolError::index_unavailable(format!(
                "no graph.sqlite under {}",
                prism.display()
            )));
        }
        Ok(Self { workspace })
    }

    fn kg(&self) -> Result<SqliteKgStore, ToolError> {
        SqliteKgStore::open(self.workspace.join(".prism/graph.sqlite"))
            .map_err(|e| ToolError::index_unavailable(e.to_string()))
    }

    fn meta(&self) -> Result<SqliteMetaStore, ToolError> {
        SqliteMetaStore::open(self.workspace.join(".prism/meta.sqlite"))
            .map_err(|e| ToolError::index_unavailable(e.to_string()))
    }
}

#[derive(Debug, Serialize)]
pub struct ToolSuccess {
    pub tool: String,
    pub confidence_note: String,
    pub result: Value,
    pub latency_ms: u64,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ToolOutcome {
    Ok(ToolSuccess),
    Err { error: ToolError },
}

fn audit(tool: &str, latency_ms: u64, hit_count: u64) {
    emit_index_event(&IndexEvent::QueryFinished {
        op: format!("mcp:{tool}"),
        latency_ms,
        hit_count,
    });
    info!(tool = %tool, latency_ms, hit_count, "mcp tool call");
}

pub fn list_tools_schema() -> Value {
    json!([
        {
            "name": "index_status",
            "description": "Return index freshness, graph cardinality, and on-disk size. Prefer this before other structural tools.",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }
        },
        {
            "name": "resolve_symbol",
            "description": "Lookup symbols by exact name. Returns ids, paths, and confidence. Use before neighbors/impact.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Exact symbol name" },
                    "file": { "type": "string", "description": "Optional path substring filter" },
                    "limit": { "type": "integer", "default": 20 }
                },
                "required": ["name"],
                "additionalProperties": false
            }
        },
        {
            "name": "neighbors",
            "description": "1-hop graph neighbors for a node id. Edge kinds optional (e.g. CALLS,IMPORTS). Returns confidence on each edge.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string" },
                    "kind": { "type": "string", "description": "Comma-separated edge kinds" },
                    "dir": { "type": "string", "enum": ["outgoing", "incoming", "both"], "default": "outgoing" },
                    "limit": { "type": "integer", "default": 50 }
                },
                "required": ["id"],
                "additionalProperties": false
            }
        },
        {
            "name": "impact",
            "description": "Depth-limited HEURISTIC blast-radius candidates from a seed node id. Not precise refactor safety — T1 only.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string" },
                    "depth": { "type": "integer", "default": 2, "maximum": 8 },
                    "limit": { "type": "integer", "default": 100 }
                },
                "required": ["id"],
                "additionalProperties": false
            }
        },
        {
            "name": "repo_map",
            "description": "Lightweight orientation: path-prefix communities and degree hubs. Heuristic architecture sketch.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "hub_limit": { "type": "integer", "default": 15 }
                },
                "additionalProperties": false
            }
        }
    ])
}

pub fn call_tool(ctx: &ToolContext, name: &str, arguments: Value) -> ToolOutcome {
    if !ALLOWED_TOOLS.contains(&name) {
        return ToolOutcome::Err {
            error: ToolError {
                code: ToolErrorCode::InvalidArgs,
                message: format!("tool '{name}' is not allowlisted"),
                hint: Some(format!("Allowed: {}", ALLOWED_TOOLS.join(", "))),
            },
        };
    }
    match name {
        "index_status" => tool_index_status(ctx),
        "resolve_symbol" => tool_resolve(ctx, arguments),
        "neighbors" => tool_neighbors(ctx, arguments),
        "impact" => tool_impact(ctx, arguments),
        "repo_map" => tool_repo_map(ctx, arguments),
        _ => ToolOutcome::Err {
            error: ToolError::invalid_args(format!("unknown tool {name}")),
        },
    }
}

fn ok(tool: &str, note: &str, result: Value, started: Instant, hits: u64) -> ToolOutcome {
    let latency_ms = started.elapsed().as_millis() as u64;
    audit(tool, latency_ms, hits);
    ToolOutcome::Ok(ToolSuccess {
        tool: tool.into(),
        confidence_note: note.into(),
        result,
        latency_ms,
    })
}

fn tool_index_status(ctx: &ToolContext) -> ToolOutcome {
    let started = Instant::now();
    let kg = match ctx.kg() {
        Ok(k) => k,
        Err(e) => return ToolOutcome::Err { error: e },
    };
    let meta = match ctx.meta() {
        Ok(m) => m,
        Err(e) => return ToolOutcome::Err { error: e },
    };
    let stats = match kg.index_stats() {
        Ok(s) => s,
        Err(e) => {
            return ToolOutcome::Err {
                error: ToolError::index_unavailable(e.to_string()),
            }
        }
    };
    let files = meta.list_file_paths().map(|v| v.len()).unwrap_or(0);
    let graph_bytes = std::fs::metadata(ctx.workspace.join(".prism/graph.sqlite"))
        .map(|m| m.len())
        .unwrap_or(0);
    let result = json!({
        "workspace": ctx.workspace.display().to_string(),
        "files_hashed": files,
        "nodes": stats.nodes,
        "edges": stats.edges,
        "files_indexed": stats.files_indexed,
        "graph_sqlite_bytes": graph_bytes,
        "tier": "T1",
    });
    ok(
        "index_status",
        "Index metadata only; no heuristic edges.",
        result,
        started,
        1,
    )
}

fn tool_resolve(ctx: &ToolContext, args: Value) -> ToolOutcome {
    let started = Instant::now();
    let name = match args.get("name").and_then(|v| v.as_str()) {
        Some(n) if !n.is_empty() => n,
        _ => {
            return ToolOutcome::Err {
                error: ToolError::scope_unresolved("resolve_symbol requires non-empty 'name'"),
            }
        }
    };
    let file = args.get("file").and_then(|v| v.as_str());
    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
    let kg = match ctx.kg() {
        Ok(k) => k,
        Err(e) => return ToolOutcome::Err { error: e },
    };
    let hits = match kg.resolve_symbol(name, file, limit) {
        Ok(h) => h,
        Err(e) => {
            return ToolOutcome::Err {
                error: ToolError::index_unavailable(e.to_string()),
            }
        }
    };
    if hits.is_empty() {
        return ToolOutcome::Err {
            error: ToolError::scope_unresolved(format!(
                "no symbols named '{name}'{}",
                file.map(|f| format!(" in paths containing '{f}'"))
                    .unwrap_or_default()
            )),
        };
    }
    let n = hits.len() as u64;
    ok(
        "resolve_symbol",
        "Symbol identities from T1 extraction (confidence on each node).",
        json!(hits),
        started,
        n,
    )
}

fn tool_neighbors(ctx: &ToolContext, args: Value) -> ToolOutcome {
    let started = Instant::now();
    let id = match args.get("id").and_then(|v| v.as_str()) {
        Some(i) if !i.is_empty() => i,
        _ => {
            return ToolOutcome::Err {
                error: ToolError::scope_unresolved(
                    "neighbors requires node 'id' from resolve_symbol",
                ),
            }
        }
    };
    let kinds = parse_edge_kinds(args.get("kind").and_then(|v| v.as_str()));
    let dir = match args
        .get("dir")
        .and_then(|v| v.as_str())
        .unwrap_or("outgoing")
    {
        "incoming" => EdgeDirection::Incoming,
        "both" => EdgeDirection::Both,
        _ => EdgeDirection::Outgoing,
    };
    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as usize;
    let kg = match ctx.kg() {
        Ok(k) => k,
        Err(e) => return ToolOutcome::Err { error: e },
    };
    let hits = match kg.neighbors(id, kinds.as_deref(), dir, limit) {
        Ok(h) => h,
        Err(e) => {
            return ToolOutcome::Err {
                error: ToolError::index_unavailable(e.to_string()),
            }
        }
    };
    let n = hits.len() as u64;
    ok(
        "neighbors",
        "Edges include confidence; CALLS are heuristic at T1.",
        json!(hits),
        started,
        n,
    )
}

fn tool_impact(ctx: &ToolContext, args: Value) -> ToolOutcome {
    let started = Instant::now();
    let id = match args.get("id").and_then(|v| v.as_str()) {
        Some(i) if !i.is_empty() => i,
        _ => {
            return ToolOutcome::Err {
                error: ToolError::scope_unresolved("impact requires seed node 'id'"),
            }
        }
    };
    let depth = args.get("depth").and_then(|v| v.as_u64()).unwrap_or(2) as u32;
    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(100) as usize;
    let kg = match ctx.kg() {
        Ok(k) => k,
        Err(e) => return ToolOutcome::Err { error: e },
    };
    let hits = match kg.impact(id, depth, limit) {
        Ok(h) => h,
        Err(e) => {
            return ToolOutcome::Err {
                error: ToolError::index_unavailable(e.to_string()),
            }
        }
    };
    let n = hits.len() as u64;
    ok(
        "impact",
        "HEURISTIC T1 blast radius — wrong callees possible; not safe-rename.",
        json!({ "seed": id, "depth": depth, "candidates": hits }),
        started,
        n,
    )
}

fn tool_repo_map(ctx: &ToolContext, args: Value) -> ToolOutcome {
    let started = Instant::now();
    let hub_limit = args.get("hub_limit").and_then(|v| v.as_u64()).unwrap_or(15) as usize;
    let kg = match ctx.kg() {
        Ok(k) => k,
        Err(e) => return ToolOutcome::Err { error: e },
    };
    let map: RepoMap = match kg.repo_map(hub_limit) {
        Ok(m) => m,
        Err(e) => {
            return ToolOutcome::Err {
                error: ToolError::index_unavailable(e.to_string()),
            }
        }
    };
    let hits = (map.communities.len() + map.hubs.len()) as u64;
    ok(
        "repo_map",
        "Path-prefix communities + degree hubs; orientation only.",
        json!(map),
        started,
        hits,
    )
}

/// Dispatch helper for unit tests / eval without stdio.
pub fn dispatch_json(workspace: &Path, tool: &str, args: Value) -> Value {
    match ToolContext::open(workspace) {
        Ok(ctx) => serde_json::to_value(call_tool(&ctx, tool, args)).unwrap_or(json!({})),
        Err(e) => json!({ "error": e }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prism_core::{IncrementalIndexer, IndexOptions, WorkspaceManager};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn resolve_and_scope_unresolved() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("m.py"), b"def helper():\n    pass\n").unwrap();
        let wm = WorkspaceManager::open(dir.path()).unwrap();
        let mut idx = IncrementalIndexer::open(wm, dir.path().join(".prism")).unwrap();
        idx.run(&IndexOptions::default()).unwrap();

        let ctx = ToolContext::open(dir.path()).unwrap();
        match call_tool(&ctx, "resolve_symbol", json!({"name": "helper"})) {
            ToolOutcome::Ok(s) => assert_eq!(s.tool, "resolve_symbol"),
            ToolOutcome::Err { error } => panic!("{error:?}"),
        }
        match call_tool(&ctx, "resolve_symbol", json!({"name": "nope_xyz"})) {
            ToolOutcome::Err { error } => {
                assert_eq!(error.code, ToolErrorCode::ScopeUnresolved);
            }
            ToolOutcome::Ok(_) => panic!("expected SCOPE_UNRESOLVED"),
        }
        match call_tool(&ctx, "repo_map", json!({})) {
            ToolOutcome::Ok(s) => assert!(s.result.get("communities").is_some()),
            ToolOutcome::Err { error } => panic!("{error:?}"),
        }
    }
}
