//! Plan IR types (schemas/plan/v0).

use crate::intent::Intent;
use crate::operators::Operator;
use prism_ir::PLAN_SCHEMA_VERSION;
use serde::{Deserialize, Serialize};

/// One node in the operator DAG.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanStep {
    pub id: String,
    pub op: Operator,
    /// Free-form operator inputs (anchors, edge kinds, depth, …).
    pub inputs: serde_json::Value,
    pub depends_on: Vec<String>,
    pub est_cost_ms: u32,
    /// False for placeholders (`Slice`, `UpgradePrecision`, …) until later phases.
    pub executable: bool,
    pub why: String,
}

/// Deterministic query plan produced by a recipe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plan {
    pub schema_version: String,
    pub plan_id: String,
    pub intent: Intent,
    pub question: String,
    pub budget_tokens: u32,
    pub steps: Vec<PlanStep>,
    pub must_include: Vec<String>,
    pub drop_order: Vec<String>,
    pub gaps: Vec<String>,
    pub notes: Vec<String>,
}

impl Plan {
    pub fn new(intent: Intent, question: impl Into<String>, budget_tokens: u32) -> Self {
        let question = question.into();
        let plan_id = stable_plan_id(intent, &question, budget_tokens);
        Self {
            schema_version: PLAN_SCHEMA_VERSION.to_string(),
            plan_id,
            intent,
            question,
            budget_tokens,
            steps: Vec::new(),
            must_include: Vec::new(),
            drop_order: Vec::new(),
            gaps: Vec::new(),
            notes: Vec::new(),
        }
    }

    /// Stable ordering for golden fixtures.
    pub fn normalize(&mut self) {
        // steps stay in dependency order; only sort string lists
        self.must_include.sort();
        self.drop_order.sort();
        self.gaps.sort();
        self.notes.sort();
        for step in &mut self.steps {
            step.depends_on.sort();
        }
    }
}

/// Refuse unbounded dump — ask for anchors (ADD §22.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeUnresolved {
    pub code: String,
    pub reason: String,
    pub ask_for: Vec<String>,
    pub intent: Option<Intent>,
    pub question: String,
}

/// Result of plan-only API (`POST /v1/query/plan` contract).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", content = "data", rename_all = "snake_case")]
pub enum PlanOutcome {
    Ok(Plan),
    ScopeUnresolved(ScopeUnresolved),
}

fn stable_plan_id(intent: Intent, question: &str, budget: u32) -> String {
    // Deterministic short id without pulling xxhash into this crate.
    let mut h: u64 = 0xcbf29ce484222325;
    for b in format!("{intent}|{budget}|{question}").bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("plan:{intent}:{:016x}", h)
}
