//! Deterministic intent classification and query planner (P2 Stage A).
//!
//! No LLM in the planning path: recipes produce an operator DAG from
//! classified intent + anchors. See `docs/architecture/QUERY-PLANNER.md`.

mod intent;
mod operators;
mod plan;
mod recipe;

pub use intent::{classify_intent, extract_anchors, Intent, PlanHints};
pub use operators::Operator;
pub use plan::{Plan, PlanOutcome, PlanStep, ScopeUnresolved};
pub use recipe::{recipe_for, IntentRecipe, DEFAULT_BUDGET_TOKENS};

use anyhow::Result;

/// Build a query plan (or refuse with `SCOPE_UNRESOLVED`) for `question` + hints.
///
/// Deterministic: same inputs ⇒ same plan IR after [`Plan::normalize`].
pub fn plan_query(question: &str, hints: &PlanHints) -> Result<PlanOutcome> {
    let intent = hints
        .intent_override
        .unwrap_or_else(|| classify_intent(question, hints));

    let mut anchors = hints.anchors.clone();
    for a in extract_anchors(question) {
        if !anchors.iter().any(|x| x == &a) {
            anchors.push(a);
        }
    }
    for f in &hints.stack_frames {
        if !anchors.iter().any(|x| x == f) {
            anchors.push(f.clone());
        }
    }
    if let Some(err) = &hints.error_text {
        if !err.trim().is_empty() && !anchors.iter().any(|x| x == err) {
            anchors.push(err.clone());
        }
    }
    for p in &hints.changed_paths {
        if !anchors.iter().any(|x| x == p) {
            anchors.push(p.clone());
        }
    }

    if needs_anchors(intent) && anchors.is_empty() {
        return Ok(PlanOutcome::ScopeUnresolved(ScopeUnresolved {
            code: "SCOPE_UNRESOLVED".into(),
            reason: format!(
                "Intent `{intent}` needs anchors (symbol, path, stack frame, error, or changed path); none found in question or hints"
            ),
            ask_for: vec![
                "symbol name or qualified path".into(),
                "file path".into(),
                "stack frame / error text".into(),
                "changed path or diff hint".into(),
            ],
            intent: Some(intent),
            question: question.to_string(),
            candidates: Vec::new(),
        }));
    }

    let recipe = recipe_for(intent);
    let budget = hints.budget_tokens.unwrap_or(DEFAULT_BUDGET_TOKENS);
    let mut plan = recipe.build_plan(question, &anchors, budget);
    plan.normalize();
    Ok(PlanOutcome::Ok(plan))
}

fn needs_anchors(intent: Intent) -> bool {
    !matches!(intent, Intent::Architecture)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};

    fn fixture_dir(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../../fixtures/plans/{name}"))
    }

    fn load_hints(dir: &Path) -> (String, PlanHints) {
        let input: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(dir.join("input.json")).unwrap()).unwrap();
        let question = input["question"].as_str().unwrap().to_string();
        let mut hints = PlanHints::default();
        if let Some(b) = input.get("budget_tokens").and_then(|v| v.as_u64()) {
            hints.budget_tokens = Some(b as u32);
        }
        if let Some(intent) = input.get("intent").and_then(|v| v.as_str()) {
            hints.intent_override = Some(intent.parse().unwrap());
        }
        if let Some(arr) = input.get("anchors").and_then(|v| v.as_array()) {
            hints.anchors = arr
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect();
        }
        if let Some(arr) = input.get("stack_frames").and_then(|v| v.as_array()) {
            hints.stack_frames = arr
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect();
        }
        if let Some(err) = input.get("error_text").and_then(|v| v.as_str()) {
            hints.error_text = Some(err.to_string());
        }
        if let Some(arr) = input.get("changed_paths").and_then(|v| v.as_array()) {
            hints.changed_paths = arr
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect();
        }
        (question, hints)
    }

    fn assert_golden(name: &str) {
        let dir = fixture_dir(name);
        let (question, hints) = load_hints(&dir);
        let outcome = plan_query(&question, &hints).unwrap();
        let actual = serde_json::to_value(&outcome).unwrap();
        let expected: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(dir.join("expected.json")).unwrap()).unwrap();
        assert_eq!(
            actual, expected,
            "plan golden fixture mismatch for `{name}`"
        );
    }

    #[test]
    fn golden_repo_qa() {
        assert_golden("repo_qa");
    }

    #[test]
    fn golden_debug() {
        assert_golden("debug");
    }

    #[test]
    fn golden_impact() {
        assert_golden("impact");
    }

    #[test]
    fn golden_refactor() {
        assert_golden("refactor");
    }

    #[test]
    fn golden_architecture() {
        assert_golden("architecture");
    }

    #[test]
    fn golden_review() {
        assert_golden("review");
    }

    #[test]
    fn golden_generate() {
        assert_golden("generate");
    }

    #[test]
    fn golden_ambiguous_scope_unresolved() {
        assert_golden("ambiguous");
    }

    #[test]
    fn every_intent_produces_plan_without_llm() {
        for intent in Intent::ALL {
            let hints = PlanHints {
                intent_override: Some(*intent),
                anchors: if needs_anchors(*intent) {
                    vec!["ExampleSymbol".into()]
                } else {
                    vec![]
                },
                budget_tokens: Some(4000),
                ..Default::default()
            };
            let outcome = plan_query("fixture question about ExampleSymbol", &hints).unwrap();
            match outcome {
                PlanOutcome::Ok(plan) => {
                    assert_eq!(plan.intent, *intent);
                    assert!(!plan.steps.is_empty());
                    assert!(plan
                        .steps
                        .iter()
                        .any(|s| matches!(s.op, Operator::BudgetPack)));
                }
                PlanOutcome::ScopeUnresolved(_) => {
                    panic!("intent {intent} should produce a plan with anchors")
                }
            }
        }
    }

    /// `UPDATE_GOLDENS=1 cargo test -p prism-plan write_plan_goldens -- --ignored`
    #[test]
    #[ignore]
    fn write_plan_goldens() {
        if std::env::var("UPDATE_GOLDENS").ok().as_deref() != Some("1") {
            return;
        }
        for name in [
            "repo_qa",
            "debug",
            "impact",
            "refactor",
            "architecture",
            "review",
            "generate",
            "ambiguous",
        ] {
            let dir = fixture_dir(name);
            let (question, hints) = load_hints(&dir);
            let outcome = plan_query(&question, &hints).unwrap();
            let json = serde_json::to_string_pretty(&outcome).unwrap() + "\n";
            fs::write(dir.join("expected.json"), json).unwrap();
        }
    }
}
