//! Fragment and candidate types (ADD §11.4 / §16.2).

use serde::{Deserialize, Serialize};

/// Hierarchical layer in an Evidence Pack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackLayer {
    Arch,
    Mod,
    Core,
    Nbr,
    Diff,
    Run,
}

impl PackLayer {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Arch => "L_arch",
            Self::Mod => "L_mod",
            Self::Core => "L_core",
            Self::Nbr => "L_nbr",
            Self::Diff => "L_diff",
            Self::Run => "L_run",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FragmentKind {
    Slice,
    Signature,
    Diff,
    CfgSummary,
    Community,
    Trace,
    ErrorVerbatim,
}

/// Provenance for a fragment — required on every kept fragment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    pub node_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub edge_ids: Vec<String>,
    pub analyzer: String,
    pub tier: String,
}

impl Provenance {
    pub fn synthetic(label: &str) -> Self {
        Self {
            node_ids: vec![format!("synthetic:{label}")],
            edge_ids: vec![],
            analyzer: "prism-compile@0.0.1".into(),
            tier: "T1".into(),
        }
    }

    pub fn from_node(node_id: impl Into<String>, confidence_analyzer: &str) -> Self {
        Self::from_node_tier(node_id, confidence_analyzer, "T1")
    }

    pub fn from_node_tier(
        node_id: impl Into<String>,
        confidence_analyzer: &str,
        tier: &str,
    ) -> Self {
        Self {
            node_ids: vec![node_id.into()],
            edge_ids: vec![],
            analyzer: confidence_analyzer.into(),
            tier: tier.into(),
        }
    }

    /// True when every node id is a `synthetic:` placeholder (P12 Stage B ACC-2).
    pub fn is_synthetic_only(&self) -> bool {
        !self.node_ids.is_empty()
            && self.node_ids.iter().all(|id| id.starts_with("synthetic:"))
            && self.edge_ids.is_empty()
    }
}

/// Pre-budget candidate (selection output).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateFragment {
    pub id: String,
    pub kind: FragmentKind,
    pub layer: PackLayer,
    pub text: String,
    pub token_estimate: u32,
    pub provenance: Provenance,
    pub confidence: String,
    /// Planner / selection reason code (becomes `why_included`).
    pub why_included: String,
    /// Higher = dropped first under budget pressure. Must-include uses `0`.
    pub drop_priority: u32,
    /// Recipe role tags (matched against `Plan.must_include`).
    pub roles: Vec<String>,
    pub must_include: bool,
}

/// Fragment kept in the Evidence Pack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceFragment {
    pub id: String,
    pub kind: FragmentKind,
    pub layer: PackLayer,
    pub text: String,
    pub token_estimate: u32,
    pub provenance: Provenance,
    pub confidence: String,
    pub why_included: String,
    pub drop_priority: u32,
    pub roles: Vec<String>,
    pub must_include: bool,
}

impl From<CandidateFragment> for EvidenceFragment {
    fn from(c: CandidateFragment) -> Self {
        Self {
            id: c.id,
            kind: c.kind,
            layer: c.layer,
            text: c.text,
            token_estimate: c.token_estimate,
            provenance: c.provenance,
            confidence: c.confidence,
            why_included: c.why_included,
            drop_priority: c.drop_priority,
            roles: c.roles,
            must_include: c.must_include,
        }
    }
}

/// Rough token estimate: ceil(chars / 4), minimum 1 for non-empty.
pub fn estimate_tokens(text: &str) -> u32 {
    if text.is_empty() {
        return 0;
    }
    (text.len() as u32).div_ceil(4)
}
