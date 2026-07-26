//! Repository intelligence products (P5 Stage A).
//!
//! See `docs/architecture/REPO-INTELLIGENCE.md`.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;

use super::communities::RepoMap;
use super::SqliteKgStore;

pub const INTEL_ALGO_VERSION: &str = "intel-v0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entrypoint {
    pub node_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    pub reason: String,
    pub confidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayeringEdge {
    pub src_prefix: String,
    pub dst_prefix: String,
    pub edge_id: String,
    pub src: String,
    pub dst: String,
    pub kind: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotspot {
    pub path: String,
    pub score: u64,
    pub method: String,
    pub confidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractSurface {
    pub node_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    pub fan_in: u64,
    pub communities_touched: u64,
    pub confidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectChangesReport {
    pub algorithm: String,
    pub changed_paths: Vec<String>,
    pub dirty_files: Vec<String>,
    pub hotspots: Vec<Hotspot>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoIntelReport {
    pub algorithm: String,
    pub algo_version: String,
    pub repo_map: RepoMap,
    pub entrypoints: Vec<Entrypoint>,
    pub layering_violations: Vec<LayeringEdge>,
    pub hotspots: Vec<Hotspot>,
    pub contracts: Vec<ContractSurface>,
    pub require_t2_hint: bool,
    pub notes: Vec<String>,
}

impl SqliteKgStore {
    /// Full Stage A intelligence report (orientation + heuristics).
    pub fn repo_intel(
        &self,
        workspace: Option<&Path>,
        hub_limit: usize,
    ) -> Result<RepoIntelReport> {
        let repo_map = self.repo_map(hub_limit)?;
        let entrypoints = self.detect_entrypoints(40)?;
        let layering_violations = self.layering_hints(30)?;
        let hotspots = match workspace {
            Some(root) => match git_hotspots(root, 15) {
                Ok(h) if !h.is_empty() => h,
                _ => self.degree_file_hotspots(15)?,
            },
            None => self.degree_file_hotspots(15)?,
        };
        let contracts = self.contract_surfaces(15)?;
        let require_t2_hint = ambiguity_require_t2(self);

        let mut notes = vec![
            "LLM community naming not used; labels are path prefixes.".into(),
            format!("algo_version={INTEL_ALGO_VERSION}"),
            "Each product carries method/confidence in fields or notes.".into(),
        ];
        notes.extend(repo_map.notes.iter().cloned());
        if require_t2_hint {
            notes.push(
                "AmbiguityIndex.require_t2=true — prefer UpgradePrecision / PreciseIndex for accuracy claims."
                    .into(),
            );
        }

        Ok(RepoIntelReport {
            algorithm: "repo_intel_v0".into(),
            algo_version: INTEL_ALGO_VERSION.into(),
            repo_map,
            entrypoints,
            layering_violations,
            hotspots,
            contracts,
            require_t2_hint,
            notes,
        })
    }

    /// Heuristic entrypoints by symbol/file name patterns.
    pub fn detect_entrypoints(&self, limit: usize) -> Result<Vec<Entrypoint>> {
        let limit = limit.clamp(1, 100);
        let mut stmt = self.conn.prepare(
            "SELECT id, name, file_path, kind FROM nodes
             WHERE kind IN ('Symbol', 'File', 'Module')",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, String>(3)?,
            ))
        })?;

        let mut out = Vec::new();
        for row in rows {
            let (id, name, path, _kind) = row?;
            let n = name.as_deref().unwrap_or("");
            let p = path.as_deref().unwrap_or("");
            if let Some(reason) = entrypoint_reason(n, p) {
                out.push(Entrypoint {
                    node_id: id,
                    name,
                    file_path: path,
                    reason: reason.into(),
                    confidence: "heuristic".into(),
                });
            }
        }
        out.sort_by(|a, b| a.node_id.cmp(&b.node_id));
        out.truncate(limit);
        Ok(out)
    }

    /// IMPORTS that look like upward / cross-layer edges (best-effort).
    pub fn layering_hints(&self, limit: usize) -> Result<Vec<LayeringEdge>> {
        let mut stmt = self.conn.prepare(
            "SELECT e.id, e.src, e.dst, e.kind,
                    ns.file_path, nd.file_path
             FROM edges e
             JOIN nodes ns ON ns.id = e.src
             JOIN nodes nd ON nd.id = e.dst
             WHERE e.kind = 'IMPORTS'
             LIMIT 500",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, Option<String>>(4)?,
                r.get::<_, Option<String>>(5)?,
            ))
        })?;

        let mut out = Vec::new();
        for row in rows {
            let (eid, src, dst, kind, sp, dp) = row?;
            let Some(sp) = sp else { continue };
            let Some(dp) = dp else { continue };
            let src_prefix = path_layer(&sp);
            let dst_prefix = path_layer(&dp);
            if src_prefix == dst_prefix {
                continue;
            }
            let upward = (src_prefix.starts_with("tests") && !dst_prefix.starts_with("tests"))
                || (src_prefix.matches('/').count() > dst_prefix.matches('/').count()
                    && !dst_prefix.starts_with("tests"));
            if upward {
                out.push(LayeringEdge {
                    src_prefix,
                    dst_prefix,
                    edge_id: eid,
                    src,
                    dst,
                    kind,
                    note: "possible upward/cross-layer IMPORTS (heuristic)".into(),
                });
            }
        }
        out.truncate(limit);
        Ok(out)
    }

    /// Detect changes: dirty set ∪ hotspots for orientation.
    pub fn detect_changes(
        &self,
        workspace: Option<&Path>,
        changed_paths: &[String],
    ) -> Result<DetectChangesReport> {
        let dirty = if changed_paths.is_empty() {
            Vec::new()
        } else {
            self.dirty_set_for_paths(changed_paths)?
        };
        let hotspots = match workspace {
            Some(root) => match git_hotspots(root, 10) {
                Ok(h) if !h.is_empty() => h,
                _ => self.degree_file_hotspots(10)?,
            },
            None => self.degree_file_hotspots(10)?,
        };
        Ok(DetectChangesReport {
            algorithm: "dirty_set+hotspots_v0".into(),
            changed_paths: changed_paths.to_vec(),
            dirty_files: dirty,
            hotspots,
            notes: vec![
                "dirty_files = reverse-dep closure of changed_paths.".into(),
                "hotspots = git churn when available, else high-degree files.".into(),
                "confidence: observed (git) or heuristic (degree).".into(),
            ],
        })
    }

    fn degree_file_hotspots(&self, limit: usize) -> Result<Vec<Hotspot>> {
        let hubs = self.repo_map(limit)?.hubs;
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        for h in hubs {
            if let Some(p) = h.file_path {
                if seen.insert(p.clone()) {
                    out.push(Hotspot {
                        path: p,
                        score: h.degree,
                        method: "degree_fallback".into(),
                        confidence: "heuristic".into(),
                    });
                }
            }
            if out.len() >= limit {
                break;
            }
        }
        Ok(out)
    }

    fn contract_surfaces(&self, limit: usize) -> Result<Vec<ContractSurface>> {
        let mut stmt = self.conn.prepare(
            "SELECT n.id, n.name, n.file_path,
                    (SELECT COUNT(*) FROM edges e WHERE e.dst = n.id) AS fan_in
             FROM nodes n
             WHERE n.kind = 'Symbol'
             ORDER BY fan_in DESC, n.id
             LIMIT ?1",
        )?;
        let rows = stmt.query_map([limit as i64], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, u64>(3)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (id, name, path, fan_in) = row?;
            if fan_in == 0 {
                continue;
            }
            let communities_touched = self.count_caller_prefixes(&id).unwrap_or(1);
            out.push(ContractSurface {
                node_id: id,
                name,
                file_path: path,
                fan_in,
                communities_touched,
                confidence: "heuristic".into(),
            });
        }
        Ok(out)
    }

    fn count_caller_prefixes(&self, node_id: &str) -> Result<u64> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT ns.file_path FROM edges e
             JOIN nodes ns ON ns.id = e.src
             WHERE e.dst = ?1 AND ns.file_path IS NOT NULL",
        )?;
        let rows = stmt.query_map([node_id], |r| r.get::<_, String>(0))?;
        let mut prefixes = HashSet::new();
        for r in rows {
            prefixes.insert(path_layer(&r?));
        }
        Ok(prefixes.len() as u64)
    }
}

fn entrypoint_reason(name: &str, path: &str) -> Option<&'static str> {
    let nl = name.to_ascii_lowercase();
    let pl = path.to_ascii_lowercase();
    if nl == "main" || nl == "__main__" || nl.ends_with("::main") {
        return Some("name:main");
    }
    if pl.ends_with("/main.rs") || pl.ends_with("/main.py") || pl.ends_with("/__main__.py") {
        return Some("path:main_module");
    }
    if pl.contains("/cli") || pl.contains("/bin/") || nl.contains("cli") {
        return Some("path_or_name:cli");
    }
    if nl.contains("handler") || nl.contains("router") || nl.ends_with("_app") || nl == "app" {
        return Some("name:handler_or_app");
    }
    if pl.contains("/cmd/") || pl.contains("/commands/") {
        return Some("path:commands");
    }
    None
}

fn path_layer(path: &str) -> String {
    if let Some((dir, _)) = path.rsplit_once('/') {
        let parts: Vec<_> = dir.split('/').collect();
        if parts.len() >= 2 {
            format!("{}/{}", parts[0], parts[1])
        } else if parts.len() == 1 {
            parts[0].to_string()
        } else {
            ".".into()
        }
    } else {
        ".".into()
    }
}

fn git_hotspots(workspace: &Path, limit: usize) -> Result<Vec<Hotspot>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(workspace)
        .args(["log", "--pretty=format:", "--numstat", "-n", "200"])
        .output()?;
    if !output.status.success() {
        anyhow::bail!("git log failed");
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let mut scores: HashMap<String, u64> = HashMap::new();
    for line in text.lines() {
        let parts: Vec<_> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }
        let added: u64 = parts[0].parse().unwrap_or(0);
        let deleted: u64 = parts[1].parse().unwrap_or(0);
        let path = parts[2..].join(" ");
        if path.is_empty() {
            continue;
        }
        *scores.entry(path).or_default() += added + deleted;
    }
    let mut hotspots: Vec<_> = scores
        .into_iter()
        .map(|(path, score)| Hotspot {
            path,
            score,
            method: "git_numstat".into(),
            confidence: "observed".into(),
        })
        .collect();
    hotspots.sort_by(|a, b| b.score.cmp(&a.score).then(a.path.cmp(&b.path)));
    hotspots.truncate(limit);
    Ok(hotspots)
}

fn ambiguity_require_t2(kg: &SqliteKgStore) -> bool {
    let total: u64 = kg
        .conn
        .query_row("SELECT COUNT(*) FROM edges WHERE kind = 'CALLS'", [], |r| {
            r.get(0)
        })
        .unwrap_or(0);
    if total == 0 {
        return false;
    }
    let unresolved: u64 = kg
        .conn
        .query_row(
            "SELECT COUNT(*) FROM edges WHERE kind = 'CALLS' AND dst LIKE 'unresolved:%'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let heuristic: u64 = kg
        .conn
        .query_row(
            "SELECT COUNT(*) FROM edges WHERE kind = 'CALLS' AND confidence = 'heuristic'
             AND dst NOT LIKE 'unresolved:%'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let unresolved_rate = unresolved as f64 / total as f64;
    let heuristic_rate = (heuristic + unresolved) as f64 / total as f64;
    unresolved_rate >= 0.30 || heuristic_rate >= 0.50
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kg::{KgStore, SqliteKgStore};
    use prism_ir::{
        edge_id, file_node_id, symbol_node_id, Confidence, EdgeKind, FactBundle, FactEdge,
        FactNode, NodeKind, Tier,
    };
    use tempfile::tempdir;

    fn node(path: &str, name: &str, kind: NodeKind) -> FactNode {
        let id = if kind == NodeKind::File {
            file_node_id(path)
        } else {
            symbol_node_id(path, "function", name, 0)
        };
        FactNode {
            id,
            kind,
            name: Some(name.into()),
            file_path: Some(path.into()),
            span: None,
            language: Some("rust".into()),
            analyzer: "test".into(),
            tier: Tier::T1,
            confidence: Confidence::Extracted,
            attrs: Default::default(),
        }
    }

    #[test]
    fn detects_main_entrypoint() {
        let dir = tempdir().unwrap();
        let mut kg = SqliteKgStore::open(dir.path().join("g.sqlite")).unwrap();
        let path = "src/main.rs";
        let mut b = FactBundle::new(path, "rust", "test");
        b.nodes.push(node(path, "main", NodeKind::Symbol));
        kg.begin_replace_file_subgraph(path).unwrap();
        kg.insert_facts(path, &b).unwrap();
        kg.commit_replace_file_subgraph(path).unwrap();
        let eps = kg.detect_entrypoints(10).unwrap();
        assert!(eps.iter().any(|e| e.reason.contains("main")));
    }

    #[test]
    fn intel_report_includes_method_notes() {
        let dir = tempdir().unwrap();
        let mut kg = SqliteKgStore::open(dir.path().join("g.sqlite")).unwrap();
        let path = "crates/foo/src/lib.rs";
        let mut b = FactBundle::new(path, "rust", "test");
        b.nodes.push(node(path, "lib", NodeKind::File));
        let helper = node(path, "helper", NodeKind::Symbol);
        b.nodes.push(helper.clone());
        kg.begin_replace_file_subgraph(path).unwrap();
        kg.insert_facts(path, &b).unwrap();
        kg.commit_replace_file_subgraph(path).unwrap();

        let main = node("crates/foo/src/main.rs", "main", NodeKind::Symbol);
        let mut b2 = FactBundle::new("crates/foo/src/main.rs", "rust", "test");
        b2.nodes.push(main.clone());
        b2.edges.push(FactEdge {
            id: edge_id(EdgeKind::Calls, &main.id, &helper.id, 1),
            kind: EdgeKind::Calls,
            src: main.id.clone(),
            dst: helper.id.clone(),
            file_path: Some("crates/foo/src/main.rs".into()),
            span: None,
            analyzer: "test".into(),
            tier: Tier::T1,
            confidence: Confidence::Heuristic,
            attrs: Default::default(),
        });
        kg.begin_replace_file_subgraph("crates/foo/src/main.rs")
            .unwrap();
        kg.insert_facts("crates/foo/src/main.rs", &b2).unwrap();
        kg.commit_replace_file_subgraph("crates/foo/src/main.rs")
            .unwrap();

        let report = kg.repo_intel(None, 10).unwrap();
        assert_eq!(report.algo_version, INTEL_ALGO_VERSION);
        assert!(!report.notes.is_empty());
        assert!(report
            .entrypoints
            .iter()
            .any(|e| e.name.as_deref() == Some("main")));
    }

    #[test]
    fn detect_changes_dirty_union() {
        let dir = tempdir().unwrap();
        let mut kg = SqliteKgStore::open(dir.path().join("g.sqlite")).unwrap();
        for (path, name) in [("a.py", "a"), ("b.py", "b")] {
            let mut b = FactBundle::new(path, "python", "test");
            b.nodes.push(node(path, name, NodeKind::File));
            kg.begin_replace_file_subgraph(path).unwrap();
            kg.insert_facts(path, &b).unwrap();
            kg.commit_replace_file_subgraph(path).unwrap();
        }
        let a = file_node_id("a.py");
        let b = file_node_id("b.py");
        let mut edge_bundle = FactBundle::new("b.py", "python", "test");
        edge_bundle.nodes.push(node("b.py", "b", NodeKind::File));
        edge_bundle.edges.push(FactEdge {
            id: edge_id(EdgeKind::Imports, &b, &a, 0),
            kind: EdgeKind::Imports,
            src: b,
            dst: a,
            file_path: Some("b.py".into()),
            span: None,
            analyzer: "test".into(),
            tier: Tier::T1,
            confidence: Confidence::Extracted,
            attrs: Default::default(),
        });
        kg.begin_replace_file_subgraph("b.py").unwrap();
        kg.insert_facts("b.py", &edge_bundle).unwrap();
        kg.commit_replace_file_subgraph("b.py").unwrap();

        let rep = kg.detect_changes(None, &["a.py".into()]).unwrap();
        assert!(rep.dirty_files.contains(&"a.py".into()));
        assert!(!rep.notes.is_empty());
    }
}
