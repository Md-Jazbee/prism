//! Path-prefix communities + hub detection (P1 Stage D + P12 Stage C hub ranking v2).

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

/// Language / stdlib / unresolved noise that must not appear as architecture hubs (ACC-4).
fn is_noise_hub(node_id: &str, name: Option<&str>) -> bool {
    if node_id.starts_with("unresolved:") {
        return true;
    }
    let n = name.unwrap_or("").to_ascii_lowercase();
    matches!(
        n.as_str(),
        "into"
            | "clone"
            | "unwrap"
            | "expect"
            | "ok"
            | "err"
            | "some"
            | "none"
            | "default"
            | "new"
            | "from"
            | "as_ref"
            | "as_str"
            | "to_string"
            | "to_owned"
            | "push"
            | "insert"
            | "get"
            | "len"
            | "is_empty"
            | "iter"
            | "collect"
            | "map"
            | "filter"
            | "and_then"
            | "unwrap_or"
            | "unwrap_or_else"
            | "unwrap_or_default"
            | "format"
            | "print"
            | "println"
            | "vec"
            | "string"
            | "box"
            | "rc"
            | "arc"
            | "mutex"
            | "lock"
            | "drop"
            | "clone_from"
            | "eq"
            | "partialeq"
            | "hash"
            | "debug"
            | "display"
            | "self"
            | "super"
            | "crate"
    )
}

/// Fixture / vendored path prefixes excluded from communities unless first-party.
fn is_noise_path(path: &str) -> bool {
    let p = path.replace('\\', "/").to_ascii_lowercase();
    p.starts_with("fixtures/repos/")
        || p.starts_with("target/")
        || p.starts_with("graphify-out/")
        || p.contains("/node_modules/")
        || p.starts_with("vendor/")
        || p.starts_with("third_party/")
}

impl SqliteKgStore {
    /// Build a lightweight repo map: path-prefix communities + filtered degree hubs.
    ///
    /// Algorithm id `path_prefix_v1+resolved_degree_hubs` (P12 Stage C): same
    /// communities as v0, but hubs exclude unresolved/builtin noise (ACC-4).
    pub fn repo_map(&self, hub_limit: usize) -> Result<RepoMap> {
        let hub_limit = hub_limit.clamp(1, 50);
        let communities = self.path_prefix_communities()?;
        let hubs = self.degree_hubs_filtered(hub_limit)?;
        Ok(RepoMap {
            algorithm: "path_prefix_v1+resolved_degree_hubs".into(),
            communities,
            hubs,
            notes: vec![
                "Communities are directory prefixes (deterministic); seeded Leiden deferred.".into(),
                "Hubs ranked by undirected edge degree over resolved nodes only (unresolved/builtin denylist — ACC-4).".into(),
                "Vendored fixtures (fixtures/repos/) excluded from community rollup.".into(),
                "Do not treat hubs as precise architectural truth; CALLS remain heuristic at T1.".into(),
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
            if is_noise_path(&path) {
                continue;
            }
            let prefix = path_prefix(&path);
            let entry = by_prefix.entry(prefix).or_insert((0, 0));
            entry.0 += 1; // files
            entry.1 += nodes;
        }

        let mut communities: Vec<_> = by_prefix
            .into_iter()
            .map(|(prefix, (files, nodes))| {
                let label = extractive_community_label(&prefix);
                Community {
                    id: format!("comm:{prefix}"),
                    label,
                    path_prefix: prefix.clone(),
                    file_count: files,
                    node_count: nodes,
                }
            })
            .collect();
        communities.sort_by(|a, b| b.file_count.cmp(&a.file_count).then(a.label.cmp(&b.label)));
        // Cap for MCP payload size
        communities.truncate(40);
        Ok(communities)
    }

    /// Degree hubs with unresolved/builtin denylist (P12 Stage C / ACC-4).
    fn degree_hubs_filtered(&self, limit: usize) -> Result<Vec<Hub>> {
        // Over-fetch then filter so denylist does not leave the list empty.
        let fetch = (limit * 8).clamp(limit, 200);
        let mut stmt = self.conn.prepare(
            "SELECT n.id, n.name, n.file_path, n.kind,
                    (SELECT COUNT(*) FROM edges e WHERE e.src = n.id OR e.dst = n.id) AS deg
             FROM nodes n
             WHERE n.kind IN ('Symbol', 'File', 'Module', 'Doc', 'Section')
             ORDER BY deg DESC, n.id
             LIMIT ?1",
        )?;
        let rows = stmt.query_map([fetch as i64], |r| {
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
            if h.degree == 0 {
                continue;
            }
            if is_noise_hub(&h.node_id, h.name.as_deref()) {
                continue;
            }
            if h.file_path
                .as_deref()
                .map(is_noise_path)
                .unwrap_or(false)
            {
                continue;
            }
            hubs.push(h);
            if hubs.len() >= limit {
                break;
            }
        }
        Ok(hubs)
    }
}

/// Extractive label from path prefix (no LLM).
fn extractive_community_label(prefix: &str) -> String {
    let trimmed = prefix.trim_end_matches('/');
    // Prefer last meaningful segment: crates/prism-compile → prism-compile
    if let Some((_, leaf)) = trimmed.rsplit_once('/') {
        leaf.to_string()
    } else if trimmed.is_empty() {
        "(root)".into()
    } else {
        trimmed.to_string()
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
    use super::{is_noise_hub, path_prefix};
    use crate::kg::{KgStore, SqliteKgStore};
    use prism_ir::{
        edge_id, file_node_id, symbol_node_id, Confidence, EdgeKind, FactBundle, FactEdge, FactNode,
        NodeKind, Tier,
    };
    use tempfile::tempdir;

    #[test]
    fn prefix_helper() {
        assert_eq!(path_prefix("src/a.rs"), "src/");
        assert_eq!(path_prefix("crates/ignore/src/lib.rs"), "crates/ignore/");
    }

    #[test]
    fn noise_hub_denylist() {
        assert!(is_noise_hub("unresolved:into", Some("into")));
        assert!(is_noise_hub("sym:x:function:unwrap:1", Some("unwrap")));
        assert!(!is_noise_hub(
            "sym:crates/prism-compile/src/select.rs:function:select_from_kg:1",
            Some("select_from_kg")
        ));
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
        assert_eq!(map.algorithm, "path_prefix_v1+resolved_degree_hubs");
    }

    #[test]
    fn hubs_exclude_unresolved_builtins() {
        let dir = tempdir().unwrap();
        let mut kg = SqliteKgStore::open(dir.path().join("g.sqlite")).unwrap();
        let path = "crates/prism-compile/src/select.rs";
        let mut b = FactBundle::new(path, "rust", "test");
        let file = file_node_id(path);
        let real = symbol_node_id(path, "function", "select_from_kg", 10);
        let noise = "unresolved:into".to_string();
        b.nodes.push(FactNode {
            id: file.clone(),
            kind: NodeKind::File,
            name: Some(path.into()),
            file_path: Some(path.into()),
            span: None,
            language: Some("rust".into()),
            analyzer: "test".into(),
            tier: Tier::T1,
            confidence: Confidence::Extracted,
            attrs: Default::default(),
        });
        b.nodes.push(FactNode {
            id: real.clone(),
            kind: NodeKind::Symbol,
            name: Some("select_from_kg".into()),
            file_path: Some(path.into()),
            span: None,
            language: Some("rust".into()),
            analyzer: "test".into(),
            tier: Tier::T1,
            confidence: Confidence::Extracted,
            attrs: Default::default(),
        });
        b.nodes.push(FactNode {
            id: noise.clone(),
            kind: NodeKind::Symbol,
            name: Some("into".into()),
            file_path: None,
            span: None,
            language: Some("rust".into()),
            analyzer: "test".into(),
            tier: Tier::T1,
            confidence: Confidence::Heuristic,
            attrs: Default::default(),
        });
        // Give noise higher degree
        for i in 0..5u32 {
            b.edges.push(FactEdge {
                id: edge_id(EdgeKind::Calls, &file, &noise, i),
                kind: EdgeKind::Calls,
                src: file.clone(),
                dst: noise.clone(),
                file_path: Some(path.into()),
                span: None,
                analyzer: "test".into(),
                tier: Tier::T1,
                confidence: Confidence::Heuristic,
                attrs: Default::default(),
            });
        }
        b.edges.push(FactEdge {
            id: edge_id(EdgeKind::Defines, &file, &real, 99),
            kind: EdgeKind::Defines,
            src: file,
            dst: real.clone(),
            file_path: Some(path.into()),
            span: None,
            analyzer: "test".into(),
            tier: Tier::T1,
            confidence: Confidence::Extracted,
            attrs: Default::default(),
        });
        kg.begin_replace_file_subgraph(path).unwrap();
        kg.insert_facts(path, &b).unwrap();
        kg.commit_replace_file_subgraph(path).unwrap();

        let map = kg.repo_map(10).unwrap();
        assert!(
            map.hubs.iter().all(|h| !h.node_id.starts_with("unresolved:")),
            "hubs={:?}",
            map.hubs
        );
        assert!(
            map.hubs.iter().any(|h| h.node_id == real || h.name.as_deref() == Some("select_from_kg")),
            "expected real hub, got {:?}",
            map.hubs
        );
    }
}
