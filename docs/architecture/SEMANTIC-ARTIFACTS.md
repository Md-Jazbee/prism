# Semantic artifact layout

**Phase:** P4 Stage A  
**Root:** `.prism/semantic/` (never mixed into hot `graph.sqlite` adjacency)

---

## Layout

```text
.prism/semantic/
  manifest.json                 # algo version, languages, counts
  by-file/
    <path-hash>__<safe_name>.json   # FunctionCfgDfg bundle for one source file
  shards/
    <shard_id>.json                 # T4 call-graph neighborhood + overlay edges
  memo/
    <memo_key>.json                 # memoized Slice results
  providers/                        # optional sink/source caches
    <id>.json
```

`path-hash` = first 16 hex of XXH3-128 of repo-relative path (stable key).  
`safe_name` = path with `/` → `__` (debug aid only).

---

## Manifest (`manifest.json`)

```json
{
  "schema_version": "0.0.1",
  "algo_version": "t3-python-cfgdfg@0.0.1",
  "language": "python",
  "files": 1,
  "functions": 2,
  "built_at": "unix:…",
  "tree_fingerprint": "…"
}
```

---

## Per-file artifact

Matches `schemas/semantic-artifact/v0`. Includes:

- `path`, `content_hash` (file bytes)  
- `functions[]` with CFG blocks/edges + DFG defs/uses/deps  
- `notes[]` for partial analysis / crash-policy soft failures  

Invalidation: rebuild when `content_hash` ≠ current file hash (Stage A: CLI rebuild; indexer hook later).

---

## Why separate from KG

| Store | Role |
|---|---|
| `graph.sqlite` | Hot T1/T2 structural queries |
| `.prism/semantic/` | Cold/on-demand T3 shards; larger, language-specific |

Orchestrator must **never** require whole-repo semantic build for `compile_context` T1/T2 paths.
