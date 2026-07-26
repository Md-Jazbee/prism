# Contributor plugin guide — adding a language

**Status:** P5 Stage C (gate passed; plugin path ready)  
**ABI:** [`schemas/plugins/LanguageExtractor.md`](../../schemas/plugins/LanguageExtractor.md)  
**Registry:** `crates/prism-extract`  
**Goal:** Add a language **without changing** the store, planner, or MCP core.

---

## Path (native first-party today)

1. **Read the ABI** — pure transform: `(path, bytes) → FactBundle`. No network, no FS writes, no process spawn.  
2. **Create crate** `crates/prism-extract-<lang>/` implementing `extract(path, bytes) -> FactBundle`.  
3. **Register** in `prism-extract`:
   - `detect_language` extension map  
   - `LanguageExtractor` impl + `extract_file` dispatch arm  
4. **Golden fixtures** under `fixtures/languages/<lang>/`:
   - source snippet  
   - `expected.json` (`FactBundle` after `normalize()`)  
5. **Unit test** in the extractor crate: extract → assert eq golden.  
6. **Run conformance:** `scripts/plugins/conformance-check.sh`  
7. **Docs:** `docs/architecture/extractors/<lang>.md` (analyzer string, limitations).  
8. **PR checklist:** ABI version unchanged (or major bump + migration note); CI green.

---

## What you must **not** touch

| Area | Why |
|---|---|
| `prism-store` schema | Core engine; facts already typed |
| Planner / recipes | Language-agnostic |
| MCP allowlist | No new tools required for T1 extract |

Optional later: PreciseImporter / SemanticBackend plugins — separate cards under `schemas/plugins/`.

---

## WASM path (deferred — ADR-0001)

`prism-plugin-host` (wasmtime WIT) is the long-term sandbox. **It is not built.** Until that crate ships, first-party languages land as native Rust crates that honor the same ABI. External contributors can still prototype against the ABI + goldens; hosting moves to WASM without changing Fact IR.

Do **not** claim the WASM host is proven until an example plugin runs under wasmtime in-tree.

---

## Conformance suite

```bash
./scripts/plugins/conformance-check.sh
```

Runs extractor golden tests for every registered language. CI invokes the same script via `cargo test -p prism-extract-python -p prism-extract-rust` (and new crates as added).

See [TEST-MATRIX.md](../architecture/TEST-MATRIX.md).
