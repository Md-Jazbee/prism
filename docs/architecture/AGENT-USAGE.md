# Agent usage guide (P4 Stage C)

## Primary path: `compile_context` first

For almost all repo Q&A, impact, architecture, review, generate, and **debug** intents:

1. **`compile_context`** — one call returns a budgeted Evidence Pack + EXPLAIN  
2. Answer from the pack citations; inspect `gaps` / `explain.drops` if unsure  
3. Only then use micro-tools (`resolve_symbol`, `neighbors`, `impact`, `repo_map`) for a *targeted* follow-up  

Do **not** open dozens of files via grep/read when `compile_context` can answer.

Optional: `query_plan` to inspect the operator DAG without packing.

## Debug / slice

| Need | Do |
|---|---|
| Crash / stack / “why” | `compile_context` with `intent=debug`, stack frames + error text |
| Local/interproc slice only | `prism semantic slice --file … --line …` |
| Security sink/source | Same debug path + optional provider anchors ([SINK-SOURCE-HOOKS.md](./SINK-SOURCE-HOOKS.md)) |

Error/stack verbatim and criterion slices are **never** dropped under budget pressure ([DEBUG-PACK-GATES.md](./DEBUG-PACK-GATES.md)).

See [DEBUG-RECIPES.md](./DEBUG-RECIPES.md).

## Precision (T2)

| Need | Do |
|---|---|
| Casual impact / neighborhood | `impact` or `compile_context` — accept **heuristic** labels |
| Accuracy claim / safe refactor | `require_precise=true` or `prism precise rename-dry-run` |
| Missing overlay | Expect `PRECISION_REQUIRED` → `prism precise import` (see SCIP-RUNBOOK) |
| Dual candidates | Surface uncertainty; do not silently pick |

See [PRECISION-GATING.md](./PRECISION-GATING.md).

## Anti-patterns

| Anti-pattern | Do instead |
|---|---|
| Ten reads / greps to “explore” | One `compile_context` |
| Whole-file dump on a crash | `compile_context` (debug) — Slice is executable |
| Grep for every call site first | `compile_context` (impact intent) or `resolve`→`neighbors` |
| Recursive directory listing for architecture | `compile_context` (architecture) or `repo_map` |
| Claiming rename safety from unlabeled `impact` | `rename-dry-run` / `require_precise=true` after T2 import |
| Dumping whole modules when scope is unclear | Expect `SCOPE_UNRESOLVED`; ask for an anchor |

## Hop budget

Prefer **1** `compile_context` hop. Structural micro-tool chains should stay in **1–4** calls. If you exceed ~8 hops, stop and re-scope.

Ambiguous questions without anchors return `SCOPE_UNRESOLVED` instead of dumping the repo.

## Errors to honor

| Code | Agent action |
|---|---|
| `SCOPE_UNRESOLVED` | Ask user for symbol / path / stack / error |
| `BUDGET_EXCEEDED` | Raise `budget_tokens` or narrow anchors |
| `INDEX_UNAVAILABLE` | Ask user to run `prism index` |
| `PRECISION_REQUIRED` | Import PreciseIndex or explicitly continue with labeled heuristic |
| `SEMANTIC_PARTIAL` | Fall back to T1 neighborhood; rebuild `prism semantic build` if needed |
