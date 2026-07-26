//! Structured evidence gaps (P12 Stage B).
//!
//! An unfilled recipe role becomes a gap with a repair action — never a
//! placeholder fragment that looks like evidence.

use serde::{Deserialize, Serialize};

/// Why a must-include (or requested) role could not be filled.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WhyAbsent {
    /// No matching node/edge in the KG.
    NoSuchNode,
    /// Anchors did not resolve (wrong seed / under-specified).
    SeedUnresolved,
    /// Evidence exists but below the required precision tier.
    BelowTier,
    /// Would have fit but was dropped under budget (should not happen for must-include).
    Budget,
    /// Path classified as vendored/fixture/generated and question did not anchor there.
    PathClassExcluded,
    /// Operator / recipe note carried forward from the plan.
    PlanNote,
    /// Other / upgrade uncertainty.
    Other,
}

impl WhyAbsent {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NoSuchNode => "no_such_node",
            Self::SeedUnresolved => "seed_unresolved",
            Self::BelowTier => "below_tier",
            Self::Budget => "budget",
            Self::PathClassExcluded => "path_class_excluded",
            Self::PlanNote => "plan_note",
            Self::Other => "other",
        }
    }
}

/// Honest absence of evidence for a role.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceGap {
    /// Recipe role that could not be filled (or `plan` for planner notes).
    pub role: String,
    pub why_absent: WhyAbsent,
    /// Machine-actionable next step for the agent (refusal-repair style).
    pub repair: String,
    /// Optional human-readable detail.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl EvidenceGap {
    pub fn new(role: impl Into<String>, why_absent: WhyAbsent, repair: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            why_absent,
            repair: repair.into(),
            detail: None,
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Convert a free-form plan gap string into a structured gap.
    pub fn from_plan_note(note: &str) -> Self {
        Self::new(
            "plan",
            WhyAbsent::PlanNote,
            "Inspect plan.gaps / EXPLAIN notes",
        )
        .with_detail(note)
    }

    /// Display form used in EXPLAIN / logs.
    pub fn summary(&self) -> String {
        match &self.detail {
            Some(d) => format!(
                "gap role={} why={} repair={} ({d})",
                self.role,
                self.why_absent.as_str(),
                self.repair
            ),
            None => format!(
                "gap role={} why={} repair={}",
                self.role,
                self.why_absent.as_str(),
                self.repair
            ),
        }
    }
}
