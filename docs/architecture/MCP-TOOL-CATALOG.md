# MCP tool catalog (P2 Stage C)

**Server:** `prism mcp <workspace>` (stdio JSON-RPC 2.0, protocol `2024-11-05`)  
**Allowlist only** — no write / rename tools until P3.  
**Primary tool:** `compile_context`

| Tool | Inputs | Returns | Confidence |
|---|---|---|---|
| **`compile_context`** | `question`, optional budget/intent/anchors/stack/error/changed_paths | Evidence Pack + EXPLAIN | per-fragment provenance |
| `query_plan` | same hints as compile | Plan IR (operator DAG) | recipe notes / gaps |
| `index_status` | — | freshness, node/edge counts, sqlite bytes | N/A (metadata) |
| `resolve_symbol` | `name`, optional `file`, `limit` | symbol ids + paths | per-node |
| `neighbors` | `id`, optional `kind`, `dir`, `limit` | edge+node pairs | per-edge (`CALLS` = heuristic) |
| `impact` | `id`, `depth`≤8, `limit` | depth-grouped candidates | **always heuristic at T1** |
| `repo_map` | optional `hub_limit` | path-prefix communities + hubs | orientation only |

Every successful tool response includes `confidence_note` and `latency_ms`. Failures use the [error model](./MCP-ERROR-MODEL.md).

## Safety

- Tools are read-only against `.prism/`.
- Every Evidence Pack fragment carries provenance (`node_ids`, analyzer, tier).
- Agents must prefer `compile_context` over grep/read loops ([AGENT-USAGE.md](./AGENT-USAGE.md)).

## ADD §25 mapping

| ADD tool | Status |
|---|---|
| `compile_context` | ✅ primary (P2 Stage C) |
| `query_plan` | ✅ MCP + CLI |
| `index_status` | ✅ |
| `resolve_symbol` | ✅ |
| `neighbors` | ✅ |
| `impact` | ✅ heuristic |
| `repo_map` | ✅ path-prefix + hubs |
| `slice` | placeholder until P4 |
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
