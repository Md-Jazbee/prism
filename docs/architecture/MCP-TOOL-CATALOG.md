# MCP tool catalog (P5 Stage A · schemas externalized P6 Stage A)

**Server:** `prism mcp <workspace>` (stdio JSON-RPC 2.0, protocol `2024-11-05`)  
**Allowlist only** — no write / apply-rename tools (dry-run is CLI-only).  
**Primary tool:** `compile_context`  
**Contract of record:** [`schemas/mcp-tools/v1/`](../../schemas/mcp-tools/v1/) (gap G-11)  
**Transport ADR:** [ADR-0003](./adr/0003-mcp-transport-hand-rolled.md) — hand-rolled stdio; `rmcp` deferred  
**Gating:** [PRECISION-GATING.md](./PRECISION-GATING.md)  
**Intel:** [REPO-INTELLIGENCE.md](./REPO-INTELLIGENCE.md)

| Tool | Inputs | Returns | Confidence |
|---|---|---|---|
| **`compile_context`** | `question`, optional budget/intent/anchors/…, `require_precise` | Evidence Pack + EXPLAIN | per-fragment; accuracy claims need T2 when `require_precise` |
| `query_plan` | same hints as compile | Plan IR (operator DAG) | recipe notes / gaps |
| `index_status` | — | freshness, node/edge counts, sqlite bytes | N/A (metadata) |
| `resolve_symbol` | `name`, optional `file`, `limit` | symbol ids + paths | per-node |
| `neighbors` | `id`, optional `kind`, `dir`, `limit` | edge+node pairs | per-edge (`CALLS` may be heuristic or precise) |
| `impact` | `id`, `depth`≤8, `limit`, optional `require_precise` | depth-grouped candidates | default **heuristic**; gated when `require_precise` |
| `repo_map` | optional `hub_limit`, `full_intel` | communities + hubs (or full `RepoIntelReport`) | orientation / heuristic |
| `entrypoints` | optional `limit` | heuristic mains/cli/handlers | heuristic |
| `detect_changes` | optional `changed_paths[]` | dirty reverse-deps + hotspots | observed (git) or heuristic |

Every successful tool response includes `confidence_note` and `latency_ms`. Failures use the [error model](./MCP-ERROR-MODEL.md) (`PRECISION_REQUIRED` for gated accuracy claims).

## Safety

- Tools are read-only against `.prism/`.
- Safe rename dry-run is **CLI/script only** — never applies edits ([SAFE-RENAME-DRY-RUN.md](./SAFE-RENAME-DRY-RUN.md)).
- Heuristic answers stay labeled; never silently upgraded to precise.
- Agents must prefer `compile_context` over grep/read loops ([AGENT-USAGE.md](./AGENT-USAGE.md)).

## ADD §25 mapping

| ADD tool | Status |
|---|---|
| `compile_context` | ✅ primary |
| `query_plan` | ✅ |
| `index_status` | ✅ |
| `resolve_symbol` | ✅ |
| `neighbors` | ✅ |
| `impact` | ✅ + `require_precise` gate |
| `repo_map` | ✅ + optional full intel |
| `entrypoints` | ✅ P5 |
| `detect_changes` | ✅ P5 |
| `slice` | executable (P4 Stage B; depth-capped interproc) |
| rename apply | ❌ dry-run only (CLI) |

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
