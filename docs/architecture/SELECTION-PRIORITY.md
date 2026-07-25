# Selection priority (P2 Stage B)

Deterministic ordering when assembling candidate fragments (ADD §17.1).

| Priority | Source | Notes |
|---:|---|---|
| 1 | Explicit anchors in question / hints | symbols, paths, stack, errors |
| 2 | Diff / dirty worktree intersection | review / impact seeds |
| 3 | Resolve anchors (T1 today; T2 later) | `ResolveSymbol` |
| 4 | Operator expansion per intent | Expand / Impact / Slice(placeholder) |
| 5 | Architecture neighborhood | `CommunityOf` / hubs |
| 6 | Embedding / keyword fallback | **not** in default v0 recipes; low confidence only |

## Intent expansion (summary)

See [INTENT-RECIPES.md](./INTENT-RECIPES.md). Compiler maps recipe `must_include` roles onto fragments and tags optional drops via `drop_priority`.

## Anti-patterns (rejected)

- Top-k similar chunks without graph binding  
- Whole-file inclusion by default  
- Multiple exemplars “just in case”  
- Vendored code unless implicated  
