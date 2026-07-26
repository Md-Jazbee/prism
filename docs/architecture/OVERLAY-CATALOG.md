# Overlay catalog

**Phase:** P7 Stage C  
**Fixtures:** [`fixtures/views/golden/`](../../fixtures/views/golden/)  
**Screenshots:** [`fixtures/views/screenshots/`](../../fixtures/views/screenshots/)

| Overlay / kind | What it shows | Distinctive cue |
|---|---|---|
| `architecture_map` | Communities + hubs | Layered; community rectangles |
| `impact_cone` | Blast radius from seed | Radial rings; heuristic dashed hops |
| `slice_path` | Criterion path | Linear path; T3/T4 precise solids |
| `pack_map` | Included fragments + drops | Visual EXPLAIN rail for drop reasons |
| `hotspot_heat` | Churn / degree heat | Hexagon + heat radius |
| `ambiguity_heat` | Duplicate names (T2 CTA) | Diamonds sharing labels |
| `layering_violations` | Illegal layer edges | Layered + `LAYER_VIOLATION` edges |

## Visual EXPLAIN

`pack_map.drops[]` reason codes render as annotations (`visualExplain` / SVG comments in Mermaid). Clicking a drop id answers “why is this missing?” without reading raw JSON.
