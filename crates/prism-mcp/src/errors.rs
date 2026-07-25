//! MCP / product error model (P1 Stage C).

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Product-level error codes returned inside tool JSON (not always MCP protocol errors).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ToolErrorCode {
    /// Ambiguous or missing anchors — refuse unbounded dump; ask for a symbol/path.
    ScopeUnresolved,
    /// Index missing or stale relative to expected snapshot.
    IndexUnavailable,
    /// Tool arguments invalid.
    InvalidArgs,
    /// Internal failure.
    Internal,
}

#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[error("{code:?}: {message}")]
pub struct ToolError {
    pub code: ToolErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

impl ToolError {
    pub fn scope_unresolved(message: impl Into<String>) -> Self {
        Self {
            code: ToolErrorCode::ScopeUnresolved,
            message: message.into(),
            hint: Some(
                "Provide a concrete symbol name or node id; do not dump the whole repository."
                    .into(),
            ),
        }
    }

    pub fn index_unavailable(message: impl Into<String>) -> Self {
        Self {
            code: ToolErrorCode::IndexUnavailable,
            message: message.into(),
            hint: Some("Run `prism index <workspace>` then retry.".into()),
        }
    }

    pub fn invalid_args(message: impl Into<String>) -> Self {
        Self {
            code: ToolErrorCode::InvalidArgs,
            message: message.into(),
            hint: None,
        }
    }
}
