# Python T1 extractor

**Crate:** `prism-extract-python`  
**Analyzer:** `tree-sitter-python@0.23`  
**Tier:** T1 (syntactic)  
**ABI:** [LanguageExtractor.md](../../../schemas/plugins/LanguageExtractor.md) (frozen P1 Stage A)

## What is extracted

| Fact | Confidence | Notes |
|---|---|---|
| `File` node | `extracted` | One per path |
| `function_definition` / `class_definition` → `Symbol` | `extracted` | `DEFINES` + `CONTAINS` from file |
| `import` / `from … import` → `IMPORTS` | `extracted` | Target is `module:{name}`; cross-file resolve is Stage B+ |
| `call` → `CALLS` | `heuristic` | Same-file name match; else `unresolved:{name}` |
| Class bases → `EXTENDS` | `heuristic` | Best-effort identifier / attribute |

## Resolution-cheap policy

- Only same-file symbol names resolve `CALLS`.
- Unresolved callees are **first-class** nodes (`unresolved:…`), never silently dropped.
- Attribute calls use the rightmost name (`obj.method` → `method`).

## Known failure modes (not claimed precise)

- Dynamic imports (`__import__`, `importlib`, string-built module paths)
- `getattr` / monkey-patched callables
- Decorators that rewrite call sites
- Relative imports beyond recording the module string
- Methods resolved across modules or via MRO
- Generics / typing constructs that are not defs

## Golden fixture

`fixtures/languages/python/simple_module.py` ↔ `expected.json`
