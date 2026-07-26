# Panel UX — Evidence & Graph (P8)

## Evidence panel

- Shows last Evidence Pack: intent, tokens used/budget, question.
- Citations as buttons → `prism.evidencePeek` (file span + graph seed).
- Fragment list by layer; EXPLAIN toggle; Copy for LLM (redacted + local audit).
- Transport mode + degradation note in header (daemon vs CLI).

## Graph panel

- Hosts `@prism/graph-view` `mountCytoscape` with `schemas/graph-view/v1` only.
- Node select → `prism.focusNode` → impact_cone refresh.
- Survives theme switch via VS Code CSS variables; reload re-pushes session state.

## Single source of truth

Host `PrismSession` owns last pack / last view. Webviews are pure views (`postMessage`).
