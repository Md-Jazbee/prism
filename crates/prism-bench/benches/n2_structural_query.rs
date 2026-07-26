//! N2 bench: structural query latency (resolve / neighbors / impact).

use criterion::{criterion_group, criterion_main, Criterion};
use prism_bench::{index_workspace, open_kg, write_mini_workspace};
use prism_store::EdgeDirection;
use tempfile::tempdir;

fn structural_queries(c: &mut Criterion) {
    let dir = tempdir().expect("tempdir");
    write_mini_workspace(dir.path()).expect("fixture");
    index_workspace(dir.path()).expect("index");
    let kg = open_kg(dir.path()).expect("open kg");

    let seeds = kg
        .resolve_symbol("entry", None, 5)
        .expect("resolve")
        .into_iter()
        .map(|n| n.id)
        .collect::<Vec<_>>();
    let seed = seeds
        .first()
        .cloned()
        .expect("fixture should yield an entry symbol");

    c.bench_function("n2_resolve_symbol", |b| {
        b.iter(|| {
            let hits = kg.resolve_symbol("helper", None, 20).expect("resolve");
            std::hint::black_box(hits);
        });
    });

    c.bench_function("n2_neighbors", |b| {
        b.iter(|| {
            let hits = kg
                .neighbors(&seed, None, EdgeDirection::Outgoing, 50)
                .expect("neighbors");
            std::hint::black_box(hits);
        });
    });

    c.bench_function("n2_impact_depth2", |b| {
        b.iter(|| {
            let hits = kg.impact(&seed, 2, 100).expect("impact");
            std::hint::black_box(hits);
        });
    });
}

criterion_group!(benches, structural_queries);
criterion_main!(benches);
