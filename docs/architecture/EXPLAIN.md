# EXPLAIN CONTEXT report format (P2 Stage C)

Every successful `compile_context` Evidence Pack includes `explain` (also on CLI `prism compile`).

```json
{
  "plan_id": "plan:repo_qa:…",
  "budget_tokens": 4000,
  "tokens_used": 95,
  "must_include_ok": true,
  "fragments": [
    {
      "fragment_id": "frag:…",
      "why_included": "primary_symbol_definition",
      "token_estimate": 10,
      "must_include": true,
      "kept": true
    }
  ],
  "drops": [
    {
      "fragment_id": "frag:opt:…",
      "reason": "budget_pressure: drop_priority=90 …",
      "drop_priority": 90,
      "token_estimate": 80
    }
  ],
  "notes": [
    "must-include fragments cannot be budget-evicted",
    "extractive default; no abstractive code summaries"
  ]
}
```

| Field | Meaning |
|---|---|
| `why_included` | Selection reason code (recipe role / operator) |
| `kept` | Present in pack hierarchy |
| `must_include_ok` | Always true on `status=ok` packs |
| `drops` | Optional fragments removed under budget |

See [EVIDENCE-PACK.md](./EVIDENCE-PACK.md).
