//! Path-prefix communities + hub detection (P1 Stage D lightweight).

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::SqliteKgStore;

/// A community labeled by path prefix (directory cluster).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Community {
    pub id: String,
    pub label: String,
    pub path_prefix: String,
    pub file_count: u64,
    pub node_count: u64,
}

/// High-degree symbol/file hubs for orientation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hub {
    pub node_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    pub degree: u64,
    pub kind: String,
}

/// Repo orientation map returned by `repo_map`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoMap {
    pub algorithm: String,
    pub communities: Vec<Community>,
    pub hubs: Vec<Hub>,
    pub notes: Vec<String>,
}

impl SqliteKgStore {
    /// Build a lightweight repo map: path-prefix communities + degree hubs.
    ///
    /// Refresh policy: recompute on demand (Stage D). Incremental community
    /// refresh on edit is deferred; dirty lists from Stage B remain advisory.
    pub fn repo_map(&self, hub_limit: usize) -> Result<RepoMap> {
        let hub_limit = hub_limit.clamp(1, 50);
        let communities = self.path_prefix_communities()?;
        let hubs = self.degree_hubs(hub_limit)?;
        Ok(RepoMap {
            algorithm: "path_prefix_v0+degree_hubs".into(),
            communities,
            hubs,
            notes: vec![
                "Communities are directory prefixes (deterministic), not Leiden yet.".into(),
                "Hubs ranked by undirected edge degree; CALLS are heuristic at T1.".into(),
                "Do not treat hubs as precise architectural truth.".into(),
            ],
        })
    }

    fn path_prefix_communities(&self) -> Result<Vec<Community>> {
        let mut stmt = self.conn.prepare(
            "SELECT file_path, COUNT(*) FROM nodes
             WHERE file_path IS NOT NULL
             GROUP BY file_path",
        )?;
        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, u64>(1)?)))?;

        let mut by_prefix: HashMap<String, (u64, u64)> = HashMap::new();
        for row in rows {
            let (path, nodes) = row?;
            let prefix = path_prefix(&path);
            let entry = by_prefix.entry(prefix).or_insert((0, 0));
            entry.0 += 1; // files
            entry.1 += nodes;
        }

        let mut communities: Vec<_> = by_prefix
            .into_iter()
            .map(|(prefix, (files, nodes))| Community {
                id: format!("comm:{prefix}"),
                label: prefix.trim_end_matches('/').to_string(),
                path_prefix: prefix.clone(),
                file_count: files,
                node_count: nodes,
            })
            .collect();
        communities.sort_by(|a, b| b.file_count.cmp(&a.file_count).then(a.label.cmp(&b.label)));
        // Cap for MCP payload size
        communities.truncate(40);
        Ok(communities)
    }

    fn degree_hubs(&self, limit: usize) -> Result<Vec<Hub>> {
        let mut stmt = self.conn.prepare(
            "SELECT n.id, n.name, n.file_path, n.kind,
                    (SELECT COUNT(*) FROM edges e WHERE e.src = n.id OR e.dst = n.id) AS deg
             FROM nodes n
             WHERE n.kind IN ('Symbol', 'File', 'Module')
             ORDER BY deg DESC, n.id
             LIMIT ?1",
        )?;
        let rows = stmt.query_map([limit as i64], |r| {
            Ok(Hub {
                node_id: r.get(0)?,
                name: r.get(1)?,
                file_path: r.get(2)?,
                kind: r.get(3)?,
                degree: r.get(4)?,
            })
        })?;
        let mut hubs = Vec::new();
        for r in rows {
            let h = r?;
            if h.degree > 0 {
                hubs.push(h);
            }
        }
        Ok(hubs)
    }
}

fn path_prefix(path: &str) -> String {
    if let Some((dir, _)) = path.rsplit_once('/') {
        if dir.is_empty() {
            "./".into()
        } else {
            // Use first two path segments when deep, else one.
            let parts: Vec<_> = dir.split('/').collect();
            if parts.len() >= 2 {
                format!("{}/{}/", parts[0], parts[1])
            } else {
                format!("{}/", parts[0])
            }
        }
    } else {
        "./".into()
    }
}

#[cfg(test)]
mod tests {
    use super::path_prefix;
    use crate::kg::{KgStore, SqliteKgStore};
    use prism_ir::{file_node_id, Confidence, FactBundle, FactNode, NodeKind, Tier};
    use tempfile::tempdir;

    #[test]
    fn prefix_helper() {
        assert_eq!(path_prefix("src/a.rs"), "src/");
        assert_eq!(path_prefix("crates/ignore/src/lib.rs"), "crates/ignore/");
    }

    #[test]
    fn repo_map_groups_by_prefix() {
        let dir = tempdir().unwrap();
        let mut kg = SqliteKgStore::open(dir.path().join("g.sqlite")).unwrap();
        for (path, name) in [("src/a.rs", "a"), ("src/b.rs", "b"), ("tests/t.rs", "t")] {
            let mut b = FactBundle::new(path, "rust", "test");
            b.nodes.push(FactNode {
                id: file_node_id(path),
                kind: NodeKind::File,
                name: Some(name.into()),
                file_path: Some(path.into()),
                span: None,
                language: Some("rust".into()),
                analyzer: "test".into(),
                tier: Tier::T1,
                confidence: Confidence::Extracted,
                attrs: Default::default(),
            });
            kg.begin_replace_file_subgraph(path).unwrap();
            kg.insert_facts(path, &b).unwrap();
            kg.commit_replace_file_subgraph(path).unwrap();
        }
        let map = kg.repo_map(10).unwrap();
        assert!(!map.communities.is_empty());
        assert_eq!(map.algorithm, "path_prefix_v0+degree_hubs");
    }
}
