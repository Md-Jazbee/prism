# Criterion benches (P6 Stage A)

Real benches live in **`crates/prism-bench`** so they share the workspace dependency graph with the indexer and KG store.

```bash
cargo bench -p prism-bench
```

| Bench | NFR | What it measures |
|---|---|---|
| `n1_cold_index_mini` | N1 | Cold index of a tiny multi-file fixture |
| `n1_incremental_one_file` | N1 | Re-index after one file edit |
| `n2_resolve_symbol` / `n2_neighbors` / `n2_impact_depth2` | N2 | Structural query latency |

Baselines: [`eval/scorecards/p6-stage-a-baselines.md`](../eval/scorecards/p6-stage-a-baselines.md)

This directory previously held only a README (gap G-07). The README remains as the discovery pointer; executable benches are the crate.
