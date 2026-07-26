# Marketplace listing copy (P8 draft)

**Name:** Prism  
**Short:** Budgeted repository Evidence Packs and graph orientation inside the editor.

**Full description (draft):**

Prism brings compile-first repository intelligence to VS Code and Cursor: index locally, orient with a budgeted graph view, and compile cited Evidence Packs without leaving the editor. The extension is a thin host — analysis runs in the local `prism` / `prismd` engine. Repository content is never uploaded by the extension.

**Capabilities**

- Compile Context → Evidence Pack panel with citations and EXPLAIN
- Graph panel (architecture / impact / slice views via graph-view/v1)
- Impact, slice, repo map, entrypoints commands
- Optional Cursor MCP auto-registration (visible + disableable)

**Limitations (honest)**

- Heuristic (T1) edges may appear; precise refactor claims need T2 PreciseIndex / SCIP
- Language coverage: Python + Rust primary (see language re-baseline ADR)
- Eval claims: see public interim scorecards; four-arm LLM benchmark is Phase 9
- Binary delivery: PATH / workspace build first; download-on-demand when manifest is populated (ADR-0006)
- Decorations off by default

**Screenshots:** use deterministic SVG exports from `fixtures/views/screenshots/` (architecture_map, impact_cone, slice_path).
