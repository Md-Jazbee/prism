//! Workspace / snapshot / file identity (Stage A handoff to Stage B).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Stable identity for a repository root (local solo mode).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RepositoryId {
    /// Absolute or canonical workspace root path.
    pub root: PathBuf,
}

impl RepositoryId {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

/// Snapshot identity: clean git commit vs dirty worktree stamp.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SnapshotId {
    /// Git HEAD SHA when available; `None` for non-git trees.
    pub git_commit: Option<String>,
    /// True when the worktree has uncommitted changes.
    pub dirty: bool,
    /// Content fingerprint of the tracked tree (directory Merkle root).
    pub tree_fingerprint: String,
}

/// Per-file content identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FileId {
    /// Repo-relative POSIX-ish path (forward slashes).
    pub path: String,
    /// XXH3-128 hex of file contents (or empty marker for missing).
    pub content_hash: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dirty_differs_from_clean() {
        let clean = SnapshotId {
            git_commit: Some("abc".into()),
            dirty: false,
            tree_fingerprint: "t1".into(),
        };
        let dirty = SnapshotId {
            git_commit: Some("abc".into()),
            dirty: true,
            tree_fingerprint: "t1".into(),
        };
        assert_ne!(clean, dirty);
    }
}
