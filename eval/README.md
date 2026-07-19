# Eval harness (W-EVAL) — Phase 0 skeleton

## Purpose

Every later gate needs a measurement path. This package runs gold tasks against
pinned repo snapshots and emits scorecards (structural metrics first; LLM judges optional).

## Layout

| Path | Role |
|---|---|
| `harness/` | Runner design + smoke CLI |
| `tasks/` | Versioned gold task cards (≥20) |
| `scorecards/templates/` | Columns aligned to ADD §32 |
| `baselines/` | Explore-token runbooks |
| `reports/` | Generated outputs (gitignored later if large) |

## How we know P1 saved tokens

1. Freeze pilot SHAs in `fixtures/repos/*.md`.
2. For each task, run **baseline** `frontier+explore` and `medium+explore` (grep/read/glob only); record tokens + tool calls.
3. Run the same tasks with Prism MCP tools only (P1).
4. Compare `tokens_per_task` and `tool_calls_per_task` on the structural subset; target ≥5× reduction with quality within ~10 pts of explore.

## Smoke

```bash
cd eval && uv sync && uv run prism-eval smoke
```
