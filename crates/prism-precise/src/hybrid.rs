//! Hybrid resolver: syntactic candidates → precise confirmation (P3 Stage B).

use crate::refine::edges_join_views;
use anyhow::Result;
use prism_store::{EdgeDirection, GraphEdgeView, SqliteKgStore};
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Default caps — keep UpgradePrecision on the sync path bounded.
pub const DEFAULT_MAX_UPGRADES: usize = 32;
pub const DEFAULT_MAX_LATENCY_MS: u64 = 200;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DualCandidate {
    pub site_src: String,
    pub heuristic_dst: String,
    pub precise_dst: String,
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfirmedEdge {
    pub src: String,
    pub dst: String,
    pub edge_id: String,
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HybridResolveReport {
    pub confirmed: Vec<ConfirmedEdge>,
    pub still_heuristic: usize,
    pub dual_candidates: Vec<DualCandidate>,
    pub deferred: usize,
    pub latency_ms: u64,
    pub upgrades_considered: usize,
    pub overlay_used: bool,
}

#[derive(Debug, Clone)]
pub struct HybridResolveOptions {
    pub critical_path_only: bool,
    pub max_upgrades: usize,
    pub max_latency_ms: u64,
}

impl Default for HybridResolveOptions {
    fn default() -> Self {
        Self {
            critical_path_only: false,
            max_upgrades: DEFAULT_MAX_UPGRADES,
            max_latency_ms: DEFAULT_MAX_LATENCY_MS,
        }
    }
}

/// Prefer precise CALLS/REFERENCES over heuristic for seed symbols.
pub fn hybrid_resolve(
    kg: &SqliteKgStore,
    seed_ids: &[String],
    opts: &HybridResolveOptions,
) -> Result<HybridResolveReport> {
    let started = Instant::now();
    let mut report = HybridResolveReport {
        overlay_used: false,
        ..Default::default()
    };

    let mut candidates: Vec<GraphEdgeView> = Vec::new();
    for seed in seed_ids {
        let nbrs = kg.neighbors(
            seed,
            Some(&["CALLS".into(), "REFERENCES".into()]),
            EdgeDirection::Both,
            100,
        )?;
        for n in nbrs {
            if !candidates.iter().any(|e| e.id == n.edge.id) {
                candidates.push(n.edge);
            }
        }
    }

    // Critical path: outgoing CALLS from seeds only
    if opts.critical_path_only {
        candidates.retain(|e| {
            e.kind == "CALLS" && seed_ids.iter().any(|s| s == &e.src)
        });
    }

    let precise: Vec<_> = candidates
        .iter()
        .filter(|e| e.confidence == "precise")
        .cloned()
        .collect();
    if !precise.is_empty() {
        report.overlay_used = true;
    }

    let mut ambiguous: Vec<_> = candidates
        .into_iter()
        .filter(|e| e.confidence != "precise")
        .collect();

    // Prefer unresolved first (highest value upgrades)
    ambiguous.sort_by_key(|e| {
        (
            !e.dst.starts_with("unresolved:"),
            e.id.clone(),
        )
    });

    for edge in ambiguous {
        if report.upgrades_considered >= opts.max_upgrades
            || started.elapsed().as_millis() as u64 >= opts.max_latency_ms
        {
            report.deferred += 1;
            continue;
        }
        report.upgrades_considered += 1;

        if let Some(p) = precise.iter().find(|p| edges_join_views(&edge, p)) {
            report.overlay_used = true;
            if p.dst != edge.dst {
                report.dual_candidates.push(DualCandidate {
                    site_src: edge.src.clone(),
                    heuristic_dst: edge.dst.clone(),
                    precise_dst: p.dst.clone(),
                    file_path: edge.file_path.clone(),
                });
            }
            report.confirmed.push(ConfirmedEdge {
                src: p.src.clone(),
                dst: p.dst.clone(),
                edge_id: p.id.clone(),
                file_path: p.file_path.clone(),
            });
        } else {
            // Look for any precise edge in store with same src (overlay may not be in seed hop)
            let more = kg.neighbors(
                &edge.src,
                Some(&["CALLS".into(), "REFERENCES".into()]),
                EdgeDirection::Outgoing,
                50,
            )?;
            if let Some(p) = more
                .iter()
                .map(|n| &n.edge)
                .find(|p| p.confidence == "precise" && edges_join_views(&edge, p))
            {
                report.overlay_used = true;
                if p.dst != edge.dst {
                    report.dual_candidates.push(DualCandidate {
                        site_src: edge.src.clone(),
                        heuristic_dst: edge.dst.clone(),
                        precise_dst: p.dst.clone(),
                        file_path: edge.file_path.clone(),
                    });
                }
                report.confirmed.push(ConfirmedEdge {
                    src: p.src.clone(),
                    dst: p.dst.clone(),
                    edge_id: p.id.clone(),
                    file_path: p.file_path.clone(),
                });
            } else {
                report.still_heuristic += 1;
            }
        }
    }

    // Already-precise edges count as confirmed without dual notes
    for p in precise {
        if !report.confirmed.iter().any(|c| c.edge_id == p.id) {
            report.confirmed.push(ConfirmedEdge {
                src: p.src.clone(),
                dst: p.dst.clone(),
                edge_id: p.id.clone(),
                file_path: p.file_path.clone(),
            });
        }
    }

    report.latency_ms = started.elapsed().as_millis() as u64;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use prism_ir::{
        edge_id, file_node_id, symbol_node_id, Confidence, EdgeKind, FactBundle, FactEdge, FactNode,
        NodeKind, Span, Tier,
    };
    use prism_store::{KgStore, SqliteKgStore};
    use tempfile::tempdir;

    #[test]
    fn confirms_precise_over_heuristic() {
        let dir = tempdir().unwrap();
        let mut kg = SqliteKgStore::open(dir.path().join("g.sqlite")).unwrap();
        let path = "app.py";
        let main = symbol_node_id(path, "function", "main", 0);
        let greet = symbol_node_id("lib.py", "function", "greet", 0);
        let mut bundle = FactBundle::new(path, "python", "test");
        bundle.nodes.push(FactNode {
            id: file_node_id(path),
            kind: NodeKind::File,
            name: Some(path.into()),
            file_path: Some(path.into()),
            span: None,
            language: Some("python".into()),
            analyzer: "test".into(),
            tier: Tier::T1,
            confidence: Confidence::Extracted,
            attrs: Default::default(),
        });
        bundle.nodes.push(FactNode {
            id: main.clone(),
            kind: NodeKind::Symbol,
            name: Some("main".into()),
            file_path: Some(path.into()),
            span: None,
            language: Some("python".into()),
            analyzer: "test".into(),
            tier: Tier::T1,
            confidence: Confidence::Extracted,
            attrs: Default::default(),
        });
        bundle.nodes.push(FactNode {
            id: greet.clone(),
            kind: NodeKind::Symbol,
            name: Some("greet".into()),
            file_path: Some("lib.py".into()),
            span: None,
            language: Some("python".into()),
            analyzer: "test".into(),
            tier: Tier::T2,
            confidence: Confidence::Precise,
            attrs: Default::default(),
        });
        let span = Span {
            start_byte: 10,
            end_byte: 15,
            start_line: 1,
            start_col: 0,
            end_line: 1,
            end_col: 5,
        };
        bundle.edges.push(FactEdge {
            id: edge_id(EdgeKind::Calls, &main, "unresolved:greet", 10),
            kind: EdgeKind::Calls,
            src: main.clone(),
            dst: "unresolved:greet".into(),
            file_path: Some(path.into()),
            span: Some(span),
            analyzer: "test".into(),
            tier: Tier::T1,
            confidence: Confidence::Heuristic,
            attrs: Default::default(),
        });
        bundle.edges.push(FactEdge {
            id: edge_id(EdgeKind::Calls, &main, &greet, 10),
            kind: EdgeKind::Calls,
            src: main.clone(),
            dst: greet.clone(),
            file_path: Some(path.into()),
            span: Some(span),
            analyzer: "precise".into(),
            tier: Tier::T2,
            confidence: Confidence::Precise,
            attrs: Default::default(),
        });
        kg.begin_replace_file_subgraph(path).unwrap();
        kg.insert_facts(path, &bundle).unwrap();
        kg.commit_replace_file_subgraph(path).unwrap();

        let report = hybrid_resolve(&kg, &[main], &HybridResolveOptions::default()).unwrap();
        assert!(!report.confirmed.is_empty());
        assert!(report.overlay_used);
        assert!(report.latency_ms <= DEFAULT_MAX_LATENCY_MS);
    }
}
