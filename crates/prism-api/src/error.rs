//! Product error model mirrored from MCP (ADD §22.3).

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use prism_mcp::{ToolError, ToolErrorCode};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiErrorBody {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot_id: Option<String>,
}

#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub body: ApiErrorBody,
}

impl ApiError {
    pub fn from_tool(err: ToolError, snapshot_id: Option<String>) -> Self {
        let status = match err.code {
            ToolErrorCode::ScopeUnresolved => StatusCode::UNPROCESSABLE_ENTITY,
            ToolErrorCode::BudgetExceeded => StatusCode::UNPROCESSABLE_ENTITY,
            ToolErrorCode::PrecisionRequired => StatusCode::CONFLICT,
            ToolErrorCode::IndexUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            ToolErrorCode::InvalidArgs => StatusCode::BAD_REQUEST,
            ToolErrorCode::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let code = match err.code {
            ToolErrorCode::ScopeUnresolved => "SCOPE_UNRESOLVED",
            ToolErrorCode::BudgetExceeded => "BUDGET_EXCEEDED",
            ToolErrorCode::PrecisionRequired => "PRECISION_REQUIRED",
            ToolErrorCode::IndexUnavailable => "INDEX_UNAVAILABLE",
            ToolErrorCode::InvalidArgs => "INVALID_ARGS",
            ToolErrorCode::Internal => "INTERNAL",
        };
        Self {
            status,
            body: ApiErrorBody {
                code: code.into(),
                message: err.message,
                hint: err.hint,
                snapshot_id,
            },
        }
    }

    pub fn unauthorized() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            body: ApiErrorBody {
                code: "UNAUTHORIZED".into(),
                message: "missing or invalid Prism token".into(),
                hint: Some("Send Authorization: Bearer <token> or X-Prism-Token".into()),
                snapshot_id: None,
            },
        }
    }

    pub fn invalid_args(message: impl Into<String>) -> Self {
        Self::from_tool(ToolError::invalid_args(message), None)
    }

    pub fn view_too_large(body: prism_view::ViewTooLarge) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            body: ApiErrorBody {
                code: "VIEW_TOO_LARGE".into(),
                message: body.message,
                hint: Some(format!(
                    "{}; anchors: {}",
                    body.hint,
                    body.suggested_anchors.join(", ")
                )),
                snapshot_id: body.snapshot_id,
            },
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: ApiErrorBody {
                code: "INTERNAL".into(),
                message: message.into(),
                hint: None,
                snapshot_id: None,
            },
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(serde_json::json!({ "error": self.body }))).into_response()
    }
}
