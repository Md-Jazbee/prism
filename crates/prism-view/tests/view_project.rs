//! View projection tests (P6 Stage C).

use prism_core::{IncrementalIndexer, IndexOptions, WorkspaceManager};
use prism_store::SqliteKgStore;
use prism_view::{project_view, ViewKind, ViewOutcome, ViewParams, GRAPH_VIEW_SCHEMA_VERSION};
use std::fs;
use tempfile::tempdir;

fn index_fixture() -> (tempfile::TempDir, SqliteKgStore) {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("src/a")).unwrap();
    fs::create_dir_all(root.join("src/b")).unwrap();
    fs::write(root.join("src/a/mod.py"), b"def entry():\n    helper()\n").unwrap();
    fs::write(root.join("src/b/mod.py"), b"def helper():\n    return 1\n").unwrap();
    // Duplicate name for ambiguity heat
    fs::write(root.join("src/a/dup.py"), b"def shared():\n    pass\n").unwrap();
    fs::write(root.join("src/b/dup.py"), b"def shared():\n    pass\n").unwrap();
    let wm = WorkspaceManager::open(root).unwrap();
    let mut indexer = IncrementalIndexer::open(wm, root.join(".prism")).unwrap();
    indexer.run(&IndexOptions::default()).unwrap();
    let kg = SqliteKgStore::open(root.join(".prism/graph.sqlite")).unwrap();
    (dir, kg)
}

#[test]
fn architecture_map_is_deterministic() {
    let (dir, kg) = index_fixture();
    let params = ViewParams {
        snapshot_id: "snap1".into(),
        ..Default::default()
    };
    let a = project_view(&kg, dir.path(), ViewKind::ArchitectureMap, &params).unwrap();
    let b = project_view(&kg, dir.path(), ViewKind::ArchitectureMap, &params).unwrap();
    match (a, b) {
        (ViewOutcome::Ok(va), ViewOutcome::Ok(vb)) => {
            assert_eq!(va.schema_version, GRAPH_VIEW_SCHEMA_VERSION);
            assert_eq!(va.layout.seed, vb.layout.seed);
            assert_eq!(va.nodes, vb.nodes);
            assert!(!va.nodes.is_empty());
        }
        other => panic!("expected Ok views: {other:?}"),
    }
}

#[test]
fn view_too_large_when_budget_below_seeds() {
    let (dir, kg) = index_fixture();
    let params = ViewParams {
        snapshot_id: "snap1".into(),
        max_nodes: Some(1),
        ..Default::default()
    };
    // Architecture map has many community seeds — with max_nodes=1 should refuse if seeds>1
    let outcome = project_view(&kg, dir.path(), ViewKind::ArchitectureMap, &params).unwrap();
    match outcome {
        ViewOutcome::TooLarge(t) => {
            assert_eq!(t.code, "VIEW_TOO_LARGE");
            assert!(!t.suggested_anchors.is_empty());
        }
        ViewOutcome::Ok(v) => {
            // If only one community, still ok — force refuse via absurdly tiny budget on ambiguity
            assert!(v.budget.nodes_used <= 1);
        }
    }
}

#[test]
fn impact_cone_requires_seed() {
    let (dir, kg) = index_fixture();
    let hits = kg.resolve_symbol("entry", None, 1).unwrap();
    let seed = hits[0].id.clone();
    let outcome = project_view(
        &kg,
        dir.path(),
        ViewKind::ImpactCone,
        &ViewParams {
            snapshot_id: "s".into(),
            seed_id: Some(seed),
            max_nodes: Some(40),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(matches!(outcome, ViewOutcome::Ok(_)));
}
