//! `PRECISION_REQUIRED` gate when T2 overlay is missing.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Product error when a precision-gated operation lacks T2 data.
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
#[error("PRECISION_REQUIRED: {message}")]
pub struct PrecisionRequired {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

impl PrecisionRequired {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            code: "PRECISION_REQUIRED".into(),
            message: message.into(),
            hint: Some(
                "Produce a PreciseIndex (see docs/architecture/SCIP-RUNBOOK.md) then run `prism precise import`. T1 heuristic results remain available but must stay labeled."
                    .into(),
            ),
        }
    }
}

/// What a caller is asking of the precise tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrecisionGate {
    /// Any attached overlay for the workspace is enough.
    OverlayPresent,
    /// At least one precise edge touches this symbol id.
    SymbolHasPreciseEdges,
}

/// Return `PRECISION_REQUIRED` when the gate fails.
pub fn precision_required(
    gate: PrecisionGate,
    overlay_present: bool,
    symbol_has_precise: bool,
    detail: impl Into<String>,
) -> Result<(), PrecisionRequired> {
    match gate {
        PrecisionGate::OverlayPresent if overlay_present => Ok(()),
        PrecisionGate::SymbolHasPreciseEdges if symbol_has_precise => Ok(()),
        _ => Err(PrecisionRequired::new(detail)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_overlay_errors() {
        let err = precision_required(
            PrecisionGate::OverlayPresent,
            false,
            false,
            "no precise overlay for workspace",
        )
        .unwrap_err();
        assert_eq!(err.code, "PRECISION_REQUIRED");
    }

    #[test]
    fn present_overlay_ok() {
        precision_required(PrecisionGate::OverlayPresent, true, false, "unused").unwrap();
    }
}
