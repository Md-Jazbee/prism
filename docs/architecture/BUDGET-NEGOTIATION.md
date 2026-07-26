# Budget negotiation & progressive packs

## Negotiation

| Input | Behavior |
|---|---|
| `budget_tokens` | Requested pack budget (default 4000) |
| `remaining_context_tokens` | Agent's leftover context window |
| Effective | `min(budget, remaining).clamp(256, 128000)` |

Available on MCP `compile_context` and HTTP `POST /v1/context/compile`.

## Progressive packs

Must-include is finalized **before** streaming. Layers:

1. `architecture` — stream first so the agent can start reasoning  
2. `must_include` — criterion / mandatory fragments  
3. `support` — soft-drop candidates  

Pass `progressive: true` on compile (MCP/HTTP) or use `prism_agent::progressive_layers`.
