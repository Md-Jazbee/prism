//! Lazy call-graph shards (T4) under `.prism/semantic/shards/`.

use crate::artifact::{SemanticFileArtifact, INTERPROC_ALGO_VERSION, SEMANTIC_SCHEMA_VERSION};
use crate::store::{load_file_artifact, semantic_dir};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::fs;
use std::path::Path;
use xxhash_rust::xxh3::xxh3_128;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct OverlayEdge {
    /// `DATA_FLOW` | `CONTROL_DEP` | `CALLS`
    pub kind: String,
    pub src: String,
    pub dst: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CallGraphShard {
    pub schema_version: String,
    pub algo_version: String,
    pub shard_id: String,
    pub seed: String,
    pub member_paths: Vec<String>,
    pub functions: Vec<String>,
    pub edges: Vec<OverlayEdge>,
    pub built_at: String,
}

#[derive(Debug, Clone)]
pub struct FnLoc {
    pub key: String,
    pub path: String,
    pub name: String,
    pub start_line: u32,
    pub end_line: u32,
}

/// Build (or rebuild) a neighborhood shard around `seed_fn`.
pub fn ensure_shard(
    workspace: &Path,
    seed_path: &str,
    seed_fn: &str,
    max_depth: u32,
    max_functions: usize,
) -> Result<(CallGraphShard, bool)> {
    let catalog = load_catalog(workspace)?;
    let seed_key = resolve_fn_key(&catalog, seed_path, seed_fn)
        .ok_or_else(|| anyhow::anyhow!("seed function not found: {seed_path}::{seed_fn}"))?;
    let all_edges = build_call_index(workspace, &catalog)?;
    let (member_keys, edges, truncated) =
        expand_from_edges(&seed_key, &all_edges, max_depth, max_functions);

    let mut member_paths: BTreeSet<String> = BTreeSet::new();
    for k in &member_keys {
        if let Some(loc) = catalog.get(k) {
            member_paths.insert(loc.path.clone());
        }
    }
    let member_paths: Vec<String> = member_paths.into_iter().collect();
    let shard_id = shard_id_for(&member_paths);
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let shard = CallGraphShard {
        schema_version: SEMANTIC_SCHEMA_VERSION.into(),
        algo_version: INTERPROC_ALGO_VERSION.into(),
        shard_id,
        seed: seed_key,
        member_paths,
        functions: member_keys,
        edges,
        built_at: format!("unix:{secs}"),
    };
    save_shard(workspace, &shard)?;
    Ok((shard, truncated))
}

pub fn save_shard(workspace: &Path, shard: &CallGraphShard) -> Result<()> {
    let dir = semantic_dir(workspace).join("shards");
    fs::create_dir_all(&dir)?;
    let dest = dir.join(format!("{}.json", shard.shard_id));
    fs::write(&dest, serde_json::to_string_pretty(shard)? + "\n")
        .with_context(|| format!("write {}", dest.display()))?;
    Ok(())
}

pub fn load_shard(workspace: &Path, shard_id: &str) -> Result<Option<CallGraphShard>> {
    let dest = semantic_dir(workspace)
        .join("shards")
        .join(format!("{shard_id}.json"));
    if !dest.exists() {
        return Ok(None);
    }
    Ok(Some(serde_json::from_str(&fs::read_to_string(dest)?)?))
}

pub fn shard_id_for(member_paths: &[String]) -> String {
    let joined = member_paths.join("|");
    let h = xxh3_128(format!("t4-shard|{INTERPROC_ALGO_VERSION}|{joined}").as_bytes());
    let hex = hex::encode(h.to_be_bytes());
    hex[..16.min(hex.len())].to_string()
}

/// Invalidate shards that include any of `dirty_paths`.
pub fn invalidate_shards_for(workspace: &Path, dirty_paths: &[String]) -> Result<usize> {
    let dir = semantic_dir(workspace).join("shards");
    if !dir.exists() {
        return Ok(0);
    }
    let dirty: BTreeSet<&str> = dirty_paths.iter().map(|s| s.as_str()).collect();
    let mut removed = 0usize;
    for ent in fs::read_dir(&dir)? {
        let ent = ent?;
        let p = ent.path();
        if p.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let text = fs::read_to_string(&p)?;
        let shard: CallGraphShard = match serde_json::from_str(&text) {
            Ok(s) => s,
            Err(_) => continue,
        };
        if shard.member_paths.iter().any(|m| dirty.contains(m.as_str())) {
            fs::remove_file(&p)?;
            removed += 1;
        }
    }
    Ok(removed)
}

pub fn load_catalog(workspace: &Path) -> Result<HashMap<String, FnLoc>> {
    let dir = semantic_dir(workspace).join("by-file");
    let mut out = HashMap::new();
    if !dir.exists() {
        return Ok(out);
    }
    for ent in fs::read_dir(&dir)? {
        let ent = ent?;
        let p = ent.path();
        if p.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let art: SemanticFileArtifact = serde_json::from_str(&fs::read_to_string(&p)?)?;
        for f in &art.functions {
            let key = format!("{}::{}", art.path, f.name);
            out.insert(
                key.clone(),
                FnLoc {
                    key,
                    path: art.path.clone(),
                    name: f.name.clone(),
                    start_line: f.start_line,
                    end_line: f.end_line,
                },
            );
        }
    }
    Ok(out)
}

fn resolve_fn_key(catalog: &HashMap<String, FnLoc>, path: &str, name: &str) -> Option<String> {
    let direct = format!("{path}::{name}");
    if catalog.contains_key(&direct) {
        return Some(direct);
    }
    catalog
        .values()
        .find(|l| l.name == name && (l.path == path || l.path.ends_with(path)))
        .map(|l| l.key.clone())
        .or_else(|| {
            catalog
                .values()
                .find(|l| l.name == name)
                .map(|l| l.key.clone())
        })
}

pub fn build_call_index(
    workspace: &Path,
    catalog: &HashMap<String, FnLoc>,
) -> Result<Vec<OverlayEdge>> {
    let mut by_name: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for loc in catalog.values() {
        by_name
            .entry(loc.name.clone())
            .or_default()
            .push(loc.key.clone());
    }
    let mut paths: BTreeSet<String> = BTreeSet::new();
    for loc in catalog.values() {
        paths.insert(loc.path.clone());
    }
    let mut edges = Vec::new();
    for path in paths {
        let Some(art) = load_file_artifact(workspace, &path)? else {
            continue;
        };
        for f in &art.functions {
            let src = format!("{}::{}", art.path, f.name);
            for c in &f.calls {
                if let Some(dsts) = by_name.get(&c.callee) {
                    for dst in dsts {
                        edges.push(OverlayEdge {
                            kind: "CALLS".into(),
                            src: src.clone(),
                            dst: dst.clone(),
                            path: Some(art.path.clone()),
                            line: Some(c.line),
                        });
                        edges.push(OverlayEdge {
                            kind: "CONTROL_DEP".into(),
                            src: src.clone(),
                            dst: dst.clone(),
                            path: Some(art.path.clone()),
                            line: Some(c.line),
                        });
                        edges.push(OverlayEdge {
                            kind: "DATA_FLOW".into(),
                            src: src.clone(),
                            dst: dst.clone(),
                            path: Some(art.path.clone()),
                            line: Some(c.line),
                        });
                    }
                }
            }
        }
    }
    Ok(edges)
}

pub fn expand_from_edges(
    seed: &str,
    all_edges: &[OverlayEdge],
    max_depth: u32,
    max_functions: usize,
) -> (Vec<String>, Vec<OverlayEdge>, bool) {
    let mut adj: HashMap<String, Vec<&OverlayEdge>> = HashMap::new();
    for e in all_edges {
        if e.kind != "CALLS" {
            continue;
        }
        adj.entry(e.src.clone()).or_default().push(e);
        adj.entry(e.dst.clone()).or_default().push(e);
    }

    let mut visited: BTreeSet<String> = BTreeSet::new();
    let mut kept_edges: Vec<OverlayEdge> = Vec::new();
    let mut q: VecDeque<(String, u32)> = VecDeque::new();
    q.push_back((seed.to_string(), 0));
    visited.insert(seed.to_string());
    let mut truncated = false;

    while let Some((cur, depth)) = q.pop_front() {
        if visited.len() > max_functions {
            truncated = true;
            break;
        }
        if depth >= max_depth {
            continue;
        }
        let Some(nbrs) = adj.get(&cur) else {
            continue;
        };
        for e in nbrs {
            let other = if e.src == cur { &e.dst } else { &e.src };
            kept_edges.push((*e).clone());
            for o in all_edges {
                if o.src == e.src && o.dst == e.dst && o.kind != "CALLS" {
                    kept_edges.push(o.clone());
                }
            }
            if visited.insert(other.clone()) {
                if visited.len() > max_functions {
                    truncated = true;
                    break;
                }
                q.push_back((other.clone(), depth + 1));
            }
        }
    }

    kept_edges.sort();
    kept_edges.dedup();
    (visited.into_iter().collect(), kept_edges, truncated)
}
