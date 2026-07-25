# MCP / tool error model (P2)

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
| `INVALID_ARGS` | Unknown tool / bad args | Fix tool name/args |
| `INTERNAL` | Unexpected | Retry once; report bug |

MCP `tools/call` sets `isError: true` when a `ToolError` is returned; content still includes the structured error for the model to read.
