//! T1 Python extractor via tree-sitter.

use anyhow::{anyhow, Result};
use prism_ir::{
    edge_id, file_node_id, symbol_node_id, unresolved_node_id, Confidence, EdgeKind, FactBundle,
    FactEdge, FactNode, NodeKind, Span, Tier,
};
use tree_sitter::{Node, Parser, Tree};

pub const ANALYZER: &str = "tree-sitter-python@0.23";
pub const LANGUAGE: &str = "python";

/// Extract T1 facts from a Python source file.
pub fn extract(path: &str, bytes: &[u8]) -> Result<FactBundle> {
    let mut parser = Parser::new();
    let language = tree_sitter_python::LANGUAGE.into();
    parser
        .set_language(&language)
        .map_err(|e| anyhow!("set python language: {e}"))?;
    let tree = parser
        .parse(bytes, None)
        .ok_or_else(|| anyhow!("tree-sitter failed to parse {path}"))?;

    let mut bundle = FactBundle::new(path, LANGUAGE, ANALYZER);
    let file_id = file_node_id(path);
    bundle.nodes.push(FactNode {
        id: file_id.clone(),
        kind: NodeKind::File,
        name: Some(path.to_string()),
        file_path: Some(path.to_string()),
        span: Some(node_span(tree.root_node())),
        language: Some(LANGUAGE.into()),
        analyzer: ANALYZER.into(),
        tier: Tier::T1,
        confidence: Confidence::Extracted,
        attrs: Default::default(),
    });

    let mut symbols: Vec<(String, String)> = Vec::new(); // (name, id)
    walk_defs(
        tree.root_node(),
        bytes,
        path,
        &file_id,
        &mut bundle,
        &mut symbols,
    );
    walk_imports(tree.root_node(), bytes, path, &file_id, &mut bundle);
    walk_calls(tree.root_node(), bytes, path, &symbols, &mut bundle);
    walk_inheritance(tree.root_node(), bytes, path, &symbols, &mut bundle);

    // Ensure unresolved callee nodes exist for CALLS targets.
    ensure_unresolved_nodes(&mut bundle);
    bundle.normalize();
    Ok(bundle)
}

fn ensure_unresolved_nodes(bundle: &mut FactBundle) {
    let existing: std::collections::HashSet<_> =
        bundle.nodes.iter().map(|n| n.id.clone()).collect();
    let mut to_add = Vec::new();
    for e in &bundle.edges {
        if e.dst.starts_with("unresolved:") && !existing.contains(&e.dst) {
            let name = e.dst.trim_start_matches("unresolved:").to_string();
            to_add.push(FactNode {
                id: e.dst.clone(),
                kind: NodeKind::Symbol,
                name: Some(name),
                file_path: None,
                span: None,
                language: Some(LANGUAGE.into()),
                analyzer: ANALYZER.into(),
                tier: Tier::T1,
                confidence: Confidence::Heuristic,
                attrs: {
                    let mut m = serde_json::Map::new();
                    m.insert("unresolved".into(), serde_json::Value::Bool(true));
                    m
                },
            });
        }
    }
    // Dedup by id
    let mut seen = existing;
    for n in to_add {
        if seen.insert(n.id.clone()) {
            bundle.nodes.push(n);
        }
    }
}

fn walk_defs(
    node: Node,
    src: &[u8],
    path: &str,
    file_id: &str,
    bundle: &mut FactBundle,
    symbols: &mut Vec<(String, String)>,
) {
    let kind = node.kind();
    if kind == "function_definition" || kind == "class_definition" {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = text(name_node, src);
            let symbol_kind = if kind == "function_definition" {
                "function"
            } else {
                "class"
            };
            let id = symbol_node_id(path, symbol_kind, &name, name_node.start_byte() as u32);
            let mut attrs = serde_json::Map::new();
            attrs.insert(
                "symbol_kind".into(),
                serde_json::Value::String(symbol_kind.into()),
            );
            bundle.nodes.push(FactNode {
                id: id.clone(),
                kind: NodeKind::Symbol,
                name: Some(name.clone()),
                file_path: Some(path.to_string()),
                span: Some(node_span(node)),
                language: Some(LANGUAGE.into()),
                analyzer: ANALYZER.into(),
                tier: Tier::T1,
                confidence: Confidence::Extracted,
                attrs,
            });
            let start = name_node.start_byte() as u32;
            bundle.edges.push(FactEdge {
                id: edge_id(EdgeKind::Defines, file_id, &id, start),
                kind: EdgeKind::Defines,
                src: file_id.to_string(),
                dst: id.clone(),
                file_path: Some(path.to_string()),
                span: Some(node_span(name_node)),
                analyzer: ANALYZER.into(),
                tier: Tier::T1,
                confidence: Confidence::Extracted,
                attrs: Default::default(),
            });
            bundle.edges.push(FactEdge {
                id: edge_id(EdgeKind::Contains, file_id, &id, start),
                kind: EdgeKind::Contains,
                src: file_id.to_string(),
                dst: id.clone(),
                file_path: Some(path.to_string()),
                span: Some(node_span(node)),
                analyzer: ANALYZER.into(),
                tier: Tier::T1,
                confidence: Confidence::Extracted,
                attrs: Default::default(),
            });
            symbols.push((name, id));
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_defs(child, src, path, file_id, bundle, symbols);
    }
}

fn walk_imports(node: Node, src: &[u8], path: &str, file_id: &str, bundle: &mut FactBundle) {
    let kind = node.kind();
    if kind == "import_statement" || kind == "import_from_statement" {
        let module_name = import_module_name(node, src);
        let start = node.start_byte() as u32;
        let dst = format!("module:{module_name}");
        if !bundle.nodes.iter().any(|n| n.id == dst) {
            bundle.nodes.push(FactNode {
                id: dst.clone(),
                kind: NodeKind::Module,
                name: Some(module_name.clone()),
                file_path: None,
                span: None,
                language: Some(LANGUAGE.into()),
                analyzer: ANALYZER.into(),
                tier: Tier::T1,
                confidence: Confidence::Extracted,
                attrs: Default::default(),
            });
        }
        let mut attrs = serde_json::Map::new();
        attrs.insert(
            "raw".into(),
            serde_json::Value::String(text(node, src).trim().to_string()),
        );
        bundle.edges.push(FactEdge {
            id: edge_id(EdgeKind::Imports, file_id, &dst, start),
            kind: EdgeKind::Imports,
            src: file_id.to_string(),
            dst,
            file_path: Some(path.to_string()),
            span: Some(node_span(node)),
            analyzer: ANALYZER.into(),
            tier: Tier::T1,
            confidence: Confidence::Extracted,
            attrs,
        });
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_imports(child, src, path, file_id, bundle);
    }
}

fn import_module_name(node: Node, src: &[u8]) -> String {
    if node.kind() == "import_from_statement" {
        if let Some(mod_node) = node.child_by_field_name("module_name") {
            return text(mod_node, src);
        }
    }
    // import foo / import foo.bar — first dotted_name / aliased_import
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "dotted_name" || child.kind() == "aliased_import" {
            if child.kind() == "aliased_import" {
                if let Some(n) = child.child_by_field_name("name") {
                    return text(n, src);
                }
            }
            return text(child, src);
        }
    }
    text(node, src)
        .trim()
        .replace("import ", "")
        .replace(' ', "")
}

fn walk_calls(
    node: Node,
    src: &[u8],
    path: &str,
    symbols: &[(String, String)],
    bundle: &mut FactBundle,
) {
    if node.kind() == "call" {
        if let Some(func) = node.child_by_field_name("function") {
            let callee = callee_name(func, src);
            if !callee.is_empty() && callee != "self" {
                let start = func.start_byte() as u32;
                let dst = symbols
                    .iter()
                    .find(|(n, _)| n == &callee)
                    .map(|(_, id)| id.clone())
                    .unwrap_or_else(|| unresolved_node_id(&callee));
                // Prefer defining function as src when nested; else file.
                let src_id = enclosing_function_id(node, src, path, symbols)
                    .unwrap_or_else(|| file_node_id(path));
                bundle.edges.push(FactEdge {
                    id: edge_id(EdgeKind::Calls, &src_id, &dst, start),
                    kind: EdgeKind::Calls,
                    src: src_id,
                    dst,
                    file_path: Some(path.to_string()),
                    span: Some(node_span(func)),
                    analyzer: ANALYZER.into(),
                    tier: Tier::T1,
                    confidence: Confidence::Heuristic,
                    attrs: {
                        let mut m = serde_json::Map::new();
                        m.insert("callee".into(), serde_json::Value::String(callee));
                        m
                    },
                });
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_calls(child, src, path, symbols, bundle);
    }
}

fn enclosing_function_id(
    mut node: Node,
    src: &[u8],
    path: &str,
    symbols: &[(String, String)],
) -> Option<String> {
    while let Some(parent) = node.parent() {
        if parent.kind() == "function_definition" {
            if let Some(name_node) = parent.child_by_field_name("name") {
                let name = text(name_node, src);
                return symbols
                    .iter()
                    .find(|(n, _)| n == &name)
                    .map(|(_, id)| id.clone())
                    .or_else(|| {
                        Some(symbol_node_id(
                            path,
                            "function",
                            &name,
                            name_node.start_byte() as u32,
                        ))
                    });
            }
        }
        node = parent;
    }
    None
}

fn walk_inheritance(
    node: Node,
    src: &[u8],
    path: &str,
    symbols: &[(String, String)],
    bundle: &mut FactBundle,
) {
    if node.kind() == "class_definition" {
        if let Some(name_node) = node.child_by_field_name("name") {
            let class_name = text(name_node, src);
            let class_id = symbols
                .iter()
                .find(|(n, _)| n == &class_name)
                .map(|(_, id)| id.clone())
                .unwrap_or_else(|| {
                    symbol_node_id(path, "class", &class_name, name_node.start_byte() as u32)
                });
            if let Some(super_list) = node.child_by_field_name("superclasses") {
                let mut cursor = super_list.walk();
                for child in super_list.children(&mut cursor) {
                    if child.kind() == "identifier" || child.kind() == "attribute" {
                        let base = text(child, src);
                        let simple = base.rsplit('.').next().unwrap_or(&base);
                        let dst = symbols
                            .iter()
                            .find(|(n, _)| n == simple)
                            .map(|(_, id)| id.clone())
                            .unwrap_or_else(|| unresolved_node_id(simple));
                        let start = child.start_byte() as u32;
                        bundle.edges.push(FactEdge {
                            id: edge_id(EdgeKind::Extends, &class_id, &dst, start),
                            kind: EdgeKind::Extends,
                            src: class_id.clone(),
                            dst,
                            file_path: Some(path.to_string()),
                            span: Some(node_span(child)),
                            analyzer: ANALYZER.into(),
                            tier: Tier::T1,
                            confidence: Confidence::Heuristic,
                            attrs: Default::default(),
                        });
                    }
                }
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_inheritance(child, src, path, symbols, bundle);
    }
}

fn callee_name(func: Node, src: &[u8]) -> String {
    match func.kind() {
        "identifier" => text(func, src),
        "attribute" => {
            if let Some(attr) = func.child_by_field_name("attribute") {
                text(attr, src)
            } else {
                text(func, src)
            }
        }
        _ => text(func, src),
    }
}

fn text(node: Node, src: &[u8]) -> String {
    node.utf8_text(src).unwrap_or("").to_string()
}

fn node_span(node: Node) -> Span {
    let start = node.start_position();
    let end = node.end_position();
    Span {
        start_byte: node.start_byte() as u32,
        end_byte: node.end_byte() as u32,
        start_line: start.row as u32,
        start_col: start.column as u32,
        end_line: end.row as u32,
        end_col: end.column as u32,
    }
}

/// Parse-only smoke for tests.
pub fn parse_ok(bytes: &[u8]) -> Result<Tree> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_python::LANGUAGE.into())
        .map_err(|e| anyhow!("{e}"))?;
    parser
        .parse(bytes, None)
        .ok_or_else(|| anyhow!("parse failed"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn extracts_function_and_call() {
        let src = b"def helper():\n    pass\n\ndef main():\n    helper()\n    missing()\n";
        let bundle = extract("sample.py", src).unwrap();
        assert!(bundle
            .nodes
            .iter()
            .any(|n| n.name.as_deref() == Some("helper")));
        assert!(bundle
            .nodes
            .iter()
            .any(|n| n.name.as_deref() == Some("main")));
        let calls: Vec<_> = bundle
            .edges
            .iter()
            .filter(|e| e.kind == EdgeKind::Calls)
            .collect();
        assert!(calls.iter().any(|e| !e.dst.starts_with("unresolved:")));
        assert!(calls.iter().any(|e| e.dst == "unresolved:missing"));
    }

    #[test]
    fn golden_simple_module_conformance() {
        let root =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/languages/python");
        let src = std::fs::read(root.join("simple_module.py")).expect("fixture source");
        let expected_raw =
            std::fs::read_to_string(root.join("expected.json")).expect("expected.json");
        let expected: FactBundle = serde_json::from_str(&expected_raw).expect("parse expected");
        let mut actual = extract("simple_module.py", &src).unwrap();
        actual.normalize();
        assert_eq!(
            serde_json::to_value(&actual).unwrap(),
            serde_json::to_value(&expected).unwrap(),
            "python golden fixture mismatch"
        );
    }
}
