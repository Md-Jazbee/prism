//! PreciseIndex JSON IR (schemas/precise-index/v0).

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const PRECISE_INDEX_SCHEMA_VERSION: &str = "0.0.1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreciseIndex {
    pub schema_version: String,
    pub language: String,
    pub analyzer: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<PreciseSnapshot>,
    pub symbols: Vec<PreciseSymbol>,
    pub edges: Vec<PreciseEdge>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreciseSnapshot {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_commit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tree_fingerprint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreciseSymbol {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub file_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span: Option<PreciseSpan>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scip_symbol: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreciseEdge {
    pub kind: String,
    pub src: String,
    pub dst: String,
    pub file_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span: Option<PreciseSpan>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scip_symbol: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreciseSpan {
    pub start_byte: u32,
    pub end_byte: u32,
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

pub fn load_precise_index(path: &Path) -> Result<PreciseIndex> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("read precise index {}", path.display()))?;
    let index: PreciseIndex = serde_json::from_str(&text)
        .with_context(|| format!("parse PreciseIndex {}", path.display()))?;
    if index.schema_version != PRECISE_INDEX_SCHEMA_VERSION {
        bail!(
            "unsupported PreciseIndex schema_version {} (expected {})",
            index.schema_version,
            PRECISE_INDEX_SCHEMA_VERSION
        );
    }
    if index.language.is_empty() || index.analyzer.is_empty() {
        bail!("PreciseIndex language/analyzer must be non-empty");
    }
    Ok(index)
}
