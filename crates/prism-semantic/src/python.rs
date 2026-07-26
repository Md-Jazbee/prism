//! Python T3 CFG/DFG via tree-sitter (best-effort).

use crate::artifact::{
    CallSite, CfgBlock, CfgEdge, DfgDef, DfgDep, DfgGraph, DfgUse, FunctionFlow,
    SemanticFileArtifact, ALGO_VERSION, SEMANTIC_SCHEMA_VERSION,
};
use tree_sitter::{Node, Parser};

pub fn analyze_file(
    path: &str,
    source: &str,
    content_hash: Option<String>,
) -> SemanticFileArtifact {
    let mut notes = Vec::new();
    let mut parser = Parser::new();
    if parser
        .set_language(&tree_sitter_python::LANGUAGE.into())
        .is_err()
    {
        notes.push("language_init_failed".into());
        return empty(path, content_hash, notes);
    }
    let Some(tree) = parser.parse(source, None) else {
        notes.push("parse_error".into());
        return empty(path, content_hash, notes);
    };
    let root = tree.root_node();
    if root.has_error() {
        notes.push("parse_error".into());
    }

    let mut functions = Vec::new();
    walk_functions(root, source.as_bytes(), &mut functions, &mut notes);

    SemanticFileArtifact {
        schema_version: SEMANTIC_SCHEMA_VERSION.into(),
        algo_version: ALGO_VERSION.into(),
        language: "python".into(),
        path: path.into(),
        content_hash,
        notes,
        functions,
    }
}

fn empty(path: &str, content_hash: Option<String>, notes: Vec<String>) -> SemanticFileArtifact {
    SemanticFileArtifact {
        schema_version: SEMANTIC_SCHEMA_VERSION.into(),
        algo_version: ALGO_VERSION.into(),
        language: "python".into(),
        path: path.into(),
        content_hash,
        notes,
        functions: vec![],
    }
}

fn walk_functions(node: Node, src: &[u8], out: &mut Vec<FunctionFlow>, notes: &mut Vec<String>) {
    if node.kind() == "function_definition" || node.kind() == "async_function_definition" {
        if let Some(f) = extract_function(node, src) {
            out.push(f);
        } else {
            notes.push("partial_cfg".into());
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_functions(child, src, out, notes);
    }
}

fn extract_function(node: Node, src: &[u8]) -> Option<FunctionFlow> {
    let name = node
        .child_by_field_name("name")
        .map(|n| n.utf8_text(src).unwrap_or("anon").to_string())
        .unwrap_or_else(|| "anon".into());
    let body = node.child_by_field_name("body")?;
    let start_line = node.start_position().row as u32;
    let end_line = node.end_position().row as u32;

    let lines: Vec<String> = body
        .utf8_text(src)
        .unwrap_or("")
        .lines()
        .map(|s| s.to_string())
        .collect();
    // Map body-relative lines to absolute (body starts at body.start_position)
    let body_start = body.start_position().row as u32;

    let (blocks, cfg_edges) = build_cfg(&name, body_start, end_line, &lines);
    let dfg = build_dfg(body, src, body_start);
    let calls = collect_calls(body, src);

    Some(FunctionFlow {
        name,
        start_line,
        end_line,
        blocks,
        cfg_edges,
        dfg,
        calls,
    })
}

fn collect_calls(body: Node, src: &[u8]) -> Vec<CallSite> {
    let mut out = Vec::new();
    walk_calls(body, src, &mut out);
    out
}

fn walk_calls(node: Node, src: &[u8], out: &mut Vec<CallSite>) {
    if node.kind() == "call" {
        if let Some(func) = node.child_by_field_name("function") {
            let callee = match func.kind() {
                "identifier" => func.utf8_text(src).ok().map(|s| s.to_string()),
                "attribute" => func
                    .child_by_field_name("attribute")
                    .and_then(|a| a.utf8_text(src).ok().map(|s| s.to_string())),
                _ => None,
            };
            if let Some(callee) = callee {
                if !is_keyword(&callee) {
                    out.push(CallSite {
                        callee,
                        line: node.start_position().row as u32,
                    });
                }
            }
        }
    }
    let mut c = node.walk();
    for child in node.children(&mut c) {
        walk_calls(child, src, out);
    }
}

fn build_cfg(
    fname: &str,
    body_start: u32,
    end_line: u32,
    body_lines: &[String],
) -> (Vec<CfgBlock>, Vec<CfgEdge>) {
    // Split into contiguous plain runs interrupted by control keywords.
    let mut blocks = Vec::new();
    let mut edges = Vec::new();
    let mut block_idx = 0u32;
    let mut run_start = body_start;
    let mut prev_id: Option<String> = None;

    let push_block = |blocks: &mut Vec<CfgBlock>,
                      edges: &mut Vec<CfgEdge>,
                      block_idx: &mut u32,
                      start: u32,
                      end: u32,
                      kind: &str,
                      prev: &mut Option<String>| {
        if end < start {
            return;
        }
        let id = format!("{fname}:b{block_idx}");
        *block_idx += 1;
        blocks.push(CfgBlock {
            id: id.clone(),
            start_line: start,
            end_line: end,
            kind: kind.into(),
        });
        if let Some(p) = prev.take() {
            edges.push(CfgEdge {
                src: p,
                dst: id.clone(),
                kind: "fallthrough".into(),
            });
        }
        *prev = Some(id);
    };

    for (i, line) in body_lines.iter().enumerate() {
        let abs = body_start + i as u32;
        let trimmed = line.trim_start();
        let is_ctrl = trimmed.starts_with("if ")
            || trimmed.starts_with("elif ")
            || trimmed.starts_with("else:")
            || trimmed.starts_with("else ")
            || trimmed.starts_with("while ")
            || trimmed.starts_with("for ")
            || trimmed.starts_with("try:")
            || trimmed.starts_with("except")
            || trimmed.starts_with("finally:")
            || trimmed.starts_with("return")
            || trimmed.starts_with("raise");
        if is_ctrl {
            if abs > run_start {
                push_block(
                    &mut blocks,
                    &mut edges,
                    &mut block_idx,
                    run_start,
                    abs.saturating_sub(1),
                    "plain",
                    &mut prev_id,
                );
            }
            let kind = if trimmed.starts_with("if ") || trimmed.starts_with("elif ") {
                "branch"
            } else if trimmed.starts_with("while ") || trimmed.starts_with("for ") {
                "loop"
            } else if trimmed.starts_with("return") || trimmed.starts_with("raise") {
                "exit"
            } else {
                "branch"
            };
            push_block(
                &mut blocks,
                &mut edges,
                &mut block_idx,
                abs,
                abs,
                kind,
                &mut prev_id,
            );
            run_start = abs + 1;
        }
    }
    if run_start <= end_line {
        push_block(
            &mut blocks,
            &mut edges,
            &mut block_idx,
            run_start,
            end_line,
            "plain",
            &mut prev_id,
        );
    }
    if blocks.is_empty() {
        blocks.push(CfgBlock {
            id: format!("{fname}:b0"),
            start_line: body_start,
            end_line,
            kind: "entry".into(),
        });
    } else {
        blocks[0].kind = "entry".into();
    }

    // Loop-back edges: each loop block edges to next and notes loop_back to itself successor
    for b in &blocks {
        if b.kind == "loop" {
            if let Some(next) = blocks.iter().find(|x| x.start_line > b.start_line) {
                edges.push(CfgEdge {
                    src: next.id.clone(),
                    dst: b.id.clone(),
                    kind: "loop_back".into(),
                });
            }
        }
    }

    (blocks, edges)
}

fn build_dfg(body: Node, src: &[u8], _body_start: u32) -> DfgGraph {
    let mut defs = Vec::new();
    let mut uses = Vec::new();
    collect_defs_uses(body, src, &mut defs, &mut uses);

    // Reaching defs: for each use, last def of same name with def_line <= use_line
    let mut deps = Vec::new();
    for u in &uses {
        let mut best: Option<&DfgDef> = None;
        for d in &defs {
            if d.name == u.name
                && d.line <= u.line
                && best.map(|b| d.line >= b.line).unwrap_or(true)
            {
                best = Some(d);
            }
        }
        if let Some(d) = best {
            deps.push(DfgDep {
                name: u.name.clone(),
                def_line: d.line,
                use_line: u.line,
            });
        }
    }

    DfgGraph { defs, uses, deps }
}

fn collect_defs_uses(node: Node, src: &[u8], defs: &mut Vec<DfgDef>, uses: &mut Vec<DfgUse>) {
    match node.kind() {
        "assignment" | "augmented_assignment" => {
            if let Some(left) = node.child_by_field_name("left") {
                collect_assignment_targets(left, src, defs);
            }
            if let Some(right) = node.child_by_field_name("right") {
                collect_identifier_uses(right, src, uses);
            }
        }
        "identifier" => {
            // Only count as use if not already handled as assignment target in parent —
            // simplistic: skip bare identifiers under assignment left by checking ancestors later.
            // Here: record all identifiers as uses; deps still work via line order.
            let name = node.utf8_text(src).unwrap_or("").to_string();
            if !name.is_empty() && !is_keyword(&name) {
                uses.push(DfgUse {
                    name,
                    line: node.start_position().row as u32,
                });
            }
        }
        "parameters" => {
            let mut c = node.walk();
            for child in node.children(&mut c) {
                if child.kind() == "identifier" {
                    if let Ok(name) = child.utf8_text(src) {
                        defs.push(DfgDef {
                            name: name.into(),
                            line: child.start_position().row as u32,
                        });
                    }
                }
            }
        }
        _ => {
            let mut c = node.walk();
            for child in node.children(&mut c) {
                collect_defs_uses(child, src, defs, uses);
            }
            return;
        }
    }
    let mut c = node.walk();
    for child in node.children(&mut c) {
        if node.kind() == "assignment" || node.kind() == "augmented_assignment" {
            // already handled fields
            continue;
        }
        if node.kind() == "identifier" || node.kind() == "parameters" {
            continue;
        }
        collect_defs_uses(child, src, defs, uses);
    }
}

fn collect_assignment_targets(node: Node, src: &[u8], defs: &mut Vec<DfgDef>) {
    match node.kind() {
        "identifier" => {
            if let Ok(name) = node.utf8_text(src) {
                defs.push(DfgDef {
                    name: name.into(),
                    line: node.start_position().row as u32,
                });
            }
        }
        "pattern_list" | "tuple_pattern" | "list_pattern" | "expression_list" | "tuple"
        | "list" => {
            let mut c = node.walk();
            for child in node.children(&mut c) {
                collect_assignment_targets(child, src, defs);
            }
        }
        _ => {
            let mut c = node.walk();
            for child in node.children(&mut c) {
                if child.kind() == "identifier" {
                    collect_assignment_targets(child, src, defs);
                }
            }
        }
    }
}

fn collect_identifier_uses(node: Node, src: &[u8], uses: &mut Vec<DfgUse>) {
    if node.kind() == "identifier" {
        if let Ok(name) = node.utf8_text(src) {
            if !is_keyword(name) {
                uses.push(DfgUse {
                    name: name.into(),
                    line: node.start_position().row as u32,
                });
            }
        }
        return;
    }
    let mut c = node.walk();
    for child in node.children(&mut c) {
        collect_identifier_uses(child, src, uses);
    }
}

fn is_keyword(name: &str) -> bool {
    matches!(
        name,
        "True" | "False" | "None" | "self" | "cls" | "and" | "or" | "not" | "in" | "is"
    )
}
