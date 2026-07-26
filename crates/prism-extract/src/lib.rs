//! LanguageExtractor ABI, detect, and dispatch (P1 Stage A).

use anyhow::Result;
use prism_ir::FactBundle;
use std::path::Path;

/// Native first-party extractor contract (see `schemas/plugins/LanguageExtractor.md`).
pub trait LanguageExtractor: Send + Sync {
    fn language(&self) -> &'static str;
    fn analyzer(&self) -> &'static str;
    fn extract(&self, path: &str, bytes: &[u8]) -> Result<FactBundle>;
}

pub struct PythonExtractor;
pub struct RustExtractor;
pub struct MarkdownExtractor;

impl LanguageExtractor for PythonExtractor {
    fn language(&self) -> &'static str {
        prism_extract_python::LANGUAGE
    }
    fn analyzer(&self) -> &'static str {
        prism_extract_python::ANALYZER
    }
    fn extract(&self, path: &str, bytes: &[u8]) -> Result<FactBundle> {
        prism_extract_python::extract(path, bytes)
    }
}

impl LanguageExtractor for RustExtractor {
    fn language(&self) -> &'static str {
        prism_extract_rust::LANGUAGE
    }
    fn analyzer(&self) -> &'static str {
        prism_extract_rust::ANALYZER
    }
    fn extract(&self, path: &str, bytes: &[u8]) -> Result<FactBundle> {
        prism_extract_rust::extract(path, bytes)
    }
}

impl LanguageExtractor for MarkdownExtractor {
    fn language(&self) -> &'static str {
        prism_extract_markdown::LANGUAGE
    }
    fn analyzer(&self) -> &'static str {
        prism_extract_markdown::ANALYZER
    }
    fn extract(&self, path: &str, bytes: &[u8]) -> Result<FactBundle> {
        prism_extract_markdown::extract(path, bytes)
    }
}

/// Detect language from path extension.
///
/// Markdown (P12 Stage A) is treated as a first-class documentation language so
/// repository intent becomes queryable instead of falling through to nothing.
pub fn detect_language(path: impl AsRef<Path>) -> Option<&'static str> {
    let ext = path.as_ref().extension()?.to_str()?;
    match ext.to_ascii_lowercase().as_str() {
        "py" | "pyi" => Some("python"),
        "rs" => Some("rust"),
        "md" | "markdown" | "mdown" | "mkd" | "mdx" => Some("markdown"),
        _ => None,
    }
}

/// Extract facts when a first-party extractor exists for the path; otherwise `Ok(None)`.
pub fn extract_file(path: &str, bytes: &[u8]) -> Result<Option<FactBundle>> {
    match detect_language(path) {
        Some("python") => PythonExtractor.extract(path, bytes).map(Some),
        Some("rust") => RustExtractor.extract(path, bytes).map(Some),
        Some("markdown") => MarkdownExtractor.extract(path, bytes).map(Some),
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_extensions() {
        assert_eq!(detect_language("a/b.py"), Some("python"));
        assert_eq!(detect_language("lib.rs"), Some("rust"));
        assert_eq!(detect_language("readme.md"), Some("markdown"));
        assert_eq!(detect_language("README.MD"), Some("markdown"));
        assert_eq!(detect_language("notes.txt"), None);
    }

    #[test]
    fn dispatch_python() {
        let b = extract_file("t.py", b"def f():\n    pass\n")
            .unwrap()
            .unwrap();
        assert_eq!(b.language, "python");
        assert!(!b.nodes.is_empty());
    }

    #[test]
    fn dispatch_markdown() {
        let b = extract_file("README.md", b"# Title\n\n## Setup\n")
            .unwrap()
            .unwrap();
        assert_eq!(b.language, "markdown");
        assert!(b.nodes.iter().any(|n| n.kind == prism_ir::NodeKind::Doc));
        assert!(b
            .nodes
            .iter()
            .any(|n| n.kind == prism_ir::NodeKind::Section));
    }
}
