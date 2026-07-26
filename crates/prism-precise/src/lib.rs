//! Precise tier (T2) — PreciseIndex ingest + edge refinement (P3 Stage A).
//!
//! See `docs/architecture/PRECISE-TIER.md` and `schemas/precise-index/v0`.

mod index;
mod refine;
mod require;
mod score;

pub use index::{
    load_precise_index, PreciseEdge, PreciseIndex, PreciseSnapshot, PreciseSpan, PreciseSymbol,
    PRECISE_INDEX_SCHEMA_VERSION,
};
pub use refine::{apply_overlay_to_store, refine_edges, OverlayApplyStats, RefineStats};
pub use require::{precision_required, PrecisionGate, PrecisionRequired};
pub use score::{score_call_resolution, CallEdge, ScoreReport};

use anyhow::{bail, Context, Result};
use prism_ir::{
    edge_id, file_node_id, Confidence, EdgeKind, FactBundle, FactEdge, FactNode, NodeKind, Span,
    Tier,
};
use std::path::{Path, PathBuf};

/// Manifest written under `.prism/scip/manifest.json` after a successful import.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PreciseManifest {
    pub schema_version: String,
    pub language: String,
    pub analyzer: String,
    pub artifact: String,
    pub imported_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_commit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tree_fingerprint: Option<String>,
    pub symbols: usize,
    pub edges: usize,
    pub refined: usize,
    pub inserted: usize,
}

/// Convert a PreciseIndex document into a T2 [`FactBundle`] (one virtual path for the overlay).
pub fn index_to_fact_bundle(index: &PreciseIndex) -> FactBundle {
    let mut bundle = FactBundle {
        schema_version: prism_ir::FACT_SCHEMA_VERSION.to_string(),
        analyzer: index.analyzer.clone(),
        tier: Tier::T2,
        language: index.language.clone(),
        path: format!("precise:{}", index.language),
        nodes: Vec::new(),
        edges: Vec::new(),
    };

    let mut seen_files = std::collections::HashSet::new();
    for sym in &index.symbols {
        if seen_files.insert(sym.file_path.clone()) {
            bundle.nodes.push(FactNode {
                id: file_node_id(&sym.file_path),
                kind: NodeKind::File,
                name: Some(sym.file_path.clone()),
                file_path: Some(sym.file_path.clone()),
                span: None,
                language: Some(index.language.clone()),
                analyzer: index.analyzer.clone(),
                tier: Tier::T2,
                confidence: Confidence::Precise,
                attrs: Default::default(),
            });
        }
        let mut attrs = serde_json::Map::new();
        attrs.insert(
            "symbol_kind".into(),
            serde_json::Value::String(sym.kind.clone()),
        );
        if let Some(scip) = &sym.scip_symbol {
            attrs.insert(
                "scip_symbol".into(),
                serde_json::Value::String(scip.clone()),
            );
        }
        bundle.nodes.push(FactNode {
            id: sym.id.clone(),
            kind: NodeKind::Symbol,
            name: Some(sym.name.clone()),
            file_path: Some(sym.file_path.clone()),
            span: sym.span.as_ref().map(span_from_precise),
            language: Some(index.language.clone()),
            analyzer: index.analyzer.clone(),
            tier: Tier::T2,
            confidence: Confidence::Precise,
            attrs,
        });
    }

    for edge in &index.edges {
        let kind = parse_edge_kind(&edge.kind);
        let start = edge
            .span
            .as_ref()
            .map(|s| s.start_byte)
            .unwrap_or(0);
        let mut attrs = serde_json::Map::new();
        attrs.insert(
            "overlay".into(),
            serde_json::Value::String("precise".into()),
        );
        if let Some(scip) = &edge.scip_symbol {
            attrs.insert(
                "scip_symbol".into(),
                serde_json::Value::String(scip.clone()),
            );
        }
        bundle.edges.push(FactEdge {
            id: edge_id(kind, &edge.src, &edge.dst, start),
            kind,
            src: edge.src.clone(),
            dst: edge.dst.clone(),
            file_path: Some(edge.file_path.clone()),
            span: edge.span.as_ref().map(span_from_precise),
            analyzer: index.analyzer.clone(),
            tier: Tier::T2,
            confidence: Confidence::Precise,
            attrs,
        });
    }

    bundle.normalize();
    bundle
}

fn span_from_precise(s: &PreciseSpan) -> Span {
    Span {
        start_byte: s.start_byte,
        end_byte: s.end_byte,
        start_line: s.start_line,
        start_col: s.start_col,
        end_line: s.end_line,
        end_col: s.end_col,
    }
}

fn parse_edge_kind(raw: &str) -> EdgeKind {
    match raw {
        "CALLS" | "Calls" => EdgeKind::Calls,
        "REFERENCES" | "References" => EdgeKind::References,
        "DEFINES" | "Defines" => EdgeKind::Defines,
        "IMPORTS" | "Imports" => EdgeKind::Imports,
        "EXTENDS" | "Extends" => EdgeKind::Extends,
        "IMPLEMENTS" | "Implements" => EdgeKind::Implements,
        _ => EdgeKind::References,
    }
}

/// Import PreciseIndex JSON into `.prism/scip/` and refine the KG overlay.
pub fn import_precise_index(
    workspace: &Path,
    index_path: &Path,
    git_commit: Option<String>,
    tree_fingerprint: Option<String>,
) -> Result<(PreciseManifest, OverlayApplyStats)> {
    let index = load_precise_index(index_path)?;
    let prism = workspace.join(".prism");
    let graph_path = prism.join("graph.sqlite");
    if !graph_path.exists() {
        bail!(
            "no index at {} — run `prism index` before precise import",
            prism.display()
        );
    }
    let scip_dir = prism.join("scip");
    std::fs::create_dir_all(&scip_dir)
        .with_context(|| format!("create {}", scip_dir.display()))?;

    let artifact_name = index_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("precise-index.json")
        .to_string();
    let dest = scip_dir.join(&artifact_name);
    std::fs::copy(index_path, &dest)
        .with_context(|| format!("copy index to {}", dest.display()))?;

    let bundle = index_to_fact_bundle(&index);
    let mut kg = prism_store::SqliteKgStore::open(&graph_path)?;
    let stats = apply_overlay_to_store(&mut kg, &bundle)?;

    let manifest = PreciseManifest {
        schema_version: index.schema_version.clone(),
        language: index.language.clone(),
        analyzer: index.analyzer.clone(),
        artifact: artifact_name,
        imported_at: chrono_like_now(),
        git_commit: git_commit.or_else(|| {
            index
                .snapshot
                .as_ref()
                .and_then(|s| s.git_commit.clone())
        }),
        tree_fingerprint: tree_fingerprint.or_else(|| {
            index
                .snapshot
                .as_ref()
                .and_then(|s| s.tree_fingerprint.clone())
        }),
        symbols: index.symbols.len(),
        edges: index.edges.len(),
        refined: stats.refined,
        inserted: stats.inserted,
    };
    let manifest_path = scip_dir.join("manifest.json");
    std::fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest)? + "\n",
    )
    .with_context(|| format!("write {}", manifest_path.display()))?;

    Ok((manifest, stats))
}

fn chrono_like_now() -> String {
    // Avoid chrono dep: RFC3339-ish via system time seconds is enough for Stage A.
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix:{secs}")
}

/// Read `.prism/scip/manifest.json` if present.
pub fn read_manifest(workspace: &Path) -> Result<Option<PreciseManifest>> {
    let path = workspace.join(".prism/scip/manifest.json");
    if !path.exists() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("read {}", path.display()))?;
    Ok(Some(serde_json::from_str(&text)?))
}

/// Whether a workspace has an attached precise overlay.
pub fn has_precise_overlay(workspace: &Path) -> bool {
    workspace.join(".prism/scip/manifest.json").exists()
}

/// Path helper for tests / CLI.
pub fn scip_dir(workspace: impl AsRef<Path>) -> PathBuf {
    workspace.as_ref().join(".prism/scip")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn fixture_index_loads_and_scores() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let index_path = root.join("fixtures/precise/oracle/python/precise-index.json");
        let index = load_precise_index(&index_path).expect("load fixture");
        let bundle = index_to_fact_bundle(&index);
        assert_eq!(bundle.tier, Tier::T2);
        assert!(bundle.edges.iter().all(|e| e.confidence == Confidence::Precise));

        let t1: Vec<CallEdge> = serde_json::from_str(
            &fs::read_to_string(root.join("fixtures/precise/oracle/python/t1-calls.json")).unwrap(),
        )
        .unwrap();
        let oracle: Vec<CallEdge> = serde_json::from_str(
            &fs::read_to_string(root.join("fixtures/precise/oracle/python/oracle-calls.json"))
                .unwrap(),
        )
        .unwrap();
        let t2_calls: Vec<CallEdge> = index
            .edges
            .iter()
            .filter(|e| e.kind == "CALLS")
            .map(|e| CallEdge {
                src: e.src.clone(),
                dst: e.dst.clone(),
                file_path: e.file_path.clone(),
                start_byte: e.span.as_ref().map(|s| s.start_byte),
            })
            .collect();

        let t1_score = score_call_resolution(&t1, &oracle);
        let t2_score = score_call_resolution(&t2_calls, &oracle);
        assert!(
            t2_score.precision > t1_score.precision,
            "T2 precision ({}) should beat T1 ({})",
            t2_score.precision,
            t1_score.precision
        );
        assert!(
            t2_score.recall >= t1_score.recall,
            "T2 recall should not regress"
        );
    }
}
