//! Inter-procedural Slice operator (P4 Stage B).

use crate::artifact::INTERPROC_ALGO_VERSION;
use crate::crash::SemanticPartial;
use crate::memo::{default_algo, load_memo, memo_key, params_hash, save_memo, MemoEntry};
use crate::shard::{ensure_shard, load_catalog, OverlayEdge};
use crate::slice::{local_slice, SliceCriterion};
use crate::store::{build_file_artifact, load_file_artifact};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SliceDirection {
    Backward,
    Forward,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SliceParams {
    pub direction: SliceDirection,
    pub max_depth: u32,
    pub max_functions: usize,
    pub max_spans: usize,
    pub residual_expand: bool,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(default = "default_snapshot")]
    pub snapshot_id: String,
}

fn default_snapshot() -> String {
    "adhoc".into()
}

impl Default for SliceParams {
    fn default() -> Self {
        Self {
            direction: SliceDirection::Backward,
            max_depth: 2,
            max_functions: 16,
            max_spans: 40,
            residual_expand: true,
            path: String::new(),
            line: None,
            symbol: None,
            snapshot_id: "adhoc".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterprocSpan {
    pub path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub function: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResidualItem {
    pub kind: String,
    pub from: String,
    pub to: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SliceProvenance {
    pub shard_id: String,
    pub memo_hit: bool,
    pub params_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterprocSliceReport {
    pub algo_version: String,
    pub direction: SliceDirection,
    pub criterion_path: String,
    pub criterion_line: u32,
    pub spans: Vec<InterprocSpan>,
    pub functions_visited: Vec<String>,
    pub depth_reached: u32,
    pub truncated: bool,
    pub residual: Vec<ResidualItem>,
    pub provenance: SliceProvenance,
    pub cfg_summary: String,
    pub latency_ms: u64,
}

/// Run inter-procedural slice with memoization.
pub fn interproc_slice(
    workspace: &Path,
    params: &SliceParams,
) -> Result<InterprocSliceReport, SemanticPartial> {
    let started = Instant::now();
    let params_json = serde_json::to_value(params).unwrap_or_default();
    let ph = params_hash(&params_json);
    let mk = memo_key(&params.snapshot_id, default_algo(), &ph);

    if let Ok(Some(hit)) = load_memo(workspace, &mk) {
        let mut report = hit.report;
        report.provenance.memo_hit = true;
        report.latency_ms = started.elapsed().as_millis() as u64;
        return Ok(report);
    }

    let _ = build_file_artifact(workspace, &params.path);

    let criterion_line = match (params.line, params.symbol.as_ref()) {
        (Some(l), _) => l,
        (None, Some(sym)) => {
            let art = load_file_artifact(workspace, &params.path)
                .ok()
                .flatten()
                .ok_or_else(|| SemanticPartial::new(format!("no artifact for {}", params.path)))?;
            art.functions
                .iter()
                .find(|f| f.name == *sym)
                .map(|f| f.start_line)
                .ok_or_else(|| {
                    SemanticPartial::new(format!("symbol `{sym}` not found in {}", params.path))
                })?
        }
        _ => {
            return Err(SemanticPartial::new("slice needs --line or --symbol"));
        }
    };

    let art = load_file_artifact(workspace, &params.path)
        .ok()
        .flatten()
        .ok_or_else(|| SemanticPartial::new(format!("no artifact for {}", params.path)))?;

    let seed_fn = art
        .functions
        .iter()
        .find(|f| f.start_line <= criterion_line && criterion_line <= f.end_line)
        .map(|f| f.name.clone())
        .ok_or_else(|| {
            SemanticPartial::new(format!(
                "no function covers line {criterion_line} in {}",
                params.path
            ))
        })?;

    let (shard, shard_truncated) = ensure_shard(
        workspace,
        &params.path,
        &seed_fn,
        params.max_depth,
        params.max_functions,
    )
    .map_err(|e| SemanticPartial::new(e.to_string()))?;

    let catalog = load_catalog(workspace).map_err(|e| SemanticPartial::new(e.to_string()))?;

    let mut spans: Vec<InterprocSpan> = Vec::new();
    let mut functions_visited = shard.functions.clone();
    functions_visited.sort();
    let mut residual = Vec::new();
    let mut depth_reached = 0u32;

    let local = local_slice(
        &art,
        &SliceCriterion::Line {
            path: params.path.clone(),
            line: criterion_line,
        },
    )?;
    for s in &local.spans {
        spans.push(InterprocSpan {
            path: params.path.clone(),
            start_line: s.start_line,
            end_line: s.end_line,
            function: seed_fn.clone(),
        });
    }

    let seed_key = format!("{}::{}", params.path, seed_fn);
    let call_edges: Vec<&OverlayEdge> = shard
        .edges
        .iter()
        .filter(|e| e.kind == "CALLS")
        .collect();

    let mut frontier = vec![seed_key.clone()];
    let mut seen = std::collections::BTreeSet::from([seed_key.clone()]);
    for depth in 1..=params.max_depth {
        let mut next = Vec::new();
        for cur in &frontier {
            for e in &call_edges {
                let nbr = match params.direction {
                    SliceDirection::Backward => {
                        if e.dst == *cur {
                            Some(e.src.as_str())
                        } else {
                            None
                        }
                    }
                    SliceDirection::Forward => {
                        if e.src == *cur {
                            Some(e.dst.as_str())
                        } else {
                            None
                        }
                    }
                };
                let Some(nbr) = nbr else { continue };
                if !seen.insert(nbr.to_string()) {
                    continue;
                }
                if seen.len() > params.max_functions {
                    residual.push(ResidualItem {
                        kind: "call_edge".into(),
                        from: cur.clone(),
                        to: nbr.into(),
                        reason: "max_functions".into(),
                    });
                    continue;
                }
                next.push(nbr.to_string());
                depth_reached = depth;

                if let Some(loc) = catalog.get(nbr) {
                    let _ = build_file_artifact(workspace, &loc.path);
                    if let Ok(Some(nart)) = load_file_artifact(workspace, &loc.path) {
                        let crit = SliceCriterion::Line {
                            path: loc.path.clone(),
                            line: loc.start_line,
                        };
                        if let Ok(rep) = local_slice(&nart, &crit) {
                            for s in rep.spans {
                                spans.push(InterprocSpan {
                                    path: loc.path.clone(),
                                    start_line: s.start_line,
                                    end_line: s.end_line,
                                    function: loc.name.clone(),
                                });
                            }
                        } else {
                            spans.push(InterprocSpan {
                                path: loc.path.clone(),
                                start_line: loc.start_line,
                                end_line: loc.end_line,
                                function: loc.name.clone(),
                            });
                        }
                    }
                }
            }
        }
        if next.is_empty() {
            break;
        }
        frontier = next;
    }

    let mut truncated = shard_truncated;
    if spans.len() > params.max_spans {
        truncated = true;
        for extra in spans.drain(params.max_spans..) {
            residual.push(ResidualItem {
                kind: "span_budget".into(),
                from: format!("{}:{}", extra.path, extra.start_line),
                to: format!("{}:{}", extra.path, extra.end_line),
                reason: "max_spans".into(),
            });
        }
    }

    spans.sort_by(|a, b| {
        (&a.path, a.start_line, a.end_line, &a.function).cmp(&(
            &b.path,
            b.start_line,
            b.end_line,
            &b.function,
        ))
    });
    spans.dedup();

    if !spans.iter().any(|s| {
        s.path == params.path && s.start_line <= criterion_line && s.end_line >= criterion_line
    }) {
        spans.push(InterprocSpan {
            path: params.path.clone(),
            start_line: criterion_line,
            end_line: criterion_line,
            function: seed_fn.clone(),
        });
    }

    if !params.residual_expand {
        residual.clear();
    }

    let report = InterprocSliceReport {
        algo_version: INTERPROC_ALGO_VERSION.into(),
        direction: params.direction,
        criterion_path: params.path.clone(),
        criterion_line,
        spans,
        functions_visited,
        depth_reached,
        truncated,
        residual,
        provenance: SliceProvenance {
            shard_id: shard.shard_id.clone(),
            memo_hit: false,
            params_hash: ph.clone(),
        },
        cfg_summary: format!(
            "interproc {} seed={}::{} depth={} funcs={} {}",
            match params.direction {
                SliceDirection::Backward => "backward",
                SliceDirection::Forward => "forward",
            },
            params.path,
            seed_fn,
            depth_reached,
            shard.functions.len(),
            local.cfg_summary
        ),
        latency_ms: started.elapsed().as_millis() as u64,
    };

    let _ = save_memo(
        workspace,
        &MemoEntry {
            memo_key: mk,
            snapshot_id: params.snapshot_id.clone(),
            algorithm_version: default_algo().into(),
            params_hash: ph,
            report: report.clone(),
        },
    );

    Ok(report)
}
