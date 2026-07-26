# Precision gating matrix (P3 Stage C)

**Status:** Locked for Phase 3 gate  
**See:** [UPGRADE-POLICY.md](./UPGRADE-POLICY.md) · [MCP-ERROR-MODEL.md](./MCP-ERROR-MODEL.md) · [SAFE-RENAME-DRY-RUN.md](./SAFE-RENAME-DRY-RUN.md)

---

## Principle

Heuristic T1 answers **always remain available and labeled**. Accuracy claims (safe rename, “precise impact”, refactor completeness) require T2 when the product path asserts accuracy — otherwise return `PRECISION_REQUIRED`. Never silently upgrade confidence.

---

## Tool / path matrix

| Surface | Default tier | Accuracy claim (`require_precise` / rename dry-run) | Notes |
|---|---|---|---|
| `compile_context` (repo_qa, architecture, generate, review) | T1 | — | Cheap path |
| `compile_context` (debug) | T1 + bounded UpgradePrecision | — | Heuristic OK; gaps note missing T2 |
| `compile_context` (impact / refactor) | T1 + UpgradePrecision | If `require_precise=true` and no overlay → `PRECISION_REQUIRED` | Pack still labels heuristic fragments |
| `impact` MCP/CLI | T1 heuristic | `require_precise=true` → need overlay + ≥1 precise edge on seed | Default confidence_note stays HEURISTIC |
| `neighbors` / `resolve_symbol` / `repo_map` / `index_status` | T1 | — | Never gated |
| `query_plan` | plan-only | — | Shows UpgradePrecision steps |
| `prism precise rename-dry-run` | **T2 required** | Always gated (unless `--allow-heuristic` override) | **Read-only**; no file writes |
| Any write / apply-rename | ❌ | Forbidden in P3 | Stage C ships dry-run only |

---

## Overrides

| Flag | Effect | Security note |
|---|---|---|
| `--allow-heuristic` / `allow_heuristic: true` | Bypass `PRECISION_REQUIRED`; results stay labeled `heuristic` | Explicit; never default for rename claims |
| Missing `.prism/scip/` | T1 tools work; gated ops fail closed | Import via [SCIP-RUNBOOK.md](./SCIP-RUNBOOK.md) |

---

## Agent rules

1. Prefer `compile_context` first.  
2. Do not claim rename safety from unlabeled `impact`.  
3. On `PRECISION_REQUIRED`, import PreciseIndex or use `--allow-heuristic` with an explicit caveat.  
4. Dual candidates from hybrid resolve → surface uncertainty; do not pick silently.
