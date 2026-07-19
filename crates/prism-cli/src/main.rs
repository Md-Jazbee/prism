//! `prism` CLI — Phase 0 stubs (`--help`, `index --dry-run`).

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use prism_core::{IncrementalIndexer, IndexOptions, WorkspaceManager};
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(
    name = "prism",
    version,
    about = "Prism — Repository Intelligence Platform",
    long_about = "Pre-LLM repository understanding: index → knowledge graph → context compilation.\nPhase 0: workspace identity, fingerprinting, meta.sqlite stubs."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Index a workspace (discover → hash → parse-hook stub → txn → invalidate).
    Index {
        /// Workspace root (default: cwd).
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Discover and hash without writing meta/graph updates.
        #[arg(long)]
        dry_run: bool,
    },
    /// Print workspace identity (git SHA, dirty, tree fingerprint).
    Doctor {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Index { path, dry_run } => {
            let wm = WorkspaceManager::open(&path)
                .with_context(|| format!("open workspace {}", path.display()))?;
            let prism_dir = wm.root().join(".prism");
            let mut indexer = IncrementalIndexer::open(wm, &prism_dir)?;
            let result = indexer.run(&IndexOptions { dry_run })?;
            println!(
                "indexed: discovered={} hashed={} skipped_unchanged={} secret_skipped={} wall_ms={} fingerprint={}{}",
                result.stats.files_discovered,
                result.stats.files_hashed,
                result.stats.files_skipped_unchanged,
                result.stats.files_secret_skipped,
                result.stats.wall_time_ms,
                &result.tree_fingerprint[..result.tree_fingerprint.len().min(16)],
                if dry_run { " (dry-run)" } else { "" }
            );
        }
        Commands::Doctor { path } => {
            let wm = WorkspaceManager::open(&path)?;
            let id = wm.identity()?;
            println!("root: {}", id.repository.root.display());
            println!(
                "git_commit: {}",
                id.snapshot.git_commit.as_deref().unwrap_or("(none)")
            );
            println!("dirty: {}", id.snapshot.dirty);
            println!("tree_fingerprint: {}", id.snapshot.tree_fingerprint);
            println!(
                "schema: meta={} fact={} events={}",
                prism_ir::META_SCHEMA_VERSION,
                prism_ir::FACT_SCHEMA_VERSION,
                prism_ir::EVENTS_SCHEMA_VERSION
            );
        }
    }
    Ok(())
}
