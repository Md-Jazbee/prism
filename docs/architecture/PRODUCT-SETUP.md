# Product setup — Graphify-like one-shot (CLI + MCP)

**Goal:** One installable path that takes a cold workspace to indexed + agent-ready without an IDE extension.

**Decision:** The VS Code / Cursor extension was **removed** ([ADR-0007](./adr/0007-extension-cut-cli-mcp.md)). Product surface is **CLI + MCP**.

## Surfaces

| Surface | Command |
|---|---|
| Setup | `prism setup .` |
| Readiness | `prism doctor --ready` / `--json` |
| Agent tools | `prism mcp .` (stdio) — register in Cursor MCP settings |
| HTTP accelerator | `prism daemon .` (optional) |

## What `prism setup` does

1. **Binary** — assumes `prism` is already running.
2. **Index** — builds/refreshes `.prism/graph.sqlite`.
3. **Assets** — `AGENTS.md`, `.cursor/rules/prism-compile-first.mdc`, `.prism/agent/skills.md` from the workflow catalog.
4. **MCP** — merges Prism into `.cursor/mcp.json` (or `.vscode/mcp.json`).

## Install for Cursor (agent)

```bash
cargo build -p prism-cli
./target/debug/prism setup .
./target/debug/prism doctor --ready
# Cursor picks up .cursor/mcp.json, or register user-level:
#   command: <path-to-prism>   args: ["mcp", "<workspace>"]
```

## Renderer (no IDE host)

`@prism/graph-view` remains for SVG/Mermaid export and tests. There is no in-editor webview host in-tree.

## Honest limits

- No Marketplace VSIX; binary must be on PATH or built from this repo.
- Interactive graph-in-panel UX is out of scope until a future product decision.
