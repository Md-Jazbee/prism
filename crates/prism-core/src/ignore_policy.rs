//! Ignore + secret-sensitive path policy (Stage A / W-SEC).

use std::path::Path;

/// Default secrets that must never be indexed (solo-local P0).
const SECRET_BASENAMES: &[&str] = &[
    ".env",
    ".env.local",
    ".env.production",
    "credentials.json",
    "id_rsa",
    "id_ed25519",
];

const SECRET_SUFFIXES: &[&str] = &[".pem", ".p12", ".pfx", ".key"];

/// Vendor / noise directory name segments (heuristics).
const VENDOR_DIR_NAMES: &[&str] = &[
    "node_modules",
    "vendor",
    "target",
    ".git",
    ".prism",
    "__pycache__",
    ".venv",
    "venv",
    "dist",
    "build",
];

/// Whether a path looks secret-sensitive by basename / suffix.
pub fn is_secret_sensitive(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    if SECRET_BASENAMES.contains(&name) {
        return true;
    }
    SECRET_SUFFIXES.iter().any(|sfx| name.ends_with(sfx))
}

/// Workspace ignore + secret policy wrapper around `ignore` WalkBuilder later.
#[derive(Debug, Clone, Default)]
pub struct IgnorePolicy {
    /// Extra directory names to treat as vendor (additive).
    pub extra_vendor_dirs: Vec<String>,
}

impl IgnorePolicy {
    pub fn new() -> Self {
        Self::default()
    }

    /// True if any path component is a known vendor / generated dir.
    pub fn is_vendor_path(&self, path: &Path) -> bool {
        for comp in path.components() {
            if let Some(s) = comp.as_os_str().to_str() {
                if VENDOR_DIR_NAMES.contains(&s) {
                    return true;
                }
                if self.extra_vendor_dirs.iter().any(|v| v == s) {
                    return true;
                }
            }
        }
        false
    }

    /// File should be excluded from discover/hash.
    pub fn should_skip_file(&self, path: &Path) -> bool {
        is_secret_sensitive(path) || self.is_vendor_path(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn skips_env() {
        assert!(is_secret_sensitive(Path::new("src/.env")));
        assert!(is_secret_sensitive(Path::new("secrets/id_rsa")));
        assert!(!is_secret_sensitive(Path::new("src/main.rs")));
    }

    #[test]
    fn skips_node_modules() {
        let p = IgnorePolicy::new();
        assert!(p.should_skip_file(&PathBuf::from("app/node_modules/leftpad/index.js")));
        assert!(!p.should_skip_file(&PathBuf::from("app/src/main.rs")));
    }
}
