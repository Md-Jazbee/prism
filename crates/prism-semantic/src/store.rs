//! Persist semantic artifacts under `.prism/semantic/`.

use crate::analyze_python_file;
use crate::artifact::{SemanticFileArtifact, ALGO_VERSION, SEMANTIC_SCHEMA_VERSION};
use anyhow::{Context, Result};
use prism_core::file_content_hash;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use xxhash_rust::xxh3::xxh3_128;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticManifest {
    pub schema_version: String,
    pub algo_version: String,
    pub language: String,
    pub files: usize,
    pub functions: usize,
    pub built_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tree_fingerprint: Option<String>,
}

pub fn semantic_dir(workspace: &Path) -> PathBuf {
    workspace.join(".prism/semantic")
}

pub fn path_key(rel: &str) -> String {
    let h = xxh3_128(rel.as_bytes());
    let hex = hex::encode(h.to_be_bytes());
    let safe = rel.replace('/', "__").replace('\\', "__");
    format!("{}__{}", &hex[..16.min(hex.len())], safe)
}

pub fn artifact_path(workspace: &Path, rel: &str) -> PathBuf {
    semantic_dir(workspace)
        .join("by-file")
        .join(format!("{}.json", path_key(rel)))
}

pub fn save_file_artifact(workspace: &Path, art: &SemanticFileArtifact) -> Result<PathBuf> {
    let dir = semantic_dir(workspace).join("by-file");
    fs::create_dir_all(&dir)?;
    let dest = artifact_path(workspace, &art.path);
    fs::write(&dest, serde_json::to_string_pretty(art)? + "\n")
        .with_context(|| format!("write {}", dest.display()))?;
    Ok(dest)
}

pub fn load_file_artifact(workspace: &Path, rel: &str) -> Result<Option<SemanticFileArtifact>> {
    let dest = artifact_path(workspace, rel);
    if !dest.exists() {
        // try basename-only key variants by scanning
        let dir = semantic_dir(workspace).join("by-file");
        if !dir.exists() {
            return Ok(None);
        }
        for ent in fs::read_dir(&dir)? {
            let ent = ent?;
            let p = ent.path();
            if p.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let text = fs::read_to_string(&p)?;
            let art: SemanticFileArtifact = serde_json::from_str(&text)?;
            if art.path == rel || art.path.ends_with(rel) {
                return Ok(Some(art));
            }
        }
        return Ok(None);
    }
    let text = fs::read_to_string(&dest)?;
    Ok(Some(serde_json::from_str(&text)?))
}

pub fn write_manifest(workspace: &Path, manifest: &SemanticManifest) -> Result<()> {
    let dir = semantic_dir(workspace);
    fs::create_dir_all(&dir)?;
    let path = dir.join("manifest.json");
    fs::write(&path, serde_json::to_string_pretty(manifest)? + "\n")?;
    Ok(())
}

pub fn read_manifest(workspace: &Path) -> Result<Option<SemanticManifest>> {
    let path = semantic_dir(workspace).join("manifest.json");
    if !path.exists() {
        return Ok(None);
    }
    Ok(Some(serde_json::from_str(&fs::read_to_string(path)?)?))
}

pub fn build_file_artifact(workspace: &Path, rel: &str) -> Result<SemanticFileArtifact> {
    let abs = workspace.join(rel);
    let bytes = fs::read(&abs).with_context(|| format!("read {}", abs.display()))?;
    let source = String::from_utf8_lossy(&bytes);
    let hash = Some(file_content_hash(&bytes));
    let art = analyze_python_file(rel, &source, hash);
    save_file_artifact(workspace, &art)?;
    Ok(art)
}

/// Discover `**/*.py` (skip `.prism`, venv, etc. via simple filters).
pub fn build_workspace_python(workspace: &Path) -> Result<SemanticManifest> {
    let mut files = 0usize;
    let mut functions = 0usize;
    walk_py(workspace, workspace, &mut files, &mut functions)?;
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let manifest = SemanticManifest {
        schema_version: SEMANTIC_SCHEMA_VERSION.into(),
        algo_version: ALGO_VERSION.into(),
        language: "python".into(),
        files,
        functions,
        built_at: format!("unix:{secs}"),
        tree_fingerprint: None,
    };
    write_manifest(workspace, &manifest)?;
    Ok(manifest)
}

fn walk_py(
    workspace: &Path,
    dir: &Path,
    files: &mut usize,
    functions: &mut usize,
) -> Result<()> {
    let skip = [".prism", ".git", "node_modules", "target", ".venv", "venv", "__pycache__"];
    for ent in fs::read_dir(dir)? {
        let ent = ent?;
        let name = ent.file_name();
        let name = name.to_string_lossy();
        if skip.iter().any(|s| *s == name) {
            continue;
        }
        let path = ent.path();
        if path.is_dir() {
            walk_py(workspace, &path, files, functions)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("py") {
            let rel = path
                .strip_prefix(workspace)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            match build_file_artifact(workspace, &rel) {
                Ok(art) => {
                    *files += 1;
                    *functions += art.functions.len();
                }
                Err(e) => {
                    // soft-skip
                    eprintln!("# semantic skip {rel}: {e}");
                }
            }
        }
    }
    Ok(())
}
