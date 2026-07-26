# Invalidation event contract

**Phase:** P6 Stage B  
**Stream:** `GET /v1/events` (SSE)

## Event: `index.updated`

Emitted after a successful incremental re-index (watcher debounce or `POST /v1/index`).

```json
{
  "event": "index.updated",
  "snapshot_id": "<tree_fingerprint>",
  "paths": ["src/foo.py"],
  "ts_unix_ms": 0
}
```

### Client obligations

| Change | Re-fetch |
|---|---|
| Any `index.updated` | `/v1/index/status` (confirm snapshot) |
| Paths overlap open graph view | Graph view-model (P6-C / P7) |
| Paths overlap open evidence pack | `/v1/context/compile` |
| Paths overlap slice criterion | `/v1/slice` |

Do **not** assume the previous pack remains valid across snapshot changes.
