# P8 phase gate scorecard

**Date:** 2026-07-26  
**Phase:** IDE Extension (VS Code / Cursor)  
**Status:** **PASS** (Stages A–C implemented; Marketplace publish + human TTO lab deferred to ops / P9)

| Gate item | Result | Evidence |
|---|---|---|
| Extension architecture + activation budget | ✅ | `EXTENSION-ARCHITECTURE.md`, `EXTENSION-ACTIVATION-BUDGET.md` |
| Binary delivery ADR | ✅ | ADR-0006 (PATH → workspace → download) |
| First-run onboarding | ✅ | `EXTENSION-ONBOARDING.md` + `maybeFirstRun` |
| Daemon HTTP → CLI fallback | ✅ | `extensions/vscode/src/transport/client.ts` |
| Command set (compile/peek/impact/slice/explain/repoMap/entrypoints) | ✅ | `package.json` contributes + `commands.ts` |
| Evidence + Graph panels | ✅ | webview + `@prism/graph-view` |
| Decorations policy (off by default) | ✅ | `EXTENSION-DECORATION-POLICY.md` |
| Cursor MCP auto-reg (visible/disableable) | ✅ | `agent/assets.ts` → `.cursor/mcp.json` |
| Generated AGENTS.md / rules | ✅ | `generateAgentsMd` |
| Actionable refusals | ✅ | `handleRefusal` |
| Marketplace listing copy (honest limits) | ✅ | `EXTENSION-MARKETPLACE.md` |
| Extension CI (typecheck/unit/build/VSIX) | ✅ | `.github/workflows/extension.yml` |
| Installable VSIX artifact | ✅ | `pnpm --filter prism-vscode package` |
| Cold repo → orientation → pack without terminal | ✅ protocol | Commands cover path; human lab timing still open (P7 TTO table) |
| `@vscode/test-electron` e2e | ⏸️ deferred | Vitest + mocked transport this pass (see plan defaults) |

## Commands

```bash
pnpm install
pnpm --filter @prism/graph-view build
pnpm --filter prism-vscode typecheck
pnpm --filter prism-vscode test
pnpm --filter prism-vscode build
pnpm --filter prism-vscode package
```

## Activation budget

Registration-only activate path; daemon deferred to first command. Measured wall time logged to Prism output channel; p95 ≤150ms is the exit criterion for electron runs (deferred harness).
