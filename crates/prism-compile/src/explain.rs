//! EXPLAIN CONTEXT — per-fragment why_included + drop audit.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExplainFragment {
    pub fragment_id: String,
    pub why_included: String,
    pub token_estimate: u32,
    pub must_include: bool,
    pub kept: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DropRecord {
    pub fragment_id: String,
    pub reason: String,
    pub drop_priority: u32,
    pub token_estimate: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExplainReport {
    pub plan_id: String,
    pub budget_tokens: u32,
    pub tokens_used: u32,
    pub must_include_ok: bool,
    pub fragments: Vec<ExplainFragment>,
    pub drops: Vec<DropRecord>,
    pub notes: Vec<String>,
}
