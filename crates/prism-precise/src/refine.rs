//! Edge refinement: upgrade heuristic CALLS/REFERENCES when PreciseIndex matches.

use anyhow::Result;
use prism_ir::{Confidence, EdgeKind, FactBundle, FactEdge, Tier};
use prism_store::SqliteKgStore;
use serde_json::json;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RefineStats {
    pub matched: usize,
    pub unmatched: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OverlayApplyStats {
    pub refined: usize,
    pub inserted: usize,
    pub nodes_upserted: usize,
}

/// Pure refinement of an in-memory heuristic edge list against precise edges.
pub fn refine_edges(heuristic: &[FactEdge], precise: &[FactEdge]) -> (Vec<FactEdge>, RefineStats) {
    let mut out = heuristic.to_vec();
    let mut stats = RefineStats::default();
    let mut used = vec![false; precise.len()];

    for edge in &mut out {
        if edge.confidence == Confidence::Precise {
            continue;
        }
        if edge.kind != EdgeKind::Calls && edge.kind != EdgeKind::References {
            continue;
        }
        if let Some((idx, p)) = precise.iter().enumerate().find(|(i, p)| {
            !used[*i] && p.kind == edge.kind && edges_join(edge, p)
        }) {
            used[idx] = true;
            edge.dst = p.dst.clone();
            edge.confidence = Confidence::Precise;
            edge.tier = Tier::T2;
            edge.analyzer = p.analyzer.clone();
            if let Some(span) = &p.span {
                edge.span = Some(*span);
            }
            edge.attrs
                .insert("refined_from".into(), json!("heuristic"));
            edge.attrs
                .insert("precise_analyzer".into(), json!(p.analyzer));
            stats.matched += 1;
        }
    }

    for (i, p) in precise.iter().enumerate() {
        if used[i] {
            continue;
        }
        if p.kind != EdgeKind::Calls && p.kind != EdgeKind::References {
            continue;
        }
        // Unmatched precise edges are appended (callers may insert into store).
        out.push(p.clone());
        stats.unmatched += 1;
    }

    (out, stats)
}

fn edges_join(heuristic: &FactEdge, precise: &FactEdge) -> bool {
    edges_join_ids(
        &heuristic.src,
        &heuristic.dst,
        heuristic.file_path.as_deref(),
        heuristic.span.as_ref().map(|s| (s.start_byte, s.end_byte)),
        heuristic.attrs.get("callee").and_then(|v| v.as_str()),
        &precise.src,
        &precise.dst,
        precise.file_path.as_deref(),
        precise.span.as_ref().map(|s| (s.start_byte, s.end_byte)),
    )
}

/// Join heuristic vs precise edge *views* (store rows) using the same Stage A rules.
pub fn edges_join_views(
    heuristic: &prism_store::GraphEdgeView,
    precise: &prism_store::GraphEdgeView,
) -> bool {
    edges_join_ids(
        &heuristic.src,
        &heuristic.dst,
        heuristic.file_path.as_deref(),
        None,
        None,
        &precise.src,
        &precise.dst,
        precise.file_path.as_deref(),
        None,
    )
}

#[allow(clippy::too_many_arguments)]
fn edges_join_ids(
    h_src: &str,
    h_dst: &str,
    h_file: Option<&str>,
    h_span: Option<(u32, u32)>,
    h_callee: Option<&str>,
    p_src: &str,
    p_dst: &str,
    p_file: Option<&str>,
    p_span: Option<(u32, u32)>,
) -> bool {
    if h_src == p_src && h_dst == p_dst {
        return true;
    }
    let same_file = h_file == p_file;
    if same_file {
        if let (Some((asb, aeb)), Some((bsb, beb))) = (h_span, p_span) {
            if asb < beb && bsb < aeb && h_src == p_src {
                return true;
            }
        }
    }
    if same_file && h_src == p_src {
        let hn = h_callee
            .map(|s| s.to_string())
            .or_else(|| name_from_id(h_dst));
        let pn = name_from_id(p_dst);
        if let (Some(a), Some(b)) = (hn, pn) {
            if a == b {
                return true;
            }
        }
        if let Some(un) = h_dst.strip_prefix("unresolved:") {
            if p_dst.contains(&format!(":{un}:")) || p_dst.ends_with(un) {
                return true;
            }
        }
    }
    false
}

fn name_from_id(id: &str) -> Option<String> {
    if let Some(rest) = id.strip_prefix("unresolved:") {
        return Some(rest.to_string());
    }
    let parts: Vec<&str> = id.split(':').collect();
    if parts.len() >= 5 && parts[0] == "sym" {
        return Some(parts[parts.len() - 2].to_string());
    }
    None
}

/// Upsert T2 nodes/edges and refine matching heuristic edges already in the store.
pub fn apply_overlay_to_store(
    kg: &mut SqliteKgStore,
    bundle: &FactBundle,
) -> Result<OverlayApplyStats> {
    let mut stats = OverlayApplyStats::default();

    // Collect existing heuristic CALLS/REFERENCES that may join.
    let mut heuristic = Vec::new();
    for edge in &bundle.edges {
        if edge.kind != EdgeKind::Calls && edge.kind != EdgeKind::References {
            continue;
        }
        let existing = kg.edges_for_file(edge.file_path.as_deref().unwrap_or(""))?;
        for e in existing {
            if e.confidence == "heuristic"
                && (e.kind == "CALLS" || e.kind == "REFERENCES" || e.kind == "Calls" || e.kind == "References")
            {
                if let Some(fe) = kg.load_fact_edge(&e.id)? {
                    if !heuristic.iter().any(|h: &FactEdge| h.id == fe.id) {
                        heuristic.push(fe);
                    }
                }
            }
        }
    }

    // Also scan all files referenced by symbols.
    for node in &bundle.nodes {
        if let Some(fp) = &node.file_path {
            for e in kg.edges_for_file(fp)? {
                if e.confidence == "heuristic"
                    && (e.kind == "CALLS" || e.kind == "Calls" || e.kind == "REFERENCES" || e.kind == "References")
                {
                    if let Some(fe) = kg.load_fact_edge(&e.id)? {
                        if !heuristic.iter().any(|h: &FactEdge| h.id == fe.id) {
                            heuristic.push(fe);
                        }
                    }
                }
            }
        }
    }

    let (refined_list, refine_stats) = refine_edges(&heuristic, &bundle.edges);
    stats.refined = refine_stats.matched;

    // Upsert all precise nodes.
    kg.upsert_overlay_nodes(&bundle.nodes)?;
    stats.nodes_upserted = bundle.nodes.len();

    // Write refined heuristic edges + unmatched precise edges.
    let mut written = std::collections::HashSet::new();
    for edge in &refined_list {
        if edge.confidence == Confidence::Precise {
            kg.upsert_overlay_edge(edge)?;
            written.insert(edge.id.clone());
            if heuristic.iter().any(|h| h.id == edge.id) {
                // refined in place
            } else {
                stats.inserted += 1;
            }
        }
    }
    // Ensure all precise bundle edges exist.
    for edge in &bundle.edges {
        if !written.contains(&edge.id) {
            kg.upsert_overlay_edge(edge)?;
            stats.inserted += 1;
        }
    }

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use prism_ir::{edge_id, Confidence, EdgeKind, FactEdge, Span, Tier};

    fn edge(
        kind: EdgeKind,
        src: &str,
        dst: &str,
        file: &str,
        start: u32,
        conf: Confidence,
        tier: Tier,
    ) -> FactEdge {
        FactEdge {
            id: edge_id(kind, src, dst, start),
            kind,
            src: src.into(),
            dst: dst.into(),
            file_path: Some(file.into()),
            span: Some(Span {
                start_byte: start,
                end_byte: start + 5,
                start_line: 0,
                start_col: 0,
                end_line: 0,
                end_col: 5,
            }),
            analyzer: "test".into(),
            tier,
            confidence: conf,
            attrs: {
                let mut m = serde_json::Map::new();
                if let Some(name) = dst.strip_prefix("unresolved:") {
                    m.insert("callee".into(), json!(name));
                }
                m
            },
        }
    }

    #[test]
    fn upgrades_unresolved_by_name() {
        let h = edge(
            EdgeKind::Calls,
            "sym:a.py:function:main:0",
            "unresolved:greet",
            "a.py",
            10,
            Confidence::Heuristic,
            Tier::T1,
        );
        let p = edge(
            EdgeKind::Calls,
            "sym:a.py:function:main:0",
            "sym:lib.py:function:greet:0",
            "a.py",
            10,
            Confidence::Precise,
            Tier::T2,
        );
        let (out, stats) = refine_edges(&[h], &[p]);
        assert_eq!(stats.matched, 1);
        assert_eq!(out[0].dst, "sym:lib.py:function:greet:0");
        assert_eq!(out[0].confidence, Confidence::Precise);
        assert_eq!(out[0].tier, Tier::T2);
    }
}
