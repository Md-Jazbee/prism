//! T3/T4 semantic analysis — CFG/DFG, local + inter-procedural slice (P4).
//!
//! See `docs/architecture/T3-ANALYSIS.md` · `SLICE-OPERATOR.md` · `T4-SHARDING.md`.

mod artifact;
mod crash;
mod interproc;
mod memo;
mod python;
mod shard;
mod slice;
mod store;

pub use artifact::{
    CallSite, CfgBlock, CfgEdge, DfgDep, DfgDef, DfgGraph, DfgUse, FunctionFlow,
    SemanticFileArtifact, ALGO_VERSION, INTERPROC_ALGO_VERSION, SEMANTIC_SCHEMA_VERSION,
};
pub use crash::SemanticPartial;
pub use interproc::{
    interproc_slice, InterprocSliceReport, InterprocSpan, ResidualItem, SliceDirection,
    SliceParams, SliceProvenance,
};
pub use memo::{memo_key, params_hash};
pub use shard::{
    ensure_shard, invalidate_shards_for, load_shard, CallGraphShard, OverlayEdge,
};
pub use slice::{local_slice, SliceCriterion, SliceReport, SliceSpan};
pub use store::{
    build_file_artifact, build_workspace_python, load_file_artifact, read_manifest,
    save_file_artifact, semantic_dir, write_manifest, SemanticManifest,
};

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

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

    #[test]
    fn interproc_chain_includes_callers_and_memoizes() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("pkg")).unwrap();
        fs::write(
            root.join("pkg/chain.py"),
            r#"
def leaf(x):
    return x + 1


def mid(x):
    return leaf(x)


def root(n):
    return mid(n)
"#,
        )
        .unwrap();

        build_workspace_python(root).unwrap();

        let params = SliceParams {
            direction: SliceDirection::Backward,
            max_depth: 2,
            max_functions: 16,
            max_spans: 40,
            residual_expand: true,
            path: "pkg/chain.py".into(),
            line: Some(2), // return in leaf
            symbol: None,
            snapshot_id: "test".into(),
        };
        let a = interproc_slice(root, &params).unwrap();
        let b = interproc_slice(root, &params).unwrap();
        assert!(
            a.spans.iter().any(|s| s.start_line <= 2 && s.end_line >= 2),
            "criterion covered: {:?}",
            a.spans
        );
        assert!(
            a.functions_visited.iter().any(|f| f.contains("mid"))
                || a.depth_reached >= 1,
            "expected interproc callers: visited={:?} depth={}",
            a.functions_visited,
            a.depth_reached
        );
        assert!(b.provenance.memo_hit, "second call should hit memo");
        assert_eq!(a.spans, b.spans);
        assert!(!a.provenance.params_hash.is_empty());
        assert!(!a.provenance.shard_id.is_empty());
    }

    #[test]
    fn dirty_path_invalidates_shards() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::write(root.join("only.py"), "def f():\n    return 1\n").unwrap();
        build_workspace_python(root).unwrap();
        let (shard, _) = ensure_shard(root, "only.py", "f", 1, 8).unwrap();
        assert!(root
            .join(".prism/semantic/shards")
            .join(format!("{}.json", shard.shard_id))
            .exists());
        let n = invalidate_shards_for(root, &["only.py".into()]).unwrap();
        assert_eq!(n, 1);
    }
}
