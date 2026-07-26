//! prismd — optional local accelerator (P6 Stage B).
//!
//! CLI without the daemon remains fully supported.

mod watcher;

pub use watcher::{lockfile_path, run_watcher};

use anyhow::{bail, Context, Result};
use prism_api::{serve, AppState, DaemonConfig};
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;
use tracing::{info, warn};

/// Run the daemon until shutdown.
pub async fn run(cfg: DaemonConfig) -> Result<()> {
    fs::create_dir_all(cfg.workspace.join(".prism"))
        .with_context(|| format!("create .prism under {}", cfg.workspace.display()))?;

    let lock = lockfile_path(&cfg.workspace);
    if lock.exists() {
        let old = fs::read_to_string(&lock).unwrap_or_default();
        warn!(
            path = %lock.display(),
            previous = %old.trim(),
            "daemon.lock already present — overwriting (stale lock recovery)"
        );
    }
    let pid = std::process::id();
    fs::write(&lock, format!("{pid}\n{}", cfg.bind))
        .with_context(|| format!("write {}", lock.display()))?;

    // Persist token so IDE / CLI clients can attach without guessing (P8 fix).
    let token_path = cfg.workspace.join(".prism/daemon.token");
    fs::write(&token_path, &cfg.token)
        .with_context(|| format!("write {}", token_path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&token_path, fs::Permissions::from_mode(0o600));
    }

    let state = AppState::new(&cfg);
    // Ensure an index exists so status/compile have a graph.
    if !cfg.workspace.join(".prism/graph.sqlite").exists() {
        info!("no index yet — running initial index");
        let _ = state.reindex(vec![]).await?;
    } else {
        // Refresh snapshot id from current tree without forcing full work when warm.
        let _ = state.reindex(vec![]).await?;
    }

    let addr: SocketAddr = cfg
        .bind
        .parse()
        .with_context(|| format!("invalid bind address {}", cfg.bind))?;

    if !addr.ip().is_loopback() {
        bail!(
            "refusing non-loopback bind {} — set an explicit loopback address (security default)",
            addr
        );
    }

    let watch_state = state.clone();
    let debounce = Duration::from_millis(cfg.debounce_ms.max(50));
    let watcher_handle = tokio::spawn(async move {
        if let Err(e) = run_watcher(watch_state, debounce).await {
            warn!(error = %e, "watcher stopped");
        }
    });

    let idle = cfg.idle_shutdown_secs;
    let idle_state = state.clone();
    let idle_handle = tokio::spawn(async move {
        if idle == 0 {
            return;
        }
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            let last = idle_state
                .last_activity_ms
                .load(std::sync::atomic::Ordering::Relaxed);
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            if now.saturating_sub(last) >= idle.saturating_mul(1000) {
                info!(idle_secs = idle, "idle shutdown");
                // Best-effort: exit process so axum serve ends with the runtime.
                std::process::exit(0);
            }
        }
    });

    let serve_result = serve(state, addr).await;

    watcher_handle.abort();
    idle_handle.abort();
    let _ = fs::remove_file(&lock);
    let _ = fs::remove_file(cfg.workspace.join(".prism/daemon.token"));
    serve_result
}

/// Generate a random token when the caller does not supply one.
pub fn generate_token() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("prism-local-{t:x}")
}

pub fn resolve_workspace(path: PathBuf) -> Result<PathBuf> {
    let p = if path.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        path
    };
    Ok(fs::canonicalize(&p).unwrap_or(p))
}
