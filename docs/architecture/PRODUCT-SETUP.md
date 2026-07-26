# Product setup — Graphify-like one-shot (P8/P9)

**Goal:** One installable path that takes a cold workspace to indexed + agent-ready without hunting for docs.

## Surfaces

| Surface | Command |
|---|---|
| CLI | `prism setup .` |
| IDE | **Prism: Setup Workspace** (`prism.setupWorkspace`) |
| Readiness | `prism doctor --ready` / `--json` |

## What `prism setup` does

1. **Binary** — assumes `prism` is already running (CLI). Extension may `cargo build -p prism-cli` when opened on this source tree, or use PATH / `prism.binaryPath`.
2. **Index** — `prism index` if `.prism/graph.sqlite` missing or refresh via indexer.
3. **Assets** — `prism agent generate-assets` → `AGENTS.md`, `.cursor/rules/prism-compile-first.mdc`, `.prism/agent/skills.md` (catalog is the single source of truth).
4. **MCP** — merge Prism into `.cursor/mcp.json` (or `.vscode/mcp.json`).
5. **Daemon (extension only)** — spawn/attach `prism daemon` with token written to `.prism/daemon.token`.

## Install Prism into Cursor

```bash
# From this repo (dev)
cargo build -p prism-cli
pnpm --filter @prism/graph-view build
pnpm --filter prism-vscode package
cursor --install-extension extensions/vscode/prism-vscode-*.vsix --force

# Then in Cursor: Command Palette → "Prism: Setup Workspace"
# Or CLI:
./target/debug/prism setup .
./target/debug/prism doctor --ready
```

## Honest limits

- Marketplace VSIX does **not** yet embed platform Rust binaries (ADR-0006). Cold machines without PATH/`cargo` need a binary first.
- Download-on-demand activates when `prism.downloadBaseUrl` + `binaries/manifest.json` are populated.
- Human time-to-orient lab numbers remain open; protocol lives under eval tasks.

## Failure recovery

| Symptom | Fix |
|---|---|
| UNAUTHORIZED / sticky daemon | Delete `.prism/daemon.lock` + restart via Setup (token now persisted in `.prism/daemon.token`) |
| No binary | `cargo build -p prism-cli` or set `prism.binaryPath` |
| Empty panels | Run Setup, then Repo Map / Compile Context |
