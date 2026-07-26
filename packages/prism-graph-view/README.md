# `@prism/graph-view`

Budgeted Graph View-Model renderer for Prism (Phase 7).

**Input:** `schemas/graph-view/v1` only. Never touches SQLite or the daemon store.

## Install / build

```bash
pnpm install
pnpm --filter @prism/graph-view build
pnpm --filter @prism/graph-view test
```

## API sketch

```ts
import {
  exportSvg,
  mountCytoscape,
  visualExplain,
  gestureToRequest,
} from "@prism/graph-view";

const svg = exportSvg(view);
const explain = visualExplain(view);
const cy = mountCytoscape({ container, view, onSelect: (id) => ... });
```

## Exports

| Path | Purpose |
|---|---|
| SVG / Mermaid | Docs, scorecards, screenshot-diff |
| Cytoscape mount | Interactive IDE webview (P8) |
| Interaction grammar | Bounded gestures → local/server requests |

See `docs/architecture/VISUAL-ENCODING.md` and `OVERLAY-CATALOG.md`.
