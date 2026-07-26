# Prism — Repository Intelligence Platform

Open-source **developer intelligence** that understands a repository *before* an LLM sees it.

**Phase:** 5 — Repo Intelligence + Hardening · **gate passed** (token proxies reconfirmed; LLM quality / ≥70% precision honest interim)

## Docs

| Document | Role |
|---|---|
| [Architecture Design Document](docs/architecture/ARCHITECTURE-DESIGN-DOCUMENT.md) | What & why |
| [Tech Stack & Project Structure](docs/architecture/TECH-STACK-AND-PROJECT-STRUCTURE.md) | How it is built |
| [Planning & Implementation](docs/planning/PLANNING-AND-IMPLEMENTATION.md) | Phases & gates |
| [Tasks & Progress](docs/planning/TASKS-AND-PROGRESS.md) | Living checklist + progress board |
| [Public benchmark report](docs/eval/PUBLIC-BENCHMARK-REPORT.md) | Methods, caveats, reproducibility |
| [Plugin guide](docs/contributing/plugin-guide.md) | Add a language via ABI + goldens |
| [MCP tool catalog](docs/architecture/MCP-TOOL-CATALOG.md) | Stage C tools |
| [Agent usage](docs/architecture/AGENT-USAGE.md) | Prefer structural tools |
| [KG query API](docs/architecture/KG-QUERY-API.md) | resolve / neighbors / impact |

## Quick start

```bash
cargo build -p prism-cli
cargo test --workspace

cargo run -p prism-cli -- doctor .
cargo run -p prism-cli -- index .
cargo run -p prism-cli -- index-status .
cargo run -p prism-cli -- query resolve helper .
cargo run -p prism-cli -- query repo-map .

# MCP stdio server (configure in your agent client)
cargo run -p prism-cli -- mcp .

cd eval && uv sync && uv run prism-eval smoke
uv run prism-eval p1-scorecard
uv run prism-eval p5-scorecard
```

## Workspace crates

| Crate | Owns |
|---|---|
| `prism-cli` | `prism` binary |
| `prism-core` | Workspace manager, fingerprint, incremental path |
| `prism-store` | meta/graph sqlite, query API, communities |
| `prism-ir` | IDs, confidence, fact IR, schema versions |
| `prism-obs` | Index / query metrics events |
| `prism-extract` | LanguageExtractor ABI + dispatch |
| `prism-extract-python` | tree-sitter Python T1 |
| `prism-extract-rust` | tree-sitter Rust T1 |
| `prism-mcp` | MCP structural tools (stdio) |

## License

MIT (planned) — `LICENSE` to be added at public release.
