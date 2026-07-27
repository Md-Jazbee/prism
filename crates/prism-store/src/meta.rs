//! `meta.sqlite` — snapshots, files, hashes, jobs (schema v0).

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::Path;

/// Bump when extractors change shape so content-identical files re-extract (P12).
pub const ANALYZER_PIPELINE_VERSION: &str = "p12-doc-v2-perl-java";

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
        // P12: analyzer_version column so extractor upgrades force re-extract.
        let has_analyzer: bool = conn
            .prepare("PRAGMA table_info(files)")?
            .query_map([], |r| r.get::<_, String>(1))?
            .filter_map(|c| c.ok())
            .any(|c| c == "analyzer_version");
        if !has_analyzer {
            conn.execute_batch(
                "ALTER TABLE files ADD COLUMN analyzer_version TEXT NOT NULL DEFAULT '';",
            )?;
        }
        conn.execute(
            "INSERT OR REPLACE INTO schema_meta(key, value) VALUES ('meta_schema_version', ?1)",
            params![prism_ir::META_SCHEMA_VERSION],
        )?;
        conn.execute(
            "INSERT OR REPLACE INTO schema_meta(key, value) VALUES ('analyzer_pipeline_version', ?1)",
            params![ANALYZER_PIPELINE_VERSION],
        )?;
        Ok(Self { conn })
    }

    /// Returns `(content_hash, analyzer_version)` when present.
    pub fn get_file_record(&self, path: &str) -> Result<Option<(String, String)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT content_hash, analyzer_version FROM files WHERE path = ?1")?;
        let mut rows = stmt.query(params![path])?;
        if let Some(row) = rows.next()? {
            Ok(Some((
                row.get(0)?,
                row.get::<_, String>(1).unwrap_or_default(),
            )))
        } else {
            Ok(None)
        }
    }

    pub fn get_file_hash(&self, path: &str) -> Result<Option<String>> {
        Ok(self.get_file_record(path)?.map(|(h, _)| h))
    }

    pub fn upsert_file_hash(&self, path: &str, content_hash: &str) -> Result<()> {
        self.upsert_file_hash_versioned(path, content_hash, ANALYZER_PIPELINE_VERSION)
    }

    pub fn upsert_file_hash_versioned(
        &self,
        path: &str,
        content_hash: &str,
        analyzer_version: &str,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO files(path, content_hash, analyzer_version) VALUES (?1, ?2, ?3)
             ON CONFLICT(path) DO UPDATE SET
               content_hash = excluded.content_hash,
               analyzer_version = excluded.analyzer_version,
               updated_at = datetime('now')",
            params![path, content_hash, analyzer_version],
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
        let rec = store.get_file_record("a.rs").unwrap().unwrap();
        assert_eq!(rec.1, ANALYZER_PIPELINE_VERSION);
    }
}
