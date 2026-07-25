//! T1 fact shapes (Phase 1 Stage A) — mirrors `schemas/fact-schema/v0`.

use crate::confidence::Confidence;
use crate::versions::FACT_SCHEMA_VERSION;
use serde::{Deserialize, Serialize};

/// Precision tier for a fact bundle / edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Tier {
    T1,
    T2,
    T3,
    T4,
}

impl Tier {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::T1 => "T1",
            Self::T2 => "T2",
            Self::T3 => "T3",
            Self::T4 => "T4",
        }
    }
}

/// Byte-oriented source span (0-based, end exclusive).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    pub start_byte: u32,
    pub end_byte: u32,
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

/// T1 node kinds from fact schema (+ Module for packages/files).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeKind {
    File,
    Symbol,
    Module,
    Package,
}

impl NodeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::File => "File",
            Self::Symbol => "Symbol",
            Self::Module => "Module",
            Self::Package => "Package",
        }
    }
}

/// T1 edge kinds (+ best-effort inheritance).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeKind {
    Contains,
    Imports,
    Calls,
    Defines,
    References,
    Extends,
    Implements,
}

impl EdgeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Contains => "CONTAINS",
            Self::Imports => "IMPORTS",
            Self::Calls => "CALLS",
            Self::Defines => "DEFINES",
            Self::References => "REFERENCES",
            Self::Extends => "EXTENDS",
            Self::Implements => "IMPLEMENTS",
        }
    }
}

/// A typed knowledge-graph node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactNode {
    pub id: String,
    pub kind: NodeKind,
    /// Display / symbol name when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Repo-relative path owning this node (File / Symbol).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    pub analyzer: String,
    pub tier: Tier,
    pub confidence: Confidence,
    /// Extra attributes (e.g. `symbol_kind: function|class|struct`).
    #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
    pub attrs: serde_json::Map<String, serde_json::Value>,
}

/// A typed knowledge-graph edge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactEdge {
    pub id: String,
    pub kind: EdgeKind,
    pub src: String,
    pub dst: String,
    /// File where the edge was observed (call site / import line).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,
    pub analyzer: String,
    pub tier: Tier,
    pub confidence: Confidence,
    #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
    pub attrs: serde_json::Map<String, serde_json::Value>,
}

/// Versioned extractor output for one file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactBundle {
    pub schema_version: String,
    pub analyzer: String,
    pub tier: Tier,
    pub language: String,
    pub path: String,
    pub nodes: Vec<FactNode>,
    pub edges: Vec<FactEdge>,
}

impl FactBundle {
    pub fn new(
        path: impl Into<String>,
        language: impl Into<String>,
        analyzer: impl Into<String>,
    ) -> Self {
        Self {
            schema_version: FACT_SCHEMA_VERSION.to_string(),
            analyzer: analyzer.into(),
            tier: Tier::T1,
            language: language.into(),
            path: path.into(),
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Stable sort for golden-fixture comparison.
    pub fn normalize(&mut self) {
        self.nodes.sort_by(|a, b| a.id.cmp(&b.id));
        self.edges.sort_by(|a, b| a.id.cmp(&b.id));
    }

    pub fn unresolved_call_count(&self) -> usize {
        self.edges
            .iter()
            .filter(|e| e.kind == EdgeKind::Calls && e.dst.starts_with("unresolved:"))
            .count()
    }
}

/// Deterministic file node id.
pub fn file_node_id(path: &str) -> String {
    format!("file:{path}")
}

/// Deterministic symbol id (path + kind + name + start byte).
pub fn symbol_node_id(path: &str, symbol_kind: &str, name: &str, start_byte: u32) -> String {
    format!("sym:{path}:{symbol_kind}:{name}:{start_byte}")
}

/// Synthetic unresolved callee id — first-class, never silently dropped.
pub fn unresolved_node_id(name: &str) -> String {
    format!("unresolved:{name}")
}

/// Deterministic edge id.
pub fn edge_id(kind: EdgeKind, src: &str, dst: &str, start_byte: u32) -> String {
    format!("edge:{}:{src}:{dst}:{start_byte}", kind.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unresolved_prefix_is_stable() {
        assert_eq!(unresolved_node_id("foo"), "unresolved:foo");
    }

    #[test]
    fn normalize_sorts_by_id() {
        let mut b = FactBundle::new("a.py", "python", "test");
        b.nodes.push(FactNode {
            id: "z".into(),
            kind: NodeKind::File,
            name: None,
            file_path: Some("a.py".into()),
            span: None,
            language: None,
            analyzer: "test".into(),
            tier: Tier::T1,
            confidence: Confidence::Extracted,
            attrs: Default::default(),
        });
        b.nodes.push(FactNode {
            id: "a".into(),
            kind: NodeKind::Symbol,
            name: Some("f".into()),
            file_path: Some("a.py".into()),
            span: None,
            language: None,
            analyzer: "test".into(),
            tier: Tier::T1,
            confidence: Confidence::Extracted,
            attrs: Default::default(),
        });
        b.normalize();
        assert_eq!(b.nodes[0].id, "a");
        assert_eq!(b.nodes[1].id, "z");
    }
}
