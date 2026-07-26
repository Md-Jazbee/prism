# Agent tool ergonomics (P9 Stage A)

**Goal:** make `compile_context` the obvious first tool; micro-tools secondary.

| Tool | Role | Description cue |
|---|---|---|
| `compile_context` | **PRIMARY** | One-shot budgeted Evidence Pack |
| `query_plan` | Inspect | DAG only — no pack |
| `run_workflow` / `prism workflow` | Named recipes | onboarding / review / debug / refactor_prep |
| `resolve_symbol` / `neighbors` / `impact` | Secondary | Targeted hops after a pack |
| `repo_map` / `entrypoints` | Orientation | Prefer via onboarding workflow |

## Ordering

1. MCP `instructions` and tool list put `compile_context` first.  
2. Errors include a `repair` object (`action`, `summary`, `candidates≤8`, optional `tool`).  
3. Deprecations: never remove micro-tools; document them as secondary in AGENT-USAGE.

## Budget negotiation

Agents may pass `remaining_context_tokens`; the compiler uses `min(budget_tokens, remaining)` clamped to `[256, 128000]`.
