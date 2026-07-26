//! `prismd` binary — local HTTP/SSE daemon (optional accelerator).

use anyhow::Result;
use clap::Parser;
use prism_api::DaemonConfig;
use prism_daemon::{generate_token, resolve_workspace, run};
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(
    name = "prismd",
    version,
    about = "Prism local daemon — HTTP/SSE accelerator (optional; CLI works without it)"
)]
struct Args {
    /// Workspace root.
    #[arg(default_value = ".")]
    path: PathBuf,
    /// Loopback bind address (non-loopback refused).
    #[arg(long, default_value = "127.0.0.1:7420")]
    bind: String,
    /// Auth token (or set PRISM_TOKEN). Generated if omitted.
    #[arg(long)]
    token: Option<String>,
    /// Idle shutdown seconds (0 = never).
    #[arg(long, default_value_t = 0)]
    idle_shutdown_secs: u64,
    /// File-watch debounce milliseconds.
    #[arg(long, default_value_t = 250)]
    debounce_ms: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    // Opt-in OTLP note (G-09): set PRISM_OTLP_ENDPOINT to acknowledge export intent.
    if let Ok(endpoint) = std::env::var("PRISM_OTLP_ENDPOINT") {
        if !endpoint.is_empty() {
            tracing::info!(
                %endpoint,
                "PRISM_OTLP_ENDPOINT set — spans use tracing; wire a collector-compatible exporter in a follow-up build"
            );
            prism_obs::emit_index_event(&prism_obs::IndexEvent::QueryFinished {
                op: "otlp.opt_in".into(),
                latency_ms: 0,
                hit_count: 0,
            });
        }
    }

    let args = Args::parse();
    let workspace = resolve_workspace(args.path)?;
    let token = args
        .token
        .or_else(|| std::env::var("PRISM_TOKEN").ok())
        .unwrap_or_else(generate_token);

    eprintln!("prismd token (clients must send Authorization: Bearer …): {token}");

    let cfg = DaemonConfig {
        workspace,
        token,
        bind: args.bind,
        idle_shutdown_secs: args.idle_shutdown_secs,
        debounce_ms: args.debounce_ms,
    };
    run(cfg).await
}
