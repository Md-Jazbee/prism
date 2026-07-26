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
    /// Machine-actionable next step (P9 Stage A). Bounded — never a content dump.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repair: Option<serde_json::Value>,
}

impl ToolError {
    pub fn scope_unresolved(message: impl Into<String>) -> Self {
        let message = message.into();
        let repair = prism_agent_repair("SCOPE_UNRESOLVED", &message, &[]);
        Self {
            code: ToolErrorCode::ScopeUnresolved,
            message,
            hint: Some(
                "Provide a concrete symbol name, path, stack frame, or error text; do not dump the whole repository."
                    .into(),
            ),
            repair: Some(repair),
        }
    }

    pub fn budget_exceeded(message: impl Into<String>) -> Self {
        let message = message.into();
        let repair = prism_agent_repair("BUDGET_EXCEEDED", &message, &[]);
        Self {
            code: ToolErrorCode::BudgetExceeded,
            message,
            hint: Some(
                "Raise remaining_context_tokens / budget_tokens or narrow anchors; must-include fragments cannot be dropped."
                    .into(),
            ),
            repair: Some(repair),
        }
    }

    pub fn index_unavailable(message: impl Into<String>) -> Self {
        let message = message.into();
        let repair = prism_agent_repair("INDEX_UNAVAILABLE", &message, &[]);
        Self {
            code: ToolErrorCode::IndexUnavailable,
            message,
            hint: Some("Run `prism index <workspace>` then retry.".into()),
            repair: Some(repair),
        }
    }

    pub fn invalid_args(message: impl Into<String>) -> Self {
        let message = message.into();
        Self {
            code: ToolErrorCode::InvalidArgs,
            message: message.clone(),
            hint: None,
            repair: Some(prism_agent_repair("INVALID_ARGS", &message, &[])),
        }
    }

    pub fn precision_required(message: impl Into<String>) -> Self {
        let message = message.into();
        let repair = prism_agent_repair("PRECISION_REQUIRED", &message, &[]);
        Self {
            code: ToolErrorCode::PrecisionRequired,
            message,
            hint: Some(
                "Run a language indexer → PreciseIndex, then `prism precise import`. Heuristic T1 results remain available but must stay labeled."
                    .into(),
            ),
            repair: Some(repair),
        }
    }

    pub fn with_candidates(mut self, candidates: Vec<String>) -> Self {
        let code = match self.code {
            ToolErrorCode::ScopeUnresolved => "SCOPE_UNRESOLVED",
            ToolErrorCode::BudgetExceeded => "BUDGET_EXCEEDED",
            ToolErrorCode::IndexUnavailable => "INDEX_UNAVAILABLE",
            ToolErrorCode::PrecisionRequired => "PRECISION_REQUIRED",
            ToolErrorCode::InvalidArgs => "INVALID_ARGS",
            ToolErrorCode::Internal => "INTERNAL",
        };
        self.repair = Some(prism_agent_repair(code, &self.message, &candidates));
        self
    }
}

fn prism_agent_repair(code: &str, message: &str, candidates: &[String]) -> serde_json::Value {
    // Keep prism-mcp free of a hard prism-agent dependency cycle: inline the
    // same shape as `prism_agent::repair_for` (action / summary / candidates).
    let (action, summary, tool) = match code {
        "SCOPE_UNRESOLVED" => (
            "pick_anchor",
            "Provide a concrete symbol, path, stack frame, or error text.",
            Some("compile_context"),
        ),
        "BUDGET_EXCEEDED" => (
            "reduce_budget_or_narrow",
            "Raise remaining_context_tokens / budget_tokens or narrow anchors.",
            Some("query_plan"),
        ),
        "INDEX_UNAVAILABLE" => (
            "run_index",
            "Build or refresh the local index, then retry.",
            None,
        ),
        "PRECISION_REQUIRED" => (
            "import_precise",
            "Import a PreciseIndex (SCIP) or continue with labeled heuristic only.",
            None,
        ),
        "VIEW_TOO_LARGE" => (
            "narrow_view",
            "Narrow seeds/anchors or raise max_nodes explicitly.",
            None,
        ),
        _ => ("retry_or_report", message, None),
    };
    let mut cands: Vec<String> = candidates.to_vec();
    cands.truncate(8);
    if cands.is_empty() && code == "SCOPE_UNRESOLVED" {
        cands = vec![
            "symbol name".into(),
            "file path".into(),
            "stack frame / error text".into(),
        ];
    }
    serde_json::json!({
        "action": action,
        "summary": summary,
        "tool": tool,
        "candidates": cands,
    })
}
