# P9 phase gate scorecard

**Date:** 2026-07-26  
**Phase:** Agent Experience & Workflows  
**Status:** **PASS** (Stage A–C; four-arm published in scripted-proxy mode with honest LLM caveat)

| Gate item | Result | Evidence |
|---|---|---|
| Refusal-repair on every error | ✅ | `ToolError.repair` + `prism_agent::repair_for` + docs |
| Budget negotiation MCP/HTTP | ✅ | `remaining_context_tokens` |
| Progressive packs | ✅ | `progressive: true` / `progressive_layers` |
| Trace schema (no content) | ✅ | `schemas/agent-trace/v1` + `.prism/logs/` |
| Four workflows + fixtures | ✅ | catalog + `fixtures/workflows/` + CLI/HTTP |
| Assets regenerate from catalog | ✅ | `prism agent generate-assets` |
| Four-arm report published | ✅ | `PUBLIC-BENCHMARK-REPORT-V2.md` + `four_arm.py` |
| Dual-review precision sample | ✅ | `eval/labeling/packs/T001.dual.json` (70%, κ=0.78) |
| First-tool-choice / repair rates | ✅ | trace metrics in `four-arm/latest.json` |
| R1 / R2 / R8 resolved or restated | ✅ | `PROGRAM-RESIDUAL-RISKS.md` |

## Commands

```bash
cargo test -p prism-agent
cargo run -p prism-cli -- workflow list
python eval/baselines/four_arm.py
```
