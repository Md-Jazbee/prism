# Refusal-repair contract

Every product error carries a **machine-actionable** next step. Suggestions are bounded — never another dump.

| Code | `repair.action` | Agent next step |
|---|---|---|
| `SCOPE_UNRESOLVED` | `pick_anchor` | Retry `compile_context` with `candidates[]` |
| `BUDGET_EXCEEDED` | `reduce_budget_or_narrow` | `query_plan` then recompile with higher budget / fewer anchors |
| `INDEX_UNAVAILABLE` | `run_index` | `prism index .` |
| `PRECISION_REQUIRED` | `import_precise` | `prism precise import` or drop `require_precise` |
| `VIEW_TOO_LARGE` | `narrow_view` | Narrow seeds / raise `max_nodes` |

Shape (also on MCP / HTTP errors):

```json
{
  "action": "pick_anchor",
  "summary": "…",
  "tool": "compile_context",
  "candidates": ["symbol name", "file path"]
}
```

Rust: `prism_agent::repair_for` · MCP: `ToolError.repair`.
