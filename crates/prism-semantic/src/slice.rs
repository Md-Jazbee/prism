//! Local intra-procedural slice (symbol or line criterion).

use crate::artifact::{FunctionFlow, SemanticFileArtifact};
use crate::crash::SemanticPartial;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SliceCriterion {
    Line { path: String, line: u32 },
    Symbol { path: String, symbol: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SliceSpan {
    pub start_line: u32,
    pub end_line: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SliceReport {
    pub path: String,
    pub function: String,
    pub criterion_line: u32,
    pub spans: Vec<SliceSpan>,
    pub cfg_summary: String,
    pub algo_version: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

const MAX_BLOCKS: usize = 64;

/// Backward local slice: blocks/lines that may affect the criterion.
pub fn local_slice(
    artifact: &SemanticFileArtifact,
    criterion: &SliceCriterion,
) -> Result<SliceReport, SemanticPartial> {
    let (path, line) = match criterion {
        SliceCriterion::Line { path, line } => (path.clone(), *line),
        SliceCriterion::Symbol { path, symbol } => {
            let Some(func) = artifact.functions.iter().find(|f| f.name == *symbol) else {
                return Err(SemanticPartial::new(format!(
                    "symbol `{symbol}` not found in {}",
                    artifact.path
                )));
            };
            (path.clone(), func.start_line)
        }
    };

    if path != artifact.path && !artifact.path.ends_with(&path) {
        // allow basename match
        let art_base = artifact.path.rsplit('/').next().unwrap_or(&artifact.path);
        let crit_base = path.rsplit('/').next().unwrap_or(&path);
        if art_base != crit_base {
            return Err(SemanticPartial::new(format!(
                "path mismatch: criterion={path} artifact={}",
                artifact.path
            )));
        }
    }

    let Some(func) = artifact
        .functions
        .iter()
        .find(|f| line >= f.start_line && line <= f.end_line)
    else {
        return Ok(SliceReport {
            path: artifact.path.clone(),
            function: String::new(),
            criterion_line: line,
            spans: vec![],
            cfg_summary: String::new(),
            algo_version: artifact.algo_version.clone(),
            notes: vec!["criterion_not_in_function".into()],
        });
    };

    let report = slice_function(artifact, func, line);
    Ok(report)
}

fn slice_function(artifact: &SemanticFileArtifact, func: &FunctionFlow, line: u32) -> SliceReport {
    let mut seed_blocks: HashSet<String> = HashSet::new();
    for b in &func.blocks {
        if line >= b.start_line && line <= b.end_line {
            seed_blocks.insert(b.id.clone());
        }
    }
    if seed_blocks.is_empty() && !func.blocks.is_empty() {
        // nearest block
        if let Some(b) = func.blocks.iter().min_by_key(|b| {
            if line < b.start_line {
                b.start_line - line
            } else {
                line - b.end_line
            }
        }) {
            seed_blocks.insert(b.id.clone());
        }
    }

    // Lines from data deps into criterion
    let mut seed_lines: HashSet<u32> = HashSet::new();
    seed_lines.insert(line);
    for dep in &func.dfg.deps {
        if dep.use_line == line {
            seed_lines.insert(dep.def_line);
            for b in &func.blocks {
                if dep.def_line >= b.start_line && dep.def_line <= b.end_line {
                    seed_blocks.insert(b.id.clone());
                }
            }
        }
    }

    // Walk CFG predecessors
    let mut preds: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for e in &func.cfg_edges {
        preds.entry(e.dst.clone()).or_default().push(e.src.clone());
    }

    let mut reached = seed_blocks.clone();
    let mut q: VecDeque<String> = seed_blocks.iter().cloned().collect();
    while let Some(id) = q.pop_front() {
        if reached.len() >= MAX_BLOCKS {
            break;
        }
        if let Some(ps) = preds.get(&id) {
            for p in ps {
                if reached.insert(p.clone()) {
                    q.push_back(p.clone());
                }
            }
        }
    }

    let mut lines: HashSet<u32> = seed_lines;
    for b in &func.blocks {
        if reached.contains(&b.id) {
            for l in b.start_line..=b.end_line {
                lines.insert(l);
            }
        }
    }
    // Close under DFG deps within function
    let mut changed = true;
    while changed {
        changed = false;
        for dep in &func.dfg.deps {
            if lines.contains(&dep.use_line) && !lines.contains(&dep.def_line) {
                lines.insert(dep.def_line);
                changed = true;
            }
        }
    }

    let mut sorted: Vec<u32> = lines.into_iter().collect();
    sorted.sort_unstable();
    let spans = merge_spans(&sorted);

    // Always ensure criterion covered
    let spans = ensure_criterion(spans, line);

    let cfg_summary = format!(
        "function {} L{}-{} blocks={} reached={} criterion_line={}",
        func.name,
        func.start_line,
        func.end_line,
        func.blocks.len(),
        reached.len(),
        line
    );

    let mut notes = Vec::new();
    if reached.len() >= MAX_BLOCKS {
        notes.push("truncated_max_blocks".into());
    }

    SliceReport {
        path: artifact.path.clone(),
        function: func.name.clone(),
        criterion_line: line,
        spans,
        cfg_summary,
        algo_version: artifact.algo_version.clone(),
        notes,
    }
}

fn merge_spans(lines: &[u32]) -> Vec<SliceSpan> {
    if lines.is_empty() {
        return vec![];
    }
    let mut out = Vec::new();
    let mut start = lines[0];
    let mut end = lines[0];
    for &l in &lines[1..] {
        if l == end + 1 {
            end = l;
        } else {
            out.push(SliceSpan {
                start_line: start,
                end_line: end,
            });
            start = l;
            end = l;
        }
    }
    out.push(SliceSpan {
        start_line: start,
        end_line: end,
    });
    out
}

fn ensure_criterion(mut spans: Vec<SliceSpan>, line: u32) -> Vec<SliceSpan> {
    if spans
        .iter()
        .any(|s| s.start_line <= line && s.end_line >= line)
    {
        return spans;
    }
    spans.push(SliceSpan {
        start_line: line,
        end_line: line,
    });
    spans.sort_by_key(|s| s.start_line);
    spans
}
