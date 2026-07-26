# Agent usage guide

**Phase:** P9 (workflows + repair) · Source catalog: [`schemas/agent-workflow/v1/catalog.json`](../../schemas/agent-workflow/v1/catalog.json)

## Primary path: `compile_context` first

For almost all repo Q&A, impact, architecture, review, generate, and **debug** intents:

1. **`compile_context`** — one call returns a budgeted Evidence Pack + EXPLAIN  
2. Or run a **named workflow**: `prism workflow run onboarding|review|debug|refactor_prep`  
3. Answer from pack citations; inspect `gaps` / `explain.drops` / `repair` on errors  
4. Only then use micro-tools for a *targeted* follow-up  

Optional: `query_plan` to inspect the operator DAG. Pass `remaining_context_tokens` to negotiate budget; `progressive: true` for architecture-first layers.

## Workflows

| Workflow | When | Command |
|---|---|---|
| onboarding | New repo / fresh agent | `prism workflow run onboarding .` |
| review | Diff / changed paths | `prism workflow run review . --changed path` |
| debug | Stack / error | `prism workflow run debug . --error-text "…"` |
| refactor_prep | Rename / structural edit | `prism workflow run refactor_prep . --anchor Sym` |

Regenerate in-repo agent assets: `prism agent generate-assets .`

## Debug / slice

| Need | Do |
|---|---|
| Crash / stack / “why” | `compile_context` with `intent=debug`, stack frames + error text |
| Local/interproc slice only | `prism semantic slice --file … --line …` |

## Precision (T2)

| Need | Do |
|---|---|
| Casual impact | Accept **heuristic** labels |
| Accuracy claim / safe refactor | `require_precise=true` or `refactor_prep` workflow |
| Missing overlay | Expect `PRECISION_REQUIRED` + `repair.action=import_precise` |

## Errors → repair

| Code | Action |
|---|---|
| `SCOPE_UNRESOLVED` | Use `repair.candidates` as anchors |
| `BUDGET_EXCEEDED` | Raise `remaining_context_tokens` / narrow anchors |
| `INDEX_UNAVAILABLE` | `prism index .` |
| `PRECISION_REQUIRED` | `prism precise import` |
| `VIEW_TOO_LARGE` | Narrow view seeds |

See [REFUSAL-REPAIR.md](./REFUSAL-REPAIR.md) and [WORKFLOW-CATALOG.md](./WORKFLOW-CATALOG.md).

## Anti-patterns

| Anti-pattern | Do instead |
|---|---|
| Ten reads / greps to “explore” | One `compile_context` or a workflow |
| Dumping modules when scope is unclear | Honor `SCOPE_UNRESOLVED` + repair |
| Claiming rename safety from unlabeled impact | `refactor_prep` / `require_precise` |

## Hop budget

Prefer **1** `compile_context` hop (or one workflow). Micro-tool chains stay in **1–4** calls.
