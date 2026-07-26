//! Progressive packs + budget negotiation (P9 Stage A).

use prism_compile::{EvidencePack, PackLayer as FragLayer};
use serde::{Deserialize, Serialize};

/// Layers stream architecture-first so agents can start before the full pack lands.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProgressiveLayer {
    pub name: String,
    pub fragment_ids: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProgressivePack {
    pub plan_id: String,
    pub layers: Vec<ProgressiveLayer>,
}

/// Split a compiled pack into progressive layers (must-include computed already).
pub fn progressive_layers(pack: &EvidencePack) -> ProgressivePack {
    let mut arch = Vec::new();
    let mut must = Vec::new();
    let mut support = Vec::new();
    for f in &pack.fragments {
        match f.layer {
            FragLayer::Arch => arch.push(f.id.clone()),
            _ if f.must_include => must.push(f.id.clone()),
            _ => support.push(f.id.clone()),
        }
    }
    if !pack.hierarchy.l_arch.is_empty() {
        arch = pack.hierarchy.l_arch.clone();
    }
    if arch.is_empty() {
        if let Some(first) = must
            .first()
            .cloned()
            .or_else(|| pack.fragments.first().map(|f| f.id.clone()))
        {
            arch.push(first);
        }
    }
    ProgressivePack {
        plan_id: pack.meta.plan_id.clone(),
        layers: vec![
            ProgressiveLayer {
                name: "architecture".into(),
                fragment_ids: arch,
                notes: vec!["Stream first — agent may begin reasoning".into()],
            },
            ProgressiveLayer {
                name: "must_include".into(),
                fragment_ids: must,
                notes: vec!["Must-include finalized before streaming began".into()],
            },
            ProgressiveLayer {
                name: "support".into(),
                fragment_ids: support,
                notes: vec!["Soft-drop candidates under budget".into()],
            },
        ],
    }
}

/// Agents declare remaining context; compiler targets the smaller of remaining and requested.
pub fn negotiate_budget(requested: u32, remaining_context_tokens: Option<u32>) -> u32 {
    match remaining_context_tokens {
        Some(rem) if rem > 0 => requested.min(rem).clamp(256, 128_000),
        _ => requested.clamp(256, 128_000),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negotiate_picks_min() {
        assert_eq!(negotiate_budget(4000, Some(1500)), 1500);
        assert_eq!(negotiate_budget(4000, None), 4000);
        assert_eq!(negotiate_budget(100, Some(50)), 256);
    }
}
