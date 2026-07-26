//! Host adapters for agent MCP / rules registration (P11 Stage B).

use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostKind {
    Cursor,
    Vscode,
    Claude,
    Generic,
}

impl HostKind {
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "cursor" => Ok(Self::Cursor),
            "vscode" | "code" | "vs-code" => Ok(Self::Vscode),
            "claude" | "claude-code" => Ok(Self::Claude),
            "generic" | "stdio" => Ok(Self::Generic),
            other => bail!("unknown host '{other}' (expected cursor|vscode|claude|generic)"),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cursor => "cursor",
            Self::Vscode => "vscode",
            Self::Claude => "claude",
            Self::Generic => "generic",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct HostStatus {
    pub host: String,
    pub registered: bool,
    pub paths: Vec<String>,
    pub detail: String,
}

#[derive(Debug, Serialize)]
pub struct HostActionReport {
    pub host: String,
    pub action: String,
    pub ok: bool,
    pub paths: Vec<String>,
    pub detail: String,
}

fn prism_bin() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(str::to_string))
        .unwrap_or_else(|| "prism".into())
}

fn mcp_server_entry(root: &Path, bin: &str) -> serde_json::Value {
    serde_json::json!({
        "command": bin,
        "args": ["mcp", root.display().to_string()],
    })
}

fn merge_mcp_json(path: &Path, root: &Path, bin: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut root_obj: serde_json::Value = if path.exists() {
        serde_json::from_str(&fs::read_to_string(path)?)
            .unwrap_or_else(|_| serde_json::json!({ "mcpServers": {} }))
    } else {
        serde_json::json!({ "mcpServers": {} })
    };
    if !root_obj
        .get("mcpServers")
        .map(|v| v.is_object())
        .unwrap_or(false)
    {
        root_obj["mcpServers"] = serde_json::json!({});
    }
    root_obj["mcpServers"]["prism"] = mcp_server_entry(root, bin);
    fs::write(
        path,
        format!("{}\n", serde_json::to_string_pretty(&root_obj)?),
    )?;
    Ok(())
}

fn remove_mcp_server(path: &Path) -> Result<bool> {
    if !path.exists() {
        return Ok(false);
    }
    let mut root_obj: serde_json::Value = serde_json::from_str(&fs::read_to_string(path)?)
        .unwrap_or_else(|_| serde_json::json!({ "mcpServers": {} }));
    let removed = root_obj
        .get_mut("mcpServers")
        .and_then(|s| s.as_object_mut())
        .map(|m| m.remove("prism").is_some())
        .unwrap_or(false);
    if removed {
        fs::write(
            path,
            format!("{}\n", serde_json::to_string_pretty(&root_obj)?),
        )?;
    }
    Ok(removed)
}

fn mcp_has_prism_at(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
        .and_then(|v| {
            v.get("mcpServers")
                .and_then(|s| s.get("prism"))
                .map(|_| true)
        })
        .unwrap_or(false)
}

const CLAUDE_SECTION_START: &str = "<!-- prism:host-adapter:start -->";
const CLAUDE_SECTION_END: &str = "<!-- prism:host-adapter:end -->";

fn claude_section(bin: &str, root: &Path) -> String {
    format!(
        r#"{start}
## Prism

Prefer Prism for repository questions. Ensure `prism` is on PATH (see `scripts/install.sh` / `scripts/install.ps1`). Then:

1. `prism setup .` (index + AGENTS.md + MCP) if not already ready
2. Call MCP `compile_context` before explore loops
3. Never ask for an API key for core indexing — local-first

MCP stdio (if the host needs a manual entry):

```json
{{
  "command": "{bin}",
  "args": ["mcp", "{root}"]
}}
```
{end}
"#,
        start = CLAUDE_SECTION_START,
        end = CLAUDE_SECTION_END,
        bin = bin,
        root = root.display()
    )
}

fn upsert_claude_md(path: &Path, bin: &str, root: &Path) -> Result<()> {
    let section = claude_section(bin, root);
    let existing = if path.exists() {
        fs::read_to_string(path)?
    } else {
        String::new()
    };
    let next = if let (Some(s), Some(e)) = (
        existing.find(CLAUDE_SECTION_START),
        existing.find(CLAUDE_SECTION_END),
    ) {
        let end = e + CLAUDE_SECTION_END.len();
        format!(
            "{}{}{}",
            &existing[..s],
            section.trim_end(),
            &existing[end..]
        )
    } else if existing.is_empty() {
        section
    } else {
        format!("{}\n\n{}", existing.trim_end(), section)
    };
    fs::write(path, next)?;
    Ok(())
}

fn remove_claude_section(path: &Path) -> Result<bool> {
    if !path.exists() {
        return Ok(false);
    }
    let existing = fs::read_to_string(path)?;
    let Some(s) = existing.find(CLAUDE_SECTION_START) else {
        return Ok(false);
    };
    let Some(e) = existing.find(CLAUDE_SECTION_END) else {
        return Ok(false);
    };
    let end = e + CLAUDE_SECTION_END.len();
    let mut next = format!("{}{}", &existing[..s], &existing[end..]);
    while next.contains("\n\n\n") {
        next = next.replace("\n\n\n", "\n\n");
    }
    fs::write(path, next.trim_start())?;
    Ok(true)
}

/// Register Prism for a host (idempotent merge).
pub fn host_install(root: &Path, host: HostKind) -> Result<HostActionReport> {
    let bin = prism_bin();
    let mut paths = Vec::new();
    let detail = match host {
        HostKind::Cursor => {
            let path = root.join(".cursor/mcp.json");
            merge_mcp_json(&path, root, &bin)?;
            paths.push(path.display().to_string());
            format!("merged Prism MCP into {}", paths[0])
        }
        HostKind::Vscode => {
            let path = root.join(".vscode/mcp.json");
            merge_mcp_json(&path, root, &bin)?;
            paths.push(path.display().to_string());
            format!("merged Prism MCP into {}", paths[0])
        }
        HostKind::Claude => {
            let md = root.join("CLAUDE.md");
            upsert_claude_md(&md, &bin, root)?;
            paths.push(md.display().to_string());
            // Also drop a portable MCP snippet beside Claude guidance.
            let mcp = root.join(".mcp.prism.json");
            fs::write(
                &mcp,
                format!(
                    "{}\n",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "mcpServers": { "prism": mcp_server_entry(root, &bin) }
                    }))?
                ),
            )?;
            paths.push(mcp.display().to_string());
            "wrote CLAUDE.md Prism section + .mcp.prism.json snippet".into()
        }
        HostKind::Generic => {
            let snippet = serde_json::json!({
                "mcpServers": { "prism": mcp_server_entry(root, &bin) }
            });
            let path = root.join(".mcp.prism.json");
            fs::write(
                &path,
                format!("{}\n", serde_json::to_string_pretty(&snippet)?),
            )?;
            paths.push(path.display().to_string());
            format!(
                "wrote portable stdio snippet to {} — paste into your host MCP settings",
                paths[0]
            )
        }
    };
    Ok(HostActionReport {
        host: host.as_str().into(),
        action: "install".into(),
        ok: true,
        paths,
        detail,
    })
}

/// Remove Prism host registration (does not delete unrelated MCP servers).
pub fn host_uninstall(root: &Path, host: HostKind) -> Result<HostActionReport> {
    let mut paths = Vec::new();
    let detail = match host {
        HostKind::Cursor => {
            let path = root.join(".cursor/mcp.json");
            let removed = remove_mcp_server(&path)?;
            paths.push(path.display().to_string());
            if removed {
                format!("removed prism from {}", paths[0])
            } else {
                format!("prism was not registered in {}", paths[0])
            }
        }
        HostKind::Vscode => {
            let path = root.join(".vscode/mcp.json");
            let removed = remove_mcp_server(&path)?;
            paths.push(path.display().to_string());
            if removed {
                format!("removed prism from {}", paths[0])
            } else {
                format!("prism was not registered in {}", paths[0])
            }
        }
        HostKind::Claude => {
            let md = root.join("CLAUDE.md");
            let removed_md = remove_claude_section(&md)?;
            if md.exists() {
                paths.push(md.display().to_string());
            }
            let mcp = root.join(".mcp.prism.json");
            let removed_mcp = if mcp.exists() {
                fs::remove_file(&mcp)?;
                paths.push(mcp.display().to_string());
                true
            } else {
                false
            };
            format!("claude section removed={removed_md}; snippet removed={removed_mcp}")
        }
        HostKind::Generic => {
            let path = root.join(".mcp.prism.json");
            if path.exists() {
                fs::remove_file(&path)?;
                paths.push(path.display().to_string());
                format!("removed {}", paths[0])
            } else {
                "no .mcp.prism.json present".into()
            }
        }
    };
    Ok(HostActionReport {
        host: host.as_str().into(),
        action: "uninstall".into(),
        ok: true,
        paths,
        detail,
    })
}

pub fn host_status(root: &Path, host: Option<HostKind>) -> Result<Vec<HostStatus>> {
    let kinds = match host {
        Some(h) => vec![h],
        None => vec![
            HostKind::Cursor,
            HostKind::Vscode,
            HostKind::Claude,
            HostKind::Generic,
        ],
    };
    let mut out = Vec::new();
    for h in kinds {
        out.push(match h {
            HostKind::Cursor => {
                let path = root.join(".cursor/mcp.json");
                HostStatus {
                    host: h.as_str().into(),
                    registered: mcp_has_prism_at(&path),
                    paths: vec![path.display().to_string()],
                    detail: if mcp_has_prism_at(&path) {
                        "prism MCP present".into()
                    } else {
                        "not registered".into()
                    },
                }
            }
            HostKind::Vscode => {
                let path = root.join(".vscode/mcp.json");
                HostStatus {
                    host: h.as_str().into(),
                    registered: mcp_has_prism_at(&path),
                    paths: vec![path.display().to_string()],
                    detail: if mcp_has_prism_at(&path) {
                        "prism MCP present".into()
                    } else {
                        "not registered".into()
                    },
                }
            }
            HostKind::Claude => {
                let md = root.join("CLAUDE.md");
                let has = md
                    .exists()
                    .then(|| fs::read_to_string(&md).ok())
                    .flatten()
                    .map(|s| s.contains(CLAUDE_SECTION_START))
                    .unwrap_or(false);
                HostStatus {
                    host: h.as_str().into(),
                    registered: has,
                    paths: vec![md.display().to_string()],
                    detail: if has {
                        "CLAUDE.md Prism section present".into()
                    } else {
                        "not registered".into()
                    },
                }
            }
            HostKind::Generic => {
                let path = root.join(".mcp.prism.json");
                HostStatus {
                    host: h.as_str().into(),
                    registered: path.exists(),
                    paths: vec![path.display().to_string()],
                    detail: if path.exists() {
                        "portable snippet present".into()
                    } else {
                        "not registered".into()
                    },
                }
            }
        });
    }
    Ok(out)
}

/// Prefer Cursor when `.cursor/` exists; else VS Code; used by `prism setup`.
pub fn register_default_mcp(root: &Path) -> Result<PathBuf> {
    let bin = prism_bin();
    let cursor_dir = root.join(".cursor");
    let target = if cursor_dir.exists() || !root.join(".vscode").exists() {
        cursor_dir.join("mcp.json")
    } else {
        root.join(".vscode/mcp.json")
    };
    merge_mcp_json(&target, root, &bin).with_context(|| format!("write {}", target.display()))?;
    Ok(target)
}

pub fn mcp_has_prism(root: &Path) -> bool {
    mcp_has_prism_at(&root.join(".cursor/mcp.json"))
        || mcp_has_prism_at(&root.join(".vscode/mcp.json"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn cursor_install_uninstall_roundtrip() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let rep = host_install(root, HostKind::Cursor).unwrap();
        assert!(rep.ok);
        assert!(mcp_has_prism(root));
        let st = host_status(root, Some(HostKind::Cursor)).unwrap();
        assert!(st[0].registered);
        host_uninstall(root, HostKind::Cursor).unwrap();
        assert!(!mcp_has_prism(root));
    }

    #[test]
    fn claude_section_idempotent() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        host_install(root, HostKind::Claude).unwrap();
        host_install(root, HostKind::Claude).unwrap();
        let md = fs::read_to_string(root.join("CLAUDE.md")).unwrap();
        assert_eq!(md.matches(CLAUDE_SECTION_START).count(), 1);
        host_uninstall(root, HostKind::Claude).unwrap();
        let md = fs::read_to_string(root.join("CLAUDE.md")).unwrap_or_default();
        assert!(!md.contains(CLAUDE_SECTION_START));
    }
}
