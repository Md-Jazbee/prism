//! `meta.sqlite` — snapshots, files, hashes, jobs (schema v0).

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::Path;

/// Metadata store backed by SQLite WAL.
pub struct SqliteMetaStore {
    conn: Connection,
}

impl SqliteMetaStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)
            .with_context(|| format!("open meta.sqlite at {}", path.display()))?;
        conn.execute_batch(
            "
            PRAGMA journal_mode=WAL;
            PRAGMA synchronous=NORMAL;
            CREATE TABLE IF NOT EXISTS schema_meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                git_commit TEXT,
                dirty INTEGER NOT NULL,
                tree_fingerprint TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS files (
                path TEXT PRIMARY KEY,
                content_hash TEXT NOT NULL,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS jobs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                kind TEXT NOT NULL,
                status TEXT NOT NULL,
                detail TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            ",
        )?;
        conn.execute(
            "INSERT OR REPLACE INTO schema_meta(key, value) VALUES ('meta_schema_version', ?1)",
            params![prism_ir::META_SCHEMA_VERSION],
        )?;
        Ok(Self { conn })
    }

    pub fn get_file_hash(&self, path: &str) -> Result<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT content_hash FROM files WHERE path = ?1")?;
        let mut rows = stmt.query(params![path])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    pub fn upsert_file_hash(&self, path: &str, content_hash: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO files(path, content_hash) VALUES (?1, ?2)
             ON CONFLICT(path) DO UPDATE SET
               content_hash = excluded.content_hash,
               updated_at = datetime('now')",
            params![path, content_hash],
        )?;
        Ok(())
    }

    pub fn delete_file_hash(&self, path: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM files WHERE path = ?1", params![path])?;
        Ok(())
    }

    pub fn list_file_paths(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare("SELECT path FROM files ORDER BY path")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn upsert_snapshot(
        &self,
        git_commit: Option<&str>,
        dirty: bool,
        tree_fingerprint: &str,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO snapshots(git_commit, dirty, tree_fingerprint) VALUES (?1, ?2, ?3)",
            params![git_commit, dirty as i32, tree_fingerprint],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn wal_upsert_and_skip() {
        let dir = tempdir().unwrap();
        let store = SqliteMetaStore::open(dir.path().join("meta.sqlite")).unwrap();
        assert!(store.get_file_hash("a.rs").unwrap().is_none());
        store.upsert_file_hash("a.rs", "abc").unwrap();
        assert_eq!(store.get_file_hash("a.rs").unwrap().as_deref(), Some("abc"));
        store.upsert_file_hash("a.rs", "def").unwrap();
        assert_eq!(store.get_file_hash("a.rs").unwrap().as_deref(), Some("def"));
    }
}
