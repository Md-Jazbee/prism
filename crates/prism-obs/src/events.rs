//! Named event schema (`schemas/events/v0`) + P1 extract events.

use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// Counters collected during discover → hash → extract.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct IndexStats {
    pub files_discovered: u64,
    pub files_skipped_unchanged: u64,
    pub files_hashed: u64,
    pub files_secret_skipped: u64,
    pub files_extracted: u64,
    pub files_extract_skipped: u64,
    pub nodes_written: u64,
    pub edges_written: u64,
    pub unresolved_calls: u64,
    pub wall_time_ms: u64,
}

/// Index pipeline event for metrics / logs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum IndexEvent {
    IndexStarted {
        schema_version: String,
        root: String,
    },
    IndexFinished {
        stats: IndexStats,
    },
    FileSkippedSecret {
        path: String,
    },
    /// Language not supported / no extractor — skip gracefully.
    FileExtractSkipped {
        path: String,
        reason: String,
    },
    FileExtracted {
        path: String,
        language: String,
        nodes: u64,
        edges: u64,
        unresolved_calls: u64,
    },
    /// Structural query latency (design NFR: local P95 &lt;50ms).
    QueryFinished {
        op: String,
        latency_ms: u64,
        hit_count: u64,
    },
    /// Precise-tier hybrid upgrade (P3 Stage B).
    PrecisionUpgrade {
        confirmed: u64,
        still_heuristic: u64,
        dual_candidates: u64,
        deferred: u64,
        latency_ms: u64,
        overlay_used: bool,
    },
    /// Inter-procedural slice (P4 Stage B).
    SliceFinished {
        depth_reached: u64,
        functions_visited: u64,
        spans: u64,
        truncated: bool,
        memo_hit: bool,
        latency_ms: u64,
        shard_build_ms: u64,
    },
}

/// Emit an index event via `tracing` (JSON-friendly fields).
pub fn emit_index_event(event: &IndexEvent) {
    match event {
        IndexEvent::IndexStarted {
            schema_version,
            root,
        } => {
            info!(
                event = "index_started",
                schema_version = %schema_version,
                root = %root,
                "index started"
            );
        }
        IndexEvent::IndexFinished { stats } => {
            info!(
                event = "index_finished",
                files_discovered = stats.files_discovered,
                files_skipped_unchanged = stats.files_skipped_unchanged,
                files_hashed = stats.files_hashed,
                files_secret_skipped = stats.files_secret_skipped,
                files_extracted = stats.files_extracted,
                files_extract_skipped = stats.files_extract_skipped,
                nodes_written = stats.nodes_written,
                edges_written = stats.edges_written,
                unresolved_calls = stats.unresolved_calls,
                wall_time_ms = stats.wall_time_ms,
                "index finished"
            );
        }
        IndexEvent::FileSkippedSecret { path } => {
            warn!(event = "file_skipped_secret", path = %path, "secret-sensitive path skipped");
        }
        IndexEvent::FileExtractSkipped { path, reason } => {
            info!(
                event = "file_extract_skipped",
                path = %path,
                reason = %reason,
                "extract skipped"
            );
        }
        IndexEvent::FileExtracted {
            path,
            language,
            nodes,
            edges,
            unresolved_calls,
        } => {
            info!(
                event = "file_extracted",
                path = %path,
                language = %language,
                nodes = nodes,
                edges = edges,
                unresolved_calls = unresolved_calls,
                "file extracted"
            );
        }
        IndexEvent::QueryFinished {
            op,
            latency_ms,
            hit_count,
        } => {
            info!(
                event = "query_finished",
                op = %op,
                latency_ms = latency_ms,
                hit_count = hit_count,
                "query finished"
            );
        }
        IndexEvent::PrecisionUpgrade {
            confirmed,
            still_heuristic,
            dual_candidates,
            deferred,
            latency_ms,
            overlay_used,
        } => {
            info!(
                event = "precision_upgrade",
                confirmed = confirmed,
                still_heuristic = still_heuristic,
                dual_candidates = dual_candidates,
                deferred = deferred,
                latency_ms = latency_ms,
                overlay_used = overlay_used,
                "precision upgrade"
            );
        }
        IndexEvent::SliceFinished {
            depth_reached,
            functions_visited,
            spans,
            truncated,
            memo_hit,
            latency_ms,
            shard_build_ms,
        } => {
            info!(
                event = "slice_finished",
                depth_reached = depth_reached,
                functions_visited = functions_visited,
                spans = spans,
                truncated = truncated,
                memo_hit = memo_hit,
                latency_ms = latency_ms,
                shard_build_ms = shard_build_ms,
                "slice finished"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_serde_tagged() {
        let e = IndexEvent::IndexFinished {
            stats: IndexStats {
                files_discovered: 3,
                ..Default::default()
            },
        };
        let v = serde_json::to_value(&e).unwrap();
        assert_eq!(v["event"], "index_finished");
        assert_eq!(v["stats"]["files_discovered"], 3);
    }

    #[test]
    fn file_extracted_serde() {
        let e = IndexEvent::FileExtracted {
            path: "a.py".into(),
            language: "python".into(),
            nodes: 2,
            edges: 1,
            unresolved_calls: 1,
        };
        let v = serde_json::to_value(&e).unwrap();
        assert_eq!(v["event"], "file_extracted");
        assert_eq!(v["unresolved_calls"], 1);
    }
}
