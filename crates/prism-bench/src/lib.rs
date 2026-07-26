//! Shared helpers for N1/N2 criterion benches (P6 Stage A).

use anyhow::Result;
use prism_core::{IncrementalIndexer, IndexOptions, WorkspaceManager};
use prism_store::SqliteKgStore;
use std::fs;
use std::path::{Path, PathBuf};

const PY_FIXTURE: &str = r#"
def helper():
    return 1

def entry():
    helper()
    missing()
"#;

const RS_FIXTURE: &str = r#"
fn helper() -> i32 { 1 }
fn entry() { let _ = helper(); missing(); }
"#;

/// Tiny multi-file workspace suitable for cold/incremental index benches.
pub fn write_mini_workspace(root: &Path) -> Result<()> {
    fs::create_dir_all(root.join("src/py"))?;
    fs::create_dir_all(root.join("src/rs"))?;
    for i in 0..8 {
        fs::write(
            root.join(format!("src/py/mod_{i}.py")),
            format!("# fixture {i}\n{PY_FIXTURE}"),
        )?;
        fs::write(
            root.join(format!("src/rs/mod_{i}.rs")),
            format!("// fixture {i}\n{RS_FIXTURE}"),
        )?;
    }
    Ok(())
}

pub fn index_workspace(root: &Path) -> Result<()> {
    let wm = WorkspaceManager::open(root)?;
    let prism = root.join(".prism");
    let mut indexer = IncrementalIndexer::open(wm, &prism)?;
    indexer.run(&IndexOptions { dry_run: false })?;
    Ok(())
}

pub fn open_kg(root: &Path) -> Result<SqliteKgStore> {
    SqliteKgStore::open(root.join(".prism/graph.sqlite"))
}

pub fn touch_one_file(root: &Path) -> Result<PathBuf> {
    let path = root.join("src/py/mod_0.py");
    let mut body = fs::read_to_string(&path)?;
    body.push_str("\n# edit\n");
    fs::write(&path, body)?;
    Ok(path)
}
