# Rust T1 extractor

**Crate:** `prism-extract-rust`  
**Analyzer:** `tree-sitter-rust@0.23`  
**Tier:** T1 (syntactic)  
**ABI:** [LanguageExtractor.md](../../../schemas/plugins/LanguageExtractor.md) (frozen P1 Stage A)

## What is extracted

| Fact | Confidence | Notes |
|---|---|---|
| `File` node | `extracted` | One per path |
| `fn` / `struct` / `enum` / `trait` / `mod` → `Symbol` | `extracted` | `DEFINES` + `CONTAINS` |
| `impl` methods → `Symbol` (`symbol_kind: method`) | `extracted` | Detected via parent `impl_item` |
| `use` → `IMPORTS` | `extracted` | Target `module:{path}`; cross-crate resolve is Stage B+ |
| `call_expression` → `CALLS` | `heuristic` | Same-file name match; else `unresolved:{name}` |

## Resolution-cheap policy

- Only same-file symbol names resolve `CALLS`.
- Unresolved callees are **first-class** (`unresolved:…`).
- Scoped / field calls use the rightmost identifier (`foo::bar` / `self.baz` → `bar` / `baz`).

## Known failure modes (not claimed precise)

- Macros (`println!`, custom macros expanding to calls)
- Trait method resolution / UFCS across crates
- Generics and associated functions on type parameters
- `dyn Trait` / object-safe dispatch
- Re-exports and `pub use` aliasing beyond the `use` path string
- Methods resolved only via trait impls in other modules

## Golden fixture

`fixtures/languages/rust/simple_mod.rs` ↔ `expected.json`
