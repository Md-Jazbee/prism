# MCP / tool error model (P1 Stage C)

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
| `SCOPE_UNRESOLVED` | Missing/blank name or id; zero resolve hits | Ask for a symbol/path anchor; **do not** dump the repo |
| `INDEX_UNAVAILABLE` | No `.prism/graph.sqlite` or open failure | Tell user to run `prism index` |
| `INVALID_ARGS` | Unknown tool / bad allowlist | Fix tool name/args |
| `INTERNAL` | Unexpected | Retry once; report bug |

MCP `tools/call` sets `isError: true` when a `ToolError` is returned; content still includes the structured error for the model to read.
