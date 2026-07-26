# P7 phase gate scorecard

**Date:** 2026-07-26  
**Phase:** Visual Repository Intelligence  
**Status:** **PASS** (Stage A–C complete; time-to-orient protocol specified — human timing deferred to P8/P9 lab runs)

| Gate item | Result | Evidence |
|---|---|---|
| Projection / LOD / layout / aggregation docs | ✅ | `PROJECTION-OPERATORS`, `LOD-POLICY`, `LAYOUT-SELECTION-MATRIX`, `AGGREGATION-SEMANTICS` |
| Time-to-orient task set | ✅ | [`eval/tasks/time-to-orient.md`](../tasks/time-to-orient.md) |
| Renderer consumes only graph-view/v1 | ✅ | `@prism/graph-view` — no store imports |
| Interaction grammar bounded | ✅ | `gestureToRequest` + docs |
| Visual encoding + legend + a11y labels | ✅ | `encode/style.ts`, SVG legend |
| Overlay catalog (7 kinds) + fixtures | ✅ | `fixtures/views/golden/` |
| Visual EXPLAIN for pack drops | ✅ | `visualExplain` + pack_map golden |
| Budget adherence on fixtures | ✅ | `assertBudgetOk` in vitest |
| Screenshot-diff suite green | ✅ | `fixtures/views/screenshots/*.svg` |
| Heuristic ≠ authoritative styling | ✅ | dashed vs solid encoding |

## Commands

```bash
pnpm install
pnpm --filter @prism/graph-view test
# refresh baselines after intentional layout changes:
UPDATE_SHOTS=1 pnpm --filter @prism/graph-view test
```

## Time-to-orient (lab)

Protocol is ready. **Numeric human deltas** are not fabricated here; run the task set when a webview/host is available (P8) and paste medians into this table.

| Task | Text median (s) | Visual median (s) | Δ | Notes |
|---|---:|---:|---:|---|
| TTO-1…TTO-6 | — | — | — | Pending lab run |
