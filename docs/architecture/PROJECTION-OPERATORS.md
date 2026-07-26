# Projection operator catalog

**Phase:** P7 Stage A  
**Consumes:** `schemas/graph-view/v1` (+ KG via `prism-view` on the server)  
**Rule:** Every operator has a **budget cost**. Operators that would explode past `max_nodes` return `VIEW_TOO_LARGE` with anchors — they never silently truncate seeds.

## Server-side operators (`prism-view`)

| Operator | Input | Output | Budget cost | Notes |
|---|---|---|---|---|
| `project(kind, params)` | view kind + seed/anchors/question | `GraphView` or `VIEW_TOO_LARGE` | full view budget | Primary entry; used by HTTP/CLI |
| `collapse_to_community` | architecture_map | community super-nodes | O(#communities) | Default LOD for orientation |
| `expand_hubs` | communities | + high-degree symbols | soft-drop after seeds | Drop priority > 0 |
| `impact_expand` | seed id + depth | cone nodes/edges | O(fan-out^depth) capped | Heuristic hops marked |
| `slice_expand` | seed id | path neighborhood | O(hops × degree) | Criterion never elided |
| `pack_project` | question/anchors | fragment nodes | pack fragment count | Same selection as compile |
| `heat_project` | intel report | hotspot / ambiguity / layering nodes | O(report size) | No fabricated edges |

## Client-side operators (`prism-graph-view`)

These act on an **already budgeted** `GraphView`. They do not query the store.

| Operator | Input | Output | Budget cost | Notes |
|---|---|---|---|---|
| `filter_by_tier` | view + min tier | subset | 0 (local) | Keeps citations |
| `filter_by_confidence` | view + allowed set | subset | 0 | Heuristic may be hidden, not restyled as precise |
| `filter_edge_kinds` | view + kinds | subset | 0 | Orphans may be ghosted |
| `focus_neighborhood` | view + node id + hops | subset | local hop budget (default 2) | Pins focus node |
| `collapse_group` | view + group id | one super-node | −(members−1) | Aggregated edge inherits **weakest** confidence |
| `promote_lod` / `demote_lod` | view + lod_rank band | filtered | 0 | See [LOD-POLICY.md](./LOD-POLICY.md) |

## Cost accounting

- **Server projection** spends the render budget (`max_nodes` / `max_edges`).
- **Client filters** only hide; they do not invent nodes. Expanding beyond the current view requires a **new** `POST /v1/view` with narrower anchors or a higher explicit budget.
- Interaction gestures map to these operators — see [INTERACTION-GRAMMAR.md](./INTERACTION-GRAMMAR.md).
