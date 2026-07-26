# Hybrid resolution (P3 Stage B)

**Status:** Live  
**Crate:** [`prism-precise`](../../crates/prism-precise) (`hybrid` module)  
**Planner:** `UpgradePrecision` operator (executable)  
**Policy:** [UPGRADE-POLICY.md](./UPGRADE-POLICY.md) · **Signals:** [AMBIGUITY-INDEX.md](./AMBIGUITY-INDEX.md)

---

## Algorithm

```text
seeds (ResolveSymbol)
  → collect CALLS / REFERENCES touching seeds (1-hop)
  → partition: precise | heuristic | unresolved
  → for each heuristic/unresolved edge (critical path only when flagged):
        if a precise edge joins the same site → confirm (prefer precise dst)
        else if dual candidates (heuristic dst ≠ precise dst) → keep both + uncertainty note
        else leave labeled heuristic / unresolved
  → stop when upgrade budget exhausted (count or latency)
```

Join rules reuse Stage A [ID-MAPPING.md](./ID-MAPPING.md) (exact ids → span overlap → callee name / `unresolved:` upgrade).

---

## Latency bound

| Knob | Default | Meaning |
|---|---|---|
| `est_cost_ms` on plan step | 200 | Planner cost sketch |
| `max_upgrades` | 32 | Cap edges touched per `UpgradePrecision` |
| `max_latency_ms` | 200 | Soft wall-clock budget; excess edges → `deferred` |

Upgrade is **synchronous but bounded**. Anything beyond the budget is listed in `deferred` for optional background re-import / Stage C product gates — not unbounded LSP fan-out on the request path.

---

## Outputs

| Field | Meaning |
|---|---|
| `confirmed` | Sites where precise overlay matched |
| `still_heuristic` | No precise match; stay labeled |
| `dual_candidates` | Heuristic and precise disagree |
| `deferred` | Skipped due to budget |
| `latency_ms` | Measured wall time |

Emitted as obs event `precision_upgrade` (`prism-obs`).

---

## Interaction with Evidence Packs

`select_from_kg` executes `UpgradePrecision` when present and **prefers `confidence=precise`** fragments over heuristic duplicates. Dual candidates add a pack gap / uncertainty note — never silent overwrite.
