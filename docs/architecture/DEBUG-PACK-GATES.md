# Debug pack quality gates (P4 Stage C)

**Status:** Locked  
**Enforced in:** `prism-compile` budget packer + unit tests

---

## Hard invariants

1. **Must-include roles** for debug (`error_or_stack_verbatim`, `primary_frame_body`) are never budget-evicted.  
2. Fragments with `FragmentKind::ErrorVerbatim` are forced must-include.  
3. Fragments tagged `primary_frame_body` or `criterion_slice` are forced must-include.  
4. If must-include cannot fit → `BUDGET_EXCEEDED` (never soft-drop truth).  
5. Optional neighbors / architecture prose drop first (plan `drop_order`).

---

## Soft quality checks (scorecard)

| Check | Pass condition |
|---|---|
| Slice operator executable on debug plan | `steps[slice].executable == true` |
| Pack contains error or frame role | At least one must-include role present after synthetic compile |
| Runtime enrichment | **Not required** for Phase 4 gate |

---

## EXPLAIN notes

Packs should retain notes that must-include cannot be budget-evicted. Agents trust `explain.drops` only for optional fragments.
