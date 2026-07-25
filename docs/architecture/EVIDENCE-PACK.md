# Evidence Pack (P2 Stage B)

**Status:** Live — selection + budget + EXPLAIN  
**Crate:** [`prism-compile`](../../crates/prism-compile)  
**Schema:** [`schemas/evidence-pack/v0`](../../schemas/evidence-pack/v0/evidence-pack.schema.json) · `PACK_SCHEMA_VERSION` = `0.0.1`  
**CLI:** `prism compile "<question>"` · **HTTP (contract):** `POST /v1/context/compile`

## Shape (ADD §16.2)

```text
EvidencePack
  meta: intent, budget, tokens_used, question, plan_id, schema_version
  hierarchy: L_arch / L_mod / L_core / L_nbr / L_diff / L_run  (fragment id lists)
  fragments[]: id, kind, layer, text, token_estimate, provenance, confidence, why_included, …
  citations: C1 → fragment / node ids
  gaps: unresolved / missing tier notes (from plan)
  drops: budget-pressure audit
  explain: EXPLAIN CONTEXT report
```

## Pipeline

```text
question → plan_query → select (KG or synthetic) → pack_under_budget → EvidencePack
                      ↘ SCOPE_UNRESOLVED
                                              ↘ BUDGET_EXCEEDED (must-include > budget)
```

## Must-include invariant

Fragments whose `roles` intersect `Plan.must_include` are **never** dropped for budget.
If their total `token_estimate` exceeds `budget_tokens`, the compiler returns `BUDGET_EXCEEDED`
instead of silently truncating truth.

Proof: unit test `must_include_never_budget_evicted` + fixture `fixtures/packs/budget_drop/`.

## EXPLAIN

Every pack carries `explain` with per-fragment `why_included`, `kept`, and `drops[]`.
Round-trip covered by `explain_roundtrip` golden + serde test.

## CLI

```bash
prism compile "What does \`Client.send\` do?"           # needs .prism index
prism compile "What does \`Helper\` do?" --synthetic    # offline recipe candidates
prism compile "tell me about the code"                  # SCOPE_UNRESOLVED
```

## Related

- [SELECTION-PRIORITY.md](./SELECTION-PRIORITY.md)
- [REDUCTION.md](./REDUCTION.md)
- [QUERY-PLANNER.md](./QUERY-PLANNER.md)
- Labeling: [`eval/labeling/`](../../eval/labeling/README.md)
