//! T3 semantic analysis — intra-procedural CFG/DFG + local slice (P4 Stage A).
//!
//! See `docs/architecture/T3-ANALYSIS.md`.

mod artifact;
mod crash;
mod python;
mod slice;
mod store;

pub use artifact::{
    CfgBlock, CfgEdge, DfgDep, DfgDef, DfgUse, FunctionFlow, SemanticFileArtifact, DfgGraph,
    ALGO_VERSION, SEMANTIC_SCHEMA_VERSION,
};
pub use crash::SemanticPartial;
pub use slice::{local_slice, SliceCriterion, SliceReport, SliceSpan};
pub use store::{
    build_file_artifact, load_file_artifact, read_manifest, save_file_artifact, semantic_dir,
    write_manifest, SemanticManifest,
};

use anyhow::Result;
use std::path::Path;

/// Analyze Python source bytes into a semantic file artifact (never panics).
pub fn analyze_python_file(path: &str, source: &str, content_hash: Option<String>) -> SemanticFileArtifact {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        python::analyze_file(path, source, content_hash.clone())
    })) {
        Ok(art) => art,
        Err(_) => SemanticFileArtifact {
            schema_version: SEMANTIC_SCHEMA_VERSION.into(),
            algo_version: ALGO_VERSION.into(),
            language: "python".into(),
            path: path.into(),
            content_hash,
            notes: vec!["panic_caught".into()],
            functions: vec![],
        },
    }
}

/// Build semantic artifacts for all `.py` files under workspace (discover via walk).
pub fn build_workspace_python(workspace: &Path) -> Result<SemanticManifest> {
    store::build_workspace_python(workspace)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn fixture_contains_criterion_and_idempotent() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let src_path = root.join("fixtures/slices/python/sample.py");
        let source = fs::read_to_string(&src_path).unwrap();
        let art = analyze_python_file("sample.py", &source, None);
        assert!(!art.functions.is_empty(), "expected functions: {:?}", art.notes);

        let crit = SliceCriterion::Line {
            path: "sample.py".into(),
            line: 15, // return y inside bug()
        };
        let a = local_slice(&art, &crit).unwrap();
        let b = local_slice(&art, &crit).unwrap();
        assert_eq!(a.spans, b.spans);
        assert!(
            a.spans.iter().any(|s| s.start_line <= 15 && s.end_line >= 15),
            "slice must contain criterion line: {:?}",
            a.spans
        );
        assert!(!a.cfg_summary.is_empty());
    }

    #[test]
    fn broken_source_does_not_panic() {
        let art = analyze_python_file("bad.py", "def broken(\n", None);
        assert!(art.functions.is_empty() || !art.notes.is_empty());
    }
}
