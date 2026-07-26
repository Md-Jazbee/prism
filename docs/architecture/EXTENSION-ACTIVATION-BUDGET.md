# Extension activation & performance budget (P8)

## Budget (exit criterion)

| Metric | Budget | Notes |
|---|---|---|
| Extension host activation wall time | **≤ 150 ms** p95 on cold workspace open (no index build) | Measured from `activate()` entry to first `resolve` of activation promise |
| Work on idle start | Contribution registration only | No daemon spawn, no index, no network |
| First command / panel open | May start daemon + health check | Counted separately from activation |
| Daemon spawn to `/health` ok | **≤ 3 s** p95 (warm binary on PATH) | Excludes first-run index |

## Activation events

```json
"activationEvents": [
  "onStartupFinished",
  "workspaceContains:.prism/**",
  "onView:prism.evidence",
  "onView:prism.graph",
  "onCommand:prism.compileContext"
]
```

Prefer `onStartupFinished` over `*` so editor chrome paints first. Heavy work is deferred until a Prism command, view, or explicit first-run action.

## Measurement

Unit tests assert activation does not call transport until a command runs. Manual / CI note: record activation ms in the P8 scorecard when running `@vscode/test-electron` (deferred in this pass; protocol ready).
