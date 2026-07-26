//! HTTP API smoke tests (P6 Stage B/C).

use axum::body::Body;
use http::{Request, StatusCode};
use http_body_util::BodyExt;
use prism_api::{router, AppState, DaemonConfig};
use prism_core::{IncrementalIndexer, IndexOptions, WorkspaceManager};
use serde_json::Value;
use std::fs;
use tempfile::TempDir;
use tower::ServiceExt;

struct Fixture {
    _dir: TempDir,
    state: AppState,
    token: String,
}

async fn setup() -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    fs::write(
        root.join("m.py"),
        b"def entry():\n    helper()\n\ndef helper():\n    return 1\n",
    )
    .unwrap();
    let wm = WorkspaceManager::open(&root).unwrap();
    let mut indexer = IncrementalIndexer::open(wm, root.join(".prism")).unwrap();
    indexer.run(&IndexOptions::default()).unwrap();

    let token = "test-token".to_string();
    let cfg = DaemonConfig::loopback(root, token.clone());
    let state = AppState::new(&cfg);
    let _ = state.reindex(vec![]).await.unwrap();
    Fixture {
        _dir: dir,
        state,
        token,
    }
}

async fn json_get(state: AppState, token: &str, path: &str) -> (StatusCode, Value) {
    let app = router(state);
    let req = Request::builder()
        .uri(path)
        .header("x-prism-token", token)
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let v: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, v)
}

async fn json_post(state: AppState, token: &str, path: &str, body: Value) -> (StatusCode, Value) {
    let app = router(state);
    let req = Request::builder()
        .method("POST")
        .uri(path)
        .header("x-prism-token", token)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let v: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, v)
}

#[tokio::test]
async fn health_and_status_and_compile() {
    let fx = setup().await;

    let app = router(fx.state.clone());
    let req = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let (st, body) = json_get(fx.state.clone(), &fx.token, "/v1/index/status").await;
    assert_eq!(st, StatusCode::OK);
    assert!(body.get("snapshot_id").is_some());
    assert!(body["nodes"].as_u64().unwrap_or(0) > 0);

    let (st, body) = json_post(
        fx.state.clone(),
        &fx.token,
        "/v1/query/plan",
        serde_json::json!({
            "question": "What does entry do?",
            "anchors": ["entry"],
            "intent": "repo_qa"
        }),
    )
    .await;
    assert_eq!(st, StatusCode::OK, "{body}");
    assert!(body.get("plan").is_some());

    let (st, body) = json_post(
        fx.state.clone(),
        &fx.token,
        "/v1/context/compile",
        serde_json::json!({
            "question": "Explain entry",
            "anchors": ["entry"],
            "intent": "repo_qa",
            "budget_tokens": 2000
        }),
    )
    .await;
    assert_eq!(st, StatusCode::OK, "{body}");
    assert!(body.get("pack").is_some());

    // P6 gate: status → view → pack over HTTP
    let (st, body) = json_post(
        fx.state.clone(),
        &fx.token,
        "/v1/view",
        serde_json::json!({
            "view_kind": "architecture_map",
            "max_nodes": 80
        }),
    )
    .await;
    assert_eq!(st, StatusCode::OK, "{body}");
    assert!(body.get("view").is_some(), "{body}");
    assert_eq!(body["view"]["schema_version"], "graph-view/v1");

    let (st, body) = json_post(
        fx.state.clone(),
        &fx.token,
        "/v1/view",
        serde_json::json!({
            "view_kind": "architecture_map",
            "max_nodes": 0
        }),
    )
    .await;
    // max_nodes=0 may refuse or return empty depending on seed count
    assert!(
        st == StatusCode::OK || st == StatusCode::UNPROCESSABLE_ENTITY,
        "{st} {body}"
    );
    if st == StatusCode::UNPROCESSABLE_ENTITY {
        assert_eq!(body["error"]["code"], "VIEW_TOO_LARGE");
    }

    let (st, body) = json_get(
        fx.state.clone(),
        &fx.token,
        "/v1/intel/entrypoints?limit=10",
    )
    .await;
    assert_eq!(st, StatusCode::OK, "{body}");
    assert!(body.get("entrypoints").is_some());

    let (st, _) = json_get(fx.state, "bad-token", "/v1/index/status").await;
    assert_eq!(st, StatusCode::UNAUTHORIZED);
}
