# Host adapter matrix (P11 Stage B)

Idempotent writers used by `prism host` and (for the default host) `prism setup`.

| Host | Files touched | Install behavior | Uninstall behavior |
|---|---|---|---|
| `cursor` | `.cursor/mcp.json` | Merge `mcpServers.prism` | Remove only `prism` key |
| `vscode` | `.vscode/mcp.json` | Merge `servers.prism` (`type: stdio`) | Remove only `prism` key from `servers` (and legacy `mcpServers`) |
| `claude` | `CLAUDE.md`, `.mcp.prism.json` | Upsert marked `## Prism` section + portable snippet | Strip marked section; delete snippet |
| `generic` | `.mcp.prism.json` | Write portable stdio MCP snippet | Delete snippet |

## Merge rules

- Never delete unrelated MCP servers.
- `command` is the absolute path of the running `prism` binary when available.
- `args` are always `["mcp", "<workspace-root>"]`.

## CLI

```bash
prism host install cursor
prism host install claude
prism host status --json
prism host uninstall vscode
```

`prism setup` registers the default host: Cursor when `.cursor/` exists (or neither IDE dir exists); otherwise VS Code.
