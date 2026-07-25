//! Graph store trait + SQLite adjacency (P1 Stage A writes real facts).

use anyhow::{Context, Result};
use prism_ir::FactBundle;
use rusqlite::{params, Connection};
use std::path::Path;

/// Polyglot persistence surface — SQLite now, Kuzu later.
pub trait KgStore {
    fn begin_replace_file_subgraph(&mut self, file_path: &str) -> Result<()>;
    /// Insert nodes/edges for the file currently mid-replace (call between begin and commit).
    fn insert_facts(&mut self, file_path: &str, bundle: &FactBundle) -> Result<()>;
    fn commit_replace_file_subgraph(&mut self, file_path: &str) -> Result<()>;
    fn invalidate_file_subgraph(&mut self, file_path: &str) -> Result<()>;
}

/// SQLite adjacency store.
pub struct SqliteKgStore {
    conn: Connection,
    /// Paths currently mid-replace inside an open transaction.
    pending: Option<String>,
}

impl SqliteKgStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)
            .with_context(|| format!("open graph.sqlite at {}", path.display()))?;
        conn.execute_batch(
            "
            PRAGMA journal_mode=WAL;
            PRAGMA synchronous=NORMAL;
            CREATE TABLE IF NOT EXISTS file_subgraphs (
                path TEXT PRIMARY KEY,
                stub_payload TEXT NOT NULL DEFAULT '',
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS nodes (
                id TEXT PRIMARY KEY,
                kind TEXT NOT NULL,
                file_path TEXT,
                attrs_json TEXT NOT NULL DEFAULT '{}'
            );
            CREATE TABLE IF NOT EXISTS edges (
                id TEXT PRIMARY KEY,
                kind TEXT NOT NULL,
                src TEXT NOT NULL,
                dst TEXT NOT NULL,
                file_path TEXT,
                confidence TEXT NOT NULL,
                attrs_json TEXT NOT NULL DEFAULT '{}'
            );
            CREATE INDEX IF NOT EXISTS idx_nodes_file ON nodes(file_path);
            CREATE INDEX IF NOT EXISTS idx_edges_file ON edges(file_path);
            ",
        )?;
        Ok(Self {
            conn,
            pending: None,
        })
    }

    /// Count nodes persisted for a file (tests / diagnostics).
    pub fn count_nodes_for_file(&self, file_path: &str) -> Result<u64> {
        let n: u64 = self.conn.query_row(
            "SELECT COUNT(*) FROM nodes WHERE file_path = ?1",
            params![file_path],
            |r| r.get(0),
        )?;
        Ok(n)
    }

    pub fn count_edges_for_file(&self, file_path: &str) -> Result<u64> {
        let n: u64 = self.conn.query_row(
            "SELECT COUNT(*) FROM edges WHERE file_path = ?1",
            params![file_path],
            |r| r.get(0),
        )?;
        Ok(n)
    }
}

impl KgStore for SqliteKgStore {
    fn begin_replace_file_subgraph(&mut self, file_path: &str) -> Result<()> {
        self.conn.execute_batch("BEGIN IMMEDIATE;")?;
        // Crash-safe replace: delete prior file-local facts, then insert.
        self.conn
            .execute("DELETE FROM nodes WHERE file_path = ?1", params![file_path])?;
        self.conn
            .execute("DELETE FROM edges WHERE file_path = ?1", params![file_path])?;
        // Also drop unresolved nodes that were only referenced from this file's edges
        // (unresolved nodes have file_path NULL — leave them; Stage B GC can prune).
        self.pending = Some(file_path.to_string());
        Ok(())
    }

    fn insert_facts(&mut self, file_path: &str, bundle: &FactBundle) -> Result<()> {
        if self.pending.as_deref() != Some(file_path) {
            anyhow::bail!("insert_facts without matching begin_replace for {file_path}");
        }
        for node in &bundle.nodes {
            let attrs = serde_json::to_string(node).unwrap_or_else(|_| "{}".into());
            // Prefer node.file_path; unresolved nodes stay with NULL file_path.
            let fp = node.file_path.as_deref().or(
                if node.id.starts_with("unresolved:") || node.id.starts_with("module:") {
                    None
                } else {
                    Some(file_path)
                },
            );
            self.conn.execute(
                "INSERT INTO nodes(id, kind, file_path, attrs_json) VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(id) DO UPDATE SET
                   kind = excluded.kind,
                   file_path = COALESCE(excluded.file_path, nodes.file_path),
                   attrs_json = excluded.attrs_json",
                params![node.id, node.kind.as_str(), fp, attrs],
            )?;
        }
        for edge in &bundle.edges {
            let attrs = serde_json::to_string(edge).unwrap_or_else(|_| "{}".into());
            self.conn.execute(
                "INSERT INTO edges(id, kind, src, dst, file_path, confidence, attrs_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT(id) DO UPDATE SET
                   kind = excluded.kind,
                   src = excluded.src,
                   dst = excluded.dst,
                   file_path = excluded.file_path,
                   confidence = excluded.confidence,
                   attrs_json = excluded.attrs_json",
                params![
                    edge.id,
                    edge.kind.as_str(),
                    edge.src,
                    edge.dst,
                    file_path,
                    edge.confidence.as_str(),
                    attrs,
                ],
            )?;
        }
        Ok(())
    }

    fn commit_replace_file_subgraph(&mut self, file_path: &str) -> Result<()> {
        let payload = format!("facts:schema={}", prism_ir::FACT_SCHEMA_VERSION);
        self.conn.execute(
            "INSERT INTO file_subgraphs(path, stub_payload) VALUES (?1, ?2)
             ON CONFLICT(path) DO UPDATE SET
               stub_payload = excluded.stub_payload,
               updated_at = datetime('now')",
            params![file_path, payload],
        )?;
        self.conn.execute_batch("COMMIT;")?;
        self.pending = None;
        Ok(())
    }

    fn invalidate_file_subgraph(&mut self, file_path: &str) -> Result<()> {
        self.conn.execute_batch("BEGIN IMMEDIATE;")?;
        self.conn
            .execute("DELETE FROM nodes WHERE file_path = ?1", params![file_path])?;
        self.conn
            .execute("DELETE FROM edges WHERE file_path = ?1", params![file_path])?;
        self.conn.execute(
            "DELETE FROM file_subgraphs WHERE path = ?1",
            params![file_path],
        )?;
        self.conn.execute_batch("COMMIT;")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prism_ir::{file_node_id, Confidence, FactNode, NodeKind, Tier};
    use tempfile::tempdir;

    #[test]
    fn replace_insert_and_invalidate() {
        let dir = tempdir().unwrap();
        let mut kg = SqliteKgStore::open(dir.path().join("graph.sqlite")).unwrap();
        let mut bundle = FactBundle::new("a.rs", "rust", "test");
        bundle.nodes.push(FactNode {
            id: file_node_id("a.rs"),
            kind: NodeKind::File,
            name: Some("a.rs".into()),
            file_path: Some("a.rs".into()),
            span: None,
            language: Some("rust".into()),
            analyzer: "test".into(),
            tier: Tier::T1,
            confidence: Confidence::Extracted,
            attrs: Default::default(),
        });
        kg.begin_replace_file_subgraph("a.rs").unwrap();
        kg.insert_facts("a.rs", &bundle).unwrap();
        kg.commit_replace_file_subgraph("a.rs").unwrap();
        assert_eq!(kg.count_nodes_for_file("a.rs").unwrap(), 1);
        kg.invalidate_file_subgraph("a.rs").unwrap();
        assert_eq!(kg.count_nodes_for_file("a.rs").unwrap(), 0);
    }
}
