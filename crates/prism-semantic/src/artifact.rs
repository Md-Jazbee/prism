//! Semantic artifact IR (schemas/semantic-artifact/v0).

use serde::{Deserialize, Serialize};

pub const SEMANTIC_SCHEMA_VERSION: &str = "0.0.1";
pub const ALGO_VERSION: &str = "t3-python-cfgdfg@0.0.1";
/// Inter-procedural shard / slice algorithm (P4 Stage B).
pub const INTERPROC_ALGO_VERSION: &str = "t4-python-interproc@0.0.1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticFileArtifact {
    pub schema_version: String,
    pub algo_version: String,
    pub language: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
    pub functions: Vec<FunctionFlow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionFlow {
    pub name: String,
    pub start_line: u32,
    pub end_line: u32,
    pub blocks: Vec<CfgBlock>,
    pub cfg_edges: Vec<CfgEdge>,
    pub dfg: DfgGraph,
    /// Direct call sites inside this function (name-level; Stage B inter-proc).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub calls: Vec<CallSite>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallSite {
    pub callee: String,
    pub line: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CfgBlock {
    pub id: String,
    pub start_line: u32,
    pub end_line: u32,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CfgEdge {
    pub src: String,
    pub dst: String,
    pub kind: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DfgGraph {
    pub defs: Vec<DfgDef>,
    pub uses: Vec<DfgUse>,
    pub deps: Vec<DfgDep>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DfgDef {
    pub name: String,
    pub line: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DfgUse {
    pub name: String,
    pub line: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DfgDep {
    pub name: String,
    pub def_line: u32,
    pub use_line: u32,
}
