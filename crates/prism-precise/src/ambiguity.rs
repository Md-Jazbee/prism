//! Ambiguity index over CALLS edges (P3 Stage B).

use serde::{Deserialize, Serialize};

/// Rates of heuristic / unresolved CALLS — feeds optional UpgradePrecision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AmbiguityIndex {
    pub total_calls: u64,
    pub precise_calls: u64,
    pub heuristic_calls: u64,
    pub unresolved_calls: u64,
    pub heuristic_rate: f64,
    pub unresolved_rate: f64,
    /// True when optional UpgradePrecision should run (impact policy).
    pub require_t2: bool,
}

impl AmbiguityIndex {
    pub const UNRESOLVED_THRESHOLD: f64 = 0.30;
    pub const HEURISTIC_THRESHOLD: f64 = 0.50;

    pub fn from_counts(
        total_calls: u64,
        precise_calls: u64,
        heuristic_calls: u64,
        unresolved_calls: u64,
    ) -> Self {
        let heuristic_rate = if total_calls == 0 {
            0.0
        } else {
            (heuristic_calls + unresolved_calls) as f64 / total_calls as f64
        };
        let unresolved_rate = if total_calls == 0 {
            0.0
        } else {
            unresolved_calls as f64 / total_calls as f64
        };
        let require_t2 = total_calls > 0
            && (unresolved_rate >= Self::UNRESOLVED_THRESHOLD
                || heuristic_rate >= Self::HEURISTIC_THRESHOLD);
        Self {
            total_calls,
            precise_calls,
            heuristic_calls,
            unresolved_calls,
            heuristic_rate,
            unresolved_rate,
            require_t2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn require_t2_on_high_unresolved() {
        let a = AmbiguityIndex::from_counts(10, 1, 2, 7);
        assert!(a.require_t2);
        assert!((a.unresolved_rate - 0.7).abs() < 1e-9);
    }

    #[test]
    fn empty_graph_no_require() {
        let a = AmbiguityIndex::from_counts(0, 0, 0, 0);
        assert!(!a.require_t2);
    }
}
