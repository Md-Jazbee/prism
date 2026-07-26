# ADR-0007: VS Code / Cursor extension cut — CLI + MCP are the product surface

**Date:** 2026-07-26  
**Status:** Accepted  
**Supersedes:** ADR-0006 (extension binary delivery)  
**Related:** P8 (retired)

## Context

Phase 8 shipped `extensions/vscode` as a thin host over `prism` / `prismd`. In practice the Cursor **user-level MCP** (`prism mcp`) plus CLI (`prism setup`, `compile`, `query`, …) already deliver the same intelligence path. The extension added a second language ecosystem (pnpm/esbuild/vsce), CI, and docs without unique value beyond an interactive webview that `@prism/graph-view` can serve as SVG/Mermaid outside the IDE.

## Decision

1. **Remove** `extensions/vscode`, `.github/workflows/extension.yml`, EXTENSION-* architecture docs, ADR-0006, and the P8 phase-gate scorecard from the repository.
2. **Keep** `@prism/graph-view`, `prism setup` / `doctor --ready`, daemon `.prism/daemon.token` persistence, MCP stdio, and HTTP/SSE daemon.
3. **Retire P8** in planning: marked *cut — superseded by CLI/MCP*; agent-facing install path is `prism setup` + MCP registration (see PRODUCT-SETUP.md).
4. IDE peek/panels remain **design-only** in IDE-INTEGRATION.md until a future product decision reopens an extension.

## Consequences

- One installable story for agents: CLI binary + MCP (Graphify-like `prism setup`).
- No Marketplace VSIX in-tree; no extension activation/CI burden.
- G-14 / R8 restated: no IDE extension *by choice*, not by gap.
