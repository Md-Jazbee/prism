# Program residual risks (after Phase 9)

Known failure modes after the P9 gate.

| ID | Risk | Status | Mitigation / next |
|---|---|---|---|
| R1 | LLM quality gap vs frontier+explore | **Restated** | Four-arm **published** in scripted-proxy mode ([PUBLIC-BENCHMARK-REPORT-V2.md](./PUBLIC-BENCHMARK-REPORT-V2.md)); live LLM judges remain opt-in (`PRISM_FOUR_ARM_LLM=1`). G1 holds on proxy (−2 pts). |
| R2 | Context precision vs ≥70% north star | **Closed on sample** | Dual-review T001: **70%** precision, κ=0.78 (`eval/labeling/packs/T001.dual.json`). Expand n≥20 ongoing. |
| R3 | Heuristic CALLS without PreciseIndex | Accepted for T1 | SCIP import / UpgradePrecision |
| R4 | Path-prefix communities ≠ true architecture | Accepted | Optional Leiden later |
| R5 | Language coverage uneven (Python/Rust first) | Accepted | Plugin guide; ADR-0002 |
| R6 | Eval contamination / non-repro | Mitigated | Pinned SHAs + suite version |
| R7 | Agents bypass `compile_context` | **Measured** | Trace first-tool-choice on fixtures + Prism arms; workflows + generated AGENTS.md |
| R8 | IDE extension not shipped | **Waived (choice)** | ADR-0007 — CLI + MCP; no VSIX |
| R9 | Multi-tenant / shared index | Deferred | Phase 10 optional |
| R10 | Narrative overlap Graphify/GitNexus | Mitigated | Packs + slice + eval |
| R11 | WASM host claim | **Waived** | ADR-0001 |
| R12 | No HTTP/LSP/daemon | **Closed** | P6 |
| R13 | Structural query P95 unmeasured | **Mitigated** | prism-bench + CI |
| R14 | No visual surface | **Closed** | P7 `@prism/graph-view` |
| R15 | No agent-side assets | **Closed** | P9 catalog → AGENTS.md / rules / skills |

## Honest note

P9 closes the interaction-half Definition of Done for agent assets and publishes a four-arm report. **Live frontier LLM scoring** is still an opt-in follow-on — we do not invent API scores.
