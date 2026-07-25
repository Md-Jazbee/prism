# Plugin ABI — LanguageExtractor (frozen for P1 Stage A)

Status: **frozen for P1 Stage A** (2026-07-25)  
Fact schema: `FACT_SCHEMA_VERSION` = `0.0.1` (`schemas/fact-schema/v0`)  
First-party P1 extractors are **native Rust** crates; this ABI describes the contract both must honor.

Breaking fact shape → bump major in `schemas/fact-schema/` and `prism-ir::FACT_SCHEMA_VERSION`.
Plugins declaring incompatible majors are rejected at load time.

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

## Output (`FactBundle`)

| Field | Type | Notes |
|---|---|---|
| `schema_version` | string | Echo of `FACT_SCHEMA_VERSION` |
| `nodes` | FactNode[] | T1 kinds only in P1 |
| `edges` | FactEdge[] | Confidence required on every edge |
| `analyzer` | string | e.g. `tree-sitter-python@0.23` |
| `tier` | `T1` | Higher tiers via other backends |
| `language` | string | `python` / `rust` / … |
| `path` | string | Same as input path |

## Confidence enums

`extracted` | `heuristic` | `precise` | `observed`

## Resolution-cheap policy (P1 Stage A)

- Same-file name match for `CALLS` → `confidence: heuristic`, `dst` = local symbol id
- Otherwise emit `CALLS` to `unresolved:{name}` — **first-class**, never silently dropped
- Imports → `IMPORTS` with `extracted` confidence (module path as string attrs; cross-file resolve is Stage B+)
- Do **not** claim precise: dynamic imports, macros, trait method resolution, generics, `getattr` / `__import__`

## Node / edge kinds (T1)

**Nodes:** `File`, `Symbol`, `Module`, `Package`  
**Edges:** `CONTAINS`, `IMPORTS`, `CALLS`, `DEFINES`, `REFERENCES`, plus best-effort `EXTENDS` / `IMPLEMENTS`

## Golden fixtures

Every language crate ships fixtures under `fixtures/languages/<lang>/` with expected facts.
CI compares extractor output to golden JSON (normalized: sort nodes/edges by `id`).

## Versioning

| Change | Action |
|---|---|
| Add optional attrs | Minor / docs only |
| Rename/remove required field or kind | Bump fact schema major |
| New language crate | Same major; new analyzer string |
