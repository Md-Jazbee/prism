# Debug & security recipes v1 (P4 Stage C)

**Status:** Locked for Phase 4 gate  
**See:** [SLICE-OPERATOR.md](./SLICE-OPERATOR.md) · [INTENT-RECIPES.md](./INTENT-RECIPES.md) · [DEBUG-PACK-GATES.md](./DEBUG-PACK-GATES.md)

---

## Debug recipe (executable)

```text
ResolveSymbol (stack / error loci)
  → UpgradePrecision (mandatory, critical_path_only, ≤200ms)
  → Slice (backward, max_depth=2, residual_expand)
  → DiffIntersect (worktree / since main)
  → Expand (CALLS signatures, depth 1)
  → BudgetPack (debug)
```

| Must-include | Never drop under budget |
|---|---|
| `error_or_stack_verbatim` | Yes |
| `primary_frame_body` (slice spans) | Yes |

Gaps may note truncated Slice residuals or missing T2 overlay; they do **not** authorize dropping criterion / stack.

---

## Security profile (same operators)

Security intents reuse the **debug DAG** with extra criteria:

1. Optional [SINK-SOURCE-HOOKS.md](./SINK-SOURCE-HOOKS.md) providers supply sink/source loci.  
2. Union those loci with stack/error anchors as Slice criteria.  
3. Prefer `backward` from sinks (or `forward` from sources) with the same depth caps.  
4. Pack must still keep error/stack verbatim when present.

No separate `Intent::Security` in v1 — agents pass `intent=debug` plus sink/source anchors (or a future MCP hint).

---

## Agent path

1. `compile_context` with stack frames + error text (and optional path:line).  
2. Read pack Core layer (error + slice) before any explore loop.  
3. Only then Expand residuals / neighbors if the answer is incomplete.

See [AGENT-USAGE.md](./AGENT-USAGE.md).
