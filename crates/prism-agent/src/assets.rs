//! Generate AGENTS.md / rules / skills from the workflow catalog (single source).

use crate::catalog::load_embedded_catalog;
use anyhow::Result;
use std::fs;
use std::path::Path;

const BANNER: &str = "<!-- prism:generated from schemas/agent-workflow/v1 — regenerate via `prism agent generate-assets` -->\n";

pub fn generate_agents_md(workspace: &Path) -> Result<std::path::PathBuf> {
    let body = agents_markdown()?;
    let out = workspace.join("AGENTS.md");
    fs::write(&out, format!("{BANNER}{body}"))?;
    Ok(out)
}

pub fn generate_cursor_rule(workspace: &Path) -> Result<std::path::PathBuf> {
    let body = agents_markdown()?;
    let dir = workspace.join(".cursor/rules");
    fs::create_dir_all(&dir)?;
    let out = dir.join("prism-compile-first.mdc");
    fs::write(
        &out,
        format!(
            "---\ndescription: Prefer Prism compile_context / workflows before explore loops\nglobs:\nalwaysApply: true\n---\n\n{body}"
        ),
    )?;
    Ok(out)
}

pub fn generate_skill_markdown(workspace: &Path) -> Result<std::path::PathBuf> {
    let catalog = load_embedded_catalog()?;
    let mut md = String::from("# Prism agent skills (generated)\n\n");
    for w in &catalog.workflows {
        md.push_str(&format!("## /prism-{}\n\n", w.id.replace('_', "-")));
        md.push_str(&format!("**Trigger:** {}\n\n", w.trigger));
        md.push_str("**Steps:**\n");
        for s in &w.steps {
            md.push_str(&format!("1. `{}` (`{}`)\n", s.tool, s.id));
        }
        md.push_str(&format!(
            "\n**Expected pack:** {}\n\n**Refusals:** {}\n\n",
            w.expected_pack_shape,
            w.refusal_points.join(", ")
        ));
    }
    let dir = workspace.join(".prism/agent");
    fs::create_dir_all(&dir)?;
    let out = dir.join("skills.md");
    fs::write(&out, format!("{BANNER}{md}"))?;
    Ok(out)
}

fn agents_markdown() -> Result<String> {
    let catalog = load_embedded_catalog()?;
    let mut md = String::from(
        "# AGENTS.md — Prism guidance (generated)\n\n\
> Prefer `compile_context` (or a named workflow) before explore loops.\n\n\
## Primary path\n\n\
1. Call **compile_context** or `prism workflow run <id>`.\n\
2. Answer from Evidence Pack citations; inspect gaps / EXPLAIN drops.\n\
3. Use micro-tools only for targeted follow-ups.\n\n\
## Workflows\n\n",
    );
    for w in &catalog.workflows {
        md.push_str(&format!(
            "### `{}` — {}\n\n- Trigger: {}\n- Tools: {}\n- Pack: {}\n\n",
            w.id,
            w.title,
            w.trigger,
            w.steps
                .iter()
                .map(|s| s.tool.as_str())
                .collect::<Vec<_>>()
                .join(" → "),
            w.expected_pack_shape
        ));
    }
    md.push_str(
        "## Refusals → next action\n\n\
| Code | Do this |\n|---|---|\n\
| SCOPE_UNRESOLVED | Pick a symbol / path / stack frame |\n\
| BUDGET_EXCEEDED | Raise `remaining_context_tokens` / narrow anchors |\n\
| INDEX_UNAVAILABLE | `prism index .` |\n\
| PRECISION_REQUIRED | `prism precise import` or continue labeled heuristic |\n\
| VIEW_TOO_LARGE | Narrow seeds / anchors |\n\n\
## Anti-patterns\n\n\
- Do not open dozens of files via grep/read when compile_context can answer.\n\
- Do not claim rename safety from unlabeled impact.\n",
    );
    Ok(md)
}
