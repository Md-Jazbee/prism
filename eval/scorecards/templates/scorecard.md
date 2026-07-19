# Scorecard template — columns aligned to ADD §32 (Evaluation Metrics)

| Column | Description | Source |
|---|---|---|
| task_id | Gold task id | tasks/ |
| repo | Pilot repo | tasks/ |
| commit_sha | Frozen snapshot | fixtures/repos/ |
| protocol | frontier+explore / medium+explore / prism+mcp | harness |
| tokens_in | Prompt / input tokens | agent / provider |
| tokens_out | Completion tokens | agent / provider |
| tool_calls | Number of tool invocations | agent transcript |
| latency_ms | Wall time for task | harness |
| quality_score | 0–100 graded checklist / judge | human or LLM-judge |
| must_include_hit | Required spans present | structural |
| notes | Freeform | harness |

## Aggregation

- Structural subset mean token ratio: `explore_tokens / prism_tokens` (P1 gate ≥5×)
- Quality delta vs explore: absolute points (P1 within ~10)
- Index latency P50/P95 (W-OBS) — placeholders until P1 benches land
