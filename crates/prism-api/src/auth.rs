//! Loopback token handshake (P6 Stage B).

use crate::error::ApiError;
use crate::state::AppState;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;

pub async fn require_token(
    axum::extract::State(state): axum::extract::State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Health is public for process liveness; everything under /v1 needs a token.
    let path = request.uri().path();
    if path == "/health" || path == "/v1/health" {
        return Ok(next.run(request).await);
    }

    let header_token = request
        .headers()
        .get("x-prism-token")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let bearer = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::to_string);

    let provided = header_token.or(bearer);
    match provided {
        Some(t) if t == state.token => {
            state.touch();
            Ok(next.run(request).await)
        }
        _ => Err(ApiError::unauthorized()),
    }
}
