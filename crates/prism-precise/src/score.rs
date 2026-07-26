//! Call-resolution precision/recall vs oracle (P3 Stage A eval).

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Minimal call edge for scoring (T1 dump, PreciseIndex, or oracle).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CallEdge {
    pub src: String,
    pub dst: String,
    pub file_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_byte: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoreReport {
    pub true_positives: usize,
    pub false_positives: usize,
    pub false_negatives: usize,
    pub precision: f64,
    pub recall: f64,
}

/// Score predicted CALLS against oracle CALLS.
///
/// Match key: `(file_path, src, start_byte)` when start_byte present on both;
/// else `(file_path, src)`. A prediction is a TP when its `dst` equals the oracle
/// `dst` for the same site.
pub fn score_call_resolution(predicted: &[CallEdge], oracle: &[CallEdge]) -> ScoreReport {
    let mut tp = 0usize;
    let mut fp = 0usize;
    let mut matched_oracle = HashSet::new();

    for pred in predicted {
        if let Some((idx, ora)) = oracle
            .iter()
            .enumerate()
            .find(|(i, o)| !matched_oracle.contains(i) && same_site(pred, o))
        {
            if pred.dst == ora.dst {
                tp += 1;
                matched_oracle.insert(idx);
            } else {
                fp += 1;
            }
        }
    }

    let fn_ = oracle.len().saturating_sub(matched_oracle.len());
    let precision = if tp + fp == 0 {
        0.0
    } else {
        tp as f64 / (tp + fp) as f64
    };
    let recall = if oracle.is_empty() {
        0.0
    } else {
        tp as f64 / oracle.len() as f64
    };
    ScoreReport {
        true_positives: tp,
        false_positives: fp,
        false_negatives: fn_,
        precision,
        recall,
    }
}

fn same_site(a: &CallEdge, b: &CallEdge) -> bool {
    if a.file_path != b.file_path || a.src != b.src {
        return false;
    }
    match (a.start_byte, b.start_byte) {
        (Some(x), Some(y)) => x == y,
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perfect_score() {
        let edges = vec![CallEdge {
            src: "a".into(),
            dst: "b".into(),
            file_path: "f.py".into(),
            start_byte: Some(1),
        }];
        let s = score_call_resolution(&edges, &edges);
        assert_eq!(s.precision, 1.0);
        assert_eq!(s.recall, 1.0);
    }

    #[test]
    fn unresolved_is_wrong_vs_resolved_oracle() {
        let pred = vec![CallEdge {
            src: "main".into(),
            dst: "unresolved:greet".into(),
            file_path: "app.py".into(),
            start_byte: Some(10),
        }];
        let ora = vec![CallEdge {
            src: "main".into(),
            dst: "sym:lib.py:function:greet:0".into(),
            file_path: "app.py".into(),
            start_byte: Some(10),
        }];
        let s = score_call_resolution(&pred, &ora);
        assert_eq!(s.true_positives, 0);
        assert_eq!(s.false_positives, 1);
        assert_eq!(s.recall, 0.0);
    }
}
