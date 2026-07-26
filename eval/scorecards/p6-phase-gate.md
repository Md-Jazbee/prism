# P6 phase gate scorecard

**Date:** 2026-07-26  
**Phase:** Consolidation & Interaction Substrate  
**Status:** **PASS** (Stage A–C complete)

| Gate item | Result | Evidence |
|---|---|---|
| §12 gaps closed or waived with expiry | ✅ | [DRIFT-CLOSURE-P6A.md](../docs/eval/DRIFT-CLOSURE-P6A.md) |
| Non-CLI client: status → view → pack over HTTP | ✅ | `POST /v1/index/status`, `/v1/view`, `/v1/context/compile` |
| `schemas/graph-view/v1` frozen + fixtures | ✅ | `schemas/graph-view/v1/` + `fixtures/views/` |
| Oversized view → `VIEW_TOO_LARGE` | ✅ | `prism-view` refuse path + golden |
| LSP hover / codelens | ✅ | `prism-lsp` / `prism lsp` |
| N1/N2 benches present (no Stage A regress job failure) | ✅ | `crates/prism-bench` + CI bench smoke |
| CLI works without daemon | ✅ | Hard rule preserved |

## Commands

```bash
cargo test -p prism-view
cargo test -p prism-api --test http_smoke
prism view architecture_map .
```
