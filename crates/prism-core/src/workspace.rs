//! Workspace Manager — roots, VCS identity, dirty stamp.

use crate::fingerprint::{file_content_hash, merkle_combine};
use crate::ignore_policy::IgnorePolicy;
use anyhow::{Context, Result};
use ignore::WalkBuilder;
use prism_ir::{RepositoryId, SnapshotId};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Resolved workspace identity for indexing.
#[derive(Debug, Clone)]
pub struct WorkspaceIdentity {
    pub repository: RepositoryId,
    pub snapshot: SnapshotId,
}

/// Manages a single local workspace root (solo mode).
#[derive(Debug)]
pub struct WorkspaceManager {
    root: PathBuf,
    policy: IgnorePolicy,
}

impl WorkspaceManager {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        let root = root
            .canonicalize()
            .with_context(|| format!("canonicalize workspace root {}", root.display()))?;
        Ok(Self {
            root,
            policy: IgnorePolicy::new(),
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn policy(&self) -> &IgnorePolicy {
        &self.policy
    }

    /// Discover files respecting `.gitignore` + vendor/secret heuristics.
    pub fn discover_files(&self) -> Result<Vec<PathBuf>> {
        let mut out = Vec::new();
        let walker = WalkBuilder::new(&self.root)
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .build();

        for entry in walker {
            let entry = entry?;
            if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                continue;
            }
            let path = entry.path();
            let rel = path.strip_prefix(&self.root).unwrap_or(path);
            if self.policy.should_skip_file(rel) {
                continue;
            }
            out.push(path.to_path_buf());
        }
        out.sort();
        Ok(out)
    }

    /// Compute SnapshotId: git SHA + dirty flag + tree Merkle of hashed files.
    pub fn identity(&self) -> Result<WorkspaceIdentity> {
        let (git_commit, dirty) = self.git_state()?;
        let files = self.discover_files()?;
        let mut leaves: Vec<(String, String)> = Vec::with_capacity(files.len());
        for abs in &files {
            let rel = abs
                .strip_prefix(&self.root)
                .unwrap_or(abs)
                .to_string_lossy()
                .replace('\\', "/");
            let bytes = fs::read(abs).with_context(|| format!("read {}", abs.display()))?;
            leaves.push((rel, file_content_hash(&bytes)));
        }
        let tree_fingerprint = merkle_combine(&leaves);
        Ok(WorkspaceIdentity {
            repository: RepositoryId::new(self.root.clone()),
            snapshot: SnapshotId {
                git_commit,
                dirty,
                tree_fingerprint,
            },
        })
    }

    fn git_state(&self) -> Result<(Option<String>, bool)> {
        if !self.root.join(".git").exists() {
            return Ok((None, false));
        }
        let sha = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&self.root)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string());

        let dirty = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&self.root)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| !o.stdout.is_empty())
            .unwrap_or(false);

        Ok((sha, dirty))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn edit_changes_tree_fingerprint() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        fs::write(root.join("a.txt"), b"one").unwrap();
        let wm = WorkspaceManager::open(root).unwrap();
        let before = wm.identity().unwrap().snapshot.tree_fingerprint;
        fs::write(root.join("a.txt"), b"two").unwrap();
        let after = wm.identity().unwrap().snapshot.tree_fingerprint;
        assert_ne!(before, after);
    }

    #[test]
    fn secret_env_not_discovered() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("ok.rs"), b"fn main() {}").unwrap();
        fs::write(dir.path().join(".env"), b"SECRET=1").unwrap();
        let wm = WorkspaceManager::open(dir.path()).unwrap();
        let files = wm.discover_files().unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("ok.rs"));
    }

    #[test]
    fn planted_env_under_docs_not_discovered() {
        // P12 Stage A residual: secrets under docs/ must still be skipped.
        let dir = tempdir().unwrap();
        let docs = dir.path().join("docs");
        fs::create_dir_all(&docs).unwrap();
        fs::write(docs.join(".env"), b"API_KEY=planted-secret\n").unwrap();
        fs::write(docs.join("ok.md"), b"# Ok\n\nSafe.\n").unwrap();
        let wm = WorkspaceManager::open(dir.path()).unwrap();
        let files = wm.discover_files().unwrap();
        assert!(
            files.iter().any(|p| p.ends_with("ok.md")),
            "expected ok.md discovered: {files:?}"
        );
        assert!(
            !files.iter().any(|p| p
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n == ".env")),
            ".env under docs must not be discovered: {files:?}"
        );
    }
}
