//! Minimal MCP stdio JSON-RPC server (tools/list + tools/call).
//!
//! Speaks newline-delimited JSON-RPC 2.0 compatible with common MCP clients.
//! Tool logic lives in [`crate::tools`] so eval/CLI can call without stdio.

use crate::tools::{call_tool, list_tools_schema, ToolContext, ToolOutcome};
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use tracing::{error, info};

pub fn serve_stdio(workspace: PathBuf) -> Result<()> {
    let ctx = ToolContext::open(&workspace).with_context(|| {
        format!(
            "open MCP workspace {} (run prism index first)",
            workspace.display()
        )
    })?;
    info!(root = %workspace.display(), "prism MCP server ready (stdio)");

    let stdin = std::io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let mut stdout = std::io::stdout().lock();
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let req: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                error!(error = %e, "invalid json-rpc line");
                continue;
            }
        };
        if let Some(resp) = handle_request(&ctx, req) {
            writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
            stdout.flush()?;
        }
    }
    Ok(())
}

fn handle_request(ctx: &ToolContext, req: Value) -> Option<Value> {
    let id = req.get("id").cloned();
    let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let params = req.get("params").cloned().unwrap_or(json!({}));

    // Notifications have no id — acknowledge silently for initialized.
    if id.is_none() {
        if method == "notifications/initialized" {
            info!("client initialized");
        }
        return None;
    }

    let result = match method {
        "initialize" => json!({
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": {} },
            "serverInfo": {
                "name": "prism",
                "version": env!("CARGO_PKG_VERSION")
            },
            "instructions": "Prefer Prism structural tools (resolve_symbol, neighbors, impact, repo_map) over grep/read loops. Always check index_status first. impact is HEURISTIC at T1."
        }),
        "ping" => json!({}),
        "tools/list" => json!({ "tools": list_tools_schema() }),
        "tools/call" => {
            let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let arguments = params.get("arguments").cloned().unwrap_or(json!({}));
            let outcome = call_tool(ctx, name, arguments);
            match outcome {
                ToolOutcome::Ok(success) => {
                    let text = serde_json::to_string_pretty(&success).unwrap_or_default();
                    json!({
                        "content": [{ "type": "text", "text": text }],
                        "structuredContent": success,
                        "isError": false
                    })
                }
                ToolOutcome::Err { error } => {
                    let text = serde_json::to_string_pretty(&error).unwrap_or_default();
                    json!({
                        "content": [{ "type": "text", "text": text }],
                        "structuredContent": { "error": error },
                        "isError": true
                    })
                }
            }
        }
        _ => {
            return Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32601, "message": format!("Method not found: {method}") }
            }));
        }
    };

    Some(json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    }))
}
