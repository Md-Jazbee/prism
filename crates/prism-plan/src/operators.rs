//! Operator catalog v1 (ADD §19.2).

use serde::{Deserialize, Serialize};
use std::fmt;

/// Query-plan operators. Placeholders are declared but not executed until later phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Operator {
    ResolveSymbol,
    Expand,
    /// Placeholder until P4 semantic slicing.
    Slice,
    Impact,
    DiffIntersect,
    FindTests,
    CommunityOf,
    /// Placeholder until P3 precise tier.
    UpgradePrecision,
    /// Low-confidence fallback; flagged, not a success path.
    KeywordEmbedFallback,
    BudgetPack,
}

impl Operator {
    /// Rough relative cost sketch (ms) for cost-based planning (v1 constants).
    pub fn est_cost_ms(self) -> u32 {
        match self {
            Operator::ResolveSymbol => 5,
            Operator::Expand => 15,
            Operator::Impact => 25,
            Operator::CommunityOf => 20,
            Operator::DiffIntersect => 30,
            Operator::FindTests => 20,
            Operator::BudgetPack => 10,
            Operator::Slice => 80,
            Operator::UpgradePrecision => 200,
            Operator::KeywordEmbedFallback => 50,
        }
    }

    /// Whether this operator is executable against the live P1 KG today.
    pub fn executable_in_v1(self) -> bool {
        matches!(
            self,
            Operator::ResolveSymbol
                | Operator::Expand
                | Operator::Impact
                | Operator::CommunityOf
                | Operator::DiffIntersect
                | Operator::FindTests
                | Operator::BudgetPack
        )
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = serde_json::to_value(self)
            .ok()
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_else(|| format!("{self:?}"));
        write!(f, "{s}")
    }
}
