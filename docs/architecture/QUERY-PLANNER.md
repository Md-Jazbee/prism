# Query Planner (P2 Stage A)

**Status:** Live — deterministic recipes, plan-only API  
**Crate:** [`prism-plan`](../../crates/prism-plan)  
**Schema:** [`schemas/plan/v0/plan.schema.json`](../../schemas/plan/v0/plan.schema.json) · `PLAN_SCHEMA_VERSION` = `0.0.1`  
**CLI:** `prism query plan "<question>"` · **HTTP (contract):** `POST /v1/query/plan` (ADD §22)

## Role

Translate intent + hints into an **operator DAG** (ADD §19). No LLM in the planning path.

```text
question + hints → classify intent → recipe → Plan IR
                 ↘ missing anchors → SCOPE_UNRESOLVED
```

## Operator catalog v1

| Operator | Executable now? | Notes |
|---|---|---|
| `resolve_symbol` | ✅ | P1 KG |
| `expand` | ✅ | neighbors stand-in |
| `impact` | ✅ | heuristic T1 |
| `diff_intersect` | ✅* | seeds from changed paths / dirty stamp (*shallow) |
| `find_tests` | ✅* | heuristic name/path (*Stage B materializes) |
| `community_of` | ✅ | `repo_map` |
| `budget_pack` | 🟡 | declared; Evidence Pack in Stage B |
| `slice` | ❌ placeholder | P4 |
| `upgrade_precision` | ❌ placeholder | P3 |
| `keyword_embed_fallback` | ❌ | low-confidence only; not in default recipes |

## Cost model (sketch)

Each operator carries a constant `est_cost_ms`. Recipes prefer cheap T1 ops; `UpgradePrecision` / `Slice` appear only where ADD recipes require them and are marked `executable: false`.

## Plan IR (summary)

| Field | Meaning |
|---|---|
| `plan_id` | Deterministic id from intent + question + budget |
| `intent` | Classified or overridden |
| `steps[]` | DAG nodes: `op`, `inputs`, `depends_on`, `est_cost_ms`, `executable`, `why` |
| `must_include` | Fragments Stage B must not drop |
| `drop_order` | Budget pressure priorities (ADD §18.1) |
| `gaps` | Missing tiers / non-executable ops |

## Plan-only API contract

### CLI

```bash
prism query plan "What does \`Client.send\` do?" [--intent repo_qa] [--budget 4000] [workspace]
prism query plan "tell me about the code"   # → SCOPE_UNRESOLVED
```

Stdout: JSON [`PlanOutcome`](../../crates/prism-plan/src/plan.rs)  
Stderr: `# plan status=… intent=… steps=…`

### HTTP (future)

`POST /v1/query/plan`

```json
{
  "question": "…",
  "budget_tokens": 4000,
  "intent": null,
  "anchors": [],
  "stack_frames": [],
  "error_text": null,
  "changed_paths": []
}
```

**200** — `{ "status": "ok", "data": { …Plan } }`  
**200** — `{ "status": "scope_unresolved", "data": { "code": "SCOPE_UNRESOLVED", … } }`  
(Product refuse, not a 5xx — matches MCP error model.)

## Example plans

See [INTENT-RECIPES.md](./INTENT-RECIPES.md) and `fixtures/plans/{debug,impact,repo_qa}/`.

### Debug (ADD §19.4 shape)

`ResolveSymbol → UpgradePrecision (placeholder) → Slice (placeholder) → DiffIntersect → Expand → BudgetPack`

### Impact

`ResolveSymbol → Impact → BudgetPack`

### Repo-QA

`ResolveSymbol → Expand (signatures) → BudgetPack`

## Errors

| Code | When |
|---|---|
| `SCOPE_UNRESOLVED` | Intent needs anchors and none were found — refuse unbounded dump |

## Non-goals (Stage A)

- Executing the DAG / building Evidence Packs (Stage B)
- MCP `compile_context` as primary tool (Stage C)
- LLM intent classification
