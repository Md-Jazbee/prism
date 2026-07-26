//! Confidence provenance for facts (planning Stage B; ADD precision ladder).

use serde::{Deserialize, Serialize};

/// Minimum confidence set required at P0 Stage B exit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    /// Deterministic structural extraction (AST / tree-sitter / markdown parse).
    Extracted,
    /// Plausible but incomplete resolution (e.g. heuristic CALLS).
    Heuristic,
    /// Compiler/SCIP/LSP-backed precise binding.
    Precise,
    /// Runtime / dynamic observation (unused until later phases).
    Observed,
    /// Documentation *claims* it, but code does not prove it (P12 Stage A doc layer).
    /// Never satisfies a precision gate — a doc can be stale or wrong.
    Asserted,
}

impl Confidence {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Extracted => "extracted",
            Self::Heuristic => "heuristic",
            Self::Precise => "precise",
            Self::Observed => "observed",
            Self::Asserted => "asserted",
        }
    }

    /// Whether this confidence level may back a precision-gated claim.
    /// `asserted` (documentation) never qualifies — that is the whole point of the label.
    pub fn is_precise(self) -> bool {
        matches!(self, Self::Precise)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_roundtrip() {
        let c = Confidence::Heuristic;
        let json = serde_json::to_string(&c).unwrap();
        assert_eq!(json, "\"heuristic\"");
        let back: Confidence = serde_json::from_str(&json).unwrap();
        assert_eq!(back, c);
    }

    #[test]
    fn asserted_roundtrips_and_is_not_precise() {
        let json = serde_json::to_string(&Confidence::Asserted).unwrap();
        assert_eq!(json, "\"asserted\"");
        let back: Confidence = serde_json::from_str(&json).unwrap();
        assert_eq!(back, Confidence::Asserted);
        assert!(!Confidence::Asserted.is_precise());
        assert!(Confidence::Precise.is_precise());
    }
}
