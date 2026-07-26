//! Incremental re-index path (P1 Stage A / P6 Stage B Rayon fan-out):
//! discover → parallel hash+extract (T1) → sequential txn → invalidate.

use crate::fingerprint::file_content_hash;
use crate::workspace::WorkspaceManager;
use anyhow::Result;
use prism_extract::extract_file;
use prism_ir::{FactBundle, FileId};
use prism_obs::{emit_index_event, IndexEvent, IndexStats};
use prism_store::{KgStore, SqliteKgStore, SqliteMetaStore};
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::info;

#[derive(Debug, Clone, Default)]
pub struct IndexOptions {
    /// When true, skip writing any facts (CLI `index --dry-run`).
    pub dry_run: bool,
}

#[derive(Debug, Clone)]
pub struct IndexResult {
    pub stats: IndexStats,
    pub tree_fingerprint: String,
    pub files: Vec<FileId>,
}

enum Prepared {
    Secret {
        rel: String,
    },
    Unchanged {
        file: FileId,
    },
    Extracted {
        file: FileId,
        bundle: Option<FactBundle>,
    },
    ReadError {
        rel: String,
        error: String,
    },
}

/// Orchestrates incremental indexing with T1 extractors.
pub struct IncrementalIndexer {
    workspace: WorkspaceManager,
    meta: SqliteMetaStore,
    kg: SqliteKgStore,
}

impl IncrementalIndexer {
    pub fn open(workspace: WorkspaceManager, prism_dir: impl AsRef<Path>) -> Result<Self> {
        let prism_dir = prism_dir.as_ref();
        fs::create_dir_all(prism_dir)?;
        fs::create_dir_all(prism_dir.join("blobs"))?;
        fs::create_dir_all(prism_dir.join("logs"))?;
        let meta = SqliteMetaStore::open(prism_dir.join("meta.sqlite"))?;
        let kg = SqliteKgStore::open(prism_dir.join("graph.sqlite"))?;
        Ok(Self {
            workspace,
            meta,
            kg,
        })
    }

    /// End-to-end index path against the workspace.
    pub fn run(&mut self, opts: &IndexOptions) -> Result<IndexResult> {
        let started = Instant::now();
        let root = self.workspace.root().display().to_string();
        emit_index_event(&IndexEvent::IndexStarted {
            schema_version: prism_ir::META_SCHEMA_VERSION.to_string(),
            root: root.clone(),
        });

        let identity = self.workspace.identity()?;
        let discovered = self.workspace.discover_files()?;
        let mut stats = IndexStats {
            files_discovered: discovered.len() as u64,
            ..Default::default()
        };

        // Snapshot prior hashes for parallel skip decisions (SQLite stays single-threaded).
        // P12: skip only when content_hash AND analyzer_pipeline_version match.
        let prior_records: std::collections::HashMap<String, (String, String)> = self
            .meta
            .list_file_paths()?
            .into_iter()
            .filter_map(|p| self.meta.get_file_record(&p).ok().flatten().map(|r| (p, r)))
            .collect();
        let pipeline = prism_store::ANALYZER_PIPELINE_VERSION;

        let root_path: PathBuf = self.workspace.root().to_path_buf();
        let prepared: Vec<Prepared> = discovered
            .par_iter()
            .map(|abs| {
                let rel = abs
                    .strip_prefix(&root_path)
                    .unwrap_or(abs)
                    .to_string_lossy()
                    .replace('\\', "/");

                if crate::ignore_policy::is_secret_sensitive(abs) {
                    return Prepared::Secret { rel };
                }

                let bytes = match fs::read(abs) {
                    Ok(b) => b,
                    Err(e) => {
                        return Prepared::ReadError {
                            rel,
                            error: e.to_string(),
                        }
                    }
                };
                let content_hash = file_content_hash(&bytes);
                let unchanged = prior_records
                    .get(&rel)
                    .map(|(h, ver)| h == &content_hash && ver == pipeline)
                    .unwrap_or(false);
                if unchanged {
                    return Prepared::Unchanged {
                        file: FileId {
                            path: rel,
                            content_hash,
                        },
                    };
                }
                match extract_file(&rel, &bytes) {
                    Ok(bundle) => Prepared::Extracted {
                        file: FileId {
                            path: rel,
                            content_hash,
                        },
                        bundle,
                    },
                    Err(e) => Prepared::ReadError {
                        rel,
                        error: e.to_string(),
                    },
                }
            })
            .collect();

        let mut file_ids = Vec::new();
        for item in prepared {
            match item {
                Prepared::Secret { rel } => {
                    stats.files_secret_skipped += 1;
                    emit_index_event(&IndexEvent::FileSkippedSecret { path: rel });
                }
                Prepared::Unchanged { file } => {
                    stats.files_hashed += 1;
                    stats.files_skipped_unchanged += 1;
                    file_ids.push(file);
                }
                Prepared::ReadError { rel, error } => {
                    anyhow::bail!("index read/extract failed for {rel}: {error}");
                }
                Prepared::Extracted { file, bundle } => {
                    stats.files_hashed += 1;
                    file_ids.push(file.clone());
                    match &bundle {
                        Some(bundle) => {
                            let unresolved = bundle.unresolved_call_count() as u64;
                            emit_index_event(&IndexEvent::FileExtracted {
                                path: file.path.clone(),
                                language: bundle.language.clone(),
                                nodes: bundle.nodes.len() as u64,
                                edges: bundle.edges.len() as u64,
                                unresolved_calls: unresolved,
                            });
                            stats.files_extracted += 1;
                            stats.nodes_written += bundle.nodes.len() as u64;
                            stats.edges_written += bundle.edges.len() as u64;
                            stats.unresolved_calls += unresolved;

                            if !opts.dry_run {
                                self.kg.begin_replace_file_subgraph(&file.path)?;
                                self.kg.insert_facts(&file.path, bundle)?;
                                self.kg.commit_replace_file_subgraph(&file.path)?;
                                self.meta.upsert_file_hash(&file.path, &file.content_hash)?;
                            }
                        }
                        None => {
                            emit_index_event(&IndexEvent::FileExtractSkipped {
                                path: file.path.clone(),
                                reason: "unsupported_language".into(),
                            });
                            stats.files_extract_skipped += 1;
                            if !opts.dry_run {
                                self.kg.begin_replace_file_subgraph(&file.path)?;
                                self.kg.commit_replace_file_subgraph(&file.path)?;
                                self.meta.upsert_file_hash(&file.path, &file.content_hash)?;
                            }
                        }
                    }
                }
            }
        }

        // invalidate: drop subgraphs for paths that disappeared
        if !opts.dry_run {
            let previous = self.meta.list_file_paths()?;
            let current: std::collections::HashSet<_> =
                file_ids.iter().map(|f| f.path.clone()).collect();
            for old in previous {
                if !current.contains(&old) {
                    self.kg.invalidate_file_subgraph(&old)?;
                    self.meta.delete_file_hash(&old)?;
                    info!(path = %old, "invalidated removed file subgraph");
                }
            }
            self.meta.upsert_snapshot(
                identity.snapshot.git_commit.as_deref(),
                identity.snapshot.dirty,
                &identity.snapshot.tree_fingerprint,
            )?;
        }

        stats.wall_time_ms = started.elapsed().as_millis() as u64;
        emit_index_event(&IndexEvent::IndexFinished {
            stats: stats.clone(),
        });

        Ok(IndexResult {
            stats,
            tree_fingerprint: identity.snapshot.tree_fingerprint,
            files: file_ids,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn incremental_skips_unchanged_on_second_run() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        fs::write(root.join("a.rs"), b"fn a() {}").unwrap();
        fs::write(root.join("b.rs"), b"fn b() {}").unwrap();

        let prism = root.join(".prism");
        let wm = WorkspaceManager::open(root).unwrap();
        let mut indexer = IncrementalIndexer::open(wm, &prism).unwrap();
        let first = indexer.run(&IndexOptions::default()).unwrap();
        assert_eq!(first.stats.files_discovered, 2);
        assert_eq!(first.stats.files_skipped_unchanged, 0);
        assert_eq!(first.stats.files_hashed, 2);
        assert_eq!(first.stats.files_extracted, 2);
        assert!(first.stats.nodes_written > 0);

        let wm2 = WorkspaceManager::open(root).unwrap();
        let mut indexer2 = IncrementalIndexer::open(wm2, &prism).unwrap();
        let second = indexer2.run(&IndexOptions::default()).unwrap();
        assert_eq!(second.stats.files_skipped_unchanged, 2);

        fs::write(root.join("a.rs"), b"fn a() { /* edit */ }").unwrap();
        let wm3 = WorkspaceManager::open(root).unwrap();
        let mut indexer3 = IncrementalIndexer::open(wm3, &prism).unwrap();
        let third = indexer3.run(&IndexOptions::default()).unwrap();
        assert_eq!(third.stats.files_skipped_unchanged, 1);
        assert_eq!(third.stats.files_hashed, 2);
    }

    #[test]
    fn dry_run_does_not_persist_hashes() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.rs"), b"fn a() {}").unwrap();
        let prism = dir.path().join(".prism");
        let wm = WorkspaceManager::open(dir.path()).unwrap();
        let mut indexer = IncrementalIndexer::open(wm, &prism).unwrap();
        indexer.run(&IndexOptions { dry_run: true }).unwrap();

        let meta = SqliteMetaStore::open(prism.join("meta.sqlite")).unwrap();
        assert!(meta.list_file_paths().unwrap().is_empty());
    }

    #[test]
    fn extracts_python_and_persists_nodes() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("m.py"), b"def foo():\n    bar()\n").unwrap();
        let prism = dir.path().join(".prism");
        let wm = WorkspaceManager::open(dir.path()).unwrap();
        let mut indexer = IncrementalIndexer::open(wm, &prism).unwrap();
        let result = indexer.run(&IndexOptions::default()).unwrap();
        assert_eq!(result.stats.files_extracted, 1);
        assert!(result.stats.unresolved_calls >= 1);

        let kg = SqliteKgStore::open(prism.join("graph.sqlite")).unwrap();
        assert!(kg.count_nodes_for_file("m.py").unwrap() >= 1);
    }
}
