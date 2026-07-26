# P6 Stage A — Drift closure report

**Opened:** 2026-07-26  
**Phase:** Consolidation & Interaction Substrate — Stage A  
**Source:** [planning §12 gap register](../planning/PLANNING-AND-IMPLEMENTATION.md#12-post-phase-5-repository-re-analysis--gap-register)

Standing rule: no claim without an artifact. Each row is `built`, `waived` (dated + expiry), or `deferred` (named stage).

| Gap | Summary | Resolution | Artifact | Expiry |
|---|---|---|---|---|
| **G-01** | No HTTP/SSE API | **built** | `prism-api` + [HTTP-API-V1.md](../architecture/HTTP-API-V1.md) | — |
| **G-02** | No LSP server | deferred | Stage C (`prism-lsp`) | P6-C |
| **G-03** | WASM host claimed “proven”, not built | **waived** | [ADR-0001](../architecture/adr/0001-wasm-plugin-host-deferred.md) — claim amended | P8 |
| **G-04** | Language re-baseline undocumented | **waived** | [ADR-0002](../architecture/adr/0002-language-rebaseline-python-rust.md) | P9 |
| **G-05** | MCP not on `rmcp` | **waived** | [ADR-0003](../architecture/adr/0003-mcp-transport-hand-rolled.md) | P8 |
| **G-06** | N2 P95 never measured | **built** | criterion `n2_structural_query` + [baselines](../../eval/scorecards/p6-stage-a-baselines.md) | — |
| **G-07** | `benches/` README-only; no CI gate | **built** | `crates/prism-bench` + CI `bench` job | — |
| **G-08** | No `LICENSE` / `deny.toml` | **built** | root `LICENSE`, `deny.toml`, CI `cargo-deny` | — |
| **G-09** | No OTLP exporter | **waived (partial)** | Env `PRISM_OTLP_ENDPOINT` opt-in hook in `prismd`; full OTLP SDK exporter deferred | P7 |
| **G-10** | No Tokio/Rayon | **built** | Tokio in `prismd`; Rayon fan-out in `IncrementalIndexer` | — |
| **G-11** | `schemas/mcp-tools/v1` missing | **built** | `schemas/mcp-tools/v1/` + conformance test | — |
| **G-12** | Four-arm LLM / precision ≥70% | deferred | P9 Stage C (out of P6 Stage A scope) | P9-C |
| **G-13** | No visual surface | deferred | P6-C view-model → P7 renderer | P7 |
| **G-14** | No IDE extension | waived | P8 → **cut** ADR-0007 | CLI+MCP |
| **G-15** | No agent workflow assets | deferred | P9 | P9 |

## Crate inventory note

| Planned crate | As-built | ADR |
|---|---|---|
| `prism-graph` | folded into `prism-store` | [ADR-0004](../architecture/adr/0004-crate-consolidation-store.md) |
| `prism-intel` | `prism-store::intel` | ADR-0004 |
| `prism-plugin-host` | not built | ADR-0001 |

## Stage A exit checklist (live)

- [x] G-03…G-11 each `built`, `waived`, or `deferred` with named stage
- [x] N1/N2 have recorded numbers (see baselines; may miss targets)
- [x] Docs no longer claim a proven WASM host
- [x] `cargo deny` + bench jobs present in CI
