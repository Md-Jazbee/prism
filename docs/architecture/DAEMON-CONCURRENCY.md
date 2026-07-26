# Concurrency & cancellation (prismd)

**Phase:** P6 Stage B

## Single-writer discipline

- Index mutations take `AppState.index_lock` (Tokio mutex).
- Rayon fan-out is used only for **hash + extract**; SQLite writes remain sequential on the calling task.
- Readers open SQLite per request (snapshot consistency via WAL); they never write.

## Request lifecycle

1. Auth middleware validates token and touches `last_activity_ms`.
2. Handler runs; if the client disconnects, the future is dropped.
3. Long compile/slice work should be cancelled by dropping the HTTP request (UI supersede = abort fetch).

## Superseded-request policy

- Clients assign their own request ids; the daemon does not demux by id in Stage B.
- A newer UI request **must cancel** the previous HTTP call. Ignoring responses without cancellation can still burn CPU until the old future finishes.

## Property to preserve

Concurrent edits + queries must not corrupt `graph.sqlite`. The writer lock + per-file subgraph replace transactions are the guardrails.
