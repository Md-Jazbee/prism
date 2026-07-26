# Layout selection matrix

**Phase:** P7 Stage A  
**IR coordinates:** always present on `GraphView.nodes[].x/y` (seeded grid from `prism-view`).  
**Renderer:** may refine with ELK/Cytoscape under the **same layout seed**, memoized under `.prism/views/` when writing caches.

| View kind | Algorithm (IR) | Renderer refinement | Determinism strategy |
|---|---|---|---|
| `architecture_map` | `layered` | ELK layered (LEFT_RIGHT) | Sorted node ids → layers by path depth; ELK inputs ordered by id |
| `layering_violations` | `layered` | ELK layered | Same; violation edges emphasized in encoding |
| `impact_cone` | `radial` | concentric / breadth rings | Hop depth → ring; angle by sorted id |
| `hotspot_heat` | `radial` | concentric by heat rank | Rank = heat desc, id asc |
| `ambiguity_heat` | `radial` | concentric by ambiguity group | Group name sorted |
| `slice_path` | `path` | linear / sugiyama path | Primary path left→right; spurs below |
| `pack_map` | `path` | linear fragment order | Fragment order = pack order; drops as side rail |

## Determinism contract

Identical `(snapshot_id, view_kind, params_key, kept node/edge ids)` ⇒ identical coordinates after refinement.

Whitespace-only source edits that do not bump `snapshot_id` leave views stable.

See [LAYOUT-DETERMINISM.md](./LAYOUT-DETERMINISM.md).
