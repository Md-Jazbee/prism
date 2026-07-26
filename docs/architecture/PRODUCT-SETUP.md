# Product setup — cold machine → indexed + agent-ready

**Goal:** One installable path that takes a cold workspace to indexed + MCP-ready without an IDE extension.

**Decision:** The VS Code / Cursor extension was **removed** ([ADR-0007](./adr/0007-extension-cut-cli-mcp.md)). Product surface is **CLI + MCP**.

**Release contract:** [RELEASE-ARTIFACTS.md](./RELEASE-ARTIFACTS.md) (P11).

## End-user install (no Rust toolchain)

Set `PRISM_GITHUB_REPO` to the GitHub `owner/repo` that publishes releases (default in scripts: `example/prism` until the public org is wired).

### macOS / Linux

```bash
curl -fsSL "https://raw.githubusercontent.com/${PRISM_GITHUB_REPO:-example/prism}/main/scripts/install.sh" | bash
# or from a checkout:
#   ./scripts/install.sh --version 0.0.1
#   ./scripts/install.sh --dry-run
#   ./scripts/install.sh --uninstall
```

Installs to `~/.local/bin/prism` (override with `PRISM_BIN_DIR` / `--bin-dir`). Checksums are verified against release `SHA256SUMS`.

### Windows (PowerShell)

```powershell
$env:PRISM_GITHUB_REPO = "example/prism"   # set your org/repo
irm "https://raw.githubusercontent.com/$env:PRISM_GITHUB_REPO/main/scripts/install.ps1" | iex
# or: .\scripts\install.ps1 -Version 0.0.1
#     .\scripts\install.ps1 -DryRun
#     .\scripts\install.ps1 -Uninstall
```

Installs to `%LOCALAPPDATA%\Prism\bin\prism.exe`.

### Package managers (draft)

| OS | Path | Location |
|---|---|---|
| macOS / Linux | Homebrew formula draft | [`packaging/homebrew/prism.rb`](../../packaging/homebrew/prism.rb) |
| Windows | Scoop manifest draft | [`packaging/scoop/prism.json`](../../packaging/scoop/prism.json) |

Replace `REPLACE_WITH_RELEASE_SHA256` after the first tagged release from [`.github/workflows/release.yml`](../../.github/workflows/release.yml).

### Upgrade

```bash
prism self-update            # latest
prism self-update --version 0.0.2
prism self-update --dry-run
```

## Workspace bootstrap

| Surface | Command |
|---|---|
| Setup | `prism setup .` |
| Readiness | `prism doctor --ready` / `--json` |
| Host adapters | `prism host install cursor\|vscode\|claude\|generic` |
| Agent tools | `prism mcp .` (stdio) |
| HTTP accelerator | `prism daemon .` (optional) |

### What `prism setup` does

1. **Binary** — uses the running `prism` (must already be on PATH / installed).
2. **Index** — builds/refreshes `.prism/graph.sqlite`.
3. **Assets** — `AGENTS.md`, `.cursor/rules/prism-compile-first.mdc`, `.prism/agent/skills.md` from the workflow catalog.
4. **MCP** — merges Prism into `.cursor/mcp.json` (or `.vscode/mcp.json`).

```bash
prism setup .
prism doctor --ready
# or register an extra host:
prism host install claude
prism host status
```

### Agent ensure-install (Graphify-like)

When an agent skill needs Prism and `prism` is missing:

1. Run the platform installer above (or `prism self-update` once a binary exists).
2. Re-resolve `command -v prism` / absolute path.
3. `prism setup .` then MCP `compile_context`.
4. **Do not** ask for an API key for core indexing — local-first.

## Contributor build (from this repo)

```bash
cargo build -p prism-cli --release
./target/release/prism setup .
./target/release/prism doctor --ready
```

## Renderer (no IDE host)

`@prism/graph-view` remains for SVG/Mermaid export and tests. There is no in-editor webview host in-tree.

## Honest limits

- Releases require a real GitHub repo + tag; until then installers resolve against `PRISM_GITHUB_REPO`.
- Homebrew/Scoop are **drafts** until sha256s are filled from a release.
- Interactive graph-in-panel UX is out of scope (ADR-0007).
- Team/shared indexes are P10 (deferred).
