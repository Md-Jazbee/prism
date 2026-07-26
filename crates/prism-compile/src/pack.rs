//! Evidence Pack IR + compile outcomes.

use crate::budget::BudgetExceeded;
use crate::explain::{DropRecord, ExplainReport};
use crate::fragment::{EvidenceFragment, PackLayer};
use prism_ir::PACK_SCHEMA_VERSION;
use prism_plan::{Intent, ScopeUnresolved};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackMeta {
    pub intent: Intent,
    pub budget_tokens: u32,
    pub tokens_used: u32,
    pub question: String,
    pub plan_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<String>,
    pub schema_version: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackHierarchy {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub l_arch: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub l_mod: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub l_core: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub l_nbr: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub l_diff: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub l_run: Vec<String>,
}

impl PackHierarchy {
    pub fn from_fragments(frags: &[EvidenceFragment]) -> Self {
        let mut h = Self::default();
        for f in frags {
            let id = f.id.clone();
            match f.layer {
                PackLayer::Arch => h.l_arch.push(id),
                PackLayer::Mod => h.l_mod.push(id),
                PackLayer::Core => h.l_core.push(id),
                PackLayer::Nbr => h.l_nbr.push(id),
                PackLayer::Diff => h.l_diff.push(id),
                PackLayer::Run => h.l_run.push(id),
            }
        }
        h
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Citation {
    pub id: String,
    pub fragment_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub node_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidencePack {
    pub meta: PackMeta,
    pub hierarchy: PackHierarchy,
    pub fragments: Vec<EvidenceFragment>,
    pub citations: Vec<Citation>,
    pub gaps: Vec<String>,
    pub drops: Vec<DropRecord>,
    pub explain: ExplainReport,
}

impl EvidencePack {
    pub fn schema_version() -> &'static str {
        PACK_SCHEMA_VERSION
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", content = "data", rename_all = "snake_case")]
pub enum CompileOutcome {
    Ok(Box<EvidencePack>),
    ScopeUnresolved(ScopeUnresolved),
    BudgetExceeded(BudgetExceeded),
}
