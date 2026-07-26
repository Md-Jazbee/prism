//! First-party vs noise path classification (P12 Stage B / ACC-6).
//!
//! Vendored snapshot fixtures and generated trees must not pollute Evidence
//! Packs or hub rankings unless the question explicitly anchors there.

/// Classify a repo-relative path for evidence / orientation scoping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathClass {
    FirstParty,
    Fixture,
    Vendored,
    Generated,
}

impl PathClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FirstParty => "first_party",
            Self::Fixture => "fixture",
            Self::Vendored => "vendored",
            Self::Generated => "generated",
        }
    }

    /// Whether this class is excluded from packs/hubs by default.
    pub fn is_noise(self) -> bool {
        !matches!(self, Self::FirstParty)
    }
}

/// Classify `path` (repo-relative, `/`-separated).
pub fn classify_path(path: &str) -> PathClass {
    let p = path.replace('\\', "/");
    let lower = p.to_ascii_lowercase();

    if lower.starts_with("fixtures/repos/")
        || lower.contains("/fixtures/repos/")
        || lower.starts_with("fixtures/repos\\")
    {
        return PathClass::Vendored;
    }
    if lower.starts_with("fixtures/") || lower.contains("/fixtures/") {
        return PathClass::Fixture;
    }
    if lower.starts_with("graphify-out/")
        || lower.starts_with("target/")
        || lower.contains("/target/")
        || lower.starts_with(".prism/")
        || lower.contains("/node_modules/")
        || lower.starts_with("node_modules/")
    {
        return PathClass::Generated;
    }
    if lower.contains("/vendor/")
        || lower.starts_with("vendor/")
        || lower.contains("/third_party/")
        || lower.starts_with("third_party/")
    {
        return PathClass::Vendored;
    }
    PathClass::FirstParty
}

/// True when evidence from `path` should be excluded unless an anchor covers it.
pub fn is_noise_path(path: &str) -> bool {
    classify_path(path).is_noise()
}

/// True when any anchor string mentions this path (substring match).
pub fn anchor_covers_path(anchors: &[String], path: &str) -> bool {
    let path = path.replace('\\', "/");
    anchors.iter().any(|a| {
        let a = a.replace('\\', "/");
        a.contains(&path) || path.contains(a.trim_matches('`'))
    })
}

/// Whether a fragment path is allowed under the path-class policy.
pub fn path_allowed(path: Option<&str>, anchors: &[String]) -> bool {
    match path {
        None => true, // unresolved / module nodes without a path
        Some(p) if !is_noise_path(p) => true,
        Some(p) => anchor_covers_path(anchors, p),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_noise_and_first_party() {
        assert_eq!(
            classify_path("crates/prism-ir/src/lib.rs"),
            PathClass::FirstParty
        );
        assert_eq!(
            classify_path("fixtures/repos/snapshots/ripgrep/src/main.rs"),
            PathClass::Vendored
        );
        assert_eq!(
            classify_path("fixtures/languages/markdown/sample.md"),
            PathClass::Fixture
        );
        assert_eq!(classify_path("target/debug/prism"), PathClass::Generated);
        assert!(is_noise_path("fixtures/repos/snapshots/httpx/x.py"));
        assert!(!is_noise_path("README.md"));
    }

    #[test]
    fn anchors_can_allow_noise() {
        let anchors = vec!["fixtures/languages/markdown/sample.md".into()];
        assert!(path_allowed(
            Some("fixtures/languages/markdown/sample.md"),
            &anchors
        ));
        assert!(!path_allowed(
            Some("fixtures/languages/markdown/sample.md"),
            &[]
        ));
    }
}
