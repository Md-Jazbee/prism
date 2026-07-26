//! Selection: plan (+ optional KG) → candidate fragments.

use crate::fragment::{estimate_tokens, CandidateFragment, FragmentKind, PackLayer, Provenance};
use crate::gap::{EvidenceGap, WhyAbsent};
use crate::path_class::path_allowed;
use anyhow::Result;
use prism_obs::{emit_index_event, IndexEvent};
use prism_plan::{Intent, Operator, Plan};
use prism_precise::{
    ambiguity_index, hybrid_resolve, HybridResolveOptions, HybridResolveReport,
    DEFAULT_MAX_LATENCY_MS, DEFAULT_MAX_UPGRADES,
};
use prism_store::{EdgeDirection, SqliteKgStore};
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct CompileOptions {
    pub workspace: Option<PathBuf>,
}

/// Live KG selection output (P12 Stage B): real fragments + honest gaps.
#[derive(Debug, Clone, Default)]
pub struct SelectionOutcome {
    pub candidates: Vec<CandidateFragment>,
    pub gaps: Vec<EvidenceGap>,
}

/// Offline / synthetic candidates from recipe roles (no KG).
///
/// Produces one fragment per `must_include` role plus a few optional neighbors
/// so budget/drop behavior is testable without an index.
///
/// **Live packs must not use this path** — see [`select_from_kg`]. Placeholders
/// here exist only for offline EXPLAIN / budget invariant demos.
pub fn select_candidates(plan: &Plan) -> Vec<CandidateFragment> {
    let anchor = first_anchor(plan).unwrap_or_else(|| "unknown".into());
    let mut out = Vec::new();

    for (i, role) in plan.must_include.iter().enumerate() {
        let (kind, layer, why, text) = role_template(plan.intent, role, &anchor);
        let token_estimate = estimate_tokens(&text);
        out.push(CandidateFragment {
            id: format!("frag:must:{i}:{role}"),
            kind,
            layer,
            text,
            token_estimate,
            provenance: Provenance::synthetic(role),
            confidence: "extracted".into(),
            why_included: why,
            drop_priority: 0,
            roles: vec![role.clone()],
            must_include: true,
        });
    }

    // Optional fragments aligned to drop_order priorities
    for (i, drop_key) in plan.drop_order.iter().enumerate() {
        let prio = 10 + (i as u32) * 10;
        let text = format!("[optional:{drop_key}] related context for {anchor}");
        out.push(CandidateFragment {
            id: format!("frag:opt:{i}:{drop_key}"),
            kind: FragmentKind::Signature,
            layer: PackLayer::Nbr,
            text: text.clone(),
            token_estimate: estimate_tokens(&text),
            provenance: Provenance::synthetic(drop_key),
            confidence: "heuristic".into(),
            why_included: format!("optional:{drop_key}"),
            drop_priority: prio,
            roles: vec![drop_key.clone()],
            must_include: false,
        });
    }

    // Architecture always gets a tiny community stub if not already must-included
    if matches!(plan.intent, Intent::Architecture)
        && !out
            .iter()
            .any(|c| c.roles.iter().any(|r| r == "community_map"))
    {
        let text = "communities: (synthetic path-prefix map)".to_string();
        out.push(CandidateFragment {
            id: "frag:arch:community".into(),
            kind: FragmentKind::Community,
            layer: PackLayer::Arch,
            text: text.clone(),
            token_estimate: estimate_tokens(&text),
            provenance: Provenance::synthetic("community"),
            confidence: "heuristic".into(),
            why_included: "community_map".into(),
            drop_priority: 0,
            roles: vec!["community_map".into()],
            must_include: true,
        });
    }

    out
}

/// Select candidates by executing cheap T1 (+ bounded T2 upgrade) plan steps against the KG.
///
/// P12 Stage B: does **not** prepend `role_template` placeholders. Unfilled roles
/// become [`EvidenceGap`]s. Synthetic-only fragments are stripped before return.
pub fn select_from_kg(
    kg: &SqliteKgStore,
    plan: &Plan,
    opts: &CompileOptions,
) -> Result<SelectionOutcome> {
    let mut out = Vec::new();
    let mut gaps: Vec<EvidenceGap> = Vec::new();
    let anchors = anchors_from_plan(plan);
    let mut gaps_extra: Vec<String> = Vec::new();

    // P12: no select_candidates() prepend — live packs cite real nodes only.
    out.extend(select_doc_prose(kg, opts, plan, &anchors)?);

    let mut seed_ids: Vec<String> = Vec::new();

    // Enrich with live KG hits where possible
    for step in &plan.steps {
        if !step.executable && !matches!(step.op, Operator::BudgetPack) {
            continue;
        }
        match step.op {
            Operator::ResolveSymbol => {
                for a in &anchors {
                    let name = strip_qual(a);
                    let hits = kg.resolve_symbol(name, None, 5)?;
                    let hits: Vec<_> = hits
                        .into_iter()
                        .filter(|h| path_allowed(h.file_path.as_deref(), &anchors))
                        .collect();
                    if hits.is_empty() {
                        gaps.push(
                            EvidenceGap::new(
                                "primary_symbol_definition",
                                WhyAbsent::SeedUnresolved,
                                format!(
                                    "Could not resolve `{name}` — pick a symbol/path from resolve_symbol / repo_map"
                                ),
                            )
                            .with_detail(a.clone()),
                        );
                        continue;
                    }
                    for (i, hit) in hits.iter().enumerate() {
                        if !seed_ids.iter().any(|s| s == &hit.id) {
                            seed_ids.push(hit.id.clone());
                        }
                        let text = format_symbol_signature(hit);
                        let slice = maybe_read_slice(opts, hit);
                        if let Some((slice_text, tokens)) = slice {
                            out.push(CandidateFragment {
                                id: format!("frag:kg:def:{}:{i}", hit.id),
                                kind: FragmentKind::Slice,
                                layer: PackLayer::Core,
                                text: slice_text,
                                token_estimate: tokens,
                                provenance: Provenance::from_node_tier(
                                    &hit.id,
                                    "prism-store",
                                    tier_for_confidence(&hit.confidence),
                                ),
                                confidence: hit.confidence.clone(),
                                why_included: "primary_symbol_definition".into(),
                                drop_priority: 0,
                                roles: vec![
                                    "primary_symbol_definition".into(),
                                    "target_symbol_definition".into(),
                                    "seed_symbols".into(),
                                    "primary_frame_body".into(),
                                    "insertion_neighborhood".into(),
                                ],
                                must_include: true,
                            });
                        }
                        out.push(CandidateFragment {
                            id: format!("frag:kg:sig:{}:{i}", hit.id),
                            kind: FragmentKind::Signature,
                            layer: PackLayer::Mod,
                            text,
                            token_estimate: estimate_tokens(&format_symbol_signature(hit)),
                            provenance: Provenance::from_node_tier(
                                &hit.id,
                                "prism-store",
                                tier_for_confidence(&hit.confidence),
                            ),
                            confidence: hit.confidence.clone(),
                            why_included: "primary_symbol_signature".into(),
                            drop_priority: 0,
                            roles: vec![
                                "primary_symbol_signature".into(),
                                "type_signatures".into(),
                                "reference_list".into(),
                            ],
                            must_include: true,
                        });

                        // 1-hop neighbors as optional signatures (prefer precise)
                        let mut nbrs = kg.neighbors(
                            &hit.id,
                            Some(&[
                                "CALLS".into(),
                                "IMPORTS".into(),
                                "DEFINES".into(),
                                "REFERENCES".into(),
                            ]),
                            EdgeDirection::Both,
                            15,
                        )?;
                        nbrs.retain(|n| path_allowed(n.node.file_path.as_deref(), &anchors));
                        nbrs.sort_by(|a, b| {
                            confidence_rank(&b.edge.confidence)
                                .cmp(&confidence_rank(&a.edge.confidence))
                                .then_with(|| a.edge.id.cmp(&b.edge.id))
                        });
                        for (j, n) in nbrs.iter().enumerate() {
                            let t = format!(
                                "neighbor {} via {} → {}",
                                n.edge.kind,
                                n.edge.confidence,
                                format_symbol_signature(&n.node)
                            );
                            out.push(CandidateFragment {
                                id: format!("frag:kg:nbr:{}:{j}", n.node.id),
                                kind: FragmentKind::Signature,
                                layer: PackLayer::Nbr,
                                text: t.clone(),
                                token_estimate: estimate_tokens(&t),
                                provenance: Provenance {
                                    node_ids: vec![n.node.id.clone()],
                                    edge_ids: vec![n.edge.id.clone()],
                                    analyzer: "prism-store".into(),
                                    tier: tier_for_confidence(&n.edge.confidence).into(),
                                },
                                confidence: n.edge.confidence.clone(),
                                why_included: "neighbor_signature".into(),
                                drop_priority: if n.edge.confidence == "precise" {
                                    20
                                } else {
                                    40
                                },
                                roles: vec!["neighbor_bodies".into()],
                                must_include: false,
                            });
                        }
                    }
                }
            }
            Operator::UpgradePrecision => {
                let policy = step
                    .inputs
                    .get("policy")
                    .and_then(|v| v.as_str())
                    .unwrap_or("mandatory");
                let critical = step
                    .inputs
                    .get("critical_path_only")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let max_upgrades =
                    step.inputs
                        .get("max_upgrades")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(DEFAULT_MAX_UPGRADES as u64) as usize;
                let max_latency_ms = step
                    .inputs
                    .get("max_latency_ms")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(DEFAULT_MAX_LATENCY_MS);

                let should_run = match policy {
                    "optional_on_ambiguity" => ambiguity_index(kg)?.require_t2,
                    _ => true,
                };

                if should_run && !seed_ids.is_empty() {
                    let report = hybrid_resolve(
                        kg,
                        &seed_ids,
                        &HybridResolveOptions {
                            critical_path_only: critical,
                            max_upgrades,
                            max_latency_ms,
                        },
                    )?;
                    emit_upgrade_event(&report);
                    out.extend(fragments_from_upgrade(&report));
                    for d in &report.dual_candidates {
                        gaps_extra.push(format!(
                            "uncertainty: dual callee for {} heuristic={} precise={}",
                            d.site_src, d.heuristic_dst, d.precise_dst
                        ));
                    }
                    if report.deferred > 0 {
                        gaps_extra.push(format!(
                            "UpgradePrecision deferred {} edges (latency/count budget)",
                            report.deferred
                        ));
                    }
                    if !report.overlay_used {
                        gaps_extra.push(
                            "UpgradePrecision ran but no precise overlay edges matched — heuristic remains labeled"
                                .into(),
                        );
                    }
                } else if policy == "optional_on_ambiguity" {
                    gaps_extra.push(
                        "UpgradePrecision skipped: ambiguity index below require_t2 threshold"
                            .into(),
                    );
                }
            }
            Operator::Slice => {
                if let Some(root) = opts.workspace.as_ref() {
                    let max_depth = step
                        .inputs
                        .get("max_depth")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(2) as u32;
                    let max_functions = step
                        .inputs
                        .get("max_functions")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(16) as usize;
                    let max_spans = step
                        .inputs
                        .get("max_spans")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(40) as usize;
                    let residual_expand = step
                        .inputs
                        .get("residual_expand")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true);
                    let direction = match step
                        .inputs
                        .get("direction")
                        .and_then(|v| v.as_str())
                        .unwrap_or("backward")
                    {
                        "forward" => prism_semantic::SliceDirection::Forward,
                        _ => prism_semantic::SliceDirection::Backward,
                    };

                    // Prefer explicit path:line anchors; else resolve first symbol hit.
                    let mut ran = false;
                    for a in &anchors {
                        if let Some((path, line)) = parse_path_line(a) {
                            let params = prism_semantic::SliceParams {
                                direction,
                                max_depth,
                                max_functions,
                                max_spans,
                                residual_expand,
                                path,
                                line: Some(line),
                                symbol: None,
                                snapshot_id: "adhoc".into(),
                            };
                            match prism_semantic::interproc_slice(root, &params) {
                                Ok(report) => {
                                    emit_slice_event(&report);
                                    out.extend(fragments_from_slice(&report));
                                    if report.truncated {
                                        gaps_extra.push(format!(
                                            "Slice truncated depth={} residual={}",
                                            report.depth_reached,
                                            report.residual.len()
                                        ));
                                    }
                                    ran = true;
                                    break;
                                }
                                Err(e) => {
                                    gaps_extra.push(format!("Slice partial: {}", e.message));
                                }
                            }
                        }
                    }
                    if !ran {
                        for a in &anchors {
                            let name = strip_qual(a);
                            let hits = kg.resolve_symbol(name, None, 1)?;
                            if let Some(hit) = hits.first() {
                                if let Some(path) = hit.file_path.clone() {
                                    let params = prism_semantic::SliceParams {
                                        direction,
                                        max_depth,
                                        max_functions,
                                        max_spans,
                                        residual_expand,
                                        path,
                                        line: None,
                                        symbol: hit.name.clone(),
                                        snapshot_id: "adhoc".into(),
                                    };
                                    match prism_semantic::interproc_slice(root, &params) {
                                        Ok(report) => {
                                            emit_slice_event(&report);
                                            out.extend(fragments_from_slice(&report));
                                            if report.truncated {
                                                gaps_extra.push(format!(
                                                    "Slice truncated depth={} residual={}",
                                                    report.depth_reached,
                                                    report.residual.len()
                                                ));
                                            }
                                            ran = true;
                                            break;
                                        }
                                        Err(e) => {
                                            gaps_extra
                                                .push(format!("Slice partial: {}", e.message));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if !ran {
                        gaps_extra.push(
                            "Slice skipped: no path:line or resolvable symbol criterion".into(),
                        );
                    }
                } else {
                    gaps_extra.push("Slice skipped: no workspace in CompileOptions".into());
                }
            }
            Operator::Impact => {
                for a in &anchors {
                    let name = strip_qual(a);
                    let seeds = kg.resolve_symbol(name, None, 3)?;
                    for seed in seeds {
                        let hits = kg.impact(&seed.id, 2, 40)?;
                        for (i, h) in hits.iter().enumerate() {
                            let t = format!(
                                "impact depth={} {} via {:?}",
                                h.depth,
                                format_symbol_signature(&h.node),
                                h.via
                            );
                            let must = h.depth <= 1;
                            let conf = if h.node.confidence == "precise" {
                                "precise"
                            } else {
                                "heuristic"
                            };
                            out.push(CandidateFragment {
                                id: format!("frag:kg:impact:{}:{i}", h.node.id),
                                kind: FragmentKind::Signature,
                                layer: if must {
                                    PackLayer::Core
                                } else {
                                    PackLayer::Nbr
                                },
                                text: t.clone(),
                                token_estimate: estimate_tokens(&t),
                                provenance: Provenance::from_node_tier(
                                    &h.node.id,
                                    "prism-store",
                                    tier_for_confidence(conf),
                                ),
                                confidence: conf.into(),
                                why_included: if must {
                                    "impact_cone_depth_1".into()
                                } else {
                                    "impact_cone_depth_2plus".into()
                                },
                                drop_priority: if must { 0 } else { 50 + h.depth * 10 },
                                roles: if must {
                                    vec!["impact_cone_depth_1".into(), "seed_symbols".into()]
                                } else {
                                    vec!["depth_3_plus_impact".into()]
                                },
                                must_include: must,
                            });
                        }
                    }
                }
            }
            Operator::CommunityOf => {
                let map = kg.repo_map(10)?;
                let eps = kg.detect_entrypoints(8).unwrap_or_default();
                let summary = format!(
                    "algorithm={} communities={} hubs={} bridges={} entrypoints={}",
                    map.algorithm,
                    map.communities.len(),
                    map.hubs.len(),
                    map.bridges.len(),
                    eps.len()
                );
                let mut lines = vec![summary];
                for c in map.communities.iter().take(5) {
                    lines.push(format!(
                        "  community {} files≈{} label={}",
                        c.id, c.file_count, c.label
                    ));
                }
                for h in map.hubs.iter().take(3) {
                    lines.push(format!("  hub {} degree={}", h.node_id, h.degree));
                }
                for e in eps.iter().take(3) {
                    lines.push(format!(
                        "  entry {} ({})",
                        e.name.as_deref().unwrap_or(&e.node_id),
                        e.reason
                    ));
                }
                let text = lines.join("\n");
                let mut node_ids: Vec<String> = map
                    .communities
                    .iter()
                    .take(5)
                    .map(|c| c.id.clone())
                    .collect();
                node_ids.extend(map.hubs.iter().take(3).map(|h| h.node_id.clone()));
                if node_ids.is_empty() {
                    node_ids.push("repo_map:empty".into());
                }
                out.push(CandidateFragment {
                    id: "frag:kg:repo_map".into(),
                    kind: FragmentKind::Community,
                    layer: PackLayer::Arch,
                    text: text.clone(),
                    token_estimate: estimate_tokens(&text),
                    provenance: Provenance {
                        node_ids,
                        edge_ids: vec![],
                        analyzer: "prism-store".into(),
                        tier: "T1".into(),
                    },
                    confidence: "heuristic".into(),
                    why_included: "community_map".into(),
                    drop_priority: 0,
                    roles: vec!["community_map".into(), "hub_nodes".into()],
                    must_include: true,
                });
            }
            Operator::DiffIntersect => {
                for (i, a) in anchors.iter().enumerate() {
                    if a.contains('/') || a.ends_with(".py") || a.ends_with(".rs") {
                        let text = format!("diff seed path: {a}");
                        out.push(CandidateFragment {
                            id: format!("frag:diff:{i}"),
                            kind: FragmentKind::Diff,
                            layer: PackLayer::Diff,
                            text: text.clone(),
                            token_estimate: estimate_tokens(&text),
                            provenance: Provenance::from_node(format!("path:{a}"), "prism-compile"),
                            confidence: "extracted".into(),
                            why_included: "diff_hunks".into(),
                            drop_priority: 0,
                            roles: vec!["diff_hunks".into()],
                            must_include: true,
                        });
                    }
                }
            }
            _ => {}
        }
    }

    // Error / stack verbatim for debug
    if matches!(plan.intent, Intent::Debug) {
        for a in &anchors {
            if a.contains("Error") || a.contains("error") || a.contains(':') {
                let text = a.clone();
                out.push(CandidateFragment {
                    id: format!("frag:error:{}", out.len()),
                    kind: FragmentKind::ErrorVerbatim,
                    layer: PackLayer::Core,
                    text: text.clone(),
                    token_estimate: estimate_tokens(&text),
                    provenance: Provenance::from_node("error:verbatim", "prism-compile"),
                    confidence: "extracted".into(),
                    why_included: "error_or_stack_verbatim".into(),
                    drop_priority: 0,
                    roles: vec!["error_or_stack_verbatim".into()],
                    must_include: true,
                });
            }
        }
    }

    // Convert free-form upgrade/slice notes into structured gaps (not fragments).
    for g in gaps_extra {
        gaps.push(EvidenceGap::from_plan_note(&g));
    }

    // Prefer precise over heuristic when same fragment id
    out.sort_by(|a, b| {
        a.id.cmp(&b.id)
            .then_with(|| confidence_rank(&b.confidence).cmp(&confidence_rank(&a.confidence)))
    });
    out.dedup_by(|a, b| a.id == b.id);

    // Strip synthetic-only placeholders (ACC-2).
    out.retain(|c| !c.provenance.is_synthetic_only());

    // Re-apply must_include from plan roles
    for c in &mut out {
        if plan.must_include.iter().any(|r| c.roles.contains(r)) {
            c.must_include = true;
            c.drop_priority = 0;
        }
    }

    Ok(SelectionOutcome {
        candidates: out,
        gaps,
    })
}

/// Pull extractive Doc/Section spans for prose roles (P12 Stage A+B).
///
/// Caps volume aggressively: at most one `product_thesis` must-include (README
/// preferred) plus a few optional architecture_prose sections. Never attach
/// code roles like `primary_symbol_definition` to doc fragments.
fn select_doc_prose(
    kg: &SqliteKgStore,
    opts: &CompileOptions,
    plan: &Plan,
    anchors: &[String],
) -> Result<Vec<CandidateFragment>> {
    let want_prose = matches!(plan.intent, Intent::Architecture | Intent::RepoQa)
        || anchors.iter().any(|a| {
            let l = a.to_ascii_lowercase();
            l.ends_with(".md") || l.contains("readme") || l.starts_with("docs/")
        });
    if !want_prose {
        return Ok(Vec::new());
    }

    // Prefer Docs; Sections are optional fillers (ORDER BY kind would starve
    // later Docs if we mixed kinds under a small LIMIT).
    let mut docs = kg.list_nodes_by_kinds(&["Doc"], 200)?;
    docs.extend(kg.list_nodes_by_kinds(&["Section"], 40)?);
    let mut out = Vec::new();
    let root = opts.workspace.as_ref();

    // Prefer product-facing docs for thesis; score lower = better.
    // Question tokens boost relevant docs; anchors must not steal product_thesis.
    let question = plan.question.as_str();
    let mut ranked: Vec<(i32, &prism_store::GraphNodeView)> = docs
        .iter()
        .filter(|n| path_allowed(n.file_path.as_deref(), anchors))
        .map(|n| (doc_priority(n, anchors, question), n))
        .collect();
    ranked.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.id.cmp(&b.1.id)));

    let mut thesis_taken = false;
    let mut prose_count = 0usize;
    // Keep packs lean for ACC-5 (≤½ Graphify budget) while covering thesis + key docs.
    const MAX_PROSE: usize = 4;

    for (prio, node) in ranked {
        if prose_count >= MAX_PROSE {
            break;
        }
        // Skip low-priority noise unless explicitly anchored.
        if prio > 50 && !anchors.is_empty() {
            let anchored = node
                .file_path
                .as_ref()
                .map(|p| crate::path_class::anchor_covers_path(anchors, p))
                .unwrap_or(false);
            if !anchored {
                continue;
            }
        }
        if prio > 80 {
            continue;
        }

        let text = match (root, node.file_path.as_deref()) {
            (Some(ws), Some(path)) => read_doc_excerpt(ws, path, node),
            _ => format!(
                "{} {} {}",
                node.kind,
                node.name.as_deref().unwrap_or(""),
                node.file_path.as_deref().unwrap_or("")
            ),
        };

        // Thesis is reserved for product-facing Docs (README / ADD / setup),
        // never a random lexically-grounded architecture note.
        let is_thesis_candidate = !thesis_taken
            && node.kind == "Doc"
            && is_product_thesis_path(node.file_path.as_deref());
        let role = if is_thesis_candidate {
            "product_thesis"
        } else {
            "architecture_prose"
        };
        let must = is_thesis_candidate && plan.must_include.iter().any(|r| r == "product_thesis");

        if is_thesis_candidate {
            thesis_taken = true;
        }
        prose_count += 1;

        out.push(CandidateFragment {
            id: format!("frag:doc:{}:{}", node.id, prose_count),
            kind: FragmentKind::Slice,
            layer: if role == "product_thesis" {
                PackLayer::Arch
            } else {
                PackLayer::Core
            },
            text: text.clone(),
            token_estimate: estimate_tokens(&text),
            provenance: Provenance::from_node(&node.id, "prism-extract-markdown"),
            confidence: "asserted".into(),
            why_included: role.into(),
            drop_priority: if must { 0 } else { 20 + prose_count as u32 },
            roles: vec![role.into()],
            must_include: must,
        });
    }
    Ok(out)
}

/// Paths eligible for the single `product_thesis` slot (ACC-1).
fn is_product_thesis_path(path: Option<&str>) -> bool {
    let Some(path) = path else {
        return false;
    };
    let lower = path.replace('\\', "/").to_ascii_lowercase();
    lower == "readme.md"
        || lower == "agents.md"
        || lower.contains("architecture-design-document")
        || lower.contains("product-setup")
}

fn doc_priority(node: &prism_store::GraphNodeView, anchors: &[String], question: &str) -> i32 {
    let path = node.file_path.as_deref().unwrap_or("").replace('\\', "/");
    let lower = path.to_ascii_lowercase();
    let q = question.to_ascii_lowercase();
    let is_doc = node.kind == "Doc";
    let name = node.name.as_deref().unwrap_or("").to_ascii_lowercase();

    // Question-topic boosts (before generic ranking).
    if is_doc {
        if (q.contains("workflow")
            || q.contains("agent")
            || q.contains("refusal")
            || q.contains("scope_unresolved"))
            && (lower == "agents.md" || lower.contains("agent-usage") || lower.contains("refusal"))
        {
            return 2;
        }
        if (q.contains("mcp") || q.contains("tool"))
            && (lower.contains("mcp-tool") || lower.contains("agent-usage"))
        {
            return 2;
        }
        if (q.contains("install")
            || q.contains("setup")
            || q.contains("bootstrap")
            || q.contains("cold"))
            && (lower.contains("product-setup") || lower == "readme.md")
        {
            return 2;
        }
        if (q.contains("non-goal")
            || q.contains("not trying")
            || q.contains("precision tier")
            || q.contains("confidence"))
            && lower.contains("architecture-design-document")
        {
            return 2;
        }
        if q.contains("phase 12") || q.contains("graphify") || q.contains("accuracy") {
            if lower.contains("planning-and-implementation")
                || lower.contains("repo-feature-summary")
                || lower.contains("p12")
            {
                return 3;
            }
        }
    }

    // Product thesis candidates outrank lexical anchors so README wins ACC-1.
    if is_doc && lower == "readme.md" {
        return 1;
    }
    if is_doc && lower == "agents.md" {
        return 4;
    }
    if is_doc && (lower.contains("architecture-design-document") || lower.contains("product-setup"))
    {
        return 5;
    }
    if anchors
        .iter()
        .any(|a| crate::path_class::anchor_covers_path(std::slice::from_ref(a), &path))
    {
        return 8;
    }
    // Light lexical overlap on path/name vs question tokens.
    let mut overlap = 0i32;
    for tok in q.split(|c: char| !c.is_ascii_alphanumeric()) {
        if tok.len() < 4 {
            continue;
        }
        if lower.contains(tok) || name.contains(tok) {
            overlap += 1;
        }
    }
    let boost = overlap.min(6);

    if is_doc && lower.ends_with("/readme.md") {
        return 12 - boost.min(4);
    }
    // Prefer whole-doc spans over many tiny sections of the same file.
    if lower == "readme.md" || lower == "agents.md" {
        return 18 - boost.min(4);
    }
    if lower.starts_with("docs/") && node.kind == "Section" {
        return 25 - boost.min(8);
    }
    if lower.starts_with("docs/") && is_doc {
        return 20 - boost.min(10);
    }
    if node.kind == "Section" {
        return 40 - boost.min(10);
    }
    if is_doc {
        return 30 - boost.min(10);
    }
    90
}

fn read_doc_excerpt(
    workspace: &std::path::Path,
    rel: &str,
    node: &prism_store::GraphNodeView,
) -> String {
    let path = workspace.join(rel);
    let Ok(bytes) = std::fs::read(&path) else {
        return format!("{} {}", node.kind, node.name.as_deref().unwrap_or(rel));
    };
    let title = node.name.as_deref().unwrap_or(rel);
    let raw = String::from_utf8_lossy(&bytes);
    let excerpt = if node.kind == "Section" {
        if let Some(idx) = raw.find(&format!("# {title}")).or_else(|| raw.find(title)) {
            let end = (idx + 800).min(raw.len());
            raw[idx..end].to_string()
        } else {
            raw.chars().take(800).collect()
        }
    } else {
        // Leaner Doc excerpts keep architecture packs under Graphify's 2k budget.
        raw.chars().take(700).collect()
    };
    format!("# {title} ({rel})\n{excerpt}")
}

fn emit_upgrade_event(report: &HybridResolveReport) {
    emit_index_event(&IndexEvent::PrecisionUpgrade {
        confirmed: report.confirmed.len() as u64,
        still_heuristic: report.still_heuristic as u64,
        dual_candidates: report.dual_candidates.len() as u64,
        deferred: report.deferred as u64,
        latency_ms: report.latency_ms,
        overlay_used: report.overlay_used,
    });
}

fn emit_slice_event(report: &prism_semantic::InterprocSliceReport) {
    emit_index_event(&IndexEvent::SliceFinished {
        depth_reached: report.depth_reached as u64,
        functions_visited: report.functions_visited.len() as u64,
        spans: report.spans.len() as u64,
        truncated: report.truncated,
        memo_hit: report.provenance.memo_hit,
        latency_ms: report.latency_ms,
        shard_build_ms: 0,
    });
}

fn fragments_from_slice(report: &prism_semantic::InterprocSliceReport) -> Vec<CandidateFragment> {
    let mut out = Vec::new();
    for (i, s) in report.spans.iter().enumerate() {
        let text = format!(
            "// slice {}::{} L{}-{} (depth≤{})\n",
            s.path, s.function, s.start_line, s.end_line, report.depth_reached
        );
        out.push(CandidateFragment {
            id: format!("frag:slice:{}:{}:{}:{i}", s.path, s.start_line, s.end_line),
            kind: FragmentKind::Slice,
            layer: PackLayer::Core,
            text: text.clone(),
            token_estimate: estimate_tokens(&text),
            provenance: Provenance {
                node_ids: vec![format!("{}::{}", s.path, s.function)],
                edge_ids: vec![report.provenance.shard_id.clone()],
                analyzer: "prism-semantic".into(),
                tier: "T3".into(),
            },
            confidence: "extracted".into(),
            why_included: "primary_frame_body".into(),
            drop_priority: 0,
            roles: vec![
                "primary_frame_body".into(),
                "criterion_slice".into(),
                "seed_symbols".into(),
            ],
            must_include: true,
        });
    }
    if !report.cfg_summary.is_empty() {
        let t = report.cfg_summary.clone();
        out.push(CandidateFragment {
            id: "frag:slice:cfg_summary".into(),
            kind: FragmentKind::Signature,
            layer: PackLayer::Mod,
            text: t.clone(),
            token_estimate: estimate_tokens(&t),
            provenance: Provenance::synthetic("slice_cfg"),
            confidence: "extracted".into(),
            why_included: "cfg_summary".into(),
            drop_priority: 15,
            roles: vec!["cfg_summary".into()],
            must_include: false,
        });
    }
    out
}

/// Parse `path:line` or `path:line in symbol` style anchors.
fn parse_path_line(anchor: &str) -> Option<(String, u32)> {
    // e.g. httpx/_client.py:412 in send
    let primary = anchor.split_whitespace().next().unwrap_or(anchor);
    let (path, line_s) = primary.rsplit_once(':')?;
    if !path.contains('.') && !path.contains('/') {
        return None;
    }
    let line: u32 = line_s.parse().ok()?;
    Some((path.to_string(), line))
}

fn fragments_from_upgrade(report: &HybridResolveReport) -> Vec<CandidateFragment> {
    let mut out = Vec::new();
    for (i, c) in report.confirmed.iter().enumerate() {
        let t = format!(
            "precise confirmed {} → {} ({})",
            c.src,
            c.dst,
            c.file_path.as_deref().unwrap_or("?")
        );
        out.push(CandidateFragment {
            id: format!("frag:kg:precise:{}:{i}", c.edge_id),
            kind: FragmentKind::Signature,
            layer: PackLayer::Core,
            text: t.clone(),
            token_estimate: estimate_tokens(&t),
            provenance: Provenance {
                node_ids: vec![c.src.clone(), c.dst.clone()],
                edge_ids: vec![c.edge_id.clone()],
                analyzer: "prism-precise".into(),
                tier: "T2".into(),
            },
            confidence: "precise".into(),
            why_included: "upgrade_precision_confirmed".into(),
            drop_priority: 10,
            roles: vec!["reference_list".into(), "neighbor_bodies".into()],
            must_include: false,
        });
    }
    out
}

fn confidence_rank(c: &str) -> u8 {
    match c {
        "precise" => 3,
        "extracted" => 2,
        "observed" => 2,
        "heuristic" => 1,
        _ => 0,
    }
}

fn tier_for_confidence(c: &str) -> &'static str {
    if c == "precise" {
        "T2"
    } else {
        "T1"
    }
}

fn role_template(
    intent: Intent,
    role: &str,
    anchor: &str,
) -> (FragmentKind, PackLayer, String, String) {
    match role {
        "error_or_stack_verbatim" => (
            FragmentKind::ErrorVerbatim,
            PackLayer::Core,
            "error_or_stack_verbatim".into(),
            format!("ERROR/STACK: {anchor}"),
        ),
        "community_map" | "hub_nodes" => (
            FragmentKind::Community,
            PackLayer::Arch,
            role.into(),
            format!("[architecture] {role} for repo (intent={intent})"),
        ),
        "diff_hunks" => (
            FragmentKind::Diff,
            PackLayer::Diff,
            "diff_hunks".into(),
            format!("diff hunks intersecting {anchor}"),
        ),
        r if r.contains("signature") || r.contains("type") => (
            FragmentKind::Signature,
            PackLayer::Mod,
            r.into(),
            format!("signature: {anchor}"),
        ),
        r => (
            FragmentKind::Slice,
            PackLayer::Core,
            r.into(),
            format!("// must-include `{r}` locus near {anchor}\n"),
        ),
    }
}

fn first_anchor(plan: &Plan) -> Option<String> {
    anchors_from_plan(plan).into_iter().next()
}

fn anchors_from_plan(plan: &Plan) -> Vec<String> {
    let mut out = Vec::new();
    for step in &plan.steps {
        if let Some(arr) = step.inputs.get("anchors").and_then(|v| v.as_array()) {
            for a in arr {
                if let Some(s) = a.as_str() {
                    if !out.iter().any(|x| x == s) {
                        out.push(s.to_string());
                    }
                }
            }
        }
    }
    // Fallback: CapWords from question via backticks already in plan inputs usually
    if out.is_empty() {
        for tok in plan.question.split_whitespace() {
            let t = tok.trim_matches('`');
            if t.chars().any(|c| c.is_ascii_uppercase()) && t.len() > 1 {
                out.push(t.to_string());
                break;
            }
        }
    }
    out
}

fn strip_qual(a: &str) -> &str {
    a.rsplit(['.', ':', '/']).next().unwrap_or(a)
}

fn format_symbol_signature(n: &prism_store::GraphNodeView) -> String {
    format!(
        "{} {} in {}",
        n.kind,
        n.name.as_deref().unwrap_or(&n.id),
        n.file_path.as_deref().unwrap_or("?")
    )
}

fn maybe_read_slice(
    opts: &CompileOptions,
    hit: &prism_store::GraphNodeView,
) -> Option<(String, u32)> {
    let root = opts.workspace.as_ref()?;
    let rel = hit.file_path.as_ref()?;
    let path = root.join(rel);
    let bytes = std::fs::read(&path).ok()?;
    // Prefer attrs span if present — query attrs not on GraphNodeView; use whole-file cap
    let text = String::from_utf8_lossy(&bytes);
    let name = hit.name.as_deref().unwrap_or("");
    // Extractive: find def line containing name, take ~40 lines window
    let lines: Vec<&str> = text.lines().collect();
    let idx = lines.iter().position(|l| l.contains(name))?;
    let start = idx.saturating_sub(2);
    let end = (idx + 30).min(lines.len());
    let slice = lines[start..end].join("\n");
    let capped = if slice.len() > 2000 {
        format!("{}…", &slice[..2000])
    } else {
        slice
    };
    let tokens = estimate_tokens(&capped);
    Some((capped, tokens))
}
