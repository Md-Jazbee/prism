# Visual encoding

**Phase:** P7 Stage B  
**Package:** `@prism/graph-view`  
**Legend:** mandatory on every SVG / interactive mount.

| Signal | Encoding |
|---|---|
| Tier T1–T4 | Badge text + border weight (1–4); Okabe–Ito fill as secondary |
| Confidence `heuristic` | **Dashed** edge |
| Confidence `precise` | **Solid** edge |
| Confidence `observed` | **Dotted** edge |
| Aggregated edge | Thickness ∝ member count; confidence = weakest member |
| Heat | Node radius boost (not color-only) |
| Kind Community | Round rectangle |
| Kind Ambiguous | Diamond |
| Kind Hotspot / Layer | Hexagon |

**A11y:** every node/edge has an `aria-label` including label, kind, tier, confidence. Keyboard: Cytoscape selection; SVG is readable as structured text for exports.

Confidence is **never** color-only — stroke pattern carries the authority signal.
