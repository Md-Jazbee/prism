# Workflow catalog

**Schema:** [`schemas/agent-workflow/v1/`](../../schemas/agent-workflow/v1/)  
**Crate:** `prism-agent` · **CLI:** `prism workflow list|run` · **HTTP:** `GET /v1/workflows`, `POST /v1/workflow`

| ID | Chain | Gold tasks |
|---|---|---|
| `onboarding` | repo_map → entrypoints → compile(architecture) | TTO-1, T001 |
| `review` | compile(review) → optional impact | T005 |
| `debug` | compile(debug) | T008 |
| `refactor_prep` | compile(refactor, require_precise) | T003 |

Assets regenerate from the catalog:

```bash
prism agent generate-assets .
# → AGENTS.md, .cursor/rules/prism-compile-first.mdc, .prism/agent/skills.md
```

Fixtures: [`fixtures/workflows/`](../../fixtures/workflows/).
