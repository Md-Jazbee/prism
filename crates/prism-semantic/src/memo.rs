//! Slice memoization: keys include `(snapshot_id, algorithm_version, params_hash)`.

use crate::artifact::INTERPROC_ALGO_VERSION;
use crate::interproc::InterprocSliceReport;
use crate::store::semantic_dir;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use xxhash_rust::xxh3::xxh3_128;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoEntry {
    pub memo_key: String,
    pub snapshot_id: String,
    pub algorithm_version: String,
    pub params_hash: String,
    pub report: InterprocSliceReport,
}

pub fn params_hash(params: &serde_json::Value) -> String {
    let canonical = serde_json::to_string(params).unwrap_or_default();
    let h = xxh3_128(canonical.as_bytes());
    hex::encode(h.to_be_bytes())
}

pub fn memo_key(snapshot_id: &str, algo: &str, params_hash: &str) -> String {
    let h = xxh3_128(format!("{snapshot_id}|{algo}|{params_hash}").as_bytes());
    hex::encode(h.to_be_bytes())
}

pub fn load_memo(workspace: &Path, key: &str) -> Result<Option<MemoEntry>> {
    let path = semantic_dir(workspace)
        .join("memo")
        .join(format!("{key}.json"));
    if !path.exists() {
        return Ok(None);
    }
    Ok(Some(serde_json::from_str(&fs::read_to_string(path)?)?))
}

pub fn save_memo(workspace: &Path, entry: &MemoEntry) -> Result<()> {
    let dir = semantic_dir(workspace).join("memo");
    fs::create_dir_all(&dir)?;
    let dest = dir.join(format!("{}.json", entry.memo_key));
    fs::write(&dest, serde_json::to_string_pretty(entry)? + "\n")
        .with_context(|| format!("write {}", dest.display()))?;
    Ok(())
}

pub fn default_algo() -> &'static str {
    INTERPROC_ALGO_VERSION
}
