//! Selection: plan (+ optional KG) → candidate fragments.

use crate::fragment::{
    estimate_tokens, CandidateFragment, FragmentKind, PackLayer, Provenance,
};
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

/// Offline / synthetic candidates from recipe roles (no KG).
///
/// Produces one fragment per `must_include` role plus a few optional neighbors
/// so budget/drop behavior is testable without an index.
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
        && !out.iter().any(|c| c.roles.iter().any(|r| r == "community_map"))
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
pub fn select_from_kg(
    kg: &SqliteKgStore,
    plan: &Plan,
    opts: &CompileOptions,
) -> Result<Vec<CandidateFragment>> {
    let mut out = Vec::new();
    let mut gaps_extra: Vec<String> = Vec::new();
    let anchors = anchors_from_plan(plan);

    // Always materialize must-include role stubs first (may be enriched below)
    out.extend(select_candidates(plan));

    let mut seed_ids: Vec<String> = Vec::new();

    // Enrich / replace with live KG hits where possible
    for step in &plan.steps {
        if !step.executable && !matches!(step.op, Operator::BudgetPack) {
            continue;
        }
        match step.op {
            Operator::ResolveSymbol => {
                for a in &anchors {
                    let name = strip_qual(a);
                    let hits = kg.resolve_symbol(name, None, 5)?;
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
                            Some(&["CALLS".into(), "IMPORTS".into(), "DEFINES".into(), "REFERENCES".into()]),
                            EdgeDirection::Both,
                            15,
                        )?;
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
                let max_upgrades = step
                    .inputs
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
                let summary = format!(
                    "communities={} hubs={}",
                    map.communities.len(),
                    map.hubs.len()
                );
                let mut lines = vec![summary];
                for c in map.communities.iter().take(8) {
                    lines.push(format!(
                        "  community {} files≈{} label={}",
                        c.id, c.file_count, c.label
                    ));
                }
                for h in map.hubs.iter().take(5) {
                    lines.push(format!("  hub {} degree={}", h.node_id, h.degree));
                }
                let text = lines.join("\n");
                out.push(CandidateFragment {
                    id: "frag:kg:repo_map".into(),
                    kind: FragmentKind::Community,
                    layer: PackLayer::Arch,
                    text: text.clone(),
                    token_estimate: estimate_tokens(&text),
                    provenance: Provenance::synthetic("repo_map"),
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
                            provenance: Provenance::synthetic(a),
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
                    provenance: Provenance::synthetic("error"),
                    confidence: "extracted".into(),
                    why_included: "error_or_stack_verbatim".into(),
                    drop_priority: 0,
                    roles: vec!["error_or_stack_verbatim".into()],
                    must_include: true,
                });
            }
        }
    }

    // Prefer precise over heuristic when same fragment id stem / neighbor target
    out.sort_by(|a, b| {
        a.id.cmp(&b.id).then_with(|| {
            confidence_rank(&b.confidence).cmp(&confidence_rank(&a.confidence))
        })
    });
    out.dedup_by(|a, b| a.id == b.id);

    // Re-apply must_include from plan roles
    for c in &mut out {
        if plan.must_include.iter().any(|r| c.roles.contains(r)) {
            c.must_include = true;
            c.drop_priority = 0;
        }
    }

    // Stash upgrade uncertainty as synthetic gap fragments (visible in pack text roles)
    for (i, g) in gaps_extra.iter().enumerate() {
        out.push(CandidateFragment {
            id: format!("frag:gap:upgrade:{i}"),
            kind: FragmentKind::Signature,
            layer: PackLayer::Nbr,
            text: g.clone(),
            token_estimate: estimate_tokens(g),
            provenance: Provenance::synthetic("upgrade_gap"),
            confidence: "heuristic".into(),
            why_included: "precision_uncertainty".into(),
            drop_priority: 5,
            roles: vec!["precision_uncertainty".into()],
            must_include: false,
        });
    }

    let _ = gaps_extra;
    Ok(out)
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
