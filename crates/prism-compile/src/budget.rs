//! Budget packing with hard must-include invariant (ADD §18.1).

use crate::explain::{DropRecord, ExplainFragment, ExplainReport};
use crate::fragment::{CandidateFragment, EvidenceFragment};
use crate::pack::{Citation, EvidencePack, PackHierarchy, PackMeta};
use prism_ir::PACK_SCHEMA_VERSION;
use prism_plan::Plan;
use serde::{Deserialize, Serialize};

/// Must-include cannot fit under budget — refuse soft truncation of truth.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BudgetExceeded {
    pub code: String,
    pub reason: String,
    pub budget_tokens: u32,
    pub must_include_tokens: u32,
    pub must_include_ids: Vec<String>,
    pub plan_id: String,
}

/// Pack candidates under `plan.budget_tokens`.
///
/// Invariant: every `must_include` candidate is kept, or this returns [`BudgetExceeded`].
/// Optional fragments are filled lowest `drop_priority` first; remainder recorded in `drops`.
pub fn pack_under_budget(
    plan: &Plan,
    mut candidates: Vec<CandidateFragment>,
) -> Result<EvidencePack, BudgetExceeded> {
    // Tag must-include from roles ∩ plan.must_include
    for c in &mut candidates {
        if plan.must_include.iter().any(|r| c.roles.contains(r)) {
            c.must_include = true;
            c.drop_priority = 0;
        }
    }

    let (mut must, mut optional): (Vec<_>, Vec<_>) =
        candidates.into_iter().partition(|c| c.must_include);

    // Stable order for goldens
    must.sort_by(|a, b| a.id.cmp(&b.id));
    optional.sort_by(|a, b| {
        a.drop_priority
            .cmp(&b.drop_priority)
            .then_with(|| a.id.cmp(&b.id))
    });

    let must_tokens: u32 = must.iter().map(|c| c.token_estimate).sum();
    if must_tokens > plan.budget_tokens {
        return Err(BudgetExceeded {
            code: "BUDGET_EXCEEDED".into(),
            reason: format!(
                "must-include fragments need {must_tokens} tokens but budget is {}",
                plan.budget_tokens
            ),
            budget_tokens: plan.budget_tokens,
            must_include_tokens: must_tokens,
            must_include_ids: must.iter().map(|c| c.id.clone()).collect(),
            plan_id: plan.plan_id.clone(),
        });
    }

    let mut kept: Vec<EvidenceFragment> = must.into_iter().map(Into::into).collect();
    let mut used = must_tokens;
    let mut drops = Vec::new();
    let mut explain_frags = Vec::new();

    for f in &kept {
        explain_frags.push(ExplainFragment {
            fragment_id: f.id.clone(),
            why_included: f.why_included.clone(),
            token_estimate: f.token_estimate,
            must_include: true,
            kept: true,
        });
    }

    for c in optional {
        if used + c.token_estimate <= plan.budget_tokens {
            used += c.token_estimate;
            explain_frags.push(ExplainFragment {
                fragment_id: c.id.clone(),
                why_included: c.why_included.clone(),
                token_estimate: c.token_estimate,
                must_include: false,
                kept: true,
            });
            kept.push(c.into());
        } else {
            drops.push(DropRecord {
                fragment_id: c.id.clone(),
                reason: format!(
                    "budget_pressure: drop_priority={} (ADD §18.1 / plan drop_order)",
                    c.drop_priority
                ),
                drop_priority: c.drop_priority,
                token_estimate: c.token_estimate,
            });
            explain_frags.push(ExplainFragment {
                fragment_id: c.id.clone(),
                why_included: c.why_included.clone(),
                token_estimate: c.token_estimate,
                must_include: false,
                kept: false,
            });
        }
    }

    kept.sort_by(|a, b| a.id.cmp(&b.id));
    drops.sort_by(|a, b| a.fragment_id.cmp(&b.fragment_id));
    explain_frags.sort_by(|a, b| a.fragment_id.cmp(&b.fragment_id));

    let hierarchy = PackHierarchy::from_fragments(&kept);
    let citations: Vec<Citation> = kept
        .iter()
        .enumerate()
        .map(|(i, f)| Citation {
            id: format!("C{}", i + 1),
            fragment_id: f.id.clone(),
            node_ids: f.provenance.node_ids.clone(),
        })
        .collect();

    let mut gaps = plan.gaps.clone();
    gaps.sort();

    let explain = ExplainReport {
        plan_id: plan.plan_id.clone(),
        budget_tokens: plan.budget_tokens,
        tokens_used: used,
        must_include_ok: true,
        fragments: explain_frags,
        drops: drops.clone(),
        notes: vec![
            "must-include fragments cannot be budget-evicted".into(),
            "extractive default; no abstractive code summaries".into(),
        ],
    };

    Ok(EvidencePack {
        meta: PackMeta {
            intent: plan.intent,
            budget_tokens: plan.budget_tokens,
            tokens_used: used,
            question: plan.question.clone(),
            plan_id: plan.plan_id.clone(),
            repo: None,
            snapshot: None,
            schema_version: PACK_SCHEMA_VERSION.to_string(),
        },
        hierarchy,
        fragments: kept,
        citations,
        gaps,
        drops,
        explain,
    })
}
