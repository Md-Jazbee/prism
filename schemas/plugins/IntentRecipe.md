# Plugin ABI — IntentRecipe (P2 Stage A)

Status: **v1 draft** (2026-07-26)  
Plan schema: `PLAN_SCHEMA_VERSION` = `0.0.1` (`schemas/plan/v0`)  
Implementation: first-party `prism-plan` crate (deterministic; not a dyn-loaded plugin yet).

ADD §23 lists `IntentRecipe` as a plugin kind (selection/reduction config). This document freezes the **recipe card** contract recipes must honor.

## Pure-transform rule

`IntentRecipe` maps `(intent, anchors, budget) → Plan` with **no LLM**, no network, and no store mutation.
Optional LLM intent classification may wrap this later; the recipe itself stays deterministic.

## Recipe card fields

| Field | Type | Notes |
|---|---|---|
| `intent` | enum | `repo_qa` \| `debug` \| `impact` \| `refactor` \| `generate` \| `review` \| `architecture` |
| `seeds` | string | What anchors the plan |
| `expand` | string | Operator expansion policy |
| `must_include` | string[] | Fragments that cannot be budget-evicted (enforced in P2 Stage B) |
| `drop_order` | string[] | Budget pressure drop priorities (ADD §18.1) |
| `notes` | string[] | Caveats (e.g. “best-effort until P4”) |
| `min_tier` | T1–T4 | Preferred precision; v1 plans are T1-executable with placeholders |

## Output

Versioned [`Plan`](../plan/v0/plan.schema.json) IR: operator DAG + must-include + drop order + gaps.

## Refuse behavior

When anchors are required and missing → `SCOPE_UNRESOLVED` (do not emit an unbounded dump plan).

## Versioning

| Change | Action |
|---|---|
| Add optional note / cost constant | patch |
| Change operator shape / must-include ids | minor or major per compatibility |
| Rename intents / remove operators | major + `PLAN_SCHEMA_VERSION` |

## Golden fixtures

`fixtures/plans/<case>/{input.json,expected.json}` — see crate tests in `prism-plan`.
