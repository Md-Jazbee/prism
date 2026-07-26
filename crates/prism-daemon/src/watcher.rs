//! File watcher → debounce → incremental re-index → SSE invalidation.

use anyhow::Result;
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use prism_api::AppState;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use tracing::{info, warn};

/// Watch `workspace` (excluding `.prism/`) and reindex after `debounce`.
pub async fn run_watcher(state: AppState, debounce: Duration) -> Result<()> {
    let (tx, rx) = mpsc::channel();
    let workspace = state.workspace.clone();
    let mut watcher = RecommendedWatcher::new(
        move |res| {
            let _ = tx.send(res);
        },
        notify::Config::default(),
    )?;
    watcher.watch(&workspace, RecursiveMode::Recursive)?;
    info!(path = %workspace.display(), "file watcher started");

    let mut pending: HashSet<String> = HashSet::new();
    let mut last_change = Instant::now();
    let mut dirty = false;

    loop {
        // Poll notify channel without blocking the async runtime for too long.
        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(Ok(event)) => {
                if matches!(
                    event.kind,
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
                ) {
                    for path in event.paths {
                        if let Some(rel) = relativize(&workspace, &path) {
                            if should_ignore(&rel) {
                                continue;
                            }
                            pending.insert(rel);
                            dirty = true;
                            last_change = Instant::now();
                        }
                    }
                }
            }
            Ok(Err(e)) => warn!(error = %e, "watcher error"),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }

        if dirty && last_change.elapsed() >= debounce {
            let paths: Vec<String> = pending.drain().collect();
            dirty = false;
            info!(count = paths.len(), "debounced reindex");
            if let Err(e) = state.reindex(paths).await {
                warn!(error = %e, "reindex failed");
            }
        }

        tokio::task::yield_now().await;
    }
    Ok(())
}

fn relativize(root: &Path, path: &Path) -> Option<String> {
    path.strip_prefix(root)
        .ok()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
}

fn should_ignore(rel: &str) -> bool {
    rel.starts_with(".prism/")
        || rel.starts_with(".git/")
        || rel.starts_with("target/")
        || rel.ends_with(".sqlite")
        || rel.ends_with(".sqlite-wal")
        || rel.ends_with(".sqlite-shm")
}

/// Resolve a lockfile path used for single-instance-per-workspace.
pub fn lockfile_path(workspace: &Path) -> PathBuf {
    workspace.join(".prism/daemon.lock")
}
