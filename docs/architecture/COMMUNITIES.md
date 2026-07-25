# Community detection design (P1 Stage D)

## Algorithm (v0)

1. **Path-prefix communities** — group files by first 1–2 directory segments (`src/`, `httpx/`, `crates/ignore/`). Deterministic; no LLM naming.
2. **Degree hubs** — rank `Symbol`/`File`/`Module` nodes by undirected edge degree; top-N exposed via `repo_map`.

Future: Leiden/Louvain on import+call graph (refresh on dirty-set threshold). Not required for P1 gate.

## Refresh triggers

| Event | Action |
|---|---|
| `repo_map` / MCP call | Recompute on demand from current `graph.sqlite` |
| Single-file edit | File subgraph replace only; communities **not** incrementally patched in v0 |
| Stage B dirty list | Advisory for planners; community recompute still on-demand |

## Labeling policy

- Label = path prefix string (no generative names in P1).
- Cap communities at 40 and hubs at `hub_limit` (default 15) to keep MCP payloads small.

## Exposure

- MCP: `repo_map`
- CLI: `prism query repo-map`
