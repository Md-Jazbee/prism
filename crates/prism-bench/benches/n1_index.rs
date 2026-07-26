//! N1 benches: cold index wall time + incremental single-file edit.

use criterion::{criterion_group, criterion_main, Criterion};
use prism_bench::{index_workspace, touch_one_file, write_mini_workspace};
use tempfile::tempdir;

fn cold_index(c: &mut Criterion) {
    c.bench_function("n1_cold_index_mini", |b| {
        b.iter_with_setup(
            || {
                let dir = tempdir().expect("tempdir");
                write_mini_workspace(dir.path()).expect("fixture");
                dir
            },
            |dir| {
                index_workspace(dir.path()).expect("cold index");
            },
        );
    });
}

fn incremental_edit(c: &mut Criterion) {
    c.bench_function("n1_incremental_one_file", |b| {
        b.iter_with_setup(
            || {
                let dir = tempdir().expect("tempdir");
                write_mini_workspace(dir.path()).expect("fixture");
                index_workspace(dir.path()).expect("seed index");
                touch_one_file(dir.path()).expect("edit");
                dir
            },
            |dir| {
                index_workspace(dir.path()).expect("incremental index");
            },
        );
    });
}

criterion_group!(benches, cold_index, incremental_edit);
criterion_main!(benches);
