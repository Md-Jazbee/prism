//! Git hook installers (P11 Stage B) — Graphify-like post-commit reindex.

use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

const MARKER_START: &str = "# >>> prism hook (managed by `prism hook`)";
const MARKER_END: &str = "# <<< prism hook";

const HOOK_BODY: &str = r#"# >>> prism hook (managed by `prism hook`)
# Incremental re-index after commit. Failures are non-fatal.
if command -v prism >/dev/null 2>&1; then
  _PRISM_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || _PRISM_ROOT=""
  if [ -n "$_PRISM_ROOT" ]; then
    (cd "$_PRISM_ROOT" && prism index . >/dev/null 2>&1) || true
  fi
  unset _PRISM_ROOT
fi
# <<< prism hook
"#;

#[derive(Debug, Serialize)]
pub struct HookStatus {
    pub installed: bool,
    pub path: String,
    pub detail: String,
}

#[derive(Debug, Serialize)]
pub struct HookActionReport {
    pub action: String,
    pub ok: bool,
    pub path: String,
    pub detail: String,
}

fn hooks_dir(root: &Path) -> Result<PathBuf> {
    let git = root.join(".git");
    if !git.exists() {
        bail!("not a git repository (missing .git) — init or run from a clone");
    }
    // Worktrees: .git may be a file pointing elsewhere; fall back to git command.
    if git.is_dir() {
        return Ok(git.join("hooks"));
    }
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--git-path", "hooks"])
        .current_dir(root)
        .output()
        .context("git rev-parse --git-path hooks")?;
    if !out.status.success() {
        bail!("git rev-parse failed — is git available?");
    }
    let rel = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let path = PathBuf::from(&rel);
    if path.is_absolute() {
        Ok(path)
    } else {
        Ok(root.join(path))
    }
}

fn post_commit_path(root: &Path) -> Result<PathBuf> {
    Ok(hooks_dir(root)?.join("post-commit"))
}

fn has_prism_section(content: &str) -> bool {
    content.contains(MARKER_START) && content.contains(MARKER_END)
}

fn strip_prism_section(content: &str) -> String {
    let Some(start) = content.find(MARKER_START) else {
        return content.to_string();
    };
    let Some(end_rel) = content[start..].find(MARKER_END) else {
        return content.to_string();
    };
    let end = start + end_rel + MARKER_END.len();
    let mut next = String::new();
    next.push_str(content[..start].trim_end());
    let after = content[end..].trim_start_matches(['\r', '\n']);
    if !next.is_empty() && !after.is_empty() {
        next.push('\n');
    }
    next.push_str(after);
    if !next.is_empty() && !next.ends_with('\n') {
        next.push('\n');
    }
    next
}

pub fn hook_status(root: &Path) -> Result<HookStatus> {
    let path = post_commit_path(root)?;
    if !path.exists() {
        return Ok(HookStatus {
            installed: false,
            path: path.display().to_string(),
            detail: "no post-commit hook".into(),
        });
    }
    let content = fs::read_to_string(&path)?;
    let installed = has_prism_section(&content);
    Ok(HookStatus {
        installed,
        path: path.display().to_string(),
        detail: if installed {
            "prism post-commit section present".into()
        } else {
            "post-commit exists but has no prism section".into()
        },
    })
}

pub fn hook_install(root: &Path) -> Result<HookActionReport> {
    let path = post_commit_path(root)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let existing = if path.exists() {
        fs::read_to_string(&path)?
    } else {
        String::from("#!/bin/sh\n")
    };

    let next = if has_prism_section(&existing) {
        // Refresh body in place (idempotent).
        let stripped = strip_prism_section(&existing);
        if stripped.trim().is_empty() {
            format!("#!/bin/sh\n{HOOK_BODY}")
        } else {
            format!("{}\n\n{HOOK_BODY}", stripped.trim_end())
        }
    } else if existing.trim().is_empty() {
        format!("#!/bin/sh\n{HOOK_BODY}")
    } else {
        let mut base = existing;
        if !base.ends_with('\n') {
            base.push('\n');
        }
        base.push('\n');
        base.push_str(HOOK_BODY);
        base
    };

    fs::write(&path, &next)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms)?;
    }

    Ok(HookActionReport {
        action: "install".into(),
        ok: true,
        path: path.display().to_string(),
        detail: "appended/refreshed prism post-commit reindex section".into(),
    })
}

pub fn hook_uninstall(root: &Path) -> Result<HookActionReport> {
    let path = post_commit_path(root)?;
    if !path.exists() {
        return Ok(HookActionReport {
            action: "uninstall".into(),
            ok: true,
            path: path.display().to_string(),
            detail: "no post-commit hook to modify".into(),
        });
    }
    let existing = fs::read_to_string(&path)?;
    if !has_prism_section(&existing) {
        return Ok(HookActionReport {
            action: "uninstall".into(),
            ok: true,
            path: path.display().to_string(),
            detail: "prism section not present — left hook untouched".into(),
        });
    }
    let next = strip_prism_section(&existing);
    // If only shebang remains, remove the file; otherwise rewrite.
    let trimmed = next.trim();
    if trimmed.is_empty() || trimmed == "#!/bin/sh" || trimmed == "#!/usr/bin/env bash" {
        fs::remove_file(&path)?;
        Ok(HookActionReport {
            action: "uninstall".into(),
            ok: true,
            path: path.display().to_string(),
            detail: "removed post-commit hook (only prism content remained)".into(),
        })
    } else {
        fs::write(&path, next)?;
        Ok(HookActionReport {
            action: "uninstall".into(),
            ok: true,
            path: path.display().to_string(),
            detail: "removed prism section; left other hook contents intact".into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::tempdir;

    fn init_git(dir: &Path) {
        Command::new("git")
            .args(["init"])
            .current_dir(dir)
            .output()
            .unwrap();
    }

    #[test]
    fn install_uninstall_roundtrip() {
        let dir = tempdir().unwrap();
        init_git(dir.path());
        let rep = hook_install(dir.path()).unwrap();
        assert!(rep.ok);
        assert!(hook_status(dir.path()).unwrap().installed);
        // Idempotent refresh
        hook_install(dir.path()).unwrap();
        let content = fs::read_to_string(post_commit_path(dir.path()).unwrap()).unwrap();
        assert_eq!(content.matches(MARKER_START).count(), 1);
        hook_uninstall(dir.path()).unwrap();
        assert!(!hook_status(dir.path()).unwrap().installed);
    }

    #[test]
    fn appends_without_clobbering() {
        let dir = tempdir().unwrap();
        init_git(dir.path());
        let path = post_commit_path(dir.path()).unwrap();
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "#!/bin/sh\necho foreign\n").unwrap();
        hook_install(dir.path()).unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("echo foreign"));
        assert!(content.contains(MARKER_START));
        hook_uninstall(dir.path()).unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("echo foreign"));
        assert!(!content.contains(MARKER_START));
    }
}
