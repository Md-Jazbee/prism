//! Named event schema stub (`schemas/events/v0`).

use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// Counters collected during discover → hash → (stub) parse.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct IndexStats {
    pub files_discovered: u64,
    pub files_skipped_unchanged: u64,
    pub files_hashed: u64,
    pub files_secret_skipped: u64,
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
    ParseHookStub {
        path: String,
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
                wall_time_ms = stats.wall_time_ms,
                "index finished"
            );
        }
        IndexEvent::FileSkippedSecret { path } => {
            warn!(event = "file_skipped_secret", path = %path, "secret-sensitive path skipped");
        }
        IndexEvent::ParseHookStub { path } => {
            info!(event = "parse_hook_stub", path = %path, "parse hook stub (P0)");
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
}
