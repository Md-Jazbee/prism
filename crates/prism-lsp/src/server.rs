//! Stdio LSP server (lsp-server + lsp-types).

use crate::LspConfig;
use anyhow::{Context, Result};
use lsp_server::{Connection, Message, Notification, Request, Response};
use lsp_types::notification::{DidOpenTextDocument, Notification as _};
use lsp_types::request::{
    CodeLensRequest, ExecuteCommand, GotoDefinition, HoverRequest, Initialize, Request as _,
    WorkspaceSymbolRequest,
};
use lsp_types::*;
use prism_compile::{compile_context, CompileOutcome};
use prism_plan::PlanHints;
use prism_store::SqliteKgStore;
use prism_view::{project_view, ViewKind, ViewParams};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tracing::{info, warn};

pub fn run_stdio(cfg: LspConfig) -> Result<()> {
    let (connection, io_threads) = Connection::stdio();
    let server_capabilities = serde_json::to_value(ServerCapabilities {
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        definition_provider: Some(OneOf::Left(true)),
        workspace_symbol_provider: Some(OneOf::Left(true)),
        code_lens_provider: Some(CodeLensOptions {
            resolve_provider: Some(false),
        }),
        execute_command_provider: Some(ExecuteCommandOptions {
            commands: vec![
                "prism.compileContext".into(),
                "prism.impact".into(),
                "prism.slice".into(),
                "prism.evidencePeek".into(),
                "prism.explain".into(),
            ],
            work_done_progress_options: Default::default(),
        }),
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        ..Default::default()
    })?;

    let init = connection.initialize(server_capabilities)?;
    let _params: InitializeParams = serde_json::from_value(init).unwrap_or_default();
    info!(workspace = %cfg.workspace.display(), "prism-lsp ready");

    let kg_path = cfg.workspace.join(".prism/graph.sqlite");
    if !kg_path.exists() {
        warn!("no graph.sqlite — symbol features will be empty until `prism index`");
    }

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    break;
                }
                let resp = handle_request(&cfg, req);
                connection.sender.send(Message::Response(resp))?;
            }
            Message::Notification(n) => {
                handle_notification(&n);
            }
            Message::Response(_) => {}
        }
    }

    io_threads.join()?;
    Ok(())
}

fn handle_notification(n: &Notification) {
    if n.method == DidOpenTextDocument::METHOD {
        // No-op: Prism indexes via CLI/daemon; LSP does not reparse.
    }
}

fn handle_request(cfg: &LspConfig, req: Request) -> Response {
    let id = req.id.clone();
    let result = match req.method.as_str() {
        Initialize::METHOD => Ok(serde_json::json!({})),
        HoverRequest::METHOD => hover(cfg, req),
        WorkspaceSymbolRequest::METHOD => workspace_symbol(cfg, req),
        CodeLensRequest::METHOD => code_lens(cfg, req),
        ExecuteCommand::METHOD => execute_command(cfg, req),
        GotoDefinition::METHOD => Ok(Value::Null),
        _ => {
            return Response::new_err(
                id,
                lsp_server::ErrorCode::MethodNotFound as i32,
                format!("unsupported {}", req.method),
            );
        }
    };
    match result {
        Ok(v) => Response::new_ok(id, v),
        Err(e) => Response::new_err(
            id,
            lsp_server::ErrorCode::InternalError as i32,
            e.to_string(),
        ),
    }
}

fn open_kg(workspace: &Path) -> Result<SqliteKgStore> {
    SqliteKgStore::open(workspace.join(".prism/graph.sqlite")).with_context(|| "open graph.sqlite")
}

fn hover(cfg: &LspConfig, req: Request) -> Result<Value> {
    let params: HoverParams = serde_json::from_value(req.params)?;
    let path = uri_to_path(&params.text_document_position_params.text_document.uri);
    let line = params.text_document_position_params.position.line + 1;
    let word = format!(
        "{}:L{line}",
        path.file_name().and_then(|s| s.to_str()).unwrap_or("file")
    );

    let kg = open_kg(&cfg.workspace)?;
    // Prefer resolving basename-ish symbols from path stem.
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("main");
    let hits = kg.resolve_symbol(stem, Some(&path_rel(&cfg.workspace, &path)), 5)?;
    let (label, note) = if let Some(h) = hits.first() {
        (
            h.name.clone().unwrap_or_else(|| h.id.clone()),
            format!("Prism hover · {} · confidence={}", h.kind, h.confidence),
        )
    } else {
        (
            word,
            "Prism hover · no KG symbol at cursor — try workspace symbol search".into(),
        )
    };

    let mut hints = PlanHints {
        anchors: vec![label.clone()],
        budget_tokens: Some(800),
        ..Default::default()
    };
    if let Some(h) = hits.first() {
        hints.anchors.push(h.id.clone());
    }
    let summary = match compile_context(&cfg.workspace, &format!("summarize {label}"), &hints) {
        Ok(CompileOutcome::Ok(pack)) => {
            let n = pack.fragments.len();
            let tokens = pack.meta.tokens_used;
            format!("Evidence Pack: {n} fragments · ~{tokens} tokens\n{note}")
        }
        _ => note,
    };

    Ok(serde_json::to_value(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!("**{label}**\n\n{summary}"),
        }),
        range: None,
    })?)
}

fn workspace_symbol(cfg: &LspConfig, req: Request) -> Result<Value> {
    let params: WorkspaceSymbolParams = serde_json::from_value(req.params)?;
    let q = params.query.trim();
    if q.is_empty() {
        return Ok(Value::Null);
    }
    let kg = open_kg(&cfg.workspace)?;
    let hits = kg.resolve_symbol(q, None, 40)?;
    let symbols: Vec<SymbolInformation> = hits
        .into_iter()
        .filter_map(|h| {
            let path = h.file_path?;
            let uri = path_to_uri(&cfg.workspace.join(&path));
            #[allow(deprecated)]
            Some(SymbolInformation {
                name: h.name.unwrap_or(h.id),
                kind: SymbolKind::FUNCTION,
                tags: None,
                deprecated: None,
                location: Location {
                    uri,
                    range: Range::default(),
                },
                container_name: Some(path),
            })
        })
        .collect();
    Ok(serde_json::to_value(symbols)?)
}

fn code_lens(cfg: &LspConfig, req: Request) -> Result<Value> {
    let params: CodeLensParams = serde_json::from_value(req.params)?;
    let path = uri_to_path(&params.text_document.uri);
    let rel = path_rel(&cfg.workspace, &path);
    let kg = open_kg(&cfg.workspace)?;
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("main");
    let hits = kg.resolve_symbol(stem, Some(&rel), 3)?;
    let mut lenses = Vec::new();
    if let Some(h) = hits.first() {
        lenses.push(CodeLens {
            range: Range::new(Position::new(0, 0), Position::new(0, 0)),
            command: Some(Command {
                title: format!("Prism: impact {}", h.name.as_deref().unwrap_or(&h.id)),
                command: "prism.impact".into(),
                arguments: Some(vec![Value::String(h.id.clone())]),
            }),
            data: None,
        });
        lenses.push(CodeLens {
            range: Range::new(Position::new(0, 0), Position::new(0, 0)),
            command: Some(Command {
                title: "Prism: compile context".into(),
                command: "prism.compileContext".into(),
                arguments: Some(vec![Value::String(h.id.clone())]),
            }),
            data: None,
        });
    }
    Ok(serde_json::to_value(lenses)?)
}

fn execute_command(cfg: &LspConfig, req: Request) -> Result<Value> {
    let params: ExecuteCommandParams = serde_json::from_value(req.params)?;
    let arg0 = params
        .arguments
        .first()
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    match params.command.as_str() {
        "prism.compileContext" => {
            let hints = PlanHints {
                anchors: vec![arg0.clone()],
                budget_tokens: Some(2000),
                ..Default::default()
            };
            match compile_context(&cfg.workspace, &format!("explain {arg0}"), &hints)? {
                CompileOutcome::Ok(pack) => Ok(serde_json::to_value(pack)?),
                other => Ok(serde_json::to_value(other)?),
            }
        }
        "prism.impact" => {
            let kg = open_kg(&cfg.workspace)?;
            let hits = kg.impact(&arg0, 2, 50)?;
            Ok(serde_json::to_value(hits)?)
        }
        "prism.slice" => {
            let kg = open_kg(&cfg.workspace)?;
            let outcome = project_view(
                &kg,
                &cfg.workspace,
                ViewKind::SlicePath,
                &ViewParams {
                    snapshot_id: "lsp".into(),
                    seed_id: Some(arg0),
                    ..Default::default()
                },
            )?;
            Ok(serde_json::to_value(outcome)?)
        }
        "prism.evidencePeek" | "prism.explain" => Ok(serde_json::json!({
            "ok": true,
            "note": "Use the Evidence Pack panel / last compile_context result"
        })),
        other => anyhow::bail!("unknown command {other}"),
    }
}

fn uri_to_path(uri: &Uri) -> PathBuf {
    let s = uri.as_str();
    if let Some(rest) = s.strip_prefix("file://") {
        PathBuf::from(rest)
    } else {
        PathBuf::from(s)
    }
}

fn path_to_uri(path: &Path) -> Uri {
    let s = format!("file://{}", path.display());
    Uri::from_str(&s).unwrap_or_else(|_| Uri::from_str("file:///").expect("root uri"))
}

fn path_rel(workspace: &Path, path: &Path) -> String {
    path.strip_prefix(workspace)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
