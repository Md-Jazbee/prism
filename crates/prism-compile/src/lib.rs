//! Evidence Pack compiler (P2 Stage B).
//!
//! Plan → candidate fragments → budget pack (must-include invariant) → EXPLAIN.
//! See `docs/architecture/EVIDENCE-PACK.md`.

mod budget;
mod explain;
mod fragment;
mod gap;
mod pack;
mod path_class;
mod seed;
mod select;

pub use budget::{pack_under_budget, pack_under_budget_with_gaps, BudgetExceeded};
pub use explain::{DropRecord, ExplainFragment, ExplainReport};
pub use fragment::{
    estimate_tokens, CandidateFragment, EvidenceFragment, FragmentKind, PackLayer, Provenance,
};
pub use gap::{EvidenceGap, WhyAbsent};
pub use pack::{Citation, CompileOutcome, EvidencePack, PackHierarchy, PackMeta};
pub use path_class::{classify_path, is_noise_path, path_allowed, PathClass};
pub use seed::{ground_plan_seeds, GroundingOutcome};
pub use select::{select_candidates, select_from_kg, CompileOptions, SelectionOutcome};

use anyhow::{Context, Result};
use prism_plan::{plan_query, Plan, PlanHints, PlanOutcome, ScopeUnresolved};
use prism_store::SqliteKgStore;
use std::path::Path;

/// Compile an Evidence Pack from an existing plan + pre-built candidates (fixtures / tests).
pub fn compile_from_candidates(plan: &Plan, candidates: Vec<CandidateFragment>) -> CompileOutcome {
    match pack_under_budget(plan, candidates) {
        Ok(pack) => CompileOutcome::Ok(Box::new(pack)),
        Err(e) => CompileOutcome::BudgetExceeded(e),
    }
}

/// End-to-end: plan question → ground seeds → select from KG → budget pack.
///
/// Returns `ScopeUnresolved` when the planner refuses or ACC-3 seed grounding
/// fails (with ranked candidates); `BudgetExceeded` when must-include cannot fit.
pub fn compile_context(
    workspace: &Path,
    question: &str,
    hints: &PlanHints,
) -> Result<CompileOutcome> {
    match plan_query(question, hints)? {
        PlanOutcome::ScopeUnresolved(u) => Ok(CompileOutcome::ScopeUnresolved(u)),
        PlanOutcome::Ok(mut plan) => {
            let kg_path = workspace.join(".prism/graph.sqlite");
            if !kg_path.exists() {
                anyhow::bail!(
                    "no index at {} — run `prism index` first",
                    workspace.join(".prism").display()
                );
            }
            let kg = SqliteKgStore::open(&kg_path)
                .with_context(|| format!("open kg {}", kg_path.display()))?;
            match ground_plan_seeds(&kg, &plan)? {
                GroundingOutcome::Unresolved(u) => {
                    return Ok(CompileOutcome::ScopeUnresolved(u));
                }
                GroundingOutcome::Ok { notes } => {
                    for n in notes {
                        if !plan.notes.iter().any(|x| x == &n) {
                            plan.notes.push(n);
                        }
                    }
                }
            }
            let opts = CompileOptions {
                workspace: Some(workspace.to_path_buf()),
            };
            let selection = select_from_kg(&kg, &plan, &opts)?;
            match pack_under_budget_with_gaps(&plan, selection.candidates, selection.gaps) {
                Ok(pack) => {
                    // ACC-2: live packs must never keep synthetic placeholders.
                    pack.assert_no_placeholder_fragments()
                        .map_err(anyhow::Error::msg)?;
                    Ok(CompileOutcome::Ok(Box::new(pack)))
                }
                Err(e) => Ok(CompileOutcome::BudgetExceeded(e)),
            }
        }
    }
}

/// Compile using only the plan (synthetic candidates from recipe roles) — no KG.
/// Useful for offline EXPLAIN / budget invariant demos.
///
/// Note (P12): synthetic packs may contain placeholder provenance; live
/// [`compile_context`] strips those and emits [`EvidenceGap`]s instead.
pub fn compile_synthetic(plan: &Plan) -> CompileOutcome {
    let candidates = select_candidates(plan);
    compile_from_candidates(plan, candidates)
}

/// Re-export planner refuse type for callers.
pub type PlanRefuse = ScopeUnresolved;

#[cfg(test)]
mod tests {
    use super::*;
    use prism_plan::{plan_query, Intent, Operator, PlanHints, PlanOutcome};
    use std::fs;
    use std::path::PathBuf;

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../../fixtures/packs/{name}"))
    }

    #[test]
    fn placeholder_invariant_rejects_synthetic_only() {
        use crate::explain::ExplainReport;
        use crate::fragment::{EvidenceFragment, FragmentKind, PackLayer, Provenance};
        use crate::pack::{Citation, EvidencePack, PackHierarchy, PackMeta};
        let bad = EvidencePack {
            meta: PackMeta {
                schema_version: EvidencePack::schema_version().into(),
                plan_id: "plan:test".into(),
                intent: Intent::RepoQa,
                question: "q".into(),
                budget_tokens: 100,
                tokens_used: 1,
                repo: None,
                snapshot: None,
            },
            hierarchy: PackHierarchy::default(),
            fragments: vec![EvidenceFragment {
                id: "frag:bad".into(),
                kind: FragmentKind::Slice,
                layer: PackLayer::Core,
                text: "// must-include `x`".into(),
                token_estimate: 1,
                must_include: true,
                drop_priority: 0,
                roles: vec!["x".into()],
                why_included: "x".into(),
                confidence: "extracted".into(),
                provenance: Provenance::synthetic("x"),
            }],
            citations: vec![Citation {
                id: "C1".into(),
                fragment_id: "frag:bad".into(),
                node_ids: vec!["synthetic:x".into()],
            }],
            gaps: vec![],
            drops: vec![],
            explain: ExplainReport {
                plan_id: "plan:test".into(),
                budget_tokens: 100,
                tokens_used: 1,
                must_include_ok: true,
                fragments: vec![],
                drops: vec![],
                notes: vec![],
            },
        };
        assert!(bad.assert_no_placeholder_fragments().is_err());
    }

    #[test]
    fn must_include_never_budget_evicted() {
        let hints = PlanHints {
            intent_override: Some(Intent::RepoQa),
            anchors: vec!["Helper".into()],
            budget_tokens: Some(80),
            ..Default::default()
        };
        let plan = match plan_query("explain `Helper`", &hints).unwrap() {
            PlanOutcome::Ok(p) => p,
            other => panic!("expected plan, got {other:?}"),
        };
        // Oversized optional + small must-includes
        let mut cands = select_candidates(&plan);
        cands.push(CandidateFragment {
            id: "frag:noise".into(),
            kind: FragmentKind::Signature,
            layer: PackLayer::Nbr,
            text: "x".repeat(400),
            token_estimate: 100,
            provenance: Provenance::synthetic("noise"),
            confidence: "heuristic".into(),
            why_included: "neighbor_signature".into(),
            drop_priority: 90,
            roles: vec!["neighbor_bodies".into()],
            must_include: false,
        });
        let outcome = compile_from_candidates(&plan, cands);
        match outcome {
            CompileOutcome::Ok(pack) => {
                assert!(pack.explain.must_include_ok);
                for role in &plan.must_include {
                    assert!(
                        pack.fragments.iter().any(|f| f.roles.contains(role)),
                        "must-include role `{role}` missing from pack"
                    );
                }
                assert!(
                    pack.drops.iter().any(|d| d.fragment_id == "frag:noise"),
                    "optional noise should be dropped under tight budget"
                );
            }
            other => panic!("expected Ok pack, got {other:?}"),
        }
    }

    #[test]
    fn debug_error_and_slice_never_dropped_under_budget() {
        let hints = PlanHints {
            intent_override: Some(Intent::Debug),
            anchors: vec![
                "pkg/chain.py:2 in leaf".into(),
                "AttributeError: boom".into(),
            ],
            stack_frames: vec!["pkg/chain.py:2 in leaf".into()],
            error_text: Some("AttributeError: boom".into()),
            budget_tokens: Some(120),
            ..Default::default()
        };
        let plan = match plan_query("why crash?", &hints).unwrap() {
            PlanOutcome::Ok(p) => p,
            other => panic!("{other:?}"),
        };
        assert!(
            plan.steps
                .iter()
                .any(|s| matches!(s.op, Operator::Slice) && s.executable),
            "Slice must be executable"
        );
        let mut cands = select_candidates(&plan);
        cands.push(CandidateFragment {
            id: "frag:error:manual".into(),
            kind: FragmentKind::ErrorVerbatim,
            layer: PackLayer::Core,
            text: "AttributeError: boom".into(),
            token_estimate: 20,
            provenance: Provenance::synthetic("error"),
            confidence: "extracted".into(),
            why_included: "error_or_stack_verbatim".into(),
            drop_priority: 99, // would drop if not protected
            roles: vec!["error_or_stack_verbatim".into()],
            must_include: false,
        });
        cands.push(CandidateFragment {
            id: "frag:slice:manual".into(),
            kind: FragmentKind::Slice,
            layer: PackLayer::Core,
            text: "// criterion slice leaf\n".into(),
            token_estimate: 30,
            provenance: Provenance::synthetic("slice"),
            confidence: "extracted".into(),
            why_included: "criterion_slice".into(),
            drop_priority: 99,
            roles: vec!["criterion_slice".into(), "primary_frame_body".into()],
            must_include: false,
        });
        cands.push(CandidateFragment {
            id: "frag:noise:huge".into(),
            kind: FragmentKind::Signature,
            layer: PackLayer::Nbr,
            text: "n".repeat(2000),
            token_estimate: 500,
            provenance: Provenance::synthetic("noise"),
            confidence: "heuristic".into(),
            why_included: "neighbor_bodies".into(),
            drop_priority: 1,
            roles: vec!["neighbor_bodies".into()],
            must_include: false,
        });
        match compile_from_candidates(&plan, cands) {
            CompileOutcome::Ok(pack) => {
                assert!(
                    pack.fragments.iter().any(|f| f.id == "frag:error:manual"),
                    "error verbatim must survive: {:?}",
                    pack.drops
                );
                assert!(
                    pack.fragments.iter().any(|f| f.id == "frag:slice:manual"),
                    "criterion slice must survive: {:?}",
                    pack.drops
                );
                assert!(
                    pack.drops
                        .iter()
                        .any(|d| d.fragment_id == "frag:noise:huge"),
                    "noise should drop under pressure"
                );
            }
            other => panic!("expected Ok pack, got {other:?}"),
        }
    }

    #[test]
    fn whitespace_only_change_keeps_must_include_stable() {
        // See docs/architecture/PACK-STABILITY.md
        let hints = PlanHints {
            intent_override: Some(Intent::RepoQa),
            anchors: vec!["Helper".into()],
            budget_tokens: Some(400),
            ..Default::default()
        };
        let plan = match plan_query("explain `Helper`", &hints).unwrap() {
            PlanOutcome::Ok(p) => p,
            other => panic!("{other:?}"),
        };
        let mut cands_a = select_candidates(&plan);
        cands_a.push(CandidateFragment {
            id: "frag:opt:ws".into(),
            kind: FragmentKind::Signature,
            layer: PackLayer::Nbr,
            text: "neighbor body".into(),
            token_estimate: 20,
            provenance: Provenance::synthetic("nbr"),
            confidence: "heuristic".into(),
            why_included: "neighbor_bodies".into(),
            drop_priority: 40,
            roles: vec!["neighbor_bodies".into()],
            must_include: false,
        });
        let mut cands_b = cands_a.clone();
        // Whitespace-only mutation on optional fragment text
        if let Some(c) = cands_b.iter_mut().find(|c| c.id == "frag:opt:ws") {
            c.text = "neighbor   body\n\n".into();
            c.token_estimate = 22;
        }
        let pack_a = match compile_from_candidates(&plan, cands_a) {
            CompileOutcome::Ok(p) => p,
            other => panic!("{other:?}"),
        };
        let pack_b = match compile_from_candidates(&plan, cands_b) {
            CompileOutcome::Ok(p) => p,
            other => panic!("{other:?}"),
        };
        let must_a: Vec<_> = pack_a
            .fragments
            .iter()
            .filter(|f| f.must_include)
            .map(|f| (f.id.clone(), f.roles.clone()))
            .collect();
        let must_b: Vec<_> = pack_b
            .fragments
            .iter()
            .filter(|f| f.must_include)
            .map(|f| (f.id.clone(), f.roles.clone()))
            .collect();
        assert_eq!(
            must_a, must_b,
            "must-include ids/roles must be citation-stable under whitespace-only optional changes"
        );
    }

    #[test]
    fn budget_exceeded_when_must_include_cannot_fit() {
        let hints = PlanHints {
            intent_override: Some(Intent::RepoQa),
            anchors: vec!["Helper".into()],
            budget_tokens: Some(5),
            ..Default::default()
        };
        let plan = match plan_query("explain `Helper`", &hints).unwrap() {
            PlanOutcome::Ok(p) => p,
            other => panic!("{other:?}"),
        };
        let mut cands = select_candidates(&plan);
        for c in &mut cands {
            if c.must_include {
                c.token_estimate = 50;
                c.text = "m".repeat(200);
            }
        }
        match compile_from_candidates(&plan, cands) {
            CompileOutcome::BudgetExceeded(e) => {
                assert_eq!(e.code, "BUDGET_EXCEEDED");
                assert!(e.must_include_tokens > e.budget_tokens);
            }
            other => panic!("expected BUDGET_EXCEEDED, got {other:?}"),
        }
    }

    #[test]
    fn explain_round_trips_with_pack() {
        let hints = PlanHints {
            intent_override: Some(Intent::Impact),
            anchors: vec!["WalkBuilder".into()],
            budget_tokens: Some(4000),
            ..Default::default()
        };
        let plan = match plan_query("impact of `WalkBuilder`", &hints).unwrap() {
            PlanOutcome::Ok(p) => p,
            other => panic!("{other:?}"),
        };
        let pack = match compile_synthetic(&plan) {
            CompileOutcome::Ok(p) => p,
            other => panic!("{other:?}"),
        };
        assert!(!pack.explain.fragments.is_empty());
        let v = serde_json::to_value(&pack).unwrap();
        let back: EvidencePack = serde_json::from_value(v).unwrap();
        assert_eq!(back.meta.plan_id, pack.meta.plan_id);
        assert!(back.explain.must_include_ok);
    }

    #[test]
    fn golden_repo_qa_ok() {
        assert_pack_golden("repo_qa_ok");
    }

    #[test]
    fn golden_budget_drop() {
        assert_pack_golden("budget_drop");
    }

    #[test]
    fn golden_budget_exceeded() {
        assert_pack_golden("budget_exceeded");
    }

    #[test]
    fn golden_explain_roundtrip() {
        assert_pack_golden("explain_roundtrip");
    }

    fn assert_pack_golden(name: &str) {
        let dir = fixture(name);
        let input: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(dir.join("input.json")).unwrap()).unwrap();
        let question = input["question"].as_str().unwrap();
        let mut hints = PlanHints::default();
        if let Some(b) = input.get("budget_tokens").and_then(|v| v.as_u64()) {
            hints.budget_tokens = Some(b as u32);
        }
        if let Some(i) = input.get("intent").and_then(|v| v.as_str()) {
            hints.intent_override = Some(i.parse().unwrap());
        }
        if let Some(arr) = input.get("anchors").and_then(|v| v.as_array()) {
            hints.anchors = arr
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect();
        }
        let plan = match plan_query(question, &hints).unwrap() {
            PlanOutcome::Ok(p) => p,
            PlanOutcome::ScopeUnresolved(u) => panic!("unexpected refuse: {u:?}"),
        };

        let outcome = if let Some(cands) = input.get("candidates") {
            let candidates: Vec<CandidateFragment> = serde_json::from_value(cands.clone()).unwrap();
            compile_from_candidates(&plan, candidates)
        } else {
            compile_synthetic(&plan)
        };

        // Stable compare: normalize pack fields that are order-sensitive
        let mut actual = serde_json::to_value(&outcome).unwrap();
        normalize_outcome_json(&mut actual);

        let expected: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(dir.join("expected.json")).unwrap()).unwrap();
        assert_eq!(actual, expected, "pack golden mismatch for `{name}`");
    }

    fn normalize_outcome_json(v: &mut serde_json::Value) {
        // plan_id is deterministic from question; leave it.
        // Sort fragment ids in explain/drops if needed — pack already sorts.
        let _ = v;
    }

    /// `UPDATE_GOLDENS=1 cargo test -p prism-compile write_pack_goldens -- --ignored`
    #[test]
    #[ignore]
    fn write_pack_goldens() {
        if std::env::var("UPDATE_GOLDENS").ok().as_deref() != Some("1") {
            return;
        }
        for name in [
            "repo_qa_ok",
            "budget_drop",
            "budget_exceeded",
            "explain_roundtrip",
        ] {
            let dir = fixture(name);
            let input: serde_json::Value =
                serde_json::from_str(&fs::read_to_string(dir.join("input.json")).unwrap()).unwrap();
            let question = input["question"].as_str().unwrap();
            let mut hints = PlanHints::default();
            if let Some(b) = input.get("budget_tokens").and_then(|v| v.as_u64()) {
                hints.budget_tokens = Some(b as u32);
            }
            if let Some(i) = input.get("intent").and_then(|v| v.as_str()) {
                hints.intent_override = Some(i.parse().unwrap());
            }
            if let Some(arr) = input.get("anchors").and_then(|v| v.as_array()) {
                hints.anchors = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect();
            }
            let plan = match plan_query(question, &hints).unwrap() {
                PlanOutcome::Ok(p) => p,
                PlanOutcome::ScopeUnresolved(_) => panic!("refuse in {name}"),
            };
            let outcome = if let Some(cands) = input.get("candidates") {
                let candidates: Vec<CandidateFragment> =
                    serde_json::from_value(cands.clone()).unwrap();
                compile_from_candidates(&plan, candidates)
            } else {
                compile_synthetic(&plan)
            };
            let json = serde_json::to_string_pretty(&outcome).unwrap() + "\n";
            fs::write(dir.join("expected.json"), json).unwrap();
        }
    }
}
