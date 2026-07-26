//! MCP / product error model (P1–P3).

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Product-level error codes returned inside tool JSON (not always MCP protocol errors).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ToolErrorCode {
    /// Ambiguous or missing anchors — refuse unbounded dump; ask for a symbol/path.
    ScopeUnresolved,
    /// Must-include evidence cannot fit the token budget.
    BudgetExceeded,
    /// Index missing or stale relative to expected snapshot.
    IndexUnavailable,
    /// Precise (T2) overlay required for this claim / operation (P3).
    PrecisionRequired,
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
                "Provide a concrete symbol name, path, stack frame, or error text; do not dump the whole repository."
                    .into(),
            ),
        }
    }

    pub fn budget_exceeded(message: impl Into<String>) -> Self {
        Self {
            code: ToolErrorCode::BudgetExceeded,
            message: message.into(),
            hint: Some(
                "Raise budget_tokens or narrow anchors; must-include fragments cannot be dropped."
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

    pub fn precision_required(message: impl Into<String>) -> Self {
        Self {
            code: ToolErrorCode::PrecisionRequired,
            message: message.into(),
            hint: Some(
                "Run a language indexer → PreciseIndex, then `prism precise import`. Heuristic T1 results remain available but must stay labeled."
                    .into(),
            ),
        }
    }
}
