# Prism — Repository Intelligence Platform

Open-source **developer intelligence** that understands a repository *before* an LLM sees it.

**Phase:** 0 — Foundations (workspace identity, hashing, schemas, eval skeleton)

## Docs

| Document | Role |
|---|---|
| [Architecture Design Document](docs/architecture/ARCHITECTURE-DESIGN-DOCUMENT.md) | What & why |
| [Tech Stack & Project Structure](docs/architecture/TECH-STACK-AND-PROJECT-STRUCTURE.md) | How it is built |
| [Planning & Implementation](docs/planning/PLANNING-AND-IMPLEMENTATION.md) | Phases & gates |
| [Tasks & Progress](docs/planning/TASKS-AND-PROGRESS.md) | Living checklist + progress board |
| [Fingerprint algorithm](docs/architecture/FINGERPRINT.md) | XXH3 + Merkle |
| [Ignore policy checklist](docs/architecture/IGNORE-POLICY-CHECKLIST.md) | Stage A review |

## Quick start

```bash
# Rust toolchain (rust-toolchain.toml pins stable)
cargo build -p prism-cli
cargo test --workspace

# Workspace identity
cargo run -p prism-cli -- doctor .

# Incremental index stub (discover → hash → parse-hook → txn → invalidate)
cargo run -p prism-cli -- index . --dry-run
cargo run -p prism-cli -- index .

# Eval gold-pack smoke
cd eval && uv sync && uv run prism-eval smoke
```

## Workspace crates (P0)

| Crate | Owns |
|---|---|
| `prism-cli` | `prism` binary |
| `prism-core` | Workspace manager, fingerprint, incremental path |
| `prism-store` | `meta.sqlite`, `KgStore` + SQLite stub |
| `prism-ir` | IDs, confidence, schema versions |
| `prism-obs` | Index metrics events |

## License

MIT (planned) — `LICENSE` to be added at public release.
