# Program residual risks (after Phase 5)

Known failure modes that remain **accepted** or **tracked** after the P5 gate.

| ID | Risk | Status | Mitigation / next |
|---|---|---|---|
| R1 | LLM quality gap vs frontier+explore unmeasured | Open | `eval/baselines/` four-arm run |
| R2 | Context precision proxy 60% &lt; 70% north star | Open | Dual-review labels; pack tuning |
| R3 | Heuristic CALLS without PreciseIndex | Accepted for T1 | SCIP import / UpgradePrecision |
| R4 | Path-prefix communities ≠ true architecture | Accepted | Optional Leiden later |
| R5 | Language coverage uneven (Python/Rust first) | Accepted | Plugin guide for expansion |
| R6 | Eval contamination / non-repro | Mitigated | Pinned SHAs + suite version |
| R7 | Agents bypass `compile_context` | Mitigated | AGENT-USAGE + MCP instructions |
| R8 | IDE extension not shipped | Deferred | IDE-INTEGRATION design ready |
| R9 | Multi-tenant / shared index | Deferred | Phase 6 optional |
| R10 | Narrative overlap Graphify/GitNexus | Mitigated | Emphasize packs + slice + eval |

## Honest interim (P5 gate)

Phase 5 **passes** on engineering + token proxies + published methods, with **explicit interim** on LLM quality ≤3pts and precision ≥70%. Closing those is a post-gate eval program, not a silent claim.
