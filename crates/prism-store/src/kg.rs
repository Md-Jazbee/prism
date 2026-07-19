//! Graph store trait + SQLite adjacency stub.
//!
//! Transaction boundary: replace-file-subgraph (planning Stage B).

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::Path;

/// Polyglot persistence surface — SQLite now, Kuzu later.
pub trait KgStore {
    fn begin_replace_file_subgraph(&mut self, file_path: &str) -> Result<()>;
    fn commit_replace_file_subgraph(&mut self, file_path: &str) -> Result<()>;
    fn invalidate_file_subgraph(&mut self, file_path: &str) -> Result<()>;
}

/// SQLite adjacency stub for P0 (no real nodes yet).
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
}

impl KgStore for SqliteKgStore {
    fn begin_replace_file_subgraph(&mut self, file_path: &str) -> Result<()> {
        self.conn.execute_batch("BEGIN IMMEDIATE;")?;
        // Crash-safe replace: delete prior file-local facts, then callers insert (P1).
        self.conn
            .execute("DELETE FROM nodes WHERE file_path = ?1", params![file_path])?;
        self.conn
            .execute("DELETE FROM edges WHERE file_path = ?1", params![file_path])?;
        self.pending = Some(file_path.to_string());
        Ok(())
    }

    fn commit_replace_file_subgraph(&mut self, file_path: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO file_subgraphs(path, stub_payload) VALUES (?1, ?2)
             ON CONFLICT(path) DO UPDATE SET
               stub_payload = excluded.stub_payload,
               updated_at = datetime('now')",
            params![file_path, "parse_hook_stub"],
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
    use tempfile::tempdir;

    #[test]
    fn replace_and_invalidate() {
        let dir = tempdir().unwrap();
        let mut kg = SqliteKgStore::open(dir.path().join("graph.sqlite")).unwrap();
        kg.begin_replace_file_subgraph("a.rs").unwrap();
        kg.commit_replace_file_subgraph("a.rs").unwrap();
        kg.invalidate_file_subgraph("a.rs").unwrap();
    }
}
