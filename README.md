# Prism — Repository Intelligence Platform

Open-source **developer intelligence** that understands a repository *before* an LLM sees it.

Prism indexes your codebase into a knowledge graph, then compiles **Evidence Packs** — minimum-sufficient, provenance-bearing context under a token budget — so agents call `compile_context` once instead of grepping their way through your repo. Local-first: indexing never requires a network or an API key.

**Phase:** 11 — Install & Distribution (Stage A+B complete; cold-VM gate pending first public release). P0–P7 + P9 gated · P8 cut ([ADR-0007](docs/architecture/adr/0007-extension-cut-cli-mcp.md)) · P10 deferred.

---

## Install

No Rust toolchain required. Installers verify SHA-256 checksums against the release `SHA256SUMS` and **fail closed** on mismatch.

### macOS / Linux

```bash
export PRISM_GITHUB_REPO=Md-Jazbee/prism
curl -fsSL "https://raw.githubusercontent.com/$PRISM_GITHUB_REPO/main/scripts/install.sh" | bash
```

Installs to `~/.local/bin/prism`. Add it to `PATH` if the installer says so.

### Windows (PowerShell)

```powershell
$env:PRISM_GITHUB_REPO = "Md-Jazbee/prism"
irm "https://raw.githubusercontent.com/$env:PRISM_GITHUB_REPO/main/scripts/install.ps1" | iex
```

Installs to `%LOCALAPPDATA%\Prism\bin\prism.exe`.

### Installer options

```bash
./scripts/install.sh --version 0.0.1     # pin a release
./scripts/install.sh --bin-dir ~/bin     # custom destination
./scripts/install.sh --dry-run           # print actions, write nothing
./scripts/install.sh --uninstall         # remove the binary
```

| Env | Purpose |
|---|---|
| `PRISM_GITHUB_REPO` | `owner/repo` that publishes releases |
| `PRISM_VERSION` | Version to install (without leading `v`) |
| `PRISM_BIN_DIR` | Install directory |
| `PRISM_DOWNLOAD_BASE` | Mirror or `file://` directory serving archives + `SHA256SUMS` (air-gapped installs; requires an explicit version) |

### Package managers (draft)

Manifests exist but need sha256 values from a published release: [`packaging/homebrew/prism.rb`](packaging/homebrew/prism.rb), [`packaging/scoop/prism.json`](packaging/scoop/prism.json).

### Upgrade

```bash
prism self-update                  # latest release
prism self-update --version 0.0.2
prism self-update --dry-run
```

> **Note:** releases are published by [`.github/workflows/release.yml`](.github/workflows/release.yml) on `v*` tags. Until the first tag exists, use the from-source path below.

---

## Bootstrap a workspace

```bash
cd your-repo
prism setup .            # index + AGENTS.md/rules/skills + MCP registration
prism doctor --ready     # binary, index, hosts, hook readiness
```

`prism setup` is one shot: it builds `.prism/graph.sqlite`, generates agent assets from the workflow catalog, and merges Prism into `.cursor/mcp.json` (or `.vscode/mcp.json`).

### Register other agent hosts

```bash
prism host install cursor      # .cursor/mcp.json
prism host install vscode      # .vscode/mcp.json
prism host install claude      # CLAUDE.md section + portable snippet
prism host install generic     # .mcp.prism.json stdio snippet
prism host status --json
```

Merges are idempotent and never clobber unrelated MCP servers. See [HOST-ADAPTERS.md](docs/architecture/HOST-ADAPTERS.md).

### Optional: re-index after every commit

```bash
prism hook install       # append-only post-commit hook
prism hook status
```

### First query

```bash
prism compile "How does indexing work?" .
prism workflow run onboarding .
```

Or let your agent call MCP `compile_context` directly. Full runbook: [BOOTSTRAP.md](docs/architecture/BOOTSTRAP.md) · [PRODUCT-SETUP.md](docs/architecture/PRODUCT-SETUP.md).

---

## Build from source (contributors)

```bash
cargo build -p prism-cli --release
./target/release/prism setup .
./target/release/prism doctor --ready

cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
./scripts/install-smoke.sh          # P11 install path smoke

cd eval && uv sync && uv run prism-eval smoke
uv run prism-eval p5-scorecard
```

---

## CLI surface

| Command | Purpose |
|---|---|
| `prism setup` / `doctor` | One-shot bootstrap + readiness |
| `prism index` / `index-status` | Incremental index + freshness |
| `prism compile` | Evidence Pack under a token budget |
| `prism query` | resolve · neighbors · impact · repo-map |
| `prism mcp` | MCP stdio server for agents |
| `prism daemon` | Optional local HTTP/SSE accelerator |
| `prism lsp` | LSP surface (augments, not replaces, language servers) |
| `prism view` | Graph View-Model projection |
| `prism workflow` / `agent` | Named workflows + generated assets |
| `prism host` / `hook` / `self-update` | Install & distribution (P11) |

## Docs

| Document | Role |
|---|---|
| [Architecture Design Document](docs/architecture/ARCHITECTURE-DESIGN-DOCUMENT.md) | What & why |
| [Tech Stack & Project Structure](docs/architecture/TECH-STACK-AND-PROJECT-STRUCTURE.md) | How it is built |
| [Planning & Implementation](docs/planning/PLANNING-AND-IMPLEMENTATION.md) | Phases & gates |
| [Tasks & Progress](docs/planning/TASKS-AND-PROGRESS.md) | Living checklist + progress board |
| [Product setup](docs/architecture/PRODUCT-SETUP.md) | Install & bootstrap (canonical) |
| [Release artifacts](docs/architecture/RELEASE-ARTIFACTS.md) | Archive / checksum contract |
| [Public benchmark report v2](docs/eval/PUBLIC-BENCHMARK-REPORT-V2.md) | Four-arm methods, caveats, reproducibility |
| [Plugin guide](docs/contributing/plugin-guide.md) | Add a language via ABI + goldens |
| [MCP tool catalog](docs/architecture/MCP-TOOL-CATALOG.md) | Agent tool surface |
| [Agent usage](docs/architecture/AGENT-USAGE.md) | Prefer structural tools |
| [KG query API](docs/architecture/KG-QUERY-API.md) | resolve / neighbors / impact |

## Workspace crates

| Crate | Owns |
|---|---|
| `prism-cli` | `prism` binary (setup, host, hook, self-update) |
| `prism-core` | Workspace manager, fingerprint, incremental path |
| `prism-store` | meta/graph sqlite, query API, communities |
| `prism-ir` | IDs, confidence, fact IR, schema versions |
| `prism-obs` | Index / query metrics events |
| `prism-extract` | LanguageExtractor ABI + dispatch |
| `prism-extract-python` | tree-sitter Python T1 |
| `prism-extract-rust` | tree-sitter Rust T1 |
| `prism-precise` | Precise tier (T2) SCIP/LSP overlays |
| `prism-semantic` | CFG/DFG/CPG shards + slicing (T3/T4) |
| `prism-plan` | Intent recipes + query plan DAG |
| `prism-compile` | Evidence Pack selection, budgets, EXPLAIN |
| `prism-mcp` | MCP structural tools (stdio) |
| `prism-api` / `prism-daemon` | HTTP/SSE surface + `prismd` lifecycle |
| `prism-view` | Graph View-Model projection |
| `prism-lsp` | LSP host |
| `prism-agent` | Workflows, refusal repair, traces, asset generation |
| `prism-bench` | N1/N2 criterion benches |

## License

MIT — see [LICENSE](LICENSE).
