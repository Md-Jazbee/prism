//! Prism LSP — augments editors; does not replace rust-analyzer / pylsp.

mod server;

pub use server::run_stdio;

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct LspConfig {
    pub workspace: PathBuf,
}
