//! Seed grounding against the KG (P12 ACC-3).
//!
//! Wrong or empty seeds must not produce a confident pack about the wrong
//! symbol. When no planner anchor grounds above [`MIN_GROUND_SCORE`], refuse
//! with ranked lexical candidates so the agent can recover on a second call.

use prism_plan::{Intent, Plan, ScopeUnresolved};
use prism_store::{tokenize_seed_terms, SeedCandidate, SqliteKgStore, MIN_GROUND_SCORE};

/// Result of grounding planner anchors in the live KG.
#[derive(Debug)]
pub enum GroundingOutcome {
    /// At least one anchor grounded; proceed to selection.
    Ok { notes: Vec<String> },
    /// Refuse — include ranked candidates for repair.
    Unresolved(ScopeUnresolved),
}

/// Ground `plan` anchors against `kg`. Architecture with no anchors is allowed.
pub fn ground_plan_seeds(kg: &SqliteKgStore, plan: &Plan) -> anyhow::Result<GroundingOutcome> {
    let anchors = anchors_from_plan(plan);
    let intent = plan.intent;

    if anchors.is_empty() {
        if matches!(intent, Intent::Architecture) {
            return Ok(GroundingOutcome::Ok {
                notes: vec!["architecture: no symbol anchors required".into()],
            });
        }
        // Planner should have refused already; still enrich with lexical candidates.
        let candidates = ranked_from_question(kg, &plan.question)?;
        return Ok(GroundingOutcome::Unresolved(refuse(
            intent,
            &plan.question,
            "No anchors to ground against the index",
            candidates,
        )));
    }

    let mut grounded = 0u32;
    let mut best_weak: Option<SeedCandidate> = None;
    let mut notes = Vec::new();
    for a in &anchors {
        match kg.score_anchor(a)? {
            Some(c) if c.score >= MIN_GROUND_SCORE => {
                grounded += 1;
                notes.push(format!(
                    "seed grounded: `{}` → {} ({} score={})",
                    a, c.node_id, c.match_kind, c.score
                ));
            }
            Some(c)
                if best_weak
                    .as_ref()
                    .map(|w| c.score > w.score)
                    .unwrap_or(true) =>
            {
                best_weak = Some(c);
            }
            Some(_) | None => {}
        }
    }

    if grounded > 0 {
        return Ok(GroundingOutcome::Ok { notes });
    }

    // Architecture orientation packs should still emit docs + communities even
    // when backtick phrases in the question look like anchors but don't ground
    // (e.g. `prism setup .`). RepoQa / debug keep hard refusal (ACC-3).
    if matches!(intent, Intent::Architecture) {
        notes.push(
            "architecture: planner anchors did not ground; continuing with docs/communities".into(),
        );
        return Ok(GroundingOutcome::Ok { notes });
    }

    // No strong ground — lexical expand from question + failed anchors.
    let mut terms = tokenize_seed_terms(&plan.question);
    for a in &anchors {
        terms.push(a.clone());
    }
    terms.sort();
    terms.dedup();
    let mut candidates = kg.lexical_seed_search(&terms, 8)?;
    if let Some(w) = best_weak {
        if !candidates.iter().any(|c| c.node_id == w.node_id) {
            candidates.insert(0, w);
        }
    }
    Ok(GroundingOutcome::Unresolved(refuse(
        intent,
        &plan.question,
        format!(
            "None of the planner anchors grounded in the index (need score ≥ {MIN_GROUND_SCORE}); pick a ranked candidate and retry"
        ),
        candidates,
    )))
}

fn ranked_from_question(kg: &SqliteKgStore, question: &str) -> anyhow::Result<Vec<SeedCandidate>> {
    let terms = tokenize_seed_terms(question);
    kg.lexical_seed_search(&terms, 8)
}

fn refuse(
    intent: Intent,
    question: &str,
    reason: impl Into<String>,
    candidates: Vec<SeedCandidate>,
) -> ScopeUnresolved {
    let ranked: Vec<String> = candidates
        .iter()
        .map(|c| {
            format!(
                "{} (score={} {}{})",
                c.anchor,
                c.score,
                c.match_kind,
                c.file_path
                    .as_ref()
                    .map(|p| format!(" @ {p}"))
                    .unwrap_or_default()
            )
        })
        .collect();
    let mut ask_for = vec![
        "symbol name or qualified path".into(),
        "file path".into(),
        "stack frame / error text".into(),
    ];
    if !ranked.is_empty() {
        ask_for.insert(
            0,
            format!(
                "one of the ranked candidates (e.g. `{}`)",
                candidates[0].anchor
            ),
        );
    }
    ScopeUnresolved {
        code: "SCOPE_UNRESOLVED".into(),
        reason: reason.into(),
        ask_for,
        intent: Some(intent),
        question: question.to_string(),
        candidates: ranked,
    }
}

fn anchors_from_plan(plan: &Plan) -> Vec<String> {
    let mut out = Vec::new();
    for step in &plan.steps {
        if let Some(arr) = step.inputs.get("anchors").and_then(|v| v.as_array()) {
            for v in arr {
                if let Some(s) = v.as_str() {
                    if !out.iter().any(|x| x == s) {
                        out.push(s.to_string());
                    }
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use prism_ir::{symbol_node_id, Confidence, FactBundle, FactNode, NodeKind, Tier};
    use prism_plan::{plan_query, PlanHints, PlanOutcome};
    use prism_store::KgStore;
    use tempfile::tempdir;

    fn seed_helper(kg: &mut SqliteKgStore) {
        let mut b = FactBundle::new("a.py", "python", "test");
        b.nodes.push(FactNode {
            id: symbol_node_id("a.py", "function", "Helper", 1),
            kind: NodeKind::Symbol,
            name: Some("Helper".into()),
            file_path: Some("a.py".into()),
            span: None,
            language: Some("python".into()),
            analyzer: "test".into(),
            tier: Tier::T1,
            confidence: Confidence::Extracted,
            attrs: Default::default(),
        });
        kg.begin_replace_file_subgraph("a.py").unwrap();
        kg.insert_facts("a.py", &b).unwrap();
        kg.commit_replace_file_subgraph("a.py").unwrap();
    }

    #[test]
    fn wrong_seed_refuses_with_ranked_candidates() {
        let dir = tempdir().unwrap();
        let mut kg = SqliteKgStore::open(dir.path().join("g.sqlite")).unwrap();
        seed_helper(&mut kg);

        let hints = PlanHints {
            intent_override: Some(Intent::RepoQa),
            anchors: vec!["elper".into()], // substring of Helper — below MIN_GROUND_SCORE
            budget_tokens: Some(2000),
            ..Default::default()
        };
        let plan = match plan_query("What does elper do in a.py?", &hints).unwrap() {
            PlanOutcome::Ok(p) => p,
            other => panic!("expected plan, got {other:?}"),
        };
        match ground_plan_seeds(&kg, &plan).unwrap() {
            GroundingOutcome::Unresolved(u) => {
                assert_eq!(u.code, "SCOPE_UNRESOLVED");
                assert!(
                    !u.candidates.is_empty(),
                    "expected ranked candidates, got empty"
                );
                assert!(
                    u.candidates.iter().any(|c| c.contains("Helper")),
                    "expected Helper in candidates: {:?}",
                    u.candidates
                );
            }
            GroundingOutcome::Ok { .. } => panic!("weak seed must refuse"),
        }
    }

    #[test]
    fn exact_seed_grounds() {
        let dir = tempdir().unwrap();
        let mut kg = SqliteKgStore::open(dir.path().join("g.sqlite")).unwrap();
        seed_helper(&mut kg);
        let hints = PlanHints {
            intent_override: Some(Intent::RepoQa),
            anchors: vec!["Helper".into()],
            budget_tokens: Some(2000),
            ..Default::default()
        };
        let plan = match plan_query("What does `Helper` do?", &hints).unwrap() {
            PlanOutcome::Ok(p) => p,
            other => panic!("expected plan, got {other:?}"),
        };
        match ground_plan_seeds(&kg, &plan).unwrap() {
            GroundingOutcome::Ok { notes } => {
                assert!(notes.iter().any(|n| n.contains("Helper")));
            }
            GroundingOutcome::Unresolved(u) => panic!("expected ground ok: {u:?}"),
        }
    }
}
