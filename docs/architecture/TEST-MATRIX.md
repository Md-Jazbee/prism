# Test matrix & CI expectations (P5 Stage B)

Aligns with ADD §31 layers.

| Layer | What | Where | CI |
|---|---|---|---|
| Golden facts | Extractor output vs `fixtures/languages/*/expected.json` | `prism-extract-*` unit tests | `cargo test` + `conformance-check.sh` |
| Planner fixtures | Plan IR goldens | `fixtures/plans/*` | `cargo test -p prism-plan` |
| Pack stability | Whitespace-only source change → stable must-include roles / ids | `prism-compile` test | `cargo test -p prism-compile` |
| Pack goldens | Budget / EXPLAIN fixtures | `fixtures/packs/*` | `cargo test -p prism-compile` |
| Precise oracle | T1→T2 call P/R | `fixtures/precise/oracle` | `prism-eval p3-scorecard` |
| Debug gate | Token proxy + pack gates | `eval/` | `prism-eval p4-scorecard` |
| Adversarial | Broken syntax no panic; SCOPE_UNRESOLVED refuse dump | semantic + plan fixtures | unit tests |
| Incremental edit | Dirty reverse-deps; secret skip | store + core | unit tests |

## Incremental edit benchmark (expectation)

| Scenario | Target |
|---|---|
| Single-file edit re-index | &lt;2s on pilot repos (G4) — measured when pilots indexed in CI |
| Whitespace-only edit | Pack must-include identity stable (see property test) |
| Secret path touch | No graph nodes; `file_skipped_secret` |

## Shadow token-savings metric

Obs event `token_savings_shadow`: compare `explore_tokens_proxy` vs `pack.tokens_used` when clients report both. Not a hard CI gate until baselines land under `eval/baselines/`.
