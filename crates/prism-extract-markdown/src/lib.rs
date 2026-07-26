//! T1 documentation extractor (P12 Stage A).
//!
//! Turns a markdown file into a queryable slice of the knowledge graph:
//!
//! * one [`NodeKind::Doc`] node per file,
//! * one [`NodeKind::Section`] node per ATX heading, with byte spans,
//! * `CONTAINS` edges building the heading hierarchy (doc → h1 → h2 …),
//! * `REFERENCES` edges for relative links and intra-doc anchors,
//! * `MENTIONS` edges for inline-code identifiers — doc→code *claims* carrying
//!   [`Confidence::Asserted`], bound to real symbols later (Stage B).
//!
//! Everything here is deterministic and dependency-free so doc goldens stay
//! reproducible across machines, and no LLM or network is involved (G8).

use anyhow::Result;
use prism_ir::{
    doc_node_id, edge_id, file_node_id, section_node_id, slugify, unresolved_node_id, Confidence,
    EdgeKind, FactBundle, FactEdge, FactNode, NodeKind, Span, Tier,
};
use std::collections::{HashMap, HashSet};

pub const ANALYZER: &str = "prism-markdown@0.1";
pub const LANGUAGE: &str = "markdown";

struct Heading {
    level: u8,
    title: String,
    slug: String,
    id: String,
    start_byte: usize,
    start_line: usize,
}

/// Extract documentation facts from a markdown file.
pub fn extract(path: &str, bytes: &[u8]) -> Result<FactBundle> {
    let text = String::from_utf8_lossy(bytes);
    let mut bundle = FactBundle::new(path, LANGUAGE, ANALYZER);

    // Line table with exact byte offsets (split_inclusive keeps newlines).
    let mut lines: Vec<(usize, usize, &str)> = Vec::new(); // (start_byte, line_no, content)
    let mut byte = 0usize;
    for (line_no, raw) in text.split_inclusive('\n').enumerate() {
        let content = raw.trim_end_matches(['\n', '\r']);
        lines.push((byte, line_no, content));
        byte += raw.len();
    }
    let total_len = text.len() as u32;

    // Pass 1: collect headings (skipping fenced code) with disambiguated slugs.
    let mut headings: Vec<Heading> = Vec::new();
    let mut slug_counts: HashMap<String, u32> = HashMap::new();
    let mut fenced = 0u32;
    let mut in_fence = false;
    let mut fence_marker = ' ';
    for &(start_byte, line_no, content) in &lines {
        if let Some(marker) = fence_delimiter(content) {
            if in_fence {
                if marker == fence_marker {
                    in_fence = false;
                }
            } else {
                in_fence = true;
                fence_marker = marker;
                fenced += 1;
            }
            continue;
        }
        if in_fence {
            continue;
        }
        if let Some((level, title)) = heading(content) {
            let base = slugify(title);
            let base = if base.is_empty() {
                "section".into()
            } else {
                base
            };
            let n = slug_counts.entry(base.clone()).or_insert(0);
            let slug = if *n == 0 {
                base.clone()
            } else {
                format!("{base}-{n}")
            };
            *n += 1;
            headings.push(Heading {
                level,
                title: title.to_string(),
                id: section_node_id(path, &slug),
                slug,
                start_byte,
                start_line: line_no,
            });
        }
    }

    // Doc node: title = first level-1 heading, else file stem.
    let doc_id = doc_node_id(path);
    let title = headings
        .iter()
        .find(|h| h.level == 1)
        .map(|h| h.title.clone())
        .unwrap_or_else(|| file_stem(path));
    let mut doc_attrs = serde_json::Map::new();
    doc_attrs.insert("title".into(), serde_json::Value::String(title.clone()));
    doc_attrs.insert(
        "section_count".into(),
        serde_json::Value::from(headings.len()),
    );
    doc_attrs.insert("fenced_code_blocks".into(), serde_json::Value::from(fenced));
    let last_line = lines.last().map(|l| l.1 as u32).unwrap_or(0);
    bundle.nodes.push(FactNode {
        id: doc_id.clone(),
        kind: NodeKind::Doc,
        name: Some(title),
        file_path: Some(path.to_string()),
        span: Some(Span {
            start_byte: 0,
            end_byte: total_len,
            start_line: 0,
            start_col: 0,
            end_line: last_line,
            end_col: 0,
        }),
        language: Some(LANGUAGE.into()),
        analyzer: ANALYZER.into(),
        tier: Tier::T1,
        confidence: Confidence::Extracted,
        attrs: doc_attrs,
    });

    // Section nodes + CONTAINS hierarchy.
    let mut stack: Vec<(u8, String)> = Vec::new();
    for (i, h) in headings.iter().enumerate() {
        let end_byte = headings
            .get(i + 1)
            .map(|next| next.start_byte as u32)
            .unwrap_or(total_len);
        let end_line = headings
            .get(i + 1)
            .map(|n| n.start_line as u32)
            .unwrap_or(last_line);
        let mut attrs = serde_json::Map::new();
        attrs.insert("level".into(), serde_json::Value::from(h.level));
        attrs.insert("slug".into(), serde_json::Value::String(h.slug.clone()));
        attrs.insert("title".into(), serde_json::Value::String(h.title.clone()));
        bundle.nodes.push(FactNode {
            id: h.id.clone(),
            kind: NodeKind::Section,
            name: Some(h.title.clone()),
            file_path: Some(path.to_string()),
            span: Some(Span {
                start_byte: h.start_byte as u32,
                end_byte,
                start_line: h.start_line as u32,
                start_col: 0,
                end_line,
                end_col: 0,
            }),
            language: Some(LANGUAGE.into()),
            analyzer: ANALYZER.into(),
            tier: Tier::T1,
            confidence: Confidence::Extracted,
            attrs,
        });

        while stack
            .last()
            .map(|(lvl, _)| *lvl >= h.level)
            .unwrap_or(false)
        {
            stack.pop();
        }
        let parent = stack
            .last()
            .map(|(_, id)| id.clone())
            .unwrap_or_else(|| doc_id.clone());
        bundle.edges.push(FactEdge {
            id: edge_id(EdgeKind::Contains, &parent, &h.id, h.start_byte as u32),
            kind: EdgeKind::Contains,
            src: parent,
            dst: h.id.clone(),
            file_path: Some(path.to_string()),
            span: None,
            analyzer: ANALYZER.into(),
            tier: Tier::T1,
            confidence: Confidence::Extracted,
            attrs: Default::default(),
        });
        stack.push((h.level, h.id.clone()));
    }

    // Pass 2: links (REFERENCES) and inline-code mentions (MENTIONS), per section.
    let dir = parent_dir(path);
    let mut in_fence = false;
    let mut fence_marker = ' ';
    let mut mention_seen: HashSet<(String, String)> = HashSet::new();
    for &(start_byte, _line_no, content) in &lines {
        if let Some(marker) = fence_delimiter(content) {
            if in_fence {
                if marker == fence_marker {
                    in_fence = false;
                }
            } else {
                in_fence = true;
                fence_marker = marker;
            }
            continue;
        }
        if in_fence || heading(content).is_some() {
            continue;
        }
        let src_section = current_section(&headings, start_byte)
            .unwrap_or(&doc_id)
            .clone();

        for link in find_links(content) {
            let abs = start_byte as u32 + link.offset as u32;
            if let Some((dst, mut attrs)) = resolve_link(&link.target, path, &dir) {
                attrs.insert("text".into(), serde_json::Value::String(link.text.clone()));
                attrs.insert("raw".into(), serde_json::Value::String(link.target.clone()));
                bundle.edges.push(FactEdge {
                    id: edge_id(EdgeKind::References, &src_section, &dst, abs),
                    kind: EdgeKind::References,
                    src: src_section.clone(),
                    dst,
                    file_path: Some(path.to_string()),
                    span: None,
                    analyzer: ANALYZER.into(),
                    tier: Tier::T1,
                    confidence: Confidence::Extracted,
                    attrs,
                });
            }
        }

        for code in find_inline_code(content) {
            if let Some(token) = mention_token(&code.value) {
                if mention_seen.insert((src_section.clone(), token.clone())) {
                    let dst = unresolved_node_id(&token);
                    let abs = start_byte as u32 + code.offset as u32;
                    let mut attrs = serde_json::Map::new();
                    attrs.insert("raw".into(), serde_json::Value::String(token.clone()));
                    bundle.edges.push(FactEdge {
                        id: edge_id(EdgeKind::Mentions, &src_section, &dst, abs),
                        kind: EdgeKind::Mentions,
                        src: src_section.clone(),
                        dst,
                        file_path: Some(path.to_string()),
                        span: None,
                        analyzer: ANALYZER.into(),
                        // A doc *claims* this identifier; code has not proven it here.
                        confidence: Confidence::Asserted,
                        tier: Tier::T1,
                        attrs,
                    });
                }
            }
        }
    }

    ensure_mention_nodes(&mut bundle);
    bundle.normalize();
    Ok(bundle)
}

/// Add first-class placeholder Symbol nodes for unresolved MENTIONS targets,
/// mirroring the call-graph unresolved policy so nothing is silently dangling.
fn ensure_mention_nodes(bundle: &mut FactBundle) {
    let existing: HashSet<String> = bundle.nodes.iter().map(|n| n.id.clone()).collect();
    let mut seen = existing.clone();
    let mut to_add = Vec::new();
    for e in &bundle.edges {
        if e.kind == EdgeKind::Mentions
            && e.dst.starts_with("unresolved:")
            && !existing.contains(&e.dst)
        {
            let name = e.dst.trim_start_matches("unresolved:").to_string();
            if seen.insert(e.dst.clone()) {
                let mut attrs = serde_json::Map::new();
                attrs.insert("unresolved".into(), serde_json::Value::Bool(true));
                attrs.insert("from_doc".into(), serde_json::Value::Bool(true));
                to_add.push(FactNode {
                    id: e.dst.clone(),
                    kind: NodeKind::Symbol,
                    name: Some(name),
                    file_path: None,
                    span: None,
                    language: None,
                    analyzer: ANALYZER.into(),
                    tier: Tier::T1,
                    confidence: Confidence::Heuristic,
                    attrs,
                });
            }
        }
    }
    bundle.nodes.extend(to_add);
}

/// Returns the fence character (`` ` `` or `~`) if the line opens/closes a code fence.
fn fence_delimiter(line: &str) -> Option<char> {
    let t = line.trim_start();
    ['`', '~']
        .into_iter()
        .find(|&marker| t.starts_with(&marker.to_string().repeat(3)))
}

/// ATX heading level + title, if the line is a heading (≤3 leading spaces).
fn heading(line: &str) -> Option<(u8, &str)> {
    let leading = line.len() - line.trim_start_matches(' ').len();
    if leading > 3 {
        return None;
    }
    let t = line.trim_start_matches(' ');
    let hashes = t.len() - t.trim_start_matches('#').len();
    if hashes == 0 || hashes > 6 {
        return None;
    }
    let rest = &t[hashes..];
    if !rest.starts_with(' ') && !rest.is_empty() {
        return None; // "#foo" is not a heading
    }
    let title = rest.trim().trim_end_matches('#').trim();
    Some((hashes as u8, title))
}

fn current_section(headings: &[Heading], byte: usize) -> Option<&String> {
    headings
        .iter()
        .rev()
        .find(|h| h.start_byte <= byte)
        .map(|h| &h.id)
}

struct Link {
    text: String,
    target: String,
    offset: usize,
}

/// Minimal inline-link scanner for `[text](target)` (ignores images `![...]`).
fn find_links(line: &str) -> Vec<Link> {
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'[' && (i == 0 || bytes[i - 1] != b'!') {
            if let Some(close) = find_byte(bytes, i + 1, b']') {
                if close + 1 < bytes.len() && bytes[close + 1] == b'(' {
                    if let Some(rparen) = find_byte(bytes, close + 2, b')') {
                        let text = line[i + 1..close].to_string();
                        let target = line[close + 2..rparen].trim().to_string();
                        out.push(Link {
                            text,
                            target,
                            offset: i,
                        });
                        i = rparen + 1;
                        continue;
                    }
                }
            }
        }
        i += 1;
    }
    out
}

struct InlineCode {
    value: String,
    offset: usize,
}

/// Scan single-backtick inline code spans on a line.
fn find_inline_code(line: &str) -> Vec<InlineCode> {
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'`' {
            if let Some(close) = find_byte(bytes, i + 1, b'`') {
                let value = line[i + 1..close].to_string();
                out.push(InlineCode { value, offset: i });
                i = close + 1;
                continue;
            }
        }
        i += 1;
    }
    out
}

fn find_byte(bytes: &[u8], from: usize, target: u8) -> Option<usize> {
    (from..bytes.len()).find(|&j| bytes[j] == target)
}

/// Decide whether an inline-code span looks like a code identifier worth binding.
/// Conservative: filters plain English words and shell commands, keeps things
/// like `compile_context`, `NodeKind`, `crates/prism-ir`, `foo()`.
fn mention_token(raw: &str) -> Option<String> {
    let token = raw.trim().trim_end_matches("()");
    if token.len() < 2 || token.len() > 100 {
        return None;
    }
    if !token
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | ':' | '.' | '/' | '-'))
    {
        return None;
    }
    if !token.chars().any(|c| c.is_ascii_alphabetic()) {
        return None;
    }
    let has_marker = token.contains('_')
        || token.contains("::")
        || token.contains('.')
        || token.contains('/')
        || token.chars().any(|c| c.is_ascii_digit())
        || is_mixed_case(token);
    if !has_marker {
        return None;
    }
    Some(token.to_string())
}

fn is_mixed_case(s: &str) -> bool {
    s.chars().any(|c| c.is_ascii_uppercase()) && s.chars().any(|c| c.is_ascii_lowercase())
}

/// Resolve a markdown link target to an edge destination + attrs, or `None` for
/// external links (http/mailto/…) that are not repository edges.
fn resolve_link(
    target: &str,
    doc_path: &str,
    dir: &str,
) -> Option<(String, serde_json::Map<String, serde_json::Value>)> {
    let target = target.split_whitespace().next().unwrap_or(target); // drop `"title"`
    if target.is_empty() {
        return None;
    }
    let lower = target.to_ascii_lowercase();
    if lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("mailto:")
        || lower.starts_with("ftp://")
        || lower.starts_with("//")
    {
        return None;
    }
    let mut attrs = serde_json::Map::new();
    if let Some(anchor) = target.strip_prefix('#') {
        attrs.insert("anchor".into(), serde_json::Value::Bool(true));
        return Some((section_node_id(doc_path, &slugify(anchor)), attrs));
    }
    // Relative path (optionally with #fragment) → file node.
    let path_part = target.split('#').next().unwrap_or(target);
    let path_part = path_part.split('?').next().unwrap_or(path_part);
    if path_part.is_empty() {
        return None;
    }
    let normalized = normalize_rel(dir, path_part);
    Some((file_node_id(&normalized), attrs))
}

fn parent_dir(path: &str) -> String {
    match path.rfind('/') {
        Some(i) => path[..i].to_string(),
        None => String::new(),
    }
}

fn file_stem(path: &str) -> String {
    let file = path.rsplit('/').next().unwrap_or(path);
    match file.rfind('.') {
        Some(i) if i > 0 => file[..i].to_string(),
        _ => file.to_string(),
    }
}

/// Join `base` dir + relative `target`, resolving `.` and `..` deterministically.
fn normalize_rel(base: &str, target: &str) -> String {
    let mut stack: Vec<&str> = if base.is_empty() {
        Vec::new()
    } else {
        base.split('/').filter(|s| !s.is_empty()).collect()
    };
    for comp in target.split('/') {
        match comp {
            "" | "." => {}
            ".." => {
                stack.pop();
            }
            other => stack.push(other),
        }
    }
    stack.join("/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn headings_and_hierarchy() {
        let md = b"# Title\n\nintro\n\n## Setup\n\ntext\n\n### Detail\n\n## Usage\n";
        let b = extract("docs/guide.md", md).unwrap();
        let doc = b.nodes.iter().find(|n| n.kind == NodeKind::Doc).unwrap();
        assert_eq!(doc.name.as_deref(), Some("Title"));
        let sections: Vec<_> = b
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Section)
            .collect();
        assert_eq!(sections.len(), 4);
        // Detail nests under Setup; Setup under Title (h1); Title under the doc.
        let contains: Vec<_> = b
            .edges
            .iter()
            .filter(|e| e.kind == EdgeKind::Contains)
            .collect();
        assert!(contains
            .iter()
            .any(|e| e.dst == section_node_id("docs/guide.md", "detail")
                && e.src == section_node_id("docs/guide.md", "setup")));
        assert!(contains
            .iter()
            .any(|e| e.dst == section_node_id("docs/guide.md", "setup")
                && e.src == section_node_id("docs/guide.md", "title")));
        assert!(contains
            .iter()
            .any(|e| e.dst == section_node_id("docs/guide.md", "title")
                && e.src == doc_node_id("docs/guide.md")));
    }

    #[test]
    fn headings_inside_fence_are_ignored() {
        let md = b"# Real\n\n```\n# not a heading\n```\n\n## Also Real\n";
        let b = extract("a.md", md).unwrap();
        // Nodes are id-sorted after normalize(), so assert membership, not order.
        let sections: Vec<String> = b
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Section)
            .map(|n| n.name.clone().unwrap())
            .collect();
        assert_eq!(sections.len(), 2);
        assert!(sections.iter().any(|s| s == "Real"));
        assert!(sections.iter().any(|s| s == "Also Real"));
    }

    #[test]
    fn relative_links_and_anchors_reference() {
        let md = b"# Doc\n\nSee [the ADD](../architecture/ADD.md) and [top](#doc).\n";
        let b = extract("docs/plan/p.md", md).unwrap();
        let refs: Vec<_> = b
            .edges
            .iter()
            .filter(|e| e.kind == EdgeKind::References)
            .collect();
        assert!(refs
            .iter()
            .any(|e| e.dst == file_node_id("docs/architecture/ADD.md")));
        assert!(refs
            .iter()
            .any(|e| e.dst == section_node_id("docs/plan/p.md", "doc")));
        // external links are not edges
        let md2 = b"# X\n\n[site](https://example.com)\n";
        let b2 = extract("x.md", md2).unwrap();
        assert!(!b2.edges.iter().any(|e| e.kind == EdgeKind::References));
    }

    #[test]
    fn inline_code_becomes_asserted_mentions() {
        let md =
            b"# API\n\nCall `compile_context` on `crates/prism-ir`; run `prism setup .` too.\n";
        let b = extract("r.md", md).unwrap();
        let mentions: Vec<_> = b
            .edges
            .iter()
            .filter(|e| e.kind == EdgeKind::Mentions)
            .collect();
        assert!(mentions
            .iter()
            .all(|e| e.confidence == Confidence::Asserted));
        assert!(mentions
            .iter()
            .any(|e| e.dst == unresolved_node_id("compile_context")));
        assert!(mentions
            .iter()
            .any(|e| e.dst == unresolved_node_id("crates/prism-ir")));
        // "prism setup ." has spaces → not a single identifier → skipped
        assert!(!mentions.iter().any(|e| e.dst.contains("prism setup")));
        // placeholder nodes exist for every mention
        for e in &mentions {
            assert!(b.nodes.iter().any(|n| n.id == e.dst));
        }
    }

    #[test]
    fn duplicate_headings_get_unique_slugs() {
        let md = b"# Notes\n\n## Setup\n\n## Setup\n";
        let b = extract("d.md", md).unwrap();
        assert!(b
            .nodes
            .iter()
            .any(|n| n.id == section_node_id("d.md", "setup")));
        assert!(b
            .nodes
            .iter()
            .any(|n| n.id == section_node_id("d.md", "setup-1")));
    }

    #[test]
    fn golden_sample_conformance() {
        let root =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/languages/markdown");
        let src = std::fs::read(root.join("sample.md")).expect("fixture source");
        let expected_raw =
            std::fs::read_to_string(root.join("expected.json")).expect("expected.json");
        let expected: FactBundle = serde_json::from_str(&expected_raw).expect("parse expected");
        let mut actual = extract("sample.md", &src).unwrap();
        actual.normalize();
        assert_eq!(
            serde_json::to_value(&actual).unwrap(),
            serde_json::to_value(&expected).unwrap(),
            "markdown golden fixture mismatch"
        );
    }
}
