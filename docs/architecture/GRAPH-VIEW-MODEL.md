# Graph View-Model (v1)

**Phase:** P6 Stage C  
**Schema:** [`schemas/graph-view/v1/`](../../schemas/graph-view/v1/)  
**Crate:** `prism-view`  
**HTTP:** `POST /v1/view` · **CLI:** `prism view <kind>`

A view is a **budgeted, layout-ready projection** of the KG — the visual analogue of an Evidence Pack. Renderers (P7) consume only this schema; they never touch SQLite.

## Non-negotiables

| Rule | Behavior |
|---|---|
| Render budget | `max_nodes` / `max_edges` with deterministic drop order |
| Refuse, don't fake | Oversized seed sets return `VIEW_TOO_LARGE` with suggested anchors |
| Every element cites | Nodes/edges carry `citation.node_ids` (+ optional path/span) |
| Deterministic layout | Same `(snapshot_id, view_kind, params, kept set)` ⇒ same coordinates |
| Confidence visible | `tier` + `confidence` on every element |

## View kinds catalog

| Kind | Seeds | Expansion | Default layout | Default budget |
|---|---|---|---|---|
| `architecture_map` | path-prefix communities | hubs (soft drop) | layered | 80 / 160 |
| `impact_cone` | symbol id | depth-limited impact | radial | 80 / 160 |
| `slice_path` | symbol id | neighbor hops | path | 80 / 160 |
| `pack_map` | compile question | Evidence Pack fragments | path | 80 / 160 |
| `hotspot_heat` | intel hotspots | — | radial | 80 / 160 |
| `layering_violations` | layering hints | — | layered | 80 / 160 |
| `ambiguity_heat` | duplicate symbol names | — | radial | 80 / 160 |

## Layout determinism

See [LAYOUT-DETERMINISM.md](./LAYOUT-DETERMINISM.md). Algorithm is a seeded grid/radial/path placement over **sorted node ids** — good enough for screenshot diffs; ELK/Cytoscape replace coordinates in P7 without changing the IR.
