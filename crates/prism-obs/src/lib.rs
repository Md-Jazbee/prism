//! Observability baseline for Phase 0 (W-OBS).
//!
//! Events can be emitted to tracing logs now; OTel exporters land later.

pub mod events;

pub use events::{emit_index_event, IndexEvent, IndexStats};

/// Convenience: audit that an Evidence Pack is about to leave the machine.
pub fn emit_pack_bound_for_llm(
    plan_id: impl Into<String>,
    token_estimate: u64,
    fragment_count: u64,
    redacted: bool,
    workspace_fingerprint: impl Into<String>,
) {
    emit_index_event(&IndexEvent::PackBoundForLlm {
        plan_id: plan_id.into(),
        token_estimate,
        fragment_count,
        redacted,
        workspace_fingerprint: workspace_fingerprint.into(),
    });
}

/// Shadow token-savings metric (explore proxy vs pack).
pub fn emit_token_savings_shadow(explore_tokens_proxy: u64, pack_tokens: u64) {
    let savings_ratio = if pack_tokens == 0 {
        0.0
    } else {
        explore_tokens_proxy as f64 / pack_tokens as f64
    };
    emit_index_event(&IndexEvent::TokenSavingsShadow {
        explore_tokens_proxy,
        pack_tokens,
        savings_ratio,
    });
}
