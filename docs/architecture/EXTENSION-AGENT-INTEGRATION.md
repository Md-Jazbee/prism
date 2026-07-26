# Cursor / VS Code agent integration (P8)

## Auto-registration

On first run (when `prism.agent.autoRegisterMcp` is true), the extension merges:

```json
{
  "mcpServers": {
    "prism": {
      "command": "<resolved prism path>",
      "args": ["mcp", "<workspaceRoot>"]
    }
  }
}
```

into `.cursor/mcp.json` (preferred) or `.vscode/mcp.json`.

Toggle: **Prism: Enable/Disable Cursor MCP Registration**. Visible in the JSON file — no hidden registration.

## Generated guidance

**Prism: Generate AGENTS.md** writes:

- `AGENTS.md` (compile-first policy)
- `.cursor/rules/prism-compile-first.mdc`

Regenerate when AGENT-USAGE changes; do not hand-edit the generated banner block.

## Refusal UX

| Code | UI action |
|---|---|
| SCOPE_UNRESOLVED | Pick Anchor |
| PRECISION_REQUIRED | Open SCIP runbook |
| INDEX_UNAVAILABLE | Build Index |
| VIEW_TOO_LARGE | Show suggested anchors |
| VERSION_SKEW | Error + upgrade hint |

MCP transport remains hand-rolled stdio (ADR-0003 reaffirmed at P8); extension panel RPCs use HTTP/CLI, not MCP.
