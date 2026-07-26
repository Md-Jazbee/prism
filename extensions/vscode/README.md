# Prism VS Code / Cursor extension

Thin host for the Rust engine. **One command to get ready:** `Prism: Setup Workspace`.

## Install (Cursor)

```bash
cargo build -p prism-cli
pnpm --filter @prism/graph-view build
pnpm --filter prism-vscode package
cursor --install-extension extensions/vscode/prism-vscode-0.8.1.vsix --force
```

Reload Cursor → Command Palette → **Prism: Setup Workspace**.

That indexes the repo, writes AGENTS.md / Cursor rules, registers MCP, and starts/attaches the daemon.

See `docs/architecture/PRODUCT-SETUP.md`.

## Develop

```bash
pnpm --filter prism-vscode build
pnpm --filter prism-vscode test
```
