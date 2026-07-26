# LOD policy

**Phase:** P7 Stage A  
**Goal:** Progressive disclosure — never “show all 50k nodes.”

## Ladder

| Level | Name | Typical seeds | Target nodes | Promotion | Demotion |
|---|---|---|---:|---|---|
| L0 | Repo | path-prefix communities | ≤ 40 | click community → L1 | — |
| L1 | Subsystem | community + hubs | ≤ 80 | expand hub → L2 | collapse groups → L0 |
| L2 | Module / file | file nodes in group | ≤ 80 | expand file → L3 | collapse to community |
| L3 | Symbol | symbols in file | ≤ 80 | focus neighborhood | collapse to file |
| L4 | Detail | slice / impact / pack overlay | ≤ 80 | — | back breadcrumb |

Default budgets remain **80 / 160** (nodes / edges). LOD levels may request **lower** ceilings; they never raise ceilings past the explicit user/daemon max without an opt-in.

## `lod_rank` on nodes

| `lod_rank` | Meaning |
|---:|---|
| 0 | Seed / always keep (community, criterion, pack must-include) |
| 1 | Primary expansion (hubs, first-hop impact) |
| 2 | Secondary (soft-drop candidates) |
| ≥3 | Decorative / heat annotations |

Drop order: higher `lod_rank` first, then id ascending (see layout determinism).

## Promotion / demotion rules

1. **Promote** only via a budgeted server re-project or a client `focus_neighborhood` that stays within the current node set.
2. **Demote** by collapsing groups; aggregated edges use weakest-member confidence ([AGGREGATION-SEMANTICS.md](./AGGREGATION-SEMANTICS.md)).
3. If promotion would need more than `max_nodes` candidates at seed priority 0 → `VIEW_TOO_LARGE`.
