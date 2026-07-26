//! Intent recipe catalog v1 (ADD §17.2 + §18.1 drop order).

use crate::intent::Intent;
use crate::operators::Operator;
use crate::plan::{Plan, PlanStep};
use serde_json::json;

/// Default token budget when caller does not specify one.
pub const DEFAULT_BUDGET_TOKENS: u32 = 4_000;

/// Declarative recipe: seeds → expand operators → must-include → drop priorities.
#[derive(Debug, Clone)]
pub struct IntentRecipe {
    pub intent: Intent,
    pub seed_description: &'static str,
    pub expand_description: &'static str,
    pub must_include: &'static [&'static str],
    pub drop_order: &'static [&'static str],
    pub notes: &'static [&'static str],
}

/// Catalog lookup.
pub fn recipe_for(intent: Intent) -> IntentRecipe {
    match intent {
        Intent::RepoQa => IntentRecipe {
            intent,
            seed_description: "named symbols / paths / doc headings in the question",
            expand_description: "def + 1-hop callee/caller signatures + doc prose when present",
            must_include: &["primary_symbol_definition", "primary_symbol_signature"],
            drop_order: &[
                "embedding_fallback_seeds",
                "depth_3_plus_neighbors",
                "neighbor_bodies",
                "secondary_exemplars",
                "architecture_prose",
                "product_thesis",
                "usage_surface",
            ],
            notes: &[
                "Prefer T1 ResolveSymbol + Expand; no LLM in planner",
                "P12: unfilled roles become gaps[] — never role_template placeholders",
                "Doc/Section nodes fill architecture_prose / product_thesis when indexed",
            ],
        },
        Intent::Debug => IntentRecipe {
            intent,
            seed_description: "stack frames + error text",
            expand_description: "backward interproc slice + recent diff + callee signatures",
            must_include: &[
                "error_or_stack_verbatim",
                "primary_frame_body",
                "criterion_slice",
            ],
            drop_order: &[
                "embedding_fallback_seeds",
                "depth_3_plus_impact",
                "neighbor_bodies",
                "secondary_exemplars",
                "architecture_prose",
            ],
            notes: &[
                "Slice: backward, max_depth=2, memoized",
                "UpgradePrecision: mandatory, critical_path_only, ≤200ms",
                "never drop error/stack or criterion slice under budget pressure",
            ],
        },
        Intent::Impact => IntentRecipe {
            intent,
            seed_description: "changed symbols or named target",
            expand_description: "forward IMPACTS depth 1–3 (heuristic T1; optional T2 upgrade)",
            must_include: &["seed_symbols", "impact_cone_depth_1"],
            drop_order: &[
                "embedding_fallback_seeds",
                "depth_3_plus_impact",
                "neighbor_bodies",
                "secondary_exemplars",
                "architecture_prose",
            ],
            notes: &[
                "Impact edges are heuristic at T1 unless UpgradePrecision confirms",
                "UpgradePrecision runs only when ambiguity index require_t2",
            ],
        },
        Intent::Refactor => IntentRecipe {
            intent,
            seed_description: "target symbol",
            expand_description: "all REFERENCES (T2+ preferred)",
            must_include: &["target_symbol_definition", "reference_list"],
            drop_order: &[
                "embedding_fallback_seeds",
                "depth_3_plus_neighbors",
                "neighbor_bodies",
                "secondary_exemplars",
                "architecture_prose",
            ],
            notes: &[
                "UpgradePrecision mandatory before reference expand",
                "PRECISION_REQUIRED for safe-rename product claims without overlay (Stage C)",
            ],
        },
        Intent::Generate => IntentRecipe {
            intent,
            seed_description: "target file locus",
            expand_description: "types + one exemplar neighborhood",
            must_include: &["insertion_neighborhood", "type_signatures"],
            drop_order: &[
                "embedding_fallback_seeds",
                "extra_exemplars",
                "neighbor_bodies",
                "architecture_prose",
            ],
            notes: &["At most one exemplar in v1"],
        },
        Intent::Review => IntentRecipe {
            intent,
            seed_description: "PR / worktree diff",
            expand_description: "impact cone + related tests",
            must_include: &["diff_hunks", "impact_cone_depth_1"],
            drop_order: &[
                "embedding_fallback_seeds",
                "depth_3_plus_impact",
                "neighbor_bodies",
                "secondary_exemplars",
                "architecture_prose",
            ],
            notes: &["DiffIntersect seeds from changed_paths / dirty worktree"],
        },
        Intent::Architecture => IntentRecipe {
            intent,
            seed_description: "communities (no symbol required); optional README/docs anchors",
            expand_description: "hub nodes + boundaries + doc product thesis",
            must_include: &["community_map", "hub_nodes", "product_thesis"],
            drop_order: &[
                "embedding_fallback_seeds",
                "deep_module_bodies",
                "secondary_exemplars",
                "architecture_prose",
                "usage_surface",
            ],
            notes: &[
                "Uses CommunityOf / repo_map; anchors optional",
                "P12: product_thesis filled from Doc nodes when present; else gaps[]",
                "Hub ranking excludes unresolved/builtin noise (ACC-4)",
            ],
        },
    }
}

impl IntentRecipe {
    /// Materialize a plan IR for this recipe (deterministic operator DAG).
    pub fn build_plan(&self, question: &str, anchors: &[String], budget_tokens: u32) -> Plan {
        let mut plan = Plan::new(self.intent, question, budget_tokens);
        plan.must_include = self.must_include.iter().map(|s| (*s).to_string()).collect();
        plan.drop_order = self.drop_order.iter().map(|s| (*s).to_string()).collect();
        plan.notes = self.notes.iter().map(|s| (*s).to_string()).collect();
        plan.notes.push(format!("seeds: {}", self.seed_description));
        plan.notes
            .push(format!("expand: {}", self.expand_description));

        let steps = match self.intent {
            Intent::RepoQa => repo_qa_steps(anchors),
            Intent::Debug => debug_steps(anchors),
            Intent::Impact => impact_steps(anchors),
            Intent::Refactor => refactor_steps(anchors),
            Intent::Generate => generate_steps(anchors),
            Intent::Review => review_steps(anchors),
            Intent::Architecture => architecture_steps(anchors),
        };
        plan.steps = steps;

        // Gaps for non-executable placeholders
        for step in &plan.steps {
            if !step.executable {
                plan.gaps.push(format!(
                    "operator `{}` not executable in v1 ({})",
                    step.op, step.why
                ));
            }
        }
        if matches!(self.intent, Intent::Refactor) {
            plan.gaps.push(
                "safe-rename claims need T2 overlay when available; heuristic refs stay labeled"
                    .into(),
            );
        }

        plan
    }
}

fn step(
    id: &str,
    op: Operator,
    inputs: serde_json::Value,
    depends_on: &[&str],
    why: &str,
) -> PlanStep {
    PlanStep {
        id: id.into(),
        op,
        inputs,
        depends_on: depends_on.iter().map(|s| (*s).to_string()).collect(),
        est_cost_ms: op.est_cost_ms(),
        executable: op.executable_in_v1(),
        why: why.into(),
    }
}

fn repo_qa_steps(anchors: &[String]) -> Vec<PlanStep> {
    vec![
        step(
            "s1",
            Operator::ResolveSymbol,
            json!({ "anchors": anchors, "limit": 20 }),
            &[],
            "resolve named symbols from question",
        ),
        step(
            "s2",
            Operator::Expand,
            json!({ "edge_kinds": ["CALLS", "IMPORTS", "DEFINES"], "depth": 1, "mode": "signatures" }),
            &["s1"],
            "1-hop callee/caller signatures",
        ),
        step(
            "s3",
            Operator::BudgetPack,
            json!({ "recipe": "repo_qa" }),
            &["s2"],
            "assemble Evidence Pack under budget (Stage B)",
        ),
    ]
}

fn debug_steps(anchors: &[String]) -> Vec<PlanStep> {
    // ADD §19.4: Resolve → UpgradePrecision → Slice → DiffIntersect → Expand → BudgetPack
    vec![
        step(
            "s1",
            Operator::ResolveSymbol,
            json!({ "anchors": anchors, "limit": 20 }),
            &[],
            "resolve stack frames / error loci",
        ),
        step(
            "s2",
            Operator::UpgradePrecision,
            json!({
                "nodes_from": "s1",
                "tier": "T2",
                "critical_path_only": true,
                "policy": "mandatory",
                "max_upgrades": 32,
                "max_latency_ms": 200
            }),
            &["s1"],
            "refine ambiguous CALLS on critical path (≤200ms / 32 edges)",
        ),
        step(
            "s3",
            Operator::Slice,
            json!({
                "direction": "backward",
                "max_depth": 2,
                "max_functions": 16,
                "max_spans": 40,
                "residual_expand": true,
                "criterion_from": "s1"
            }),
            &["s2"],
            "inter-procedural backward slice (depth-capped; memoized)",
        ),
        step(
            "s4",
            Operator::DiffIntersect,
            json!({ "since": "main", "worktree": true }),
            &["s3"],
            "intersect recent dirty/diff with slice candidates",
        ),
        step(
            "s5",
            Operator::Expand,
            json!({ "edge_kinds": ["CALLS"], "depth": 1, "mode": "signatures" }),
            &["s4"],
            "callee signatures only",
        ),
        step(
            "s6",
            Operator::BudgetPack,
            json!({ "recipe": "debug" }),
            &["s5"],
            "assemble debug Evidence Pack (Stage B)",
        ),
    ]
}

fn impact_steps(anchors: &[String]) -> Vec<PlanStep> {
    vec![
        step(
            "s1",
            Operator::ResolveSymbol,
            json!({ "anchors": anchors, "limit": 20 }),
            &[],
            "resolve changed / named symbols",
        ),
        step(
            "s2",
            Operator::UpgradePrecision,
            json!({
                "nodes_from": "s1",
                "tier": "T2",
                "critical_path_only": true,
                "policy": "optional_on_ambiguity",
                "max_upgrades": 32,
                "max_latency_ms": 200
            }),
            &["s1"],
            "optional T2 refine when ambiguity index require_t2",
        ),
        step(
            "s3",
            Operator::Impact,
            json!({ "depth": 2, "limit": 100 }),
            &["s2"],
            "forward impact cone (prefer precise edges when upgraded)",
        ),
        step(
            "s4",
            Operator::BudgetPack,
            json!({ "recipe": "impact" }),
            &["s3"],
            "assemble impact Evidence Pack",
        ),
    ]
}

fn refactor_steps(anchors: &[String]) -> Vec<PlanStep> {
    vec![
        step(
            "s1",
            Operator::ResolveSymbol,
            json!({ "anchors": anchors, "limit": 5 }),
            &[],
            "resolve refactor target",
        ),
        step(
            "s2",
            Operator::UpgradePrecision,
            json!({
                "nodes_from": "s1",
                "tier": "T2",
                "policy": "mandatory",
                "max_upgrades": 32,
                "max_latency_ms": 200
            }),
            &["s1"],
            "mandatory T2 refine before reference expand (≤200ms)",
        ),
        step(
            "s3",
            Operator::Expand,
            json!({ "edge_kinds": ["REFERENCES", "CALLS"], "depth": 2, "mode": "all_refs" }),
            &["s2"],
            "references / callers — prefer precise confidence",
        ),
        step(
            "s4",
            Operator::BudgetPack,
            json!({ "recipe": "refactor" }),
            &["s3"],
            "assemble refactor Evidence Pack",
        ),
    ]
}

fn generate_steps(anchors: &[String]) -> Vec<PlanStep> {
    vec![
        step(
            "s1",
            Operator::ResolveSymbol,
            json!({ "anchors": anchors, "limit": 20 }),
            &[],
            "resolve insertion locus / nearby types",
        ),
        step(
            "s2",
            Operator::Expand,
            json!({ "edge_kinds": ["IMPORTS", "DEFINES", "CONTAINS"], "depth": 1, "mode": "types_and_one_exemplar" }),
            &["s1"],
            "type deps + single exemplar",
        ),
        step(
            "s3",
            Operator::BudgetPack,
            json!({ "recipe": "generate" }),
            &["s2"],
            "assemble generate Evidence Pack (Stage B)",
        ),
    ]
}

fn review_steps(anchors: &[String]) -> Vec<PlanStep> {
    vec![
        step(
            "s1",
            Operator::DiffIntersect,
            json!({ "anchors": anchors, "worktree": true }),
            &[],
            "seed from PR/worktree diff paths",
        ),
        step(
            "s2",
            Operator::Impact,
            json!({ "depth": 2, "limit": 100 }),
            &["s1"],
            "impact cone of changed symbols",
        ),
        step(
            "s3",
            Operator::FindTests,
            json!({ "symbols_from": "s2" }),
            &["s2"],
            "related tests (heuristic path/name match in v1)",
        ),
        step(
            "s4",
            Operator::BudgetPack,
            json!({ "recipe": "review" }),
            &["s3"],
            "assemble review Evidence Pack (Stage B)",
        ),
    ]
}

fn architecture_steps(anchors: &[String]) -> Vec<PlanStep> {
    let mut steps = vec![step(
        "s1",
        Operator::CommunityOf,
        json!({ "hub_limit": 15 }),
        &[],
        "path-prefix communities + hubs (repo_map)",
    )];
    if !anchors.is_empty() {
        steps.push(step(
            "s2",
            Operator::ResolveSymbol,
            json!({ "anchors": anchors, "limit": 10 }),
            &["s1"],
            "optional: place named symbols into communities",
        ));
        steps.push(step(
            "s3",
            Operator::BudgetPack,
            json!({ "recipe": "architecture" }),
            &["s2"],
            "assemble architecture Evidence Pack (Stage B)",
        ));
    } else {
        steps.push(step(
            "s2",
            Operator::BudgetPack,
            json!({ "recipe": "architecture" }),
            &["s1"],
            "assemble architecture Evidence Pack (Stage B)",
        ));
    }
    steps
}
