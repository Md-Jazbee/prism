# Agent traces

**Schema:** [`schemas/agent-trace/v1/`](../../schemas/agent-trace/v1/)  
**Default path:** `.prism/logs/agent-traces.jsonl` (gitignored under `.prism/`)

## Privacy

- Local by default; opt-in export only.  
- Record **tool names, codes, latencies, hop counts** — never file bodies or pack text.  
- `PRISM_TRACE=0` disables persistence (CLI: omit `--persist-trace`).

## Metrics derived

| Metric | Definition |
|---|---|
| `chose_compile_first` | First `tool_call` is `compile_context` |
| `refusal_count` | Events with `kind=refusal` or `error_code` |
| `repair_success_count` | `kind=repair` with `ok=true` |
| `hops` | Count of `tool_call` events |
