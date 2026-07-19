//! Confidence provenance for facts (planning Stage B; ADD precision ladder).

use serde::{Deserialize, Serialize};

/// Minimum confidence set required at P0 Stage B exit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    /// Deterministic structural extraction (AST / tree-sitter).
    Extracted,
    /// Plausible but incomplete resolution (e.g. heuristic CALLS).
    Heuristic,
    /// Compiler/SCIP/LSP-backed precise binding.
    Precise,
    /// Runtime / dynamic observation (unused until later phases).
    Observed,
}

impl Confidence {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Extracted => "extracted",
            Self::Heuristic => "heuristic",
            Self::Precise => "precise",
            Self::Observed => "observed",
        }
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
}
