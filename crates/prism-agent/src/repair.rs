//! Machine-actionable refusal repairs (P9 Stage A).
//!
//! Repair suggestions are **bounded lists**, never content dumps.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepairKind {
    PickAnchor,
    ReduceBudgetOrNarrow,
    RunIndex,
    ImportPrecise,
    NarrowView,
    FixArgs,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RepairAction {
    pub kind: RepairKind,
    /// Stable id for traces / UI buttons.
    pub action: String,
    pub summary: String,
    /// Optional next tool the agent should call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    /// Suggested args (no file bodies).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args_hint: Option<Value>,
    /// Bounded candidate anchors / plans (≤ 8).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub candidates: Vec<String>,
}

/// Build a repair payload for a product error code.
pub fn repair_for(
    code: &str,
    message: &str,
    candidates: impl IntoIterator<Item = impl Into<String>>,
) -> RepairAction {
    let mut cands: Vec<String> = candidates.into_iter().map(Into::into).collect();
    cands.truncate(8);

    match code {
        "SCOPE_UNRESOLVED" => RepairAction {
            kind: RepairKind::PickAnchor,
            action: "pick_anchor".into(),
            summary: "Provide a concrete symbol, path, stack frame, or error text.".into(),
            tool: Some("compile_context".into()),
            args_hint: Some(json!({
                "anchors": cands.first().cloned().into_iter().collect::<Vec<_>>(),
            })),
            candidates: if cands.is_empty() {
                vec![
                    "symbol name".into(),
                    "file path".into(),
                    "stack frame / error text".into(),
                ]
            } else {
                cands
            },
        },
        "BUDGET_EXCEEDED" => RepairAction {
            kind: RepairKind::ReduceBudgetOrNarrow,
            action: "reduce_budget_or_narrow".into(),
            summary: "Raise remaining_context_tokens / budget_tokens or narrow anchors; must-include cannot drop.".into(),
            tool: Some("query_plan".into()),
            args_hint: Some(json!({
                "budget_tokens": 2000,
                "note": "Inspect plan, then recompile with higher budget or fewer anchors"
            })),
            candidates: cands,
        },
        "INDEX_UNAVAILABLE" => RepairAction {
            kind: RepairKind::RunIndex,
            action: "run_index".into(),
            summary: "Build or refresh the local index, then retry.".into(),
            tool: None,
            args_hint: Some(json!({ "cli": "prism index ." })),
            candidates: vec!["prism index .".into()],
        },
        "PRECISION_REQUIRED" => RepairAction {
            kind: RepairKind::ImportPrecise,
            action: "import_precise".into(),
            summary: "Import a PreciseIndex (SCIP) or continue with labeled heuristic only.".into(),
            tool: None,
            args_hint: Some(json!({
                "cli": "prism precise import <precise-index.json>",
                "or": "retry without require_precise=true"
            })),
            candidates: cands,
        },
        "VIEW_TOO_LARGE" => RepairAction {
            kind: RepairKind::NarrowView,
            action: "narrow_view".into(),
            summary: "Narrow seeds/anchors or raise max_nodes explicitly.".into(),
            tool: None,
            args_hint: Some(json!({ "view_kind": "architecture_map", "max_nodes": 40 })),
            candidates: cands,
        },
        "INVALID_ARGS" => RepairAction {
            kind: RepairKind::FixArgs,
            action: "fix_args".into(),
            summary: message.to_string(),
            tool: None,
            args_hint: None,
            candidates: cands,
        },
        _ => RepairAction {
            kind: RepairKind::FixArgs,
            action: "retry_or_report".into(),
            summary: format!("{code}: {message}"),
            tool: None,
            args_hint: None,
            candidates: cands,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_repair_is_bounded() {
        let r = repair_for(
            "SCOPE_UNRESOLVED",
            "missing",
            (0..20).map(|i| format!("sym{i}")),
        );
        assert_eq!(r.action, "pick_anchor");
        assert!(r.candidates.len() <= 8);
        assert_eq!(r.tool.as_deref(), Some("compile_context"));
    }
}
