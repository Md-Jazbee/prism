# Performance envelope (P7)

| LOD ceiling (nodes) | Target interaction | Path |
|---:|---|---|
| ≤ 40 | 60 fps pan/zoom | Cytoscape canvas |
| ≤ 80 | 30 fps; layout preset (no reflow on pan) | Cytoscape |
| > 80 | Refuse (`VIEW_TOO_LARGE`) or explicit opt-in | — |
| Escape hatch | WebGL (Sigma) | **Not default** — only behind explicit confirmation |

Measurements (local vitest / export path, 2026-07-26):

| Fixture | Nodes | SVG export | Notes |
|---|---:|---:|---|
| architecture_map | 3 | < 5 ms | deterministic |
| pack_map + EXPLAIN | 2 | < 5 ms | drops annotated |
| All goldens screenshot-diff | 7 views | suite < 1 s | CI gate |

Full browser frame timing lands with the P8 webview harness; Stage B exit uses export + element construction as the proxy envelope.
