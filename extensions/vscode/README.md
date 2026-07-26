# Prism VS Code / Cursor extension (P8)

Thin host for daemon HTTP / CLI fallback. Analysis stays in Rust.

## Develop

```bash
# from repo root
pnpm install
pnpm --filter @prism/graph-view build
pnpm --filter prism-vscode build
pnpm --filter prism-vscode test
```

Press F5 with an Extension Development Host pointing at `extensions/vscode`, or:

```bash
code --extensionDevelopmentPath=extensions/vscode .
```

Requires a `prism` binary on PATH or `cargo build -p prism-cli`.

## Commands

Palette: Compile Context, Impact, Slice, Explain, Repo Map, Entrypoints, Build Index, MCP enable/disable, Generate AGENTS.md.

Activity bar: Evidence panel + Graph panel (`@prism/graph-view`).

## Docs (in-repo)

- `docs/architecture/EXTENSION-ARCHITECTURE.md`
- `docs/architecture/adr/0006-extension-binary-delivery.md`
- `docs/architecture/IDE-INTEGRATION.md`

## Package VSIX

```bash
pnpm --filter prism-vscode package
```
