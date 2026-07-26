# MCP / tool error model (P3)

Tool failures return JSON with:

```json
{
  "code": "SCOPE_UNRESOLVED",
  "message": "…",
  "hint": "…"
}
```

| Code | When | Agent behavior |
|---|---|---|
| `SCOPE_UNRESOLVED` | Missing anchors / zero resolve hits / ambiguous question | Ask for a symbol/path/stack/error; **do not** dump the repo |
| `BUDGET_EXCEEDED` | Must-include fragments exceed `budget_tokens` | Raise budget or narrow scope; never soft-truncate must-include |
| `INDEX_UNAVAILABLE` | No `.prism/graph.sqlite` or open failure | Tell user to run `prism index` |
| `PRECISION_REQUIRED` | Precision-gated op needs T2 and no overlay / precise edges | Run indexer + `prism precise import`; keep T1 answers labeled heuristic |
| `INVALID_ARGS` | Unknown tool / bad args | Fix tool name/args |
| `INTERNAL` | Unexpected | Retry once; report bug |

MCP `tools/call` sets `isError: true` when a `ToolError` is returned; content still includes the structured error for the model to read.

See [PRECISE-TIER.md](./PRECISE-TIER.md) and [SCIP-RUNBOOK.md](./SCIP-RUNBOOK.md).
