# Plugin ABI draft (P0 Stage B) — LanguageExtractor

Status: **draft for contributors** — not implemented as WASM host until later phases.
First-party P1 extractors are **native Rust** crates; this ABI describes the contract both must honor.

## Pure-transform rule

`LanguageExtractor` is a pure function of file bytes (+ language config) → versioned facts.
No network, no arbitrary filesystem writes, no process spawn.

## Input

| Field | Type | Notes |
|---|---|---|
| `path` | string | Repo-relative path |
| `bytes` | bytes | File contents |
| `language` | string | Declared or detected |
| `schema_version` | semver | Must match `FACT_SCHEMA_VERSION` major |

## Output

| Field | Type | Notes |
|---|---|---|
| `nodes` | FactNode[] | T1 kinds only in P1 |
| `edges` | FactEdge[] | Confidence required |
| `analyzer` | string | e.g. `tree-sitter-python@0.x` |
| `tier` | `T1` | Higher tiers via other backends |

## Confidence enums

`extracted` | `heuristic` | `precise` | `observed`

## Golden fixtures

Every language crate ships fixtures under `fixtures/languages/<lang>/` with expected facts.
CI compares extractor output to golden JSON (P1).

## Versioning

Breaking fact shape → bump major in `schemas/fact-schema/` and `prism-ir`.
Plugins declaring incompatible majors are rejected at load time.
