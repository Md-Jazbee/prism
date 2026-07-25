# Prism — Repository Intelligence Platform

Open-source **developer intelligence** that understands a repository *before* an LLM sees it.

**Phase:** 1 — Syntactic KG + MCP · **Stage B** (KG query API)

## Docs

| Document | Role |
|---|---|
| [Architecture Design Document](docs/architecture/ARCHITECTURE-DESIGN-DOCUMENT.md) | What & why |
| [Tech Stack & Project Structure](docs/architecture/TECH-STACK-AND-PROJECT-STRUCTURE.md) | How it is built |
| [Planning & Implementation](docs/planning/PLANNING-AND-IMPLEMENTATION.md) | Phases & gates |
| [Tasks & Progress](docs/planning/TASKS-AND-PROGRESS.md) | Living checklist + progress board |
| [Fingerprint algorithm](docs/architecture/FINGERPRINT.md) | XXH3 + Merkle |
| [Ignore policy checklist](docs/architecture/IGNORE-POLICY-CHECKLIST.md) | Stage A review |
| [Python extractor](docs/architecture/extractors/python.md) | T1 Python design |
| [Rust extractor](docs/architecture/extractors/rust.md) | T1 Rust design |
| [KG query API](docs/architecture/KG-QUERY-API.md) | resolve / neighbors / impact |
| [Incremental update](docs/architecture/INCREMENTAL-UPDATE.md) | Single-file edit path |

## Quick start

```bash
# Rust toolchain (rust-toolchain.toml pins stable)
cargo build -p prism-cli
cargo test --workspace

# Workspace identity
cargo run -p prism-cli -- doctor .

# Incremental index (discover → hash → T1 extract → txn → invalidate)
cargo run -p prism-cli -- index . --dry-run
cargo run -p prism-cli -- index .
cargo run -p prism-cli -- index-status .

# Structural queries (after index)
cargo run -p prism-cli -- query resolve helper .
cargo run -p prism-cli -- query neighbors '<node-id>' .
cargo run -p prism-cli -- query impact '<node-id>' --depth 2 .

# Eval gold-pack smoke
cd eval && uv sync && uv run prism-eval smoke
```

## Workspace crates

| Crate | Owns |
|---|---|
| `prism-cli` | `prism` binary |
| `prism-core` | Workspace manager, fingerprint, incremental path |
| `prism-store` | `meta.sqlite`, `KgStore` + fact insert + query API |
| `prism-ir` | IDs, confidence, fact IR, schema versions |
| `prism-obs` | Index / extract metrics events |
| `prism-extract` | LanguageExtractor ABI + dispatch |
| `prism-extract-python` | tree-sitter Python T1 |
| `prism-extract-rust` | tree-sitter Rust T1 |

## License

MIT (planned) — `LICENSE` to be added at public release.
