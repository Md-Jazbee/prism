# Precise-tier oracle fixtures (P3 Stage A)

Compares T1 heuristic call resolution vs T2 PreciseIndex against a labeled oracle.

## Python cross-file call

| File | Role |
|---|---|
| `python/app.py` / `lib.py` | Source under test (documented; import uses JSON facts) |
| `python/t1-calls.json` | Heuristic CALLS (same-file policy → `greet` unresolved) |
| `python/precise-index.json` | T2 PreciseIndex resolving `greet` → `lib.py` |
| `python/oracle-calls.json` | Ground-truth callees |
| `python/expected-scores.json` | Expected P/R deltas |

T2 precision must exceed T1 on this fixture (`cargo test -p prism-precise`).
