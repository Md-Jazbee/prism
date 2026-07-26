# Interaction grammar

**Phase:** P7 Stage B  
**Package:** `@prism/graph-view`  
**Rule:** Every gesture maps to a **bounded** local op or a budgeted `POST /v1/view`. Unbounded “show more” is forbidden.

| Gesture | Query | Budget | Refusal |
|---|---|---|---|
| Focus | Local neighborhood filter | ≤ current view | Stay in view; re-project to widen |
| Expand | Server `project` with `seed_id` | view `max_nodes` | `VIEW_TOO_LARGE` + anchors |
| Collapse | Local `collapseGroup` | decreases nodes | N/A |
| Filter tier / confidence | Local filter | 0 | Hides only; never invents |
| Path-between | Server `slice_path` | 80/160 default | `VIEW_TOO_LARGE` |
| Why is this here? | Local citation / drop reason | 0 | Missing citation = bug |
| Breadcrumb back | Client history | 0 | No new query |

See implementation: `packages/prism-graph-view/src/interact/grammar.ts`.
