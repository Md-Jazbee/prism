# Program residual risks (after Phase 5)

Known failure modes that remain **accepted** or **tracked** after the P5 gate.

| ID | Risk | Status | Mitigation / next |
|---|---|---|---|
| R1 | LLM quality gap vs frontier+explore unmeasured | Open | `eval/baselines/` four-arm run — **owned by P9 Stage C** |
| R2 | Context precision proxy 60% &lt; 70% north star | Open | Dual-review labels; pack tuning — **owned by P9 Stage C** |
| R3 | Heuristic CALLS without PreciseIndex | Accepted for T1 | SCIP import / UpgradePrecision |
| R4 | Path-prefix communities ≠ true architecture | Accepted | Optional Leiden later |
| R5 | Language coverage uneven (Python/Rust first) | Accepted | Plugin guide for expansion; waiver in P6 Stage A |
| R6 | Eval contamination / non-repro | Mitigated | Pinned SHAs + suite version |
| R7 | Agents bypass `compile_context` | Mitigated | AGENT-USAGE + MCP instructions; **measured on traces in P9** |
| R8 | IDE extension not shipped | Deferred | IDE-INTEGRATION design ready — **owned by P8** |
| R9 | Multi-tenant / shared index | Deferred | Phase 10 optional (renumbered from Phase 6) |
| R10 | Narrative overlap Graphify/GitNexus | Mitigated | Emphasize packs + slice + eval |
| **R11** | P5 tech-view claimed a *proven* WASM plugin host; none was built | **Waived** | [ADR-0001](../architecture/adr/0001-wasm-plugin-host-deferred.md) — claim amended; revisit by **P8** |
| **R12** | No non-CLI/non-MCP surface: no HTTP API, no LSP, no daemon | **Open** | P6 Stages B–C (gaps G-01, G-02, G-10) |
| **R13** | Structural query P95 (NFR N2) has never been measured; no perf regression gate exists | **Mitigated** | `crates/prism-bench` + CI `bench` job; baselines in `eval/scorecards/p6-stage-a-baselines.md` (hard P95 fail thresholds still TBD) |
| **R14** | No visual surface; orientation output is JSON only | **Open** | P7 (gap G-13) |
| **R15** | No agent-side assets (rules, `AGENTS.md`, workflows); adoption depends on an agent reading a doc | **Open** | P9 Stage B (gap G-15) |

## Honest interim (P5 gate)

Phase 5 **passes** on engineering + token proxies + published methods, with **explicit interim** on LLM quality ≤3pts and precision ≥70%. Closing those is a post-gate eval program, not a silent claim.

## Post-P5 re-analysis (2026-07-26)

A full repository audit added R11–R15. The pattern is consistent: the **engine** matches its design, but several documents described **surfaces and tooling that were never built**. Risks R11 and R13 are the uncomfortable ones — a gate said “proven” and a growth rule said “perf regressions fail CI”, and neither was backed by an artifact.

The program response is a new standing rule — *no claim without an artifact* — plus a dedicated reconciliation workstream (**W-DEBT**) that must close or re-waive drift at every phase exit from P6 onward. Full register: [planning §12](../planning/PLANNING-AND-IMPLEMENTATION.md#12-post-phase-5-repository-re-analysis--gap-register).
