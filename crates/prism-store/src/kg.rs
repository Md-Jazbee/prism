//! Graph store trait + SQLite adjacency (P1 Stage A/B).

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
    pub(crate) conn: Connection,
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
                name TEXT,
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
            CREATE INDEX IF NOT EXISTS idx_nodes_name ON nodes(name);
            CREATE INDEX IF NOT EXISTS idx_edges_file ON edges(file_path);
            CREATE INDEX IF NOT EXISTS idx_edges_src ON edges(src);
            CREATE INDEX IF NOT EXISTS idx_edges_dst ON edges(dst);
            CREATE INDEX IF NOT EXISTS idx_edges_kind ON edges(kind);
            ",
        )?;
        // Migrate older Stage A DBs that lack `name`.
        let has_name: bool = conn
            .prepare("PRAGMA table_info(nodes)")?
            .query_map([], |r| r.get::<_, String>(1))?
            .filter_map(|c| c.ok())
            .any(|c| c == "name");
        if !has_name {
            conn.execute_batch("ALTER TABLE nodes ADD COLUMN name TEXT;")?;
            conn.execute_batch("CREATE INDEX IF NOT EXISTS idx_nodes_name ON nodes(name);")?;
        }
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

    /// Lightweight edge rows for a file (P3 overlay join).
    pub fn edges_for_file(&self, file_path: &str) -> Result<Vec<crate::query::GraphEdgeView>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, kind, src, dst, file_path, confidence FROM edges
             WHERE file_path = ?1 ORDER BY id",
        )?;
        let rows = stmt.query_map(params![file_path], |r| {
            Ok(crate::query::GraphEdgeView {
                id: r.get(0)?,
                kind: r.get(1)?,
                src: r.get(2)?,
                dst: r.get(3)?,
                file_path: r.get(4)?,
                confidence: r.get(5)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    /// Load a full [`FactEdge`] from `attrs_json` when present.
    pub fn load_fact_edge(&self, edge_id: &str) -> Result<Option<prism_ir::FactEdge>> {
        let mut stmt = self
            .conn
            .prepare("SELECT attrs_json FROM edges WHERE id = ?1")?;
        match stmt.query_row(params![edge_id], |r| r.get::<_, String>(0)) {
            Ok(attrs) => {
                let edge: prism_ir::FactEdge = serde_json::from_str(&attrs).with_context(|| {
                    format!("deserialize FactEdge attrs for {edge_id}")
                })?;
                Ok(Some(edge))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Upsert overlay nodes without wiping the file subgraph (P3 T2 attach).
    pub fn upsert_overlay_nodes(&mut self, nodes: &[prism_ir::FactNode]) -> Result<()> {
        for node in nodes {
            let attrs = serde_json::to_string(node).unwrap_or_else(|_| "{}".into());
            self.conn.execute(
                "INSERT INTO nodes(id, kind, name, file_path, attrs_json) VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(id) DO UPDATE SET
                   kind = excluded.kind,
                   name = excluded.name,
                   file_path = COALESCE(excluded.file_path, nodes.file_path),
                   attrs_json = excluded.attrs_json",
                params![
                    node.id,
                    node.kind.as_str(),
                    node.name,
                    node.file_path,
                    attrs
                ],
            )?;
        }
        Ok(())
    }

    /// Upsert a single overlay / refined edge (P3).
    pub fn upsert_overlay_edge(&mut self, edge: &prism_ir::FactEdge) -> Result<()> {
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
                edge.file_path,
                edge.confidence.as_str(),
                attrs,
            ],
        )?;
        Ok(())
    }

    /// True if any edge touching `node_id` has confidence=precise.
    pub fn symbol_has_precise_edges(&self, node_id: &str) -> Result<bool> {
        let n: u64 = self.conn.query_row(
            "SELECT COUNT(*) FROM edges
             WHERE confidence = 'precise' AND (src = ?1 OR dst = ?1)",
            params![node_id],
            |r| r.get(0),
        )?;
        Ok(n > 0)
    }
}

impl KgStore for SqliteKgStore {
    fn begin_replace_file_subgraph(&mut self, file_path: &str) -> Result<()> {
        self.conn.execute_batch("BEGIN IMMEDIATE;")?;
        self.conn
            .execute("DELETE FROM nodes WHERE file_path = ?1", params![file_path])?;
        self.conn
            .execute("DELETE FROM edges WHERE file_path = ?1", params![file_path])?;
        self.pending = Some(file_path.to_string());
        Ok(())
    }

    fn insert_facts(&mut self, file_path: &str, bundle: &FactBundle) -> Result<()> {
        if self.pending.as_deref() != Some(file_path) {
            anyhow::bail!("insert_facts without matching begin_replace for {file_path}");
        }
        for node in &bundle.nodes {
            let attrs = serde_json::to_string(node).unwrap_or_else(|_| "{}".into());
            let fp = node.file_path.as_deref().or(
                if node.id.starts_with("unresolved:") || node.id.starts_with("module:") {
                    None
                } else {
                    Some(file_path)
                },
            );
            self.conn.execute(
                "INSERT INTO nodes(id, kind, name, file_path, attrs_json) VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(id) DO UPDATE SET
                   kind = excluded.kind,
                   name = excluded.name,
                   file_path = COALESCE(excluded.file_path, nodes.file_path),
                   attrs_json = excluded.attrs_json",
                params![node.id, node.kind.as_str(), node.name, fp, attrs],
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
