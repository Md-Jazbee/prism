# Ambiguity index (P3 Stage B)

**Status:** Live  
**API:** `SqliteKgStore::ambiguity_index` · CLI `prism precise ambiguity`  
**Feeds:** optional `UpgradePrecision` on impact ([UPGRADE-POLICY.md](./UPGRADE-POLICY.md))

---

## Definition

Over all `CALLS` edges in `graph.sqlite`:

| Metric | Formula |
|---|---|
| `total_calls` | count(`kind=CALLS`) |
| `precise_calls` | confidence = `precise` |
| `heuristic_calls` | confidence = `heuristic` and dst not `unresolved:*` |
| `unresolved_calls` | dst starts with `unresolved:` |
| `heuristic_rate` | `(heuristic + unresolved) / total` |
| `unresolved_rate` | `unresolved / total` |
| `require_t2` | `unresolved_rate ≥ 0.30` **or** `heuristic_rate ≥ 0.50` (or total=0 → false) |

Empty graph → all zeros, `require_t2=false`.

---

## Use

1. **Planner / compile:** impact recipes carry `UpgradePrecision` with `policy=optional_on_ambiguity`; executor skips when `require_t2` is false.
2. **Agents:** high unresolved rate ⇒ attach PreciseIndex (`prism precise import`) before refactor claims.
3. **Eval:** track rates before/after overlay on oracle fixtures.
4. **Repo intel (P5):** `repo_intel.require_t2_hint` mirrors this flag for architecture packs — orientation still works at T1; accuracy claims should upgrade.

---

## Auto-require T2 (product policy)

| Surface | When `require_t2` |
|---|---|
| Impact `UpgradePrecision` | Execute hybrid resolve |
| Refactor | Already mandatory regardless |
| Architecture / `repo_map` | Emit note only; do not block |
| Safe rename | Stage C gating (`PRECISION_REQUIRED`) |

---

## Non-goals

- Not a substitute for Leiden communities.
- Not a precise call graph — rates are T1/T2 mixture labels only.
