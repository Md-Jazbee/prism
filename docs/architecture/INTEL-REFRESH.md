# Intelligence refresh & invalidation (P5 Stage A)

**Status:** Locked  
**Artifacts:** optional cache under `.prism/intel/` (never hot adjacency)

---

## When to recompute

| Trigger | Action |
|---|---|
| `prism index` completes | Invalidate intel cache (tree fingerprint changed) |
| MCP/`repo_map` / `entrypoints` / `detect_changes` | Recompute on demand if cache missing or stale |
| File subgraph replace | Mark intel dirty; next read rebuilds |
| Ambiguity index | Always live query (cheap aggregate) — no cache required |

---

## Cache key

```text
cache_key = xxh3(tree_fingerprint | algo_version)
```

`algo_version` = `intel-v0` (bump when heuristics change).

Layout:

```text
.prism/intel/
  manifest.json   # fingerprint, algo, built_at
  report.json     # last RepoIntelReport
```

If `manifest.tree_fingerprint` ≠ current workspace fingerprint → delete and rebuild.

---

## Incremental policy

- Dirty file lists (`reverse_dep_files`) are **advisory** for planners.
- Stage A does **not** surgically patch communities; full recompute is fine at pilot scale.
- Hotspots from git are independent of KG dirty sets (history-based).

---

## Confidence notes

Every product must emit method + confidence strings in the report `notes[]` (see [REPO-INTELLIGENCE.md](./REPO-INTELLIGENCE.md)).
