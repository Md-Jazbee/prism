//! View-model IR (`schemas/graph-view/v1`).

use serde::{Deserialize, Serialize};

pub const GRAPH_VIEW_SCHEMA_VERSION: &str = "graph-view/v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Citation {
    pub node_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Span {
    pub start_line: u32,
    pub end_line: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ViewNode {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub tier: String,
    pub confidence: String,
    pub lod_rank: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    pub citation: Citation,
    pub x: f64,
    pub y: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heat: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ViewEdge {
    pub id: String,
    pub src: String,
    pub dst: String,
    pub kind: String,
    pub tier: String,
    pub confidence: String,
    pub citation: Citation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ViewGroup {
    pub id: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LayoutInfo {
    pub algorithm: String,
    pub seed: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BudgetUsed {
    pub max_nodes: usize,
    pub max_edges: usize,
    pub nodes_used: usize,
    pub edges_used: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DropRecord {
    pub id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphView {
    pub schema_version: String,
    pub snapshot_id: String,
    pub view_kind: String,
    pub nodes: Vec<ViewNode>,
    pub edges: Vec<ViewEdge>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<ViewGroup>,
    pub budget: BudgetUsed,
    pub layout: LayoutInfo,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub drops: Vec<DropRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ViewTooLarge {
    pub code: String,
    pub message: String,
    pub view_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot_id: Option<String>,
    pub candidate_nodes: usize,
    pub max_nodes: usize,
    pub suggested_anchors: Vec<String>,
    pub hint: String,
}

impl ViewTooLarge {
    pub fn new(
        view_kind: &str,
        snapshot_id: Option<String>,
        candidate_nodes: usize,
        max_nodes: usize,
        suggested_anchors: Vec<String>,
    ) -> Self {
        Self {
            code: "VIEW_TOO_LARGE".into(),
            message: format!(
                "view '{view_kind}' has {candidate_nodes} candidates; budget max_nodes={max_nodes} — refusing silent truncation"
            ),
            view_kind: view_kind.into(),
            snapshot_id,
            candidate_nodes,
            max_nodes,
            suggested_anchors,
            hint: "Narrow anchors (symbol/path/community) or raise max_nodes explicitly for exploration.".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ViewOutcome {
    Ok(GraphView),
    TooLarge(ViewTooLarge),
}
