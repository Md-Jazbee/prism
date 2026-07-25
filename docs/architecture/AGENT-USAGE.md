# Agent usage guide (P1 Stage C)

## Prefer structural tools over explore loops

For symbol location, call graphs, blast radius, and repo orientation:

1. `index_status` — confirm an index exists  
2. `resolve_symbol` — get a stable node id  
3. `neighbors` and/or `impact` — expand structure  
4. `repo_map` — only when you need orientation / hubs  

Do **not** open dozens of files via grep/read when these tools answer the question.

## Anti-patterns

| Anti-pattern | Do instead |
|---|---|
| Grep for every call site first | `resolve_symbol` → `neighbors` kind=CALLS |
| Recursive directory listing for architecture | `repo_map` |
| Claiming rename safety from `impact` | Wait for T2 (P3); treat impact as heuristic |
| Dumping whole modules when scope is unclear | Expect `SCOPE_UNRESOLVED`; ask user for an anchor |

## Hop budget

Structural gold tasks should complete in **1–4** Prism tool calls. If you exceed ~8 hops, stop and re-scope — Phase 2 `compile_context` will replace multi-hop thrash.
