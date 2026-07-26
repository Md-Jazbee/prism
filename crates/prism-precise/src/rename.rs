//! Rename dry-run + accuracy claim gating (P3 Stage C).

use crate::has_precise_overlay;
use crate::require::{precision_required, PrecisionGate, PrecisionRequired};
use anyhow::Result;
use prism_store::{EdgeDirection, SqliteKgStore};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// One reference / call site that would be touched by a rename.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenameSite {
    pub edge_kind: String,
    pub confidence: String,
    pub src: String,
    pub dst: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    pub edge_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenameDryRunReport {
    pub mode: String,
    pub writes: bool,
    pub old_name: String,
    pub new_name: String,
    pub tier: String,
    pub sites: Vec<RenameSite>,
    pub site_count: usize,
    pub notes: Vec<String>,
}

/// Collect REFERENCES/CALLS for a resolved symbol; gate on T2 unless `allow_heuristic`.
pub fn rename_dry_run(
    workspace: &Path,
    kg: &SqliteKgStore,
    old_name: &str,
    new_name: &str,
    allow_heuristic: bool,
) -> Result<Result<RenameDryRunReport, PrecisionRequired>> {
    let overlay = has_precise_overlay(workspace);
    if !overlay && !allow_heuristic {
        return Ok(Err(precision_required(
            PrecisionGate::OverlayPresent,
            false,
            false,
            format!(
                "safe rename dry-run for `{old_name}` requires a precise (T2) overlay; import PreciseIndex or pass allow_heuristic"
            ),
        )
        .unwrap_err()));
    }

    let hits = kg.resolve_symbol(old_name, None, 20)?;
    if hits.is_empty() {
        return Ok(Err(PrecisionRequired::new(format!(
            "no symbol named `{old_name}` to rename"
        ))));
    }

    let mut sites = Vec::new();
    let mut saw_precise = false;
    for hit in &hits {
        let nbrs = kg.neighbors(
            &hit.id,
            Some(&["REFERENCES".into(), "CALLS".into()]),
            EdgeDirection::Both,
            200,
        )?;
        for n in nbrs {
            if n.edge.confidence == "precise" {
                saw_precise = true;
            }
            if !allow_heuristic && n.edge.confidence != "precise" {
                continue;
            }
            sites.push(RenameSite {
                edge_kind: n.edge.kind.clone(),
                confidence: n.edge.confidence.clone(),
                src: n.edge.src.clone(),
                dst: n.edge.dst.clone(),
                file_path: n.edge.file_path.clone(),
                edge_id: n.edge.id.clone(),
            });
        }
        // Also include the definition itself as a rename locus
        sites.push(RenameSite {
            edge_kind: "DEFINES".into(),
            confidence: hit.confidence.clone(),
            src: format!("file:{}", hit.file_path.as_deref().unwrap_or("?")),
            dst: hit.id.clone(),
            file_path: hit.file_path.clone(),
            edge_id: format!("def:{}", hit.id),
        });
        if hit.confidence == "precise" {
            saw_precise = true;
        }
    }

    if overlay && !saw_precise && !allow_heuristic {
        return Ok(Err(precision_required(
            PrecisionGate::SymbolHasPreciseEdges,
            true,
            false,
            format!(
                "overlay present but no precise REFERENCES/CALLS for `{old_name}`; re-import index covering this symbol"
            ),
        )
        .unwrap_err()));
    }

    sites.sort_by(|a, b| a.edge_id.cmp(&b.edge_id));
    sites.dedup_by(|a, b| a.edge_id == b.edge_id);

    let tier = if saw_precise && !allow_heuristic {
        "T2"
    } else if allow_heuristic && !saw_precise {
        "T1_heuristic_override"
    } else if saw_precise {
        "T2"
    } else {
        "T1"
    };

    let mut notes = vec![
        "No files modified. Apply rename only after human review.".into(),
        "This is a dry-run procedure, not a production rename engine.".into(),
    ];
    if allow_heuristic && !saw_precise {
        notes.push("allow_heuristic: sites may be incomplete; do not claim rename safety.".into());
    }

    Ok(Ok(RenameDryRunReport {
        mode: "dry_run".into(),
        writes: false,
        old_name: old_name.into(),
        new_name: new_name.into(),
        tier: tier.into(),
        site_count: sites.len(),
        sites,
        notes,
    }))
}

/// Gate an accuracy claim on impact/refactor: need overlay (and preferably low ambiguity).
pub fn require_precise_claim(
    workspace: &Path,
    kg: &SqliteKgStore,
    seed_id: Option<&str>,
) -> Result<(), PrecisionRequired> {
    let overlay = has_precise_overlay(workspace);
    if !overlay {
        return precision_required(
            PrecisionGate::OverlayPresent,
            false,
            false,
            "accuracy claim requires precise (T2) overlay — run `prism precise import`",
        );
    }
    if let Some(id) = seed_id {
        let has = kg
            .symbol_has_precise_edges(id)
            .map_err(|e| PrecisionRequired::new(e.to_string()))?;
        precision_required(
            PrecisionGate::SymbolHasPreciseEdges,
            true,
            has,
            format!("no precise edges touching seed `{id}`"),
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn missing_overlay_precision_required() {
        let dir = tempdir().unwrap();
        let kg = SqliteKgStore::open(dir.path().join("g.sqlite")).unwrap();
        let err = rename_dry_run(dir.path(), &kg, "greet", "hello", false)
            .unwrap()
            .unwrap_err();
        assert_eq!(err.code, "PRECISION_REQUIRED");
    }
}
