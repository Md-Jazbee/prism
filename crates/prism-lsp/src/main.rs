//! `prism-lsp` binary — stdio Language Server Protocol host.

use anyhow::Result;
use clap::Parser;
use prism_lsp::{run_stdio, LspConfig};
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(
    name = "prism-lsp",
    version,
    about = "Prism LSP — evidence hover / codelens / symbols"
)]
struct Args {
    /// Workspace root (must contain `.prism/` index for full features).
    #[arg(long, default_value = ".")]
    workspace: PathBuf,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .with_writer(std::io::stderr)
        .init();

    let args = Args::parse();
    let workspace = std::fs::canonicalize(&args.workspace).unwrap_or(args.workspace);
    run_stdio(LspConfig { workspace })
}
