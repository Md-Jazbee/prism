# OpenTelemetry span model (P5 Stage B — design)

**Status:** Design; emission today via `tracing` + `IndexEvent`  
**Related:** [AUDIT-AND-REDACTION.md](../security/AUDIT-AND-REDACTION.md)

---

## Span hierarchy

```text
prism.compile_context
  ├─ plan_query
  │    └─ recipe.<intent>
  ├─ select_from_kg
  │    ├─ op.resolve_symbol
  │    ├─ op.upgrade_precision
  │    ├─ op.slice
  │    └─ op.community_of / impact / …
  └─ pack_under_budget
```

Attributes (suggested): `plan_id`, `intent`, `budget_tokens`, `tokens_used`, `workspace_fingerprint`.

---

## Metrics

| Metric | Meaning |
|---|---|
| `prism.pack.tokens_used` | Pack size |
| `prism.pack.latency_ms` | End-to-end compile |
| `prism.token_savings_shadow` | explore_proxy − pack (when both known) |
| `prism.op.latency_ms{op=…}` | Per-operator |

---

## Exporters

Stage B does not require OTLP. When enabled: prefer local collector; never ship pack bodies off-machine without `pack_bound_for_llm` + redaction.
