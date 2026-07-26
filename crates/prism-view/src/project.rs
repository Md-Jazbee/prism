//! Project KG / pack facts into a budgeted GraphView.

use crate::budget::ViewBudget;
use crate::kinds::ViewKind;
use crate::layout::{assign_coordinates, layout_seed};
use crate::model::{
    BudgetUsed, Citation, DropRecord, GraphView, LayoutInfo, ViewEdge, ViewGroup, ViewNode,
    ViewOutcome, ViewTooLarge, GRAPH_VIEW_SCHEMA_VERSION,
};
use anyhow::Result;
use prism_compile::{compile_context, CompileOutcome};
use prism_plan::PlanHints;
use prism_store::{EdgeDirection, SqliteKgStore};
use std::collections::BTreeSet;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct ViewParams {
    pub snapshot_id: String,
    pub seed_id: Option<String>,
    pub anchors: Vec<String>,
    pub question: Option<String>,
    pub path: Option<String>,
    pub max_nodes: Option<usize>,
    pub max_edges: Option<usize>,
}

struct Candidate {
    node: ViewNode,
    /// Lower = keep preferentially (seeds = 0).
    drop_priority: u32,
}

type ExpandResult = (Vec<Candidate>, Vec<ViewEdge>, Vec<ViewGroup>, Vec<String>);

/// Project a view. Refuses with `VIEW_TOO_LARGE` when seeds alone exceed budget
/// or candidate explosion exceeds `max_nodes * 20` at the coarsest expansion.
pub fn project_view(
    kg: &SqliteKgStore,
    workspace: &Path,
    kind: ViewKind,
    params: &ViewParams,
) -> Result<ViewOutcome> {
    let budget = ViewBudget {
        max_nodes: params.max_nodes.unwrap_or(crate::budget::DEFAULT_MAX_NODES),
        max_edges: params.max_edges.unwrap_or(crate::budget::DEFAULT_MAX_EDGES),
    }
    .clamp();

    let snapshot = if params.snapshot_id.is_empty() {
        "adhoc".into()
    } else {
        params.snapshot_id.clone()
    };

    let (mut candidates, mut edges, groups, notes) =
        expand(kg, workspace, kind, params, &snapshot)?;

    // Seeds must fit; otherwise refuse (never silently drop seeds).
    let seed_count = candidates.iter().filter(|c| c.drop_priority == 0).count();
    if seed_count > budget.max_nodes {
        let anchors = suggested_anchors(&candidates, params);
        return Ok(ViewOutcome::TooLarge(ViewTooLarge::new(
            kind.as_str(),
            Some(snapshot),
            seed_count,
            budget.max_nodes,
            anchors,
        )));
    }

    // Explosion guard: refuse rather than dump.
    let refuse_at = budget.max_nodes.saturating_mul(20).max(200);
    if candidates.len() > refuse_at {
        let anchors = suggested_anchors(&candidates, params);
        return Ok(ViewOutcome::TooLarge(ViewTooLarge::new(
            kind.as_str(),
            Some(snapshot),
            candidates.len(),
            budget.max_nodes,
            anchors,
        )));
    }

    // Deterministic drop order: higher drop_priority first, then id.
    candidates.sort_by(|a, b| {
        a.drop_priority
            .cmp(&b.drop_priority)
            .then(a.node.id.cmp(&b.node.id))
    });

    let mut drops = Vec::new();
    let mut kept: Vec<ViewNode> = Vec::new();
    for c in candidates {
        if kept.len() < budget.max_nodes {
            kept.push(c.node);
        } else {
            drops.push(DropRecord {
                id: c.node.id,
                reason: format!("over_max_nodes drop_priority={}", c.drop_priority),
            });
        }
    }

    let kept_ids: BTreeSet<_> = kept.iter().map(|n| n.id.clone()).collect();
    let mut kept_edges: Vec<ViewEdge> = edges
        .drain(..)
        .filter(|e| kept_ids.contains(&e.src) && kept_ids.contains(&e.dst))
        .collect();
    kept_edges.sort_by(|a, b| a.id.cmp(&b.id));
    if kept_edges.len() > budget.max_edges {
        for e in kept_edges.drain(budget.max_edges..) {
            drops.push(DropRecord {
                id: e.id,
                reason: "over_max_edges".into(),
            });
        }
    }

    let params_key = format!(
        "{}|{}|{:?}",
        params.seed_id.as_deref().unwrap_or(""),
        params.anchors.join(","),
        params.question.as_deref().unwrap_or("")
    );
    let seed = layout_seed(&snapshot, kind.as_str(), &params_key);
    let algo = kind.default_layout();
    assign_coordinates(&mut kept, algo, &seed);

    Ok(ViewOutcome::Ok(GraphView {
        schema_version: GRAPH_VIEW_SCHEMA_VERSION.into(),
        snapshot_id: snapshot,
        view_kind: kind.as_str().into(),
        nodes: kept.clone(),
        edges: kept_edges.clone(),
        groups,
        budget: BudgetUsed {
            max_nodes: budget.max_nodes,
            max_edges: budget.max_edges,
            nodes_used: kept.len(),
            edges_used: kept_edges.len(),
        },
        layout: LayoutInfo {
            algorithm: algo.into(),
            seed,
            notes: vec![
                "Coordinates are deterministic for identical (snapshot_id, view_kind, params, kept set)."
                    .into(),
            ],
        },
        drops,
        notes,
    }))
}

fn suggested_anchors(cands: &[Candidate], params: &ViewParams) -> Vec<String> {
    let mut out = params.anchors.clone();
    if let Some(s) = &params.seed_id {
        out.push(s.clone());
    }
    for c in cands.iter().filter(|c| c.drop_priority == 0).take(5) {
        out.push(c.node.id.clone());
    }
    out.sort();
    out.dedup();
    if out.is_empty() {
        out.push("provide a symbol or path anchor".into());
    }
    out
}

fn expand(
    kg: &SqliteKgStore,
    workspace: &Path,
    kind: ViewKind,
    params: &ViewParams,
    snapshot: &str,
) -> Result<ExpandResult> {
    match kind {
        ViewKind::ArchitectureMap => expand_architecture(kg),
        ViewKind::ImpactCone => expand_impact(kg, params),
        ViewKind::SlicePath => expand_slice_path(kg, params),
        ViewKind::PackMap => expand_pack_map(kg, workspace, params, snapshot),
        ViewKind::HotspotHeat => expand_hotspots(kg, workspace),
        ViewKind::LayeringViolations => expand_layering(kg),
        ViewKind::AmbiguityHeat => expand_ambiguity(kg),
    }
}

fn cite(id: &str, file: Option<&str>) -> Citation {
    Citation {
        node_ids: vec![id.into()],
        file_path: file.map(str::to_string),
        span: None,
    }
}

fn expand_architecture(kg: &SqliteKgStore) -> Result<ExpandResult> {
    let map = kg.repo_map(20)?;
    let mut cands = Vec::new();
    let mut groups = Vec::new();
    for (i, c) in map.communities.iter().enumerate() {
        groups.push(ViewGroup {
            id: c.id.clone(),
            label: c.label.clone(),
            kind: Some("community".into()),
        });
        cands.push(Candidate {
            drop_priority: 0,
            node: ViewNode {
                id: c.id.clone(),
                label: c.label.clone(),
                kind: "Community".into(),
                tier: "T1".into(),
                confidence: "heuristic".into(),
                lod_rank: 0,
                group: Some(c.id.clone()),
                citation: cite(&c.id, Some(&c.path_prefix)),
                x: 0.0,
                y: 0.0,
                heat: Some(c.file_count as f64),
            },
        });
        // Soft priority bump for later communities if budget tight — still seeds.
        let _ = i;
    }
    for (i, h) in map.hubs.iter().enumerate() {
        cands.push(Candidate {
            drop_priority: 1 + (i as u32 / 5),
            node: ViewNode {
                id: h.node_id.clone(),
                label: h.name.clone().unwrap_or_else(|| h.node_id.clone()),
                kind: h.kind.clone(),
                tier: "T1".into(),
                confidence: "heuristic".into(),
                lod_rank: 1,
                group: h
                    .file_path
                    .as_ref()
                    .map(|p| format!("comm:{}", path_prefix(p))),
                citation: cite(&h.node_id, h.file_path.as_deref()),
                x: 0.0,
                y: 0.0,
                heat: Some(h.degree as f64),
            },
        });
    }
    Ok((cands, Vec::new(), groups, map.notes))
}

fn path_prefix(path: &str) -> String {
    let p = path.replace('\\', "/");
    match p.rfind('/') {
        Some(i) => p[..=i].to_string(),
        None => "./".into(),
    }
}

fn expand_impact(kg: &SqliteKgStore, params: &ViewParams) -> Result<ExpandResult> {
    let seed = params
        .seed_id
        .clone()
        .or_else(|| params.anchors.first().cloned())
        .ok_or_else(|| anyhow::anyhow!("impact_cone requires seed_id or anchors"))?;
    let hits = kg.impact(&seed, 3, 200)?;
    let mut cands = vec![Candidate {
        drop_priority: 0,
        node: ViewNode {
            id: seed.clone(),
            label: seed.clone(),
            kind: "Symbol".into(),
            tier: "T1".into(),
            confidence: "extracted".into(),
            lod_rank: 0,
            group: None,
            citation: cite(&seed, None),
            x: 0.0,
            y: 0.0,
            heat: None,
        },
    }];
    let mut edges = Vec::new();
    for h in hits {
        cands.push(Candidate {
            drop_priority: h.depth,
            node: ViewNode {
                id: h.node.id.clone(),
                label: h.node.name.clone().unwrap_or_else(|| h.node.id.clone()),
                kind: h.node.kind.clone(),
                tier: "T1".into(),
                confidence: h.node.confidence.clone(),
                lod_rank: h.depth,
                group: None,
                citation: cite(&h.node.id, h.node.file_path.as_deref()),
                x: 0.0,
                y: 0.0,
                heat: Some(1.0 / (h.depth as f64 + 1.0)),
            },
        });
        let eid = format!("impact:{}:{}", seed, h.node.id);
        edges.push(ViewEdge {
            id: eid.clone(),
            src: seed.clone(),
            dst: h.node.id.clone(),
            kind: h.via.last().cloned().unwrap_or_else(|| "IMPACT".into()),
            tier: "T1".into(),
            confidence: "heuristic".into(),
            citation: cite(&eid, None),
        });
    }
    Ok((
        cands,
        edges,
        Vec::new(),
        vec!["Impact cone is HEURISTIC at T1 unless PreciseIndex is attached.".into()],
    ))
}

fn expand_slice_path(kg: &SqliteKgStore, params: &ViewParams) -> Result<ExpandResult> {
    // Approximate slice path via outgoing neighbors of seed (full interproc is HTTP /v1/slice).
    let seed = params
        .seed_id
        .clone()
        .or_else(|| params.anchors.first().cloned())
        .ok_or_else(|| anyhow::anyhow!("slice_path requires seed_id or anchors"))?;
    let neigh = kg.neighbors(&seed, None, EdgeDirection::Both, 40)?;
    let mut cands = vec![Candidate {
        drop_priority: 0,
        node: ViewNode {
            id: seed.clone(),
            label: seed.clone(),
            kind: "Symbol".into(),
            tier: "T1".into(),
            confidence: "extracted".into(),
            lod_rank: 0,
            group: None,
            citation: cite(&seed, params.path.as_deref()),
            x: 0.0,
            y: 0.0,
            heat: None,
        },
    }];
    let mut edges = Vec::new();
    for (i, n) in neigh.into_iter().enumerate() {
        cands.push(Candidate {
            drop_priority: 1,
            node: ViewNode {
                id: n.node.id.clone(),
                label: n.node.name.clone().unwrap_or_else(|| n.node.id.clone()),
                kind: n.node.kind.clone(),
                tier: "T1".into(),
                confidence: n.edge.confidence.clone(),
                lod_rank: 1,
                group: None,
                citation: cite(&n.node.id, n.node.file_path.as_deref()),
                x: 0.0,
                y: 0.0,
                heat: Some(1.0 - (i as f64) * 0.01),
            },
        });
        edges.push(ViewEdge {
            id: n.edge.id.clone(),
            src: n.edge.src.clone(),
            dst: n.edge.dst.clone(),
            kind: n.edge.kind.clone(),
            tier: "T1".into(),
            confidence: n.edge.confidence.clone(),
            citation: cite(&n.edge.id, n.edge.file_path.as_deref()),
        });
    }
    Ok((
        cands,
        edges,
        Vec::new(),
        vec!["slice_path view uses neighbor hops; prefer /v1/slice for interproc spans.".into()],
    ))
}

fn expand_pack_map(
    kg: &SqliteKgStore,
    workspace: &Path,
    params: &ViewParams,
    _snapshot: &str,
) -> Result<ExpandResult> {
    let question = params
        .question
        .clone()
        .unwrap_or_else(|| "architecture overview".into());
    let mut hints = PlanHints {
        anchors: params.anchors.clone(),
        budget_tokens: Some(2000),
        ..Default::default()
    };
    if let Some(s) = &params.seed_id {
        hints.anchors.push(s.clone());
    }
    let _ = kg; // pack compile opens its own kg handle
    let outcome = compile_context(workspace, &question, &hints)?;
    let pack = match outcome {
        CompileOutcome::Ok(p) => p,
        CompileOutcome::ScopeUnresolved(u) => {
            anyhow::bail!("SCOPE_UNRESOLVED: {}", u.reason);
        }
        CompileOutcome::BudgetExceeded(e) => {
            anyhow::bail!("BUDGET_EXCEEDED: {}", e.reason);
        }
    };
    let mut cands = Vec::new();
    for (i, f) in pack.fragments.iter().enumerate() {
        let id = f.id.clone();
        cands.push(Candidate {
            drop_priority: if f.must_include {
                0
            } else {
                1 + (i as u32 / 10)
            },
            node: ViewNode {
                id: id.clone(),
                label: format!("{:?}:{}", f.kind, f.id),
                kind: format!("{:?}", f.kind),
                tier: f.provenance.tier.clone(),
                confidence: f.confidence.clone(),
                lod_rank: match f.layer {
                    prism_compile::PackLayer::Arch => 0,
                    prism_compile::PackLayer::Mod => 1,
                    _ => 2,
                },
                group: Some(f.layer.as_str().into()),
                citation: Citation {
                    node_ids: f.provenance.node_ids.clone(),
                    file_path: None,
                    span: None,
                },
                x: 0.0,
                y: 0.0,
                heat: None,
            },
        });
    }
    Ok((
        cands,
        Vec::new(),
        Vec::new(),
        vec!["pack_map nodes are Evidence Pack fragments with provenance citations.".into()],
    ))
}

fn expand_hotspots(kg: &SqliteKgStore, workspace: &Path) -> Result<ExpandResult> {
    let intel = kg.repo_intel(Some(workspace), 15)?;
    let mut cands = Vec::new();
    for (i, h) in intel.hotspots.iter().enumerate() {
        let id = format!("hotspot:{}", h.path);
        cands.push(Candidate {
            drop_priority: if i < 5 { 0 } else { 1 },
            node: ViewNode {
                id: id.clone(),
                label: h.path.clone(),
                kind: "Hotspot".into(),
                tier: "T1".into(),
                confidence: "observed".into(),
                lod_rank: 1,
                group: None,
                citation: cite(&id, Some(&h.path)),
                x: 0.0,
                y: 0.0,
                heat: Some(h.score as f64),
            },
        });
    }
    Ok((
        cands,
        Vec::new(),
        Vec::new(),
        vec!["Hotspot heat from git history or degree fallback.".into()],
    ))
}

fn expand_layering(kg: &SqliteKgStore) -> Result<ExpandResult> {
    let intel = kg.repo_intel(None, 10)?;
    let mut cands = Vec::new();
    for (i, v) in intel.layering_violations.iter().enumerate() {
        let id = format!("layer:{}:{}", i, v.edge_id);
        let label = format!("{} → {} ({})", v.src_prefix, v.dst_prefix, v.kind);
        cands.push(Candidate {
            drop_priority: 0,
            node: ViewNode {
                id: id.clone(),
                label,
                kind: "LayeringViolation".into(),
                tier: "T1".into(),
                confidence: "heuristic".into(),
                lod_rank: 0,
                group: None,
                citation: cite(&v.edge_id, None),
                x: 0.0,
                y: 0.0,
                heat: Some(1.0),
            },
        });
    }
    if cands.is_empty() {
        cands.push(Candidate {
            drop_priority: 0,
            node: ViewNode {
                id: "layer:none".into(),
                label: "no layering violations detected".into(),
                kind: "Note".into(),
                tier: "T1".into(),
                confidence: "heuristic".into(),
                lod_rank: 0,
                group: None,
                citation: cite("layer:none", None),
                x: 0.0,
                y: 0.0,
                heat: None,
            },
        });
    }
    Ok((cands, Vec::new(), Vec::new(), intel.notes))
}

fn expand_ambiguity(kg: &SqliteKgStore) -> Result<ExpandResult> {
    let groups = kg.ambiguous_symbol_names(40)?;
    let mut cands = Vec::new();
    for (name, ids) in groups {
        for (id, path) in ids.iter().take(8) {
            cands.push(Candidate {
                drop_priority: 0,
                node: ViewNode {
                    id: id.clone(),
                    label: name.clone(),
                    kind: "AmbiguousSymbol".into(),
                    tier: "T1".into(),
                    confidence: "heuristic".into(),
                    lod_rank: 1,
                    group: Some(format!("ambig:{name}")),
                    citation: cite(id, path.as_deref()),
                    x: 0.0,
                    y: 0.0,
                    heat: Some(ids.len() as f64),
                },
            });
        }
    }
    cands.truncate(200);
    Ok((
        cands,
        Vec::new(),
        Vec::new(),
        vec![
            "Ambiguity heat groups duplicate symbol names — prefer T2 when claiming accuracy."
                .into(),
        ],
    ))
}
