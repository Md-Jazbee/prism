# T4 lazy sharding strategy (P4 Stage B)

**Status:** Locked for Stage B  
**See:** [SLICE-OPERATOR.md](./SLICE-OPERATOR.md) · [SEMANTIC-ARTIFACTS.md](./SEMANTIC-ARTIFACTS.md)

---

## Goal

Never build whole-monorepo CPG by default. Build **entrypoint- or neighborhood-scoped** shards on demand; invalidate dirty subsets only.

---

## When to build

| Trigger | Action |
|---|---|
| `prism semantic build` | Per-file T3 CFG/DFG (Stage A) |
| First `Slice` / `shard-build` for a seed | Build call-graph **shard** for seed neighborhood |
| File content hash changes | Drop that file’s T3 artifact + any shard whose `member_paths` include it |
| Memo miss with same params | Recompute slice; write memo |

Shards are **not** required for T1/T2 `compile_context`.

---

## Shard key

```text
shard_id = xxh3_128("t4-shard|" + algo_version + "|" + sorted(member_paths).join("|"))
```

Default membership: functions reachable within `max_depth` CALLS from the seed function (bidirectional for debug backward slices).

Layout:

```text
.prism/semantic/
  by-file/…                 # T3 (Stage A)
  shards/<shard_id>.json    # T4 call graph + overlay DATA_FLOW / CONTROL_DEP
  memo/<memo_key>.json      # memoized Slice results
```

---

## Invalidation

1. Compare each member file’s `content_hash` to current bytes.  
2. If any stale → delete shard + memos referencing `shard_id`.  
3. Rebuild only the dirty neighborhood (not full workspace).

---

## Caps (defaults)

| Cap | Default | Rationale |
|---|---|---|
| `max_depth` | 2 | Bound inter-proc fan-out |
| `max_functions` | 16 | Latency |
| `max_spans` | 40 | Pack size |

Residuals record truncated edges for optional expand (see Slice residual policy).
