# Planner upgrade policies (P3 Stage B)

**Status:** Locked for Stage B  
**See:** [HYBRID-RESOLVE.md](./HYBRID-RESOLVE.md) · [QUERY-PLANNER.md](./QUERY-PLANNER.md)

---

## When is `UpgradePrecision` inserted?

| Intent | Insertion | Mode | Rationale |
|---|---|---|---|
| **refactor** | **Mandatory** | Full seed neighborhood | Safe-rename / ref claims need T2 when available |
| **debug** | **Mandatory** | `critical_path_only` | Frame0 / error locus CALLS only |
| **impact** | **Optional** | `optional_on_ambiguity` | Insert always in plan; execute only if ambiguity index says `require_t2` |
| **review** | Optional (via impact cone) | — | Relies on impact; no extra step in v1 |
| **repo_qa** / **generate** / **architecture** | **Absent** | — | Stay cheap T1 |

---

## Mandatory vs optional

```text
mandatory  → always run hybrid resolve (skip confirmations only if no overlay;
             gaps note missing T2; Stage C may PRECISION_REQUIRED for claims)
optional_on_ambiguity → run only when AmbiguityIndex.require_t2 == true
```

Missing overlay does **not** fail the plan at Stage B. Heuristic edges remain; gaps/notes record that precise confirmation was unavailable. Product claims that need T2 escalate in Stage C via `PRECISION_REQUIRED`.

---

## Cost policy

- Prefer T1 `ResolveSymbol` / `Expand` / `Impact` first.
- `UpgradePrecision` cost sketch = **200ms**; hard caps: **32 edges**, **200ms** wall.
- Do not insert `KeywordEmbedFallback` as a substitute for precision.

---

## Plan inputs shape

```json
{
  "nodes_from": "s1",
  "tier": "T2",
  "critical_path_only": true,
  "policy": "mandatory" | "optional_on_ambiguity",
  "max_upgrades": 32,
  "max_latency_ms": 200
}
```
