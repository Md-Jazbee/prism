# Agent usage guide (P2 Stage C)

## Primary path: `compile_context` first

For almost all repo Q&A, impact, architecture, review, and generate intents:

1. **`compile_context`** — one call returns a budgeted Evidence Pack + EXPLAIN  
2. Answer from the pack citations; inspect `gaps` / `explain.drops` if unsure  
3. Only then use micro-tools (`resolve_symbol`, `neighbors`, `impact`, `repo_map`) for a *targeted* follow-up  

Do **not** open dozens of files via grep/read when `compile_context` can answer.

Optional: `query_plan` to inspect the operator DAG without packing.

## Anti-patterns

| Anti-pattern | Do instead |
|---|---|
| Ten reads / greps to “explore” | One `compile_context` |
| Grep for every call site first | `compile_context` (impact intent) or `resolve`→`neighbors` |
| Recursive directory listing for architecture | `compile_context` (architecture) or `repo_map` |
| Claiming rename safety from `impact` | Wait for T2 (P3); treat impact as heuristic |
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
