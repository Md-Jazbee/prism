//! Generate AGENTS.md / rules / skills from the workflow catalog (single source).

use crate::catalog::load_embedded_catalog;
use anyhow::Result;
use std::fs;
use std::path::Path;

const BANNER: &str = "<!-- prism:generated from schemas/agent-workflow/v1 — regenerate via `prism agent generate-assets` -->\n";

const ENSURE_INSTALL: &str = r#"## Ensure install (P11)

If `prism` is missing from PATH, install before exploring the repo. **Do not ask for an API key** for core indexing — local-first.

1. macOS/Linux: `curl -fsSL "https://raw.githubusercontent.com/${PRISM_GITHUB_REPO:-example/prism}/main/scripts/install.sh" | bash`
2. Windows (PowerShell): `irm "https://raw.githubusercontent.com/$env:PRISM_GITHUB_REPO/main/scripts/install.ps1" | iex` (set `PRISM_GITHUB_REPO` first)
3. Re-resolve the binary (`command -v prism`) and run `prism setup .`
4. Confirm with `prism doctor --ready`, then call MCP `compile_context`
5. Optional: `prism host install <cursor|vscode|claude|generic>` · `prism hook install` for post-commit reindex

Contributor fallback (this monorepo only): `cargo build -p prism-cli --release` then use `./target/release/prism`.

"#;

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
    md.push_str("## /prism-ensure-install\n\n");
    md.push_str("**Trigger:** `prism` missing on PATH, or workspace not doctor-ready\n\n");
    md.push_str("**Steps:**\n");
    md.push_str("1. Detect `command -v prism` (or Windows `where.exe prism`)\n");
    md.push_str("2. If missing, run platform installer (`scripts/install.sh` / `install.ps1`)\n");
    md.push_str("3. `prism setup .` (index + assets + default MCP)\n");
    md.push_str("4. `prism doctor --ready`\n");
    md.push_str("5. Prefer `compile_context` / named workflows next\n\n");
    md.push_str("**Expected pack:** none yet — bootstrap only\n\n");
    md.push_str("**Refusals:** INDEX_UNAVAILABLE (after install, run setup/index)\n\n");
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
3. Use micro-tools only for targeted follow-ups.\n\n",
    );
    md.push_str(ENSURE_INSTALL);
    md.push_str("## Workflows\n\n");
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
| INDEX_UNAVAILABLE | `prism setup .` or `prism index .` (ensure install first) |\n\
| PRECISION_REQUIRED | `prism precise import` or continue labeled heuristic |\n\
| VIEW_TOO_LARGE | Narrow seeds / anchors |\n\n\
## Anti-patterns\n\n\
- Do not open dozens of files via grep/read when compile_context can answer.\n\
- Do not claim rename safety from unlabeled impact.\n\
- Do not block on API keys for indexing or MCP setup.\n",
    );
    Ok(md)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn assets_include_ensure_install() {
        let dir = tempdir().unwrap();
        let agents = generate_agents_md(dir.path()).unwrap();
        let text = fs::read_to_string(agents).unwrap();
        assert!(text.contains("Ensure install"));
        assert!(text.contains("install.sh"));
        let skills = generate_skill_markdown(dir.path()).unwrap();
        let text = fs::read_to_string(skills).unwrap();
        assert!(text.contains("/prism-ensure-install"));
    }
}
