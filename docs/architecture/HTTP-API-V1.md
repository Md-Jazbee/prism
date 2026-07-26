# HTTP + SSE API v1

**Phase:** P6 Stage B  
**Transport:** `prism-api` behind `prismd`  
**Error model:** mirrors MCP (`SCOPE_UNRESOLVED`, `BUDGET_EXCEEDED`, `PRECISION_REQUIRED`, …)  
**Auth:** `Authorization: Bearer <token>` or `X-Prism-Token: <token>` (except `/health`)

Base URL (default): `http://127.0.0.1:7420`

## Endpoints

| Method | Path | Purpose |
|---|---|---|
| GET | `/health` · `/v1/health` | Liveness + `api_version` + `snapshot_id` (no auth) |
| GET | `/v1/index/status` | Freshness, node/edge counts |
| POST | `/v1/index` | Trigger incremental re-index (`{ "paths": [] }`) |
| POST | `/v1/query/plan` | Plan IR only |
| POST | `/v1/context/compile` | Evidence Pack |
| GET | `/v1/symbols?name=` | Symbol resolve |
| POST | `/v1/query/neighbors` | 1-hop neighbors |
| POST | `/v1/impact` | Blast radius (`require_precise` optional) |
| POST | `/v1/slice` | Inter-procedural slice |
| GET | `/v1/repo/map` | Communities / hubs (`full_intel=true` for full report) |
| GET | `/v1/intel/entrypoints` | Heuristic entrypoints |
| GET | `/v1/events` | **SSE** invalidation stream |

## Example

```bash
TOKEN=secret
curl -s -H "Authorization: Bearer $TOKEN" http://127.0.0.1:7420/v1/index/status
curl -s -H "Authorization: Bearer $TOKEN" -H 'content-type: application/json' \
  -d '{"question":"Explain entry","anchors":["entry"],"intent":"repo_qa"}' \
  http://127.0.0.1:7420/v1/context/compile
curl -N -H "Authorization: Bearer $TOKEN" http://127.0.0.1:7420/v1/events
```

## Errors

```json
{ "error": { "code": "SCOPE_UNRESOLVED", "message": "…", "hint": "…", "snapshot_id": "…" } }
```

| Code | HTTP |
|---|---|
| `SCOPE_UNRESOLVED` | 422 |
| `BUDGET_EXCEEDED` | 422 |
| `PRECISION_REQUIRED` | 409 |
| `INDEX_UNAVAILABLE` | 503 |
| `INVALID_ARGS` | 400 |
| `UNAUTHORIZED` | 401 |

## Cancellation

Handlers honor client disconnect (axum request drop). Superseded UI requests should cancel the in-flight HTTP call; the daemon does not queue unbounded compile work per connection.
