# MCP tool catalog v1 (P1 Stage C)

**Server:** `prism mcp <workspace>` (stdio JSON-RPC 2.0, protocol `2024-11-05`)  
**Allowlist only** — no write / rename tools until P3.

| Tool | Inputs | Returns | Confidence |
|---|---|---|---|
| `index_status` | — | freshness, node/edge counts, sqlite bytes | N/A (metadata) |
| `resolve_symbol` | `name`, optional `file`, `limit` | symbol ids + paths | per-node |
| `neighbors` | `id`, optional `kind`, `dir`, `limit` | edge+node pairs | per-edge (`CALLS` = heuristic) |
| `impact` | `id`, `depth`≤8, `limit` | depth-grouped candidates | **always heuristic at T1** |
| `repo_map` | optional `hub_limit` | path-prefix communities + hubs | orientation only |

Every successful tool response includes `confidence_note` and `latency_ms`. Failures use the [error model](./MCP-ERROR-MODEL.md).

## Safety

- Tools are read-only against `.prism/`.
- Citations: node `id` + `file_path` (+ spans inside stored attrs).
- Agents must prefer these over grep/read loops for structural questions ([AGENT-USAGE.md](./AGENT-USAGE.md)).

## ADD §25 mapping

| ADD tool | P1 status |
|---|---|
| `index_status` | ✅ |
| `resolve_symbol` | ✅ |
| `neighbors` | ✅ |
| `impact` | ✅ heuristic |
| `repo_map` | ✅ path-prefix stub → Stage D hubs |
| `slice` / `compile_context` | P2+ |
| `query_plan` (CLI today) | P2 Stage A — `prism query plan`; MCP promote with Stage C |
| `detect_changes` / `find_tests` | later |

## Client config example

```json
{
  "mcpServers": {
    "prism": {
      "command": "prism",
      "args": ["mcp", "/absolute/path/to/repo"]
    }
  }
}
```

Index the repo first: `prism index /absolute/path/to/repo`.
