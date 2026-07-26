//! Semantic communities + hub detection (P1 Stage D + P12 Stage C).
//!
//! Algorithm `louvain_v1+resolved_degree_hubs`: deterministic Louvain over a
//! file-level graph (IMPORTS / CALLS / DESCRIBES / MENTIONS / CONTAINS plus
//! soft co-directory edges), with path-prefix fallback when the structural
//! graph is too sparse. Hubs exclude unresolved/builtins.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use super::SqliteKgStore;

type FileAdj = HashMap<String, HashMap<String, f64>>;

/// A community (Louvain cluster or path-prefix fallback).
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

/// Cross-community edge (“bridge”) for orientation (heuristic).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bridge {
    pub src_file: String,
    pub dst_file: String,
    pub edge_kind: String,
    pub src_community: String,
    pub dst_community: String,
    pub weight: f64,
}

/// Repo orientation map returned by `repo_map`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoMap {
    pub algorithm: String,
    pub communities: Vec<Community>,
    pub hubs: Vec<Hub>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bridges: Vec<Bridge>,
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
    /// Build repo map: Louvain communities (or path-prefix fallback) + filtered hubs + bridges.
    ///
    /// Algorithm id `louvain_v1+resolved_degree_hubs` (P12 Stage C).
    pub fn repo_map(&self, hub_limit: usize) -> Result<RepoMap> {
        let hub_limit = hub_limit.clamp(1, 50);
        let (communities, bridges, used_louvain) = self.semantic_communities()?;
        let hubs = self.degree_hubs_filtered(hub_limit)?;
        let mut notes = vec![
            if used_louvain {
                "Communities from deterministic Louvain on file-level edges (IMPORTS/CALLS/DESCRIBES/MENTIONS/CONTAINS + co-directory; seeded by sorted file id)."
                    .into()
            } else {
                "Communities fell back to directory prefixes (structural graph too sparse for Louvain)."
                    .into()
            },
            "Hubs ranked by undirected edge degree over resolved nodes only (unresolved/builtin denylist — ACC-4)."
                .into(),
            "Vendored fixtures (fixtures/repos/) excluded from community rollup.".into(),
            "Bridges are cross-community edges, labeled heuristic.".into(),
            "Do not treat hubs as precise architectural truth; CALLS remain heuristic at T1.".into(),
        ];
        if !bridges.is_empty() {
            notes.push(format!(
                "Bridge report capped at {} cross-community edges.",
                bridges.len()
            ));
        }
        Ok(RepoMap {
            algorithm: if used_louvain {
                "louvain_v1+resolved_degree_hubs".into()
            } else {
                "path_prefix_v1+resolved_degree_hubs".into()
            },
            communities,
            hubs,
            bridges,
            notes,
        })
    }

    /// Prefer Louvain; fall back to path-prefix when &lt; 2 structural edges.
    fn semantic_communities(&self) -> Result<(Vec<Community>, Vec<Bridge>, bool)> {
        let (adj, files) = self.file_adjacency()?;
        let edge_count: usize = adj.values().map(|n| n.len()).sum::<usize>() / 2;
        if edge_count < 2 || files.len() < 3 {
            let communities = self.path_prefix_communities()?;
            return Ok((communities, Vec::new(), false));
        }
        let membership = louvain_cluster(&files, &adj);
        let communities = self.communities_from_membership(&membership)?;
        let bridges = self.bridges_from_membership(&membership, 20)?;
        Ok((communities, bridges, true))
    }

    /// Undirected weighted adjacency among non-noise files.
    fn file_adjacency(&self) -> Result<(FileAdj, Vec<String>)> {
        let mut files: HashSet<String> = HashSet::new();
        {
            let mut stmt = self
                .conn
                .prepare("SELECT DISTINCT file_path FROM nodes WHERE file_path IS NOT NULL")?;
            let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
            for r in rows {
                let p = r?;
                if !is_noise_path(&p) {
                    files.insert(p);
                }
            }
        }

        let mut adj: FileAdj = HashMap::new();
        for f in &files {
            adj.insert(f.clone(), HashMap::new());
        }

        let mut stmt = self.conn.prepare(
            "SELECT e.kind, ns.file_path, nd.file_path
             FROM edges e
             JOIN nodes ns ON ns.id = e.src
             JOIN nodes nd ON nd.id = e.dst
             WHERE e.kind IN ('IMPORTS', 'CALLS', 'DESCRIBES', 'MENTIONS', 'CONTAINS')
               AND ns.file_path IS NOT NULL AND nd.file_path IS NOT NULL",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })?;
        for row in rows {
            let (kind, src, dst) = row?;
            if src == dst || is_noise_path(&src) || is_noise_path(&dst) {
                continue;
            }
            if !files.contains(&src) || !files.contains(&dst) {
                continue;
            }
            let w = match kind.as_str() {
                "IMPORTS" | "CALLS" => 1.0,
                "DESCRIBES" => 0.5,
                "MENTIONS" => 0.25,
                "CONTAINS" => 0.35,
                _ => 0.1,
            };
            *adj.entry(src.clone())
                .or_default()
                .entry(dst.clone())
                .or_insert(0.0) += w;
            *adj.entry(dst).or_default().entry(src).or_insert(0.0) += w;
        }

        // Soft co-directory edges so Louvain has a connected graph when most
        // CALLS target unresolved: builtins (common at T1).
        let mut by_prefix: HashMap<String, Vec<String>> = HashMap::new();
        for f in &files {
            by_prefix.entry(path_prefix(f)).or_default().push(f.clone());
        }
        for members in by_prefix.values() {
            if members.len() < 2 {
                continue;
            }
            // Connect consecutive sorted members (path) — O(n) spanning tree per prefix.
            let mut sorted = members.clone();
            sorted.sort();
            for w in sorted.windows(2) {
                let (a, b) = (&w[0], &w[1]);
                *adj.entry(a.clone())
                    .or_default()
                    .entry(b.clone())
                    .or_insert(0.0) += 0.2;
                *adj.entry(b.clone())
                    .or_default()
                    .entry(a.clone())
                    .or_insert(0.0) += 0.2;
            }
        }

        let mut file_list: Vec<String> = files.into_iter().collect();
        file_list.sort();
        Ok((adj, file_list))
    }

    fn communities_from_membership(
        &self,
        membership: &HashMap<String, usize>,
    ) -> Result<Vec<Community>> {
        let mut by_comm: HashMap<usize, Vec<String>> = HashMap::new();
        for (file, cid) in membership {
            by_comm.entry(*cid).or_default().push(file.clone());
        }
        let mut communities = Vec::new();
        for (cid, mut files) in by_comm {
            files.sort();
            let prefix = majority_path_prefix(&files);
            let label = extractive_community_label(&prefix);
            let node_count = self.count_nodes_in_files(&files)?;
            communities.push(Community {
                id: format!("comm:louvain:{cid}"),
                label,
                path_prefix: prefix,
                file_count: files.len() as u64,
                node_count,
            });
        }
        communities.sort_by(|a, b| {
            b.file_count
                .cmp(&a.file_count)
                .then(a.label.cmp(&b.label))
                .then(a.id.cmp(&b.id))
        });
        communities.truncate(40);
        Ok(communities)
    }

    fn count_nodes_in_files(&self, files: &[String]) -> Result<u64> {
        if files.is_empty() {
            return Ok(0);
        }
        let mut total = 0u64;
        let mut stmt = self
            .conn
            .prepare("SELECT COUNT(*) FROM nodes WHERE file_path = ?1")?;
        for f in files {
            let n: u64 = stmt.query_row([f], |r| r.get(0))?;
            total += n;
        }
        Ok(total)
    }

    fn bridges_from_membership(
        &self,
        membership: &HashMap<String, usize>,
        limit: usize,
    ) -> Result<Vec<Bridge>> {
        let mut stmt = self.conn.prepare(
            "SELECT e.kind, ns.file_path, nd.file_path
             FROM edges e
             JOIN nodes ns ON ns.id = e.src
             JOIN nodes nd ON nd.id = e.dst
             WHERE e.kind IN ('IMPORTS', 'CALLS', 'DESCRIBES')
               AND ns.file_path IS NOT NULL AND nd.file_path IS NOT NULL",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })?;
        let mut bridges = Vec::new();
        let mut seen = HashSet::new();
        for row in rows {
            let (kind, src, dst) = row?;
            if src == dst {
                continue;
            }
            let Some(&cs) = membership.get(&src) else {
                continue;
            };
            let Some(&cd) = membership.get(&dst) else {
                continue;
            };
            if cs == cd {
                continue;
            }
            let key = if src < dst {
                format!("{src}|{dst}|{kind}")
            } else {
                format!("{dst}|{src}|{kind}")
            };
            if !seen.insert(key) {
                continue;
            }
            let w = match kind.as_str() {
                "IMPORTS" | "CALLS" => 1.0,
                "DESCRIBES" => 0.5,
                _ => 0.25,
            };
            bridges.push(Bridge {
                src_file: src,
                dst_file: dst,
                edge_kind: kind,
                src_community: format!("comm:louvain:{cs}"),
                dst_community: format!("comm:louvain:{cd}"),
                weight: w,
            });
        }
        bridges.sort_by(|a, b| {
            b.weight
                .partial_cmp(&a.weight)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.src_file.cmp(&b.src_file))
                .then(a.dst_file.cmp(&b.dst_file))
        });
        bridges.truncate(limit);
        Ok(bridges)
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
            entry.0 += 1;
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
        communities.truncate(40);
        Ok(communities)
    }

    fn degree_hubs_filtered(&self, limit: usize) -> Result<Vec<Hub>> {
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
            if h.file_path.as_deref().map(is_noise_path).unwrap_or(false) {
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

/// Deterministic Louvain (single-level + aggregation) with nodes processed in sorted order.
fn louvain_cluster(nodes: &[String], adj: &FileAdj) -> HashMap<String, usize> {
    let mut membership: HashMap<String, usize> = nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.clone(), i))
        .collect();
    let m: f64 = adj
        .values()
        .map(|nbrs| nbrs.values().sum::<f64>())
        .sum::<f64>()
        / 2.0;
    if m <= 0.0 {
        return membership;
    }

    let mut improved = true;
    let mut pass = 0;
    while improved && pass < 20 {
        improved = false;
        pass += 1;
        let mut order = nodes.to_vec();
        order.sort();
        for node in &order {
            let cur = membership[node];
            let mut best = cur;
            let mut best_delta = 0.0;
            let mut neighbor_comms: HashMap<usize, f64> = HashMap::new();
            if let Some(nbrs) = adj.get(node) {
                for (nbr, w) in nbrs {
                    let c = membership[nbr];
                    *neighbor_comms.entry(c).or_insert(0.0) += *w;
                }
            }
            let k_i: f64 = adj.get(node).map(|n| n.values().sum()).unwrap_or(0.0);
            for (&comm, &k_i_in) in &neighbor_comms {
                if comm == cur {
                    continue;
                }
                let sigma_tot = community_strength(comm, &membership, adj);
                // Standard Louvain gain approximation (Newman).
                let delta = k_i_in - (sigma_tot * k_i) / (2.0 * m);
                if delta > best_delta + 1e-12 || ((delta - best_delta).abs() < 1e-12 && comm < best)
                {
                    best_delta = delta;
                    best = comm;
                }
            }
            if best != cur {
                membership.insert(node.clone(), best);
                improved = true;
            }
        }
    }

    // Relabel community ids to dense 0..k-1 in sorted-file order for stability.
    let mut remap: HashMap<usize, usize> = HashMap::new();
    let mut next = 0usize;
    let mut ordered = nodes.to_vec();
    ordered.sort();
    for n in &ordered {
        let old = membership[n];
        if let std::collections::hash_map::Entry::Vacant(e) = remap.entry(old) {
            e.insert(next);
            next += 1;
        }
    }
    for v in membership.values_mut() {
        *v = remap[v];
    }
    membership
}

fn community_strength(comm: usize, membership: &HashMap<String, usize>, adj: &FileAdj) -> f64 {
    let mut s = 0.0;
    for (n, &c) in membership {
        if c != comm {
            continue;
        }
        if let Some(nbrs) = adj.get(n) {
            s += nbrs.values().sum::<f64>();
        }
    }
    s
}

fn majority_path_prefix(files: &[String]) -> String {
    let mut counts: HashMap<String, u64> = HashMap::new();
    for f in files {
        let p = path_prefix(f);
        *counts.entry(p).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .max_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)))
        .map(|(p, _)| p)
        .unwrap_or_else(|| "./".into())
}

fn extractive_community_label(prefix: &str) -> String {
    let trimmed = prefix.trim_end_matches('/');
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
    use super::{is_noise_hub, louvain_cluster, path_prefix};
    use crate::kg::{KgStore, SqliteKgStore};
    use prism_ir::{
        edge_id, file_node_id, symbol_node_id, Confidence, EdgeKind, FactBundle, FactEdge,
        FactNode, NodeKind, Tier,
    };
    use std::collections::HashMap;
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
    fn louvain_is_deterministic() {
        let nodes: Vec<String> = vec!["a.rs".into(), "b.rs".into(), "c.rs".into(), "d.rs".into()];
        let mut adj: HashMap<String, HashMap<String, f64>> = HashMap::new();
        for n in &nodes {
            adj.insert(n.clone(), HashMap::new());
        }
        // Two cliques: a-b and c-d
        for (x, y) in [("a.rs", "b.rs"), ("c.rs", "d.rs")] {
            adj.get_mut(x).unwrap().insert(y.into(), 1.0);
            adj.get_mut(y).unwrap().insert(x.into(), 1.0);
        }
        let m1 = louvain_cluster(&nodes, &adj);
        let m2 = louvain_cluster(&nodes, &adj);
        assert_eq!(m1, m2);
        assert_eq!(m1["a.rs"], m1["b.rs"]);
        assert_eq!(m1["c.rs"], m1["d.rs"]);
        assert_ne!(m1["a.rs"], m1["c.rs"]);
    }

    #[test]
    fn repo_map_louvain_when_edges_exist() {
        let dir = tempdir().unwrap();
        let mut kg = SqliteKgStore::open(dir.path().join("g.sqlite")).unwrap();
        // Three files with IMPORTS between a-b and b-c
        for (path, name) in [("src/a.rs", "a"), ("src/b.rs", "b"), ("src/c.rs", "c")] {
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
        // Edges live on a.rs subgraph
        let mut b = FactBundle::new("src/a.rs", "rust", "test");
        b.nodes.push(FactNode {
            id: file_node_id("src/a.rs"),
            kind: NodeKind::File,
            name: Some("a".into()),
            file_path: Some("src/a.rs".into()),
            span: None,
            language: Some("rust".into()),
            analyzer: "test".into(),
            tier: Tier::T1,
            confidence: Confidence::Extracted,
            attrs: Default::default(),
        });
        b.edges.push(FactEdge {
            id: edge_id(
                EdgeKind::Imports,
                &file_node_id("src/a.rs"),
                &file_node_id("src/b.rs"),
                1,
            ),
            kind: EdgeKind::Imports,
            src: file_node_id("src/a.rs"),
            dst: file_node_id("src/b.rs"),
            file_path: Some("src/a.rs".into()),
            span: None,
            analyzer: "test".into(),
            tier: Tier::T1,
            confidence: Confidence::Extracted,
            attrs: Default::default(),
        });
        b.edges.push(FactEdge {
            id: edge_id(
                EdgeKind::Imports,
                &file_node_id("src/b.rs"),
                &file_node_id("src/c.rs"),
                2,
            ),
            kind: EdgeKind::Imports,
            src: file_node_id("src/b.rs"),
            dst: file_node_id("src/c.rs"),
            file_path: Some("src/a.rs".into()),
            span: None,
            analyzer: "test".into(),
            tier: Tier::T1,
            confidence: Confidence::Extracted,
            attrs: Default::default(),
        });
        kg.begin_replace_file_subgraph("src/a.rs").unwrap();
        kg.insert_facts("src/a.rs", &b).unwrap();
        kg.commit_replace_file_subgraph("src/a.rs").unwrap();

        let map = kg.repo_map(10).unwrap();
        assert_eq!(map.algorithm, "louvain_v1+resolved_degree_hubs");
        assert!(!map.communities.is_empty());
        let map2 = kg.repo_map(10).unwrap();
        assert_eq!(
            map.communities.iter().map(|c| &c.id).collect::<Vec<_>>(),
            map2.communities.iter().map(|c| &c.id).collect::<Vec<_>>()
        );
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
            map.hubs
                .iter()
                .all(|h| !h.node_id.starts_with("unresolved:")),
            "hubs={:?}",
            map.hubs
        );
        assert!(
            map.hubs
                .iter()
                .any(|h| h.node_id == real || h.name.as_deref() == Some("select_from_kg")),
            "expected real hub, got {:?}",
            map.hubs
        );
    }
}
