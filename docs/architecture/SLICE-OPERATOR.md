# Slice operator contract (P4 Stage B)

**Status:** Locked for Stage B  
**Operator:** `Slice` (executable)  
**Backend:** [`prism-semantic`](../../crates/prism-semantic)  
**See:** [T4-SHARDING.md](./T4-SHARDING.md) · [T3-ANALYSIS.md](./T3-ANALYSIS.md)

---

## Inputs

```json
{
  "direction": "backward",
  "max_depth": 2,
  "max_functions": 16,
  "max_spans": 40,
  "residual_expand": true,
  "criterion_from": "s1",
  "path": "optional/repo/rel.py",
  "line": 15,
  "symbol": "optional_fn_name"
}
```

| Field | Meaning |
|---|---|
| `direction` | `backward` (debug default) or `forward` |
| `max_depth` | Inter-procedural CALLS hops |
| `max_functions` | Hard stop on visited functions |
| `max_spans` | Cap on emitted line ranges |
| `residual_expand` | If true, truncated work is listed in `residual` for later expand |
| Criterion | From plan anchors / stack frames, or explicit `path`+`line`/`symbol` |

---

## Outputs

```json
{
  "algo_version": "t4-python-interproc@0.0.1",
  "direction": "backward",
  "criterion": { "path": "…", "line": 15 },
  "spans": [{ "path": "…", "start_line": 10, "end_line": 15, "function": "leaf" }],
  "functions_visited": ["leaf", "mid"],
  "depth_reached": 1,
  "truncated": false,
  "residual": [],
  "provenance": { "shard_id": "…", "memo_hit": false, "params_hash": "…" },
  "cfg_summary": "…"
}
```

**Properties:**

- Spans always cover the criterion line (intra-proc invariant preserved).  
- Idempotent for identical `(snapshot_id, algorithm_version, params_hash)`.  
- Provenance lists shard / memo keys — never silent whole-file dumps.

---

## Errors

| Code | When |
|---|---|
| `SEMANTIC_PARTIAL` | Criterion file missing / unanalyzed / parse soft-fail |
| (soft) gaps note | Caps hit → `truncated=true` + residual; plan continues |

Missing semantic artifacts → best-effort local neighborhood from KG signatures (Stage A behavior); gap notes say slice was partial.

---

## Memoization key

```text
memo_key = xxh3_128(
  snapshot_id | algorithm_version | params_hash
)
params_hash = xxh3_128(canonical_json(direction, max_*, criterion, residual_expand))
snapshot_id = workspace tree fingerprint or "adhoc" when unknown
```

---

## Residual policy

When `max_depth` / `max_functions` / `max_spans` stops expansion:

1. Emit what was collected.  
2. Set `truncated=true`.  
3. Append residual entries: `{ kind: "call_edge"|"span_budget", from, to, reason }`.  
4. If `residual_expand` and planner later Expand runs, callees in residual may be signature-expanded (not full re-slice unless asked).

Criterion + error/stack verbatim fragments stay must-include under budget pressure (Stage C gate).
