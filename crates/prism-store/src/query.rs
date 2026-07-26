//! KG query shapes for P1 Stage B: resolve, neighbors, depth-limited impact.

use anyhow::Result;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};

use super::SqliteKgStore;

/// `(symbol_name, occurrences of (symbol_id, file_path))` for ambiguity heat.
pub type AmbiguousSymbolGroup = (String, Vec<(String, Option<String>)>);

/// Lightweight node view returned by queries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphNodeView {
    pub id: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    pub confidence: String,
}

/// Lightweight edge view returned by queries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphEdgeView {
    pub id: String,
    pub kind: String,
    pub src: String,
    pub dst: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    pub confidence: String,
}

/// One hop from a seed node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NeighborHit {
    pub edge: GraphEdgeView,
    pub node: GraphNodeView,
}

/// Depth-grouped impact candidate (heuristic; confidence on edges).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImpactHit {
    pub depth: u32,
    pub node: GraphNodeView,
    /// Edge kinds traversed on the path (joined).
    pub via: Vec<String>,
}

/// Direction for neighbor / impact expansion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EdgeDirection {
    #[default]
    Outgoing,
    Incoming,
    Both,
}

impl SqliteKgStore {
    /// Resolve symbols by exact name (optional path substring filter).
    pub fn resolve_symbol(
        &self,
        name: &str,
        path_contains: Option<&str>,
        limit: usize,
    ) -> Result<Vec<GraphNodeView>> {
        let limit = limit.clamp(1, 500);
        let mut out = Vec::new();
        if let Some(sub) = path_contains {
            let mut stmt = self.conn.prepare(
                "SELECT id, kind, name, file_path, attrs_json FROM nodes
                 WHERE name = ?1 AND file_path LIKE ?2
                 ORDER BY id LIMIT ?3",
            )?;
            let like = format!("%{sub}%");
            let rows = stmt.query_map(params![name, like, limit as i64], row_to_node)?;
            for r in rows {
                out.push(r?);
            }
        } else {
            let mut stmt = self.conn.prepare(
                "SELECT id, kind, name, file_path, attrs_json FROM nodes
                 WHERE name = ?1
                 ORDER BY id LIMIT ?2",
            )?;
            let rows = stmt.query_map(params![name, limit as i64], row_to_node)?;
            for r in rows {
                out.push(r?);
            }
        }
        Ok(out)
    }

    /// Resolve by exact node id.
    pub fn get_node(&self, id: &str) -> Result<Option<GraphNodeView>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, kind, name, file_path, attrs_json FROM nodes WHERE id = ?1")?;
        match stmt.query_row(params![id], row_to_node) {
            Ok(n) => Ok(Some(n)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// List nodes whose `kind` is in `kinds` (P12 doc/section prose selection).
    pub fn list_nodes_by_kinds(&self, kinds: &[&str], limit: usize) -> Result<Vec<GraphNodeView>> {
        let limit = limit.clamp(1, 500);
        if kinds.is_empty() {
            return Ok(Vec::new());
        }
        let placeholders = (1..=kinds.len())
            .map(|i| format!("?{i}"))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "SELECT id, kind, name, file_path, attrs_json FROM nodes
             WHERE kind IN ({placeholders})
             ORDER BY kind, id
             LIMIT ?{lim}",
            lim = kinds.len() + 1
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let mut params: Vec<rusqlite::types::Value> = kinds
            .iter()
            .map(|k| rusqlite::types::Value::Text((*k).to_string()))
            .collect();
        params.push(rusqlite::types::Value::Integer(limit as i64));
        let rows = stmt.query_map(rusqlite::params_from_iter(params), row_to_node)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    /// 1-hop neighbors filtered by optional edge kinds.
    pub fn neighbors(
        &self,
        node_id: &str,
        edge_kinds: Option<&[String]>,
        direction: EdgeDirection,
        limit: usize,
    ) -> Result<Vec<NeighborHit>> {
        let limit = limit.clamp(1, 1000);
        let edges = self.edges_touching(node_id, direction, edge_kinds, limit)?;
        let mut hits = Vec::new();
        for edge in edges {
            let other_id = if edge.src == node_id {
                edge.dst.clone()
            } else {
                edge.src.clone()
            };
            if let Some(node) = self.get_node(&other_id)? {
                hits.push(NeighborHit { edge, node });
            } else {
                // Synthetic / missing node still reported via edge endpoint stub.
                hits.push(NeighborHit {
                    node: GraphNodeView {
                        id: other_id,
                        kind: "Unknown".into(),
                        name: None,
                        file_path: edge.file_path.clone(),
                        confidence: edge.confidence.clone(),
                    },
                    edge,
                });
            }
        }
        Ok(hits)
    }

    /// Depth-limited forward impact candidates via CALLS / IMPORTS / CONTAINS / DEFINES.
    ///
    /// Heuristic: confidence is never upgraded; callers must treat results as T1.
    pub fn impact(&self, seed_id: &str, max_depth: u32, limit: usize) -> Result<Vec<ImpactHit>> {
        let max_depth = max_depth.clamp(1, 8);
        let limit = limit.clamp(1, 2000);
        let default_kinds = [
            "CALLS",
            "IMPORTS",
            "CONTAINS",
            "DEFINES",
            "EXTENDS",
            "IMPLEMENTS",
        ];
        let kinds: Vec<String> = default_kinds.iter().map(|s| (*s).to_string()).collect();

        let mut seen = HashSet::new();
        seen.insert(seed_id.to_string());
        let mut queue: VecDeque<(String, u32, Vec<String>)> = VecDeque::new();
        queue.push_back((seed_id.to_string(), 0, Vec::new()));
        let mut hits = Vec::new();

        while let Some((cur, depth, via)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }
            let neigh = self.neighbors(&cur, Some(&kinds), EdgeDirection::Outgoing, 500)?;
            for n in neigh {
                if !seen.insert(n.node.id.clone()) {
                    continue;
                }
                let mut next_via = via.clone();
                next_via.push(n.edge.kind.clone());
                let next_depth = depth + 1;
                hits.push(ImpactHit {
                    depth: next_depth,
                    node: n.node.clone(),
                    via: next_via.clone(),
                });
                if hits.len() >= limit {
                    return Ok(hits);
                }
                queue.push_back((n.node.id, next_depth, next_via));
            }
        }
        Ok(hits)
    }

    /// Files that should be considered dirty when `changed_path` is edited:
    /// the file itself plus any file that has an edge into a node owned by that file
    /// (reverse dependency list for incremental rebuild planning).
    pub fn reverse_dep_files(&self, changed_path: &str) -> Result<Vec<String>> {
        let mut dirty: HashSet<String> = HashSet::new();
        dirty.insert(changed_path.to_string());

        // Files with edges whose dst is a node defined in changed_path.
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT e.file_path
             FROM edges e
             JOIN nodes n ON n.id = e.dst
             WHERE n.file_path = ?1
               AND e.file_path IS NOT NULL
               AND e.file_path != ?1",
        )?;
        let rows = stmt.query_map(params![changed_path], |r| r.get::<_, String>(0))?;
        for r in rows {
            dirty.insert(r?);
        }

        // Also: edges of kind IMPORTS where dst module might be this file's module stem —
        // Stage B keeps this path-based only; module→file map is Stage C+.
        let mut out: Vec<_> = dirty.into_iter().collect();
        out.sort();
        Ok(out)
    }

    /// Union of reverse-dep dirty sets for a batch of changed paths.
    pub fn dirty_set_for_paths(&self, changed_paths: &[String]) -> Result<Vec<String>> {
        let mut all = HashSet::new();
        for p in changed_paths {
            for d in self.reverse_dep_files(p)? {
                all.insert(d);
            }
        }
        let mut out: Vec<_> = all.into_iter().collect();
        out.sort();
        Ok(out)
    }

    /// Index size estimate (bytes on disk for graph.sqlite) + node/edge counts.
    pub fn index_stats(&self) -> Result<IndexSizeStats> {
        let nodes: u64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM nodes", [], |r| r.get(0))?;
        let edges: u64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM edges", [], |r| r.get(0))?;
        let files: u64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM file_subgraphs", [], |r| r.get(0))?;
        Ok(IndexSizeStats {
            nodes,
            edges,
            files_indexed: files,
        })
    }

    /// Symbols that share a name across multiple files (ambiguity heat heuristic).
    ///
    /// Each entry is `(name, Vec<(symbol_id, file_path)>)`.
    pub fn ambiguous_symbol_names(&self, limit_names: usize) -> Result<Vec<AmbiguousSymbolGroup>> {
        let mut by_name: std::collections::BTreeMap<String, Vec<(String, Option<String>)>> =
            std::collections::BTreeMap::new();
        let mut stmt = self.conn.prepare(
            "SELECT id, name, file_path FROM nodes
             WHERE kind = 'Symbol' AND name IS NOT NULL
             ORDER BY name, id
             LIMIT 5000",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
            ))
        })?;
        for row in rows {
            let (id, name, path) = row?;
            by_name.entry(name).or_default().push((id, path));
        }
        let mut out: Vec<_> = by_name
            .into_iter()
            .filter(|(_, ids)| ids.len() >= 2)
            .collect();
        out.truncate(limit_names.clamp(1, 200));
        Ok(out)
    }

    /// CALLS confidence histogram for the ambiguity index (P3 Stage B).
    ///
    /// Returns `(total, precise, heuristic_resolved, unresolved)`.
    pub fn calls_confidence_counts(&self) -> Result<(u64, u64, u64, u64)> {
        let total: u64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM edges WHERE kind = 'CALLS'", [], |r| {
                    r.get(0)
                })?;
        let precise: u64 = self.conn.query_row(
            "SELECT COUNT(*) FROM edges WHERE kind = 'CALLS' AND confidence = 'precise'",
            [],
            |r| r.get(0),
        )?;
        let unresolved: u64 = self.conn.query_row(
            "SELECT COUNT(*) FROM edges WHERE kind = 'CALLS' AND dst LIKE 'unresolved:%'",
            [],
            |r| r.get(0),
        )?;
        let heuristic: u64 = self.conn.query_row(
            "SELECT COUNT(*) FROM edges
             WHERE kind = 'CALLS'
               AND confidence = 'heuristic'
               AND dst NOT LIKE 'unresolved:%'",
            [],
            |r| r.get(0),
        )?;
        Ok((total, precise, heuristic, unresolved))
    }

    fn edges_touching(
        &self,
        node_id: &str,
        direction: EdgeDirection,
        edge_kinds: Option<&[String]>,
        limit: usize,
    ) -> Result<Vec<GraphEdgeView>> {
        let where_clause = match direction {
            EdgeDirection::Outgoing => "src = ?1",
            EdgeDirection::Incoming => "dst = ?1",
            EdgeDirection::Both => "(src = ?1 OR dst = ?1)",
        };
        let sql = format!(
            "SELECT id, kind, src, dst, file_path, confidence FROM edges WHERE {where_clause} ORDER BY id"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params![node_id], row_to_edge)?;
        let kind_filter: Option<HashSet<&str>> =
            edge_kinds.map(|k| k.iter().map(|s| s.as_str()).collect());
        let mut out = Vec::new();
        for r in rows {
            let e = r?;
            if let Some(ref kinds) = kind_filter {
                if !kinds.contains(e.kind.as_str()) {
                    continue;
                }
            }
            out.push(e);
            if out.len() >= limit {
                break;
            }
        }
        Ok(out)
    }
}

/// Coarse index cardinality stats (size-on-disk measured by caller via metadata).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexSizeStats {
    pub nodes: u64,
    pub edges: u64,
    pub files_indexed: u64,
}

pub(crate) fn row_to_node(r: &rusqlite::Row<'_>) -> rusqlite::Result<GraphNodeView> {
    let id: String = r.get(0)?;
    let kind: String = r.get(1)?;
    let name: Option<String> = r.get(2)?;
    let file_path: Option<String> = r.get(3)?;
    let attrs_json: String = r.get(4)?;
    let confidence = confidence_from_attrs(&attrs_json);
    let name = name.or_else(|| name_from_attrs(&attrs_json));
    Ok(GraphNodeView {
        id,
        kind,
        name,
        file_path,
        confidence,
    })
}

fn row_to_edge(r: &rusqlite::Row<'_>) -> rusqlite::Result<GraphEdgeView> {
    Ok(GraphEdgeView {
        id: r.get(0)?,
        kind: r.get(1)?,
        src: r.get(2)?,
        dst: r.get(3)?,
        file_path: r.get(4)?,
        confidence: r.get(5)?,
    })
}

fn name_from_attrs(attrs_json: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(attrs_json).ok()?;
    v.get("name")?.as_str().map(|s| s.to_string())
}

fn confidence_from_attrs(attrs_json: &str) -> String {
    serde_json::from_str::<serde_json::Value>(attrs_json)
        .ok()
        .and_then(|v| {
            v.get("confidence")
                .and_then(|c| c.as_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "extracted".into())
}

/// Parse comma-separated edge kinds for CLI (`CALLS,IMPORTS`).
pub fn parse_edge_kinds(raw: Option<&str>) -> Option<Vec<String>> {
    raw.map(|s| {
        s.split(',')
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty())
            .collect()
    })
    .filter(|v: &Vec<String>| !v.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kg::{KgStore, SqliteKgStore};
    use prism_ir::{
        edge_id, file_node_id, symbol_node_id, unresolved_node_id, Confidence, EdgeKind,
        FactBundle, FactEdge, FactNode, NodeKind, Tier,
    };
    use tempfile::tempdir;

    fn seed(kg: &mut SqliteKgStore) {
        let mut bundle = FactBundle::new("a.py", "python", "test");
        let file = file_node_id("a.py");
        let helper = symbol_node_id("a.py", "function", "helper", 10);
        let main = symbol_node_id("a.py", "function", "main", 40);
        bundle.nodes.push(FactNode {
            id: file.clone(),
            kind: NodeKind::File,
            name: Some("a.py".into()),
            file_path: Some("a.py".into()),
            span: None,
            language: Some("python".into()),
            analyzer: "test".into(),
            tier: Tier::T1,
            confidence: Confidence::Extracted,
            attrs: Default::default(),
        });
        for (id, name) in [(&helper, "helper"), (&main, "main")] {
            bundle.nodes.push(FactNode {
                id: id.clone(),
                kind: NodeKind::Symbol,
                name: Some(name.into()),
                file_path: Some("a.py".into()),
                span: None,
                language: Some("python".into()),
                analyzer: "test".into(),
                tier: Tier::T1,
                confidence: Confidence::Extracted,
                attrs: Default::default(),
            });
        }
        bundle.edges.push(FactEdge {
            id: edge_id(EdgeKind::Calls, &main, &helper, 50),
            kind: EdgeKind::Calls,
            src: main.clone(),
            dst: helper.clone(),
            file_path: Some("a.py".into()),
            span: None,
            analyzer: "test".into(),
            tier: Tier::T1,
            confidence: Confidence::Heuristic,
            attrs: Default::default(),
        });
        bundle.edges.push(FactEdge {
            id: edge_id(EdgeKind::Calls, &main, &unresolved_node_id("missing"), 60),
            kind: EdgeKind::Calls,
            src: main,
            dst: unresolved_node_id("missing"),
            file_path: Some("a.py".into()),
            span: None,
            analyzer: "test".into(),
            tier: Tier::T1,
            confidence: Confidence::Heuristic,
            attrs: Default::default(),
        });

        kg.begin_replace_file_subgraph("a.py").unwrap();
        kg.insert_facts("a.py", &bundle).unwrap();
        kg.commit_replace_file_subgraph("a.py").unwrap();

        // Second file that calls into helper (reverse dep).
        let mut b2 = FactBundle::new("b.py", "python", "test");
        let bfile = file_node_id("b.py");
        let bmain = symbol_node_id("b.py", "function", "other", 1);
        b2.nodes.push(FactNode {
            id: bfile,
            kind: NodeKind::File,
            name: Some("b.py".into()),
            file_path: Some("b.py".into()),
            span: None,
            language: Some("python".into()),
            analyzer: "test".into(),
            tier: Tier::T1,
            confidence: Confidence::Extracted,
            attrs: Default::default(),
        });
        b2.nodes.push(FactNode {
            id: bmain.clone(),
            kind: NodeKind::Symbol,
            name: Some("other".into()),
            file_path: Some("b.py".into()),
            span: None,
            language: Some("python".into()),
            analyzer: "test".into(),
            tier: Tier::T1,
            confidence: Confidence::Extracted,
            attrs: Default::default(),
        });
        // Point at helper in a.py (same id).
        b2.edges.push(FactEdge {
            id: edge_id(EdgeKind::Calls, &bmain, &helper, 5),
            kind: EdgeKind::Calls,
            src: bmain,
            dst: helper,
            file_path: Some("b.py".into()),
            span: None,
            analyzer: "test".into(),
            tier: Tier::T1,
            confidence: Confidence::Heuristic,
            attrs: Default::default(),
        });
        kg.begin_replace_file_subgraph("b.py").unwrap();
        kg.insert_facts("b.py", &b2).unwrap();
        kg.commit_replace_file_subgraph("b.py").unwrap();
    }

    #[test]
    fn resolve_and_neighbors_and_impact() {
        let dir = tempdir().unwrap();
        let mut kg = SqliteKgStore::open(dir.path().join("graph.sqlite")).unwrap();
        seed(&mut kg);

        let hits = kg.resolve_symbol("helper", None, 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].name.as_deref(), Some("helper"));

        let helper_id = hits[0].id.clone();
        let neigh = kg
            .neighbors(&helper_id, None, EdgeDirection::Incoming, 10)
            .unwrap();
        assert!(!neigh.is_empty());

        let main = kg.resolve_symbol("main", Some("a.py"), 5).unwrap();
        assert_eq!(main.len(), 1);
        let impact = kg.impact(&main[0].id, 2, 50).unwrap();
        assert!(impact
            .iter()
            .any(|h| h.node.name.as_deref() == Some("helper")));
        assert!(impact.iter().any(|h| h.node.id.starts_with("unresolved:")));
    }

    #[test]
    fn reverse_dep_includes_dependents() {
        let dir = tempdir().unwrap();
        let mut kg = SqliteKgStore::open(dir.path().join("graph.sqlite")).unwrap();
        seed(&mut kg);
        let dirty = kg.reverse_dep_files("a.py").unwrap();
        assert!(dirty.contains(&"a.py".into()));
        assert!(dirty.contains(&"b.py".into()));
    }
}
