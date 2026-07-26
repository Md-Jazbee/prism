//! Graphify-like one-shot workspace setup (`prism setup`).

use crate::host;
use crate::hook;
use anyhow::{bail, Context, Result};
use prism_core::{IncrementalIndexer, IndexOptions, WorkspaceManager};
use serde::Serialize;
use std::path::Path;

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

/// Doctor readiness checklist v2 (P11) — install + host + index + hook.
#[derive(Debug, Serialize)]
pub struct ReadyChecklist {
    pub binary: bool,
    pub binary_path: String,
    pub binary_version: String,
    pub index: bool,
    pub agents_md: bool,
    pub cursor_rule: bool,
    pub mcp_registered: bool,
    pub hook_installed: bool,
    pub hosts: Vec<host::HostStatus>,
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

    // 4. MCP registration (default host adapter)
    let mcp_ok = if opts.register_mcp {
        let target = host::register_default_mcp(&root)?;
        steps.push(SetupStep {
            id: "mcp".into(),
            ok: true,
            detail: format!("registered Prism MCP in {}", target.display()),
        });
        true
    } else {
        let present = host::mcp_has_prism(&root);
        steps.push(SetupStep {
            id: "mcp".into(),
            ok: present,
            detail: "MCP registration skipped".into(),
        });
        present
    };

    let ready = build_ready_checklist(&root, index_ok, agents_ok, rule_ok, mcp_ok);
    let ok = ready.binary && ready.index;
    Ok(SetupReport {
        ok,
        workspace: root.display().to_string(),
        steps,
        ready,
    })
}

fn binary_meta() -> (String, String) {
    let path = std::env::current_exe()
        .ok()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "prism".into());
    let version = env!("CARGO_PKG_VERSION").to_string();
    (path, version)
}

fn build_ready_checklist(
    root: &Path,
    index_ok: bool,
    agents_ok: bool,
    rule_ok: bool,
    mcp_ok: bool,
) -> ReadyChecklist {
    let (binary_path, binary_version) = binary_meta();
    let hosts = host::host_status(root, None).unwrap_or_default();
    let hook_installed = hook::hook_status(root)
        .map(|s| s.installed)
        .unwrap_or(false);
    ReadyChecklist {
        binary: true,
        binary_path,
        binary_version,
        index: index_ok,
        agents_md: agents_ok,
        cursor_rule: rule_ok,
        mcp_registered: mcp_ok,
        hook_installed,
        hosts,
    }
}

pub fn doctor_ready(path: &Path) -> Result<ReadyChecklist> {
    let wm = WorkspaceManager::open(path)?;
    let root = wm.root();
    Ok(build_ready_checklist(
        root,
        root.join(".prism/graph.sqlite").exists(),
        root.join("AGENTS.md").exists(),
        root.join(".cursor/rules/prism-compile-first.mdc").exists(),
        host::mcp_has_prism(root),
    ))
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
