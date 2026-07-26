# P6 Stage A — N1/N2 performance baselines

**Date:** 2026-07-26  
**Harness:** `cargo bench -p prism-bench --bench n1_index --bench n2_structural_query`  
**Fixture:** mini workspace — 8 Python + 8 Rust synthetic modules (`prism-bench` helpers)  
**Host:** local kickoff (darwin); CI `bench` job re-runs with short sample size

## Targets (ADD NFRs)

| ID | Target | Notes |
|---|---|---|
| N1 cold index | &lt;5 min / 100k LOC | Mini fixture is far smaller; used for regression slope, not absolute NFR proof |
| N1 incremental edit | &lt;2 s single-file | Same |
| N2 structural query P95 | &lt;50 ms | Mini KG; expand to pilot repos before hard-fail thresholds |

## Baseline snapshot (2026-07-26 kickoff)

Criterion reports `[low mean high]`. Means below are the middle value.

| Bench | Mean | vs target |
|---|---|---|
| `n1_cold_index_mini` | **6.52 ms** | ≪ N1 cold (fixture-scale) |
| `n1_incremental_one_file` | **3.43 ms** | ≪ N1 incremental |
| `n2_resolve_symbol` | **31.8 µs** | ≪ 50 ms P95 |
| `n2_neighbors` | **12.8 µs** | ≪ 50 ms P95 |
| `n2_impact_depth2` | **20.8 µs** | ≪ 50 ms P95 |

```bash
cargo bench -p prism-bench --bench n1_index --bench n2_structural_query -- \
  --sample-size 10 --warm-up-time 1 --measurement-time 1
```

## CI policy (Stage A)

- Job `bench` runs the command above (smoke that benches compile and execute).
- Hard P95 fail gates wait until pilot-repo numbers exist; until then the gate is **presence + green run**, not a numeric ceiling on CI.
