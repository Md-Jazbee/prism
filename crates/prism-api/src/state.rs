//! Warm daemon state shared by HTTP handlers and the file watcher.

use anyhow::Result;
use prism_core::{IncrementalIndexer, IndexOptions, WorkspaceManager};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{broadcast, Mutex, RwLock};
use uuid::Uuid;

pub const PRISM_API_VERSION: &str = "0.0.1";

#[derive(Debug, Clone)]
pub struct DaemonConfig {
    pub workspace: PathBuf,
    pub token: String,
    pub bind: String,
    /// Idle shutdown seconds; 0 = never.
    pub idle_shutdown_secs: u64,
    /// File-watch debounce window.
    pub debounce_ms: u64,
}

impl DaemonConfig {
    pub fn loopback(workspace: impl Into<PathBuf>, token: impl Into<String>) -> Self {
        Self {
            workspace: workspace.into(),
            token: token.into(),
            bind: "127.0.0.1:7420".into(),
            idle_shutdown_secs: 0,
            debounce_ms: 250,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidationEvent {
    pub event: String,
    pub snapshot_id: String,
    pub paths: Vec<String>,
    pub ts_unix_ms: u64,
}

#[derive(Clone)]
pub struct AppState {
    pub workspace: PathBuf,
    pub token: String,
    pub snapshot_id: Arc<RwLock<String>>,
    pub events: broadcast::Sender<InvalidationEvent>,
    /// Single-writer gate for index mutations.
    pub index_lock: Arc<Mutex<()>>,
    pub request_seq: Arc<AtomicU64>,
    pub last_activity_ms: Arc<AtomicU64>,
}

impl AppState {
    pub fn new(cfg: &DaemonConfig) -> Self {
        let (events, _) = broadcast::channel(256);
        let now = now_ms();
        Self {
            workspace: cfg.workspace.clone(),
            token: cfg.token.clone(),
            snapshot_id: Arc::new(RwLock::new(format!("boot-{}", Uuid::new_v4()))),
            events,
            index_lock: Arc::new(Mutex::new(())),
            request_seq: Arc::new(AtomicU64::new(0)),
            last_activity_ms: Arc::new(AtomicU64::new(now)),
        }
    }

    pub fn touch(&self) {
        self.last_activity_ms.store(now_ms(), Ordering::Relaxed);
        self.request_seq.fetch_add(1, Ordering::Relaxed);
    }

    pub async fn snapshot_id(&self) -> String {
        self.snapshot_id.read().await.clone()
    }

    pub async fn reindex(&self, paths: Vec<String>) -> Result<String> {
        let _guard = self.index_lock.lock().await;
        let wm = WorkspaceManager::open(&self.workspace)?;
        let prism = self.workspace.join(".prism");
        let mut indexer = IncrementalIndexer::open(wm, &prism)?;
        let result = indexer.run(&IndexOptions { dry_run: false })?;
        let snap = result.tree_fingerprint.clone();
        *self.snapshot_id.write().await = snap.clone();
        let ev = InvalidationEvent {
            event: "index.updated".into(),
            snapshot_id: snap.clone(),
            paths,
            ts_unix_ms: now_ms(),
        };
        let _ = self.events.send(ev);
        Ok(snap)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<InvalidationEvent> {
        self.events.subscribe()
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
