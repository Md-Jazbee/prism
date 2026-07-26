//! HTTP/SSE API v1 for Prism (P6 Stage B).
//!
//! Transport only — mirrors the MCP error model; no new analysis semantics.

mod auth;
mod error;
mod routes;
mod state;

pub use auth::require_token;
pub use error::{ApiError, ApiErrorBody};
pub use routes::router;
pub use state::{AppState, DaemonConfig, InvalidationEvent, PRISM_API_VERSION};

use anyhow::Result;
use axum::Router;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing::info;

/// Bind and serve the `/v1` API until the process is cancelled.
pub async fn serve(state: AppState, addr: SocketAddr) -> Result<()> {
    let app: Router = router(state.clone()).layer(TraceLayer::new_for_http());
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!(%addr, workspace = %state.workspace.display(), "prismd listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    info!("shutdown signal received");
}
