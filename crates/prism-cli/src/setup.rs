//! Graphify-like one-shot workspace setup (`prism setup`).

use anyhow::{bail, Context, Result};
use prism_core::{IncrementalIndexer, IndexOptions, WorkspaceManager};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
pub struct SetupReport {
    pub ok: bool,
    pub workspace: String,
    pub steps: Vec<SetupStep>,
    pub ready: ReadyChecklist,
}

#[derive(Debug, Serialize)]
pub struct SetupStep {
    pub id: String,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Serialize)]
pub struct ReadyChecklist {
    pub binary: bool,
    pub index: bool,
    pub agents_md: bool,
    pub cursor_rule: bool,
    pub mcp_registered: bool,
}

pub struct SetupOpts {
    pub register_mcp: bool,
    pub skip_index: bool,
    pub generate_assets: bool,
}

impl Default for SetupOpts {
    fn default() -> Self {
        Self {
            register_mcp: true,
            skip_index: false,
            generate_assets: true,
        }
    }
}

/// Ensure index + agent assets + optional MCP registration for a cold workspace.
pub fn run_setup(path: &Path, opts: SetupOpts) -> Result<SetupReport> {
    let wm = WorkspaceManager::open(path)
        .with_context(|| format!("open workspace {}", path.display()))?;
    let root = wm.root().to_path_buf();
    let mut steps = Vec::new();

    // 1. Binary — if we got here, `prism` is executable.
    steps.push(SetupStep {
        id: "binary".into(),
        ok: true,
        detail: format!("prism CLI available (workspace {})", root.display()),
    });

    // 2. Index
    let graph = root.join(".prism/graph.sqlite");
    let index_ok = if opts.skip_index {
        steps.push(SetupStep {
            id: "index".into(),
            ok: graph.exists(),
            detail: if graph.exists() {
                "index present (skip requested)".into()
            } else {
                "no index and --skip-index set".into()
            },
        });
        graph.exists()
    } else {
        let prism_dir = root.join(".prism");
        let mut indexer = IncrementalIndexer::open(wm, &prism_dir)?;
        let result = indexer.run(&IndexOptions { dry_run: false })?;
        steps.push(SetupStep {
            id: "index".into(),
            ok: true,
            detail: format!(
                "indexed nodes={} edges={} wall_ms={}",
                result.stats.nodes_written, result.stats.edges_written, result.stats.wall_time_ms
            ),
        });
        true
    };

    // 3. Agent assets from catalog (single source of truth)
    let mut agents_ok = root.join("AGENTS.md").exists();
    let mut rule_ok = root.join(".cursor/rules/prism-compile-first.mdc").exists();
    if opts.generate_assets {
        let agents = prism_agent::generate_agents_md(&root)?;
        let rule = prism_agent::generate_cursor_rule(&root)?;
        let skills = prism_agent::generate_skill_markdown(&root)?;
        agents_ok = true;
        rule_ok = true;
        steps.push(SetupStep {
            id: "assets".into(),
            ok: true,
            detail: format!(
                "wrote {}, {}, {}",
                agents.display(),
                rule.display(),
                skills.display()
            ),
        });
    } else {
        steps.push(SetupStep {
            id: "assets".into(),
            ok: agents_ok && rule_ok,
            detail: "asset generation skipped".into(),
        });
    }

    // 4. MCP registration (Cursor / VS Code portable)
    let mcp_ok = if opts.register_mcp {
        let prism_bin = std::env::current_exe()
            .ok()
            .and_then(|p| p.to_str().map(str::to_string))
            .unwrap_or_else(|| "prism".into());
        let target = register_mcp(&root, &prism_bin)?;
        steps.push(SetupStep {
            id: "mcp".into(),
            ok: true,
            detail: format!("registered Prism MCP in {}", target.display()),
        });
        true
    } else {
        steps.push(SetupStep {
            id: "mcp".into(),
            ok: mcp_has_prism(&root),
            detail: "MCP registration skipped".into(),
        });
        mcp_has_prism(&root)
    };

    let ready = ReadyChecklist {
        binary: true,
        index: index_ok,
        agents_md: agents_ok,
        cursor_rule: rule_ok,
        mcp_registered: mcp_ok,
    };
    let ok = ready.binary && ready.index;
    Ok(SetupReport {
        ok,
        workspace: root.display().to_string(),
        steps,
        ready,
    })
}

pub fn doctor_ready(path: &Path) -> Result<ReadyChecklist> {
    let wm = WorkspaceManager::open(path)?;
    let root = wm.root();
    Ok(ReadyChecklist {
        binary: true,
        index: root.join(".prism/graph.sqlite").exists(),
        agents_md: root.join("AGENTS.md").exists(),
        cursor_rule: root.join(".cursor/rules/prism-compile-first.mdc").exists(),
        mcp_registered: mcp_has_prism(root),
    })
}

fn mcp_has_prism(root: &Path) -> bool {
    for rel in [".cursor/mcp.json", ".vscode/mcp.json"] {
        let p = root.join(rel);
        if !p.exists() {
            continue;
        }
        if let Ok(raw) = fs::read_to_string(&p) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                if v.get("mcpServers")
                    .and_then(|s| s.get("prism"))
                    .is_some()
                {
                    return true;
                }
            }
        }
    }
    false
}

fn register_mcp(root: &Path, prism_bin: &str) -> Result<PathBuf> {
    let cursor_dir = root.join(".cursor");
    let target = if cursor_dir.exists() || !root.join(".vscode").exists() {
        fs::create_dir_all(&cursor_dir)?;
        cursor_dir.join("mcp.json")
    } else {
        let vscode = root.join(".vscode");
        fs::create_dir_all(&vscode)?;
        vscode.join("mcp.json")
    };

    let mut root_obj: serde_json::Value = if target.exists() {
        serde_json::from_str(&fs::read_to_string(&target)?)
            .unwrap_or_else(|_| serde_json::json!({ "mcpServers": {} }))
    } else {
        serde_json::json!({ "mcpServers": {} })
    };
    if !root_obj.get("mcpServers").map(|v| v.is_object()).unwrap_or(false) {
        root_obj["mcpServers"] = serde_json::json!({});
    }
    root_obj["mcpServers"]["prism"] = serde_json::json!({
        "command": prism_bin,
        "args": ["mcp", root.display().to_string()],
    });
    fs::write(
        &target,
        format!("{}\n", serde_json::to_string_pretty(&root_obj)?),
    )?;
    Ok(target)
}

pub fn assert_ready(checklist: &ReadyChecklist) -> Result<()> {
    if checklist.binary && checklist.index {
        Ok(())
    } else {
        bail!(
            "workspace not ready — binary={} index={} (run `prism setup`)",
            checklist.binary,
            checklist.index
        )
    }
}
