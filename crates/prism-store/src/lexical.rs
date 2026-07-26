//! Lexical seed search over the KG (P12 ACC-3).
//!
//! Structure-first fallback: score identifiers, paths, and doc headings with
//! cheap SQL `LIKE` / exact matches. Embeddings stay out of scope.

use crate::query::{row_to_node, GraphNodeView};
use crate::SqliteKgStore;
use anyhow::Result;
use rusqlite::params;
use serde::{Deserialize, Serialize};

/// Minimum score to treat an existing planner anchor as grounded.
pub const MIN_GROUND_SCORE: u32 = 70;
/// Minimum score for a ranked candidate to appear in a refusal repair list.
pub const MIN_CANDIDATE_SCORE: u32 = 40;

/// A scored anchor suggestion from the graph vocabulary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeedCandidate {
    /// Prefer this string as a planner/hint anchor (symbol name or path).
    pub anchor: String,
    pub node_id: String,
    pub score: u32,
    /// exact | path | name_prefix | name_substr | heading | id_substr
    pub match_kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
}

impl SqliteKgStore {
    /// Score how well `anchor` resolves in the KG (ACC-3).
    pub fn score_anchor(&self, anchor: &str) -> Result<Option<SeedCandidate>> {
        let a = anchor.trim();
        if a.is_empty() {
            return Ok(None);
        }
        // Exact symbol name.
        let exact = self.resolve_symbol(a, None, 3)?;
        if let Some(hit) = exact.into_iter().next() {
            return Ok(Some(candidate_from_node(
                &hit,
                100,
                "exact",
                prefer_name(&hit, a),
            )));
        }
        // Path-shaped: file node or path substring.
        if a.contains('/') || a.contains('.') {
            if let Some(hit) = self.find_by_path_exact(a)? {
                return Ok(Some(candidate_from_node(
                    &hit,
                    90,
                    "path",
                    hit.file_path.clone().unwrap_or_else(|| a.to_string()),
                )));
            }
            if let Some(hit) = self.find_by_path_substr(a, 1)?.into_iter().next() {
                return Ok(Some(candidate_from_node(
                    &hit,
                    80,
                    "path",
                    hit.file_path.clone().unwrap_or_else(|| a.to_string()),
                )));
            }
        }
        // Prefix / substring name (symbols + sections).
        if let Some(hit) = self.find_by_name_prefix(a, 1)?.into_iter().next() {
            return Ok(Some(candidate_from_node(
                &hit,
                75,
                "name_prefix",
                prefer_name(&hit, a),
            )));
        }
        if a.len() >= 3 {
            if let Some(hit) = self.find_by_name_substr(a, 1)?.into_iter().next() {
                return Ok(Some(candidate_from_node(
                    &hit,
                    50,
                    "name_substr",
                    prefer_name(&hit, a),
                )));
            }
        }
        Ok(None)
    }

    /// Lexical search over graph vocabulary for question/anchor terms.
    pub fn lexical_seed_search(&self, terms: &[String], limit: usize) -> Result<Vec<SeedCandidate>> {
        let limit = limit.clamp(1, 30);
        let mut scored: Vec<SeedCandidate> = Vec::new();
        for term in terms {
            let t = term.trim();
            if t.len() < 3 || is_stopword(t) {
                continue;
            }
            if let Some(c) = self.score_anchor(t)? {
                push_best(&mut scored, c);
            }
            for hit in self.find_by_name_substr(t, 8)? {
                let kind = if hit.kind == "Section" || hit.kind == "Doc" {
                    "heading"
                } else {
                    "name_substr"
                };
                let score = if hit.name.as_deref() == Some(t) {
                    100
                } else if hit
                    .name
                    .as_deref()
                    .map(|n| n.to_ascii_lowercase().starts_with(&t.to_ascii_lowercase()))
                    .unwrap_or(false)
                {
                    75
                } else if hit.kind == "Section" || hit.kind == "Doc" {
                    45
                } else {
                    50
                };
                push_best(
                    &mut scored,
                    candidate_from_node(&hit, score, kind, prefer_name(&hit, t)),
                );
            }
            for hit in self.find_by_path_substr(t, 5)? {
                push_best(
                    &mut scored,
                    candidate_from_node(
                        &hit,
                        80,
                        "path",
                        hit.file_path.clone().unwrap_or_else(|| t.to_string()),
                    ),
                );
            }
        }
        scored.retain(|c| c.score >= MIN_CANDIDATE_SCORE);
        scored.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| a.anchor.cmp(&b.anchor))
                .then_with(|| a.node_id.cmp(&b.node_id))
        });
        scored.dedup_by(|a, b| a.node_id == b.node_id);
        scored.truncate(limit);
        Ok(scored)
    }

    fn find_by_path_exact(&self, path: &str) -> Result<Option<GraphNodeView>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, kind, name, file_path, attrs_json FROM nodes
             WHERE (file_path = ?1 OR id = ?2) AND kind = 'File'
             ORDER BY id LIMIT 1",
        )?;
        let id = format!("file:{path}");
        match stmt.query_row(params![path, id], row_to_node) {
            Ok(n) => Ok(Some(n)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn find_by_path_substr(&self, sub: &str, limit: usize) -> Result<Vec<GraphNodeView>> {
        let like = format!("%{sub}%");
        let mut stmt = self.conn.prepare(
            "SELECT id, kind, name, file_path, attrs_json FROM nodes
             WHERE file_path LIKE ?1
               AND kind = 'File'
               AND (file_path NOT LIKE '%fixtures/repos/%')
             ORDER BY length(file_path), id
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![like, limit as i64], row_to_node)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    fn find_by_name_prefix(&self, prefix: &str, limit: usize) -> Result<Vec<GraphNodeView>> {
        let like = format!("{prefix}%");
        let mut stmt = self.conn.prepare(
            "SELECT id, kind, name, file_path, attrs_json FROM nodes
             WHERE name LIKE ?1 COLLATE NOCASE
               AND id NOT LIKE 'unresolved:%'
               AND (file_path IS NULL OR file_path NOT LIKE '%fixtures/repos/%')
             ORDER BY length(name), id
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![like, limit as i64], row_to_node)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    fn find_by_name_substr(&self, sub: &str, limit: usize) -> Result<Vec<GraphNodeView>> {
        let like = format!("%{sub}%");
        let mut stmt = self.conn.prepare(
            "SELECT id, kind, name, file_path, attrs_json FROM nodes
             WHERE name LIKE ?1 COLLATE NOCASE
               AND id NOT LIKE 'unresolved:%'
               AND (file_path IS NULL OR file_path NOT LIKE '%fixtures/repos/%')
             ORDER BY length(name), id
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![like, limit as i64], row_to_node)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }
}

fn candidate_from_node(
    hit: &GraphNodeView,
    score: u32,
    match_kind: &str,
    anchor: String,
) -> SeedCandidate {
    SeedCandidate {
        anchor,
        node_id: hit.id.clone(),
        score,
        match_kind: match_kind.into(),
        file_path: hit.file_path.clone(),
        kind: Some(hit.kind.clone()),
    }
}

fn prefer_name(hit: &GraphNodeView, fallback: &str) -> String {
    hit.name
        .clone()
        .filter(|n| !n.is_empty())
        .or_else(|| hit.file_path.clone())
        .unwrap_or_else(|| fallback.to_string())
}

fn push_best(out: &mut Vec<SeedCandidate>, c: SeedCandidate) {
    if let Some(existing) = out.iter_mut().find(|x| x.node_id == c.node_id) {
        if c.score > existing.score {
            *existing = c;
        }
        return;
    }
    out.push(c);
}

fn is_stopword(t: &str) -> bool {
    matches!(
        t.to_ascii_lowercase().as_str(),
        "the"
            | "and"
            | "for"
            | "with"
            | "from"
            | "that"
            | "this"
            | "what"
            | "how"
            | "where"
            | "when"
            | "does"
            | "about"
            | "code"
            | "tell"
            | "me"
            | "please"
            | "into"
            | "some"
            | "have"
            | "will"
            | "your"
            | "repo"
            | "file"
            | "function"
            | "class"
            | "struct"
    )
}

/// Tokenize free text into lexical seed terms (backticks, paths, CapWords, words).
pub fn tokenize_seed_terms(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find('`') {
        rest = &rest[start + 1..];
        if let Some(end) = rest.find('`') {
            let tok = rest[..end].trim();
            if tok.len() >= 2 {
                out.push(tok.to_string());
            }
            rest = &rest[end + 1..];
        } else {
            break;
        }
    }
    for raw in text
        .split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '/' || c == '.' || c == '-'))
    {
        let t = raw.trim();
        if t.len() >= 3 && !is_stopword(t) {
            out.push(t.to_string());
        }
    }
    out.sort();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::KgStore;
    use prism_ir::{
        section_node_id, symbol_node_id, Confidence, FactBundle, FactNode, NodeKind, Tier,
    };
    use tempfile::tempdir;

    #[test]
    fn exact_and_lexical_rank() {
        let dir = tempdir().unwrap();
        let mut kg = SqliteKgStore::open(dir.path().join("g.sqlite")).unwrap();
        let mut b = FactBundle::new("src/lib.rs", "rust", "test");
        b.nodes.push(FactNode {
            id: symbol_node_id("src/lib.rs", "struct", "PlanHints", 1),
            kind: NodeKind::Symbol,
            name: Some("PlanHints".into()),
            file_path: Some("src/lib.rs".into()),
            span: None,
            language: Some("rust".into()),
            analyzer: "test".into(),
            tier: Tier::T1,
            confidence: Confidence::Extracted,
            attrs: Default::default(),
        });
        b.nodes.push(FactNode {
            id: section_node_id("README.md", "install"),
            kind: NodeKind::Section,
            name: Some("Install".into()),
            file_path: Some("README.md".into()),
            span: None,
            language: Some("markdown".into()),
            analyzer: "test".into(),
            tier: Tier::T1,
            confidence: Confidence::Asserted,
            attrs: Default::default(),
        });
        kg.begin_replace_file_subgraph("src/lib.rs").unwrap();
        kg.insert_facts("src/lib.rs", &b).unwrap();
        kg.commit_replace_file_subgraph("src/lib.rs").unwrap();

        let hit = kg.score_anchor("PlanHints").unwrap().unwrap();
        assert_eq!(hit.score, 100);
        assert_eq!(hit.match_kind, "exact");

        let ranked = kg
            .lexical_seed_search(&["PlanHints".into(), "Install".into()], 10)
            .unwrap();
        assert!(ranked.iter().any(|c| c.anchor == "PlanHints"));
        assert!(ranked
            .iter()
            .any(|c| c.match_kind == "heading" || c.anchor == "Install"));
    }

    #[test]
    fn tokenize_pulls_backticks() {
        let t = tokenize_seed_terms("What does `WalkBuilder` do in ignore?");
        assert!(t.iter().any(|x| x == "WalkBuilder"));
    }
}
