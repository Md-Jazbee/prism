//! `/v1/*` routes — status, plan, compile, query, slice, intel, SSE.

use crate::auth::require_token;
use crate::error::ApiError;
use crate::state::{AppState, PRISM_API_VERSION};
use axum::extract::{Query, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::routing::{get, post};
use axum::{Json, Router};
use futures_util::stream::Stream;
use prism_compile::{compile_context, CompileOutcome};
use prism_mcp::ToolError;
use prism_plan::{plan_query, Intent, PlanHints, PlanOutcome};
use prism_semantic::{interproc_slice, SliceDirection, SliceParams};
use prism_store::{parse_edge_kinds, EdgeDirection, SqliteKgStore, SqliteMetaStore};
use serde::Deserialize;
use serde_json::{json, Value};
use std::convert::Infallible;
use std::str::FromStr;
use std::time::Duration;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/health", get(health))
        .route("/v1/index/status", get(index_status))
        .route("/v1/index", post(index_refresh))
        .route("/v1/query/plan", post(query_plan))
        .route("/v1/context/compile", post(context_compile))
        .route("/v1/symbols", get(symbols))
        .route("/v1/query/neighbors", post(neighbors))
        .route("/v1/impact", post(impact))
        .route("/v1/slice", post(slice))
        .route("/v1/repo/map", get(repo_map))
        .route("/v1/intel/entrypoints", get(entrypoints))
        .route("/v1/events", get(events_sse))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            require_token,
        ))
        .with_state(state)
}

async fn health(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "ok": true,
        "service": "prismd",
        "api_version": PRISM_API_VERSION,
        "snapshot_id": state.snapshot_id().await,
        "workspace": state.workspace.display().to_string(),
    }))
}

async fn index_status(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let snap = state.snapshot_id().await;
    let kg = open_kg(&state)?;
    let meta = open_meta(&state)?;
    let stats = kg.index_stats().map_err(|e| {
        ApiError::from_tool(
            ToolError::index_unavailable(e.to_string()),
            Some(snap.clone()),
        )
    })?;
    let files = meta.list_file_paths().map(|v| v.len()).unwrap_or(0);
    let graph_bytes = std::fs::metadata(state.workspace.join(".prism/graph.sqlite"))
        .map(|m| m.len())
        .unwrap_or(0);
    Ok(Json(json!({
        "snapshot_id": snap,
        "workspace": state.workspace.display().to_string(),
        "files_hashed": files,
        "nodes": stats.nodes,
        "edges": stats.edges,
        "files_indexed": stats.files_indexed,
        "graph_sqlite_bytes": graph_bytes,
        "tier": "T1",
        "freshness": "warm",
    })))
}

#[derive(Debug, Deserialize, Default)]
struct IndexBody {
    #[serde(default)]
    paths: Vec<String>,
}

async fn index_refresh(
    State(state): State<AppState>,
    Json(body): Json<IndexBody>,
) -> Result<Json<Value>, ApiError> {
    let snap = state
        .reindex(body.paths.clone())
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(json!({
        "ok": true,
        "snapshot_id": snap,
        "paths": body.paths,
    })))
}

#[derive(Debug, Deserialize)]
struct PlanBody {
    question: String,
    #[serde(default)]
    budget_tokens: Option<u32>,
    #[serde(default)]
    intent: Option<String>,
    #[serde(default)]
    anchors: Vec<String>,
    #[serde(default)]
    stack_frames: Vec<String>,
    #[serde(default)]
    error_text: Option<String>,
    #[serde(default)]
    changed_paths: Vec<String>,
}

async fn query_plan(
    State(state): State<AppState>,
    Json(body): Json<PlanBody>,
) -> Result<Json<Value>, ApiError> {
    let snap = state.snapshot_id().await;
    let hints = plan_hints(&body)?;
    match plan_query(&body.question, &hints) {
        Ok(PlanOutcome::Ok(plan)) => Ok(Json(json!({
            "snapshot_id": snap,
            "plan": plan,
        }))),
        Ok(PlanOutcome::ScopeUnresolved(u)) => Err(ApiError::from_tool(
            ToolError {
                code: prism_mcp::ToolErrorCode::ScopeUnresolved,
                message: u.reason,
                hint: Some(format!("Ask for: {}", u.ask_for.join("; "))),
            },
            Some(snap),
        )),
        Err(e) => Err(ApiError::internal(e.to_string())),
    }
}

#[derive(Debug, Deserialize)]
struct CompileBody {
    question: String,
    #[serde(default)]
    budget_tokens: Option<u32>,
    #[serde(default)]
    intent: Option<String>,
    #[serde(default)]
    anchors: Vec<String>,
    #[serde(default)]
    stack_frames: Vec<String>,
    #[serde(default)]
    error_text: Option<String>,
    #[serde(default)]
    changed_paths: Vec<String>,
    #[serde(default)]
    require_precise: bool,
}

async fn context_compile(
    State(state): State<AppState>,
    Json(body): Json<CompileBody>,
) -> Result<Json<Value>, ApiError> {
    let snap = state.snapshot_id().await;
    if body.require_precise {
        let kg = open_kg(&state)?;
        if let Err(e) = prism_precise::require_precise_claim(&state.workspace, &kg, None) {
            return Err(ApiError::from_tool(
                ToolError::precision_required(e.message),
                Some(snap),
            ));
        }
    }
    let plan_body = PlanBody {
        question: body.question.clone(),
        budget_tokens: body.budget_tokens,
        intent: body.intent.clone(),
        anchors: body.anchors.clone(),
        stack_frames: body.stack_frames.clone(),
        error_text: body.error_text.clone(),
        changed_paths: body.changed_paths.clone(),
    };
    let hints = plan_hints(&plan_body)?;
    let outcome = compile_context(&state.workspace, &body.question, &hints).map_err(|e| {
        ApiError::from_tool(
            ToolError::index_unavailable(e.to_string()),
            Some(snap.clone()),
        )
    })?;
    match outcome {
        CompileOutcome::Ok(pack) => Ok(Json(json!({
            "snapshot_id": snap,
            "pack": pack,
        }))),
        CompileOutcome::ScopeUnresolved(u) => Err(ApiError::from_tool(
            ToolError {
                code: prism_mcp::ToolErrorCode::ScopeUnresolved,
                message: u.reason,
                hint: Some(format!("Ask for: {}", u.ask_for.join("; "))),
            },
            Some(snap),
        )),
        CompileOutcome::BudgetExceeded(e) => Err(ApiError::from_tool(
            ToolError::budget_exceeded(e.reason),
            Some(snap),
        )),
    }
}

#[derive(Debug, Deserialize)]
struct SymbolsQuery {
    name: String,
    #[serde(default)]
    file: Option<String>,
    #[serde(default = "default_limit_20")]
    limit: usize,
}

fn default_limit_20() -> usize {
    20
}

async fn symbols(
    State(state): State<AppState>,
    Query(q): Query<SymbolsQuery>,
) -> Result<Json<Value>, ApiError> {
    let snap = state.snapshot_id().await;
    if q.name.trim().is_empty() {
        return Err(ApiError::from_tool(
            ToolError::scope_unresolved("name is required"),
            Some(snap),
        ));
    }
    let kg = open_kg(&state)?;
    let hits = kg
        .resolve_symbol(&q.name, q.file.as_deref(), q.limit)
        .map_err(|e| {
            ApiError::from_tool(
                ToolError::index_unavailable(e.to_string()),
                Some(snap.clone()),
            )
        })?;
    if hits.is_empty() {
        return Err(ApiError::from_tool(
            ToolError::scope_unresolved(format!("no symbols named '{}'", q.name)),
            Some(snap),
        ));
    }
    Ok(Json(json!({ "snapshot_id": snap, "symbols": hits })))
}

#[derive(Debug, Deserialize)]
struct NeighborsBody {
    id: String,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default = "default_dir")]
    dir: String,
    #[serde(default = "default_limit_50")]
    limit: usize,
}

fn default_dir() -> String {
    "outgoing".into()
}
fn default_limit_50() -> usize {
    50
}

async fn neighbors(
    State(state): State<AppState>,
    Json(body): Json<NeighborsBody>,
) -> Result<Json<Value>, ApiError> {
    let snap = state.snapshot_id().await;
    let kg = open_kg(&state)?;
    let kinds = parse_edge_kinds(body.kind.as_deref());
    let dir = match body.dir.as_str() {
        "incoming" => EdgeDirection::Incoming,
        "both" => EdgeDirection::Both,
        _ => EdgeDirection::Outgoing,
    };
    let hits = kg
        .neighbors(
            &body.id,
            kinds.as_deref(),
            dir,
            body.limit,
        )
        .map_err(|e| {
            ApiError::from_tool(
                ToolError::index_unavailable(e.to_string()),
                Some(snap.clone()),
            )
        })?;
    Ok(Json(json!({ "snapshot_id": snap, "neighbors": hits })))
}

#[derive(Debug, Deserialize)]
struct ImpactBody {
    id: String,
    #[serde(default = "default_depth")]
    depth: u32,
    #[serde(default = "default_limit_100")]
    limit: usize,
    #[serde(default)]
    require_precise: bool,
}

fn default_depth() -> u32 {
    2
}
fn default_limit_100() -> usize {
    100
}

async fn impact(
    State(state): State<AppState>,
    Json(body): Json<ImpactBody>,
) -> Result<Json<Value>, ApiError> {
    let snap = state.snapshot_id().await;
    let kg = open_kg(&state)?;
    if body.require_precise {
        if let Err(e) = prism_precise::require_precise_claim(&state.workspace, &kg, Some(&body.id))
        {
            return Err(ApiError::from_tool(
                ToolError::precision_required(e.message),
                Some(snap),
            ));
        }
    }
    let hits = kg.impact(&body.id, body.depth, body.limit).map_err(|e| {
        ApiError::from_tool(
            ToolError::index_unavailable(e.to_string()),
            Some(snap.clone()),
        )
    })?;
    Ok(Json(json!({
        "snapshot_id": snap,
        "impact": hits,
        "confidence_note": if body.require_precise {
            "precise-gated"
        } else {
            "HEURISTIC at T1"
        },
    })))
}

#[derive(Debug, Deserialize)]
struct SliceBody {
    path: String,
    #[serde(default)]
    line: Option<u32>,
    #[serde(default)]
    symbol: Option<String>,
    #[serde(default = "default_depth")]
    max_depth: u32,
    #[serde(default = "default_dir_slice")]
    direction: String,
}

fn default_dir_slice() -> String {
    "backward".into()
}

async fn slice(
    State(state): State<AppState>,
    Json(body): Json<SliceBody>,
) -> Result<Json<Value>, ApiError> {
    let snap = state.snapshot_id().await;
    let direction = match body.direction.as_str() {
        "forward" => SliceDirection::Forward,
        _ => SliceDirection::Backward,
    };
    let params = SliceParams {
        direction,
        max_depth: body.max_depth,
        path: body.path,
        line: body.line,
        symbol: body.symbol,
        snapshot_id: snap.clone(),
        ..Default::default()
    };
    let report = interproc_slice(&state.workspace, &params).map_err(|e| {
        ApiError::from_tool(
            ToolError::index_unavailable(e.to_string()),
            Some(snap.clone()),
        )
    })?;
    Ok(Json(json!({ "snapshot_id": snap, "slice": report })))
}

#[derive(Debug, Deserialize)]
struct RepoMapQuery {
    #[serde(default = "default_hub")]
    hub_limit: usize,
    #[serde(default)]
    full_intel: bool,
}

fn default_hub() -> usize {
    15
}

async fn repo_map(
    State(state): State<AppState>,
    Query(q): Query<RepoMapQuery>,
) -> Result<Json<Value>, ApiError> {
    let snap = state.snapshot_id().await;
    let kg = open_kg(&state)?;
    if q.full_intel {
        let report = kg
            .repo_intel(Some(&state.workspace), q.hub_limit)
            .map_err(|e| {
                ApiError::from_tool(
                    ToolError::index_unavailable(e.to_string()),
                    Some(snap.clone()),
                )
            })?;
        Ok(Json(json!({ "snapshot_id": snap, "intel": report })))
    } else {
        let map = kg.repo_map(q.hub_limit).map_err(|e| {
            ApiError::from_tool(
                ToolError::index_unavailable(e.to_string()),
                Some(snap.clone()),
            )
        })?;
        Ok(Json(json!({ "snapshot_id": snap, "repo_map": map })))
    }
}

#[derive(Debug, Deserialize)]
struct EntrypointsQuery {
    #[serde(default = "default_limit_40")]
    limit: usize,
}

fn default_limit_40() -> usize {
    40
}

async fn entrypoints(
    State(state): State<AppState>,
    Query(q): Query<EntrypointsQuery>,
) -> Result<Json<Value>, ApiError> {
    let snap = state.snapshot_id().await;
    let kg = open_kg(&state)?;
    let eps = kg.detect_entrypoints(q.limit).map_err(|e| {
        ApiError::from_tool(
            ToolError::index_unavailable(e.to_string()),
            Some(snap.clone()),
        )
    })?;
    Ok(Json(json!({ "snapshot_id": snap, "entrypoints": eps })))
}

async fn events_sse(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|msg| match msg {
        Ok(ev) => {
            let data = serde_json::to_string(&ev).unwrap_or_else(|_| "{}".into());
            Some(Ok(Event::default().event(ev.event).data(data)))
        }
        Err(_) => None,
    });
    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}

fn open_kg(state: &AppState) -> Result<SqliteKgStore, ApiError> {
    let path = state.workspace.join(".prism/graph.sqlite");
    if !path.exists() {
        return Err(ApiError::from_tool(
            ToolError::index_unavailable(format!("no graph.sqlite under {}", path.display())),
            None,
        ));
    }
    SqliteKgStore::open(path)
        .map_err(|e| ApiError::from_tool(ToolError::index_unavailable(e.to_string()), None))
}

fn open_meta(state: &AppState) -> Result<SqliteMetaStore, ApiError> {
    SqliteMetaStore::open(state.workspace.join(".prism/meta.sqlite"))
        .map_err(|e| ApiError::from_tool(ToolError::index_unavailable(e.to_string()), None))
}

fn plan_hints(body: &PlanBody) -> Result<PlanHints, ApiError> {
    let mut hints = PlanHints {
        budget_tokens: body.budget_tokens,
        anchors: body.anchors.clone(),
        stack_frames: body.stack_frames.clone(),
        error_text: body.error_text.clone(),
        changed_paths: body.changed_paths.clone(),
        ..Default::default()
    };
    if let Some(raw) = &body.intent {
        hints.intent_override =
            Some(Intent::from_str(raw).map_err(|e| ApiError::invalid_args(e.to_string()))?);
    }
    Ok(hints)
}
