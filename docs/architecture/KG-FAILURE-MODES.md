# KG failure modes & recovery (P1 Stage B)

## Crash mid-transaction

**Write path:** `BEGIN IMMEDIATE` → delete file-local nodes/edges → `insert_facts` → update `file_subgraphs` → `COMMIT`.

| Failure point | On-disk outcome | Recovery |
|---|---|---|
| Before COMMIT | WAL rolls back; prior subgraph intact | Re-run `prism index` |
| Process kill during COMMIT | SQLite WAL recovers on next open | Re-run `prism index` if hash/meta drifted |
| Partial meta upsert after graph COMMIT | Rare window: graph newer than meta hash | Next index sees hash mismatch or skip; force delete `.prism` if corrupted |

**Expectation:** never leave a half-written file subgraph visible to readers after recovery. Readers should open a fresh `SqliteKgStore` connection (CLI does this per invocation).

## Path isolation / secrets

- Secret-sensitive paths are skipped at discover (`ignore_policy`) — no blob retention of `.env` / key material in the graph.
- Graph stores paths and spans only; not full file bodies (blobs dir reserved, unused for T1 facts).

## Query failure modes

| Case | Behavior |
|---|---|
| Empty / missing index | CLI errors asking to run `prism index` |
| Ambiguous symbol names | `resolve` returns multiple hits (caller disambiguates) |
| Wrong heuristic CALLS | Impact may include false callees — confidence stays `heuristic` |
| Unresolved callees | First-class `unresolved:*` nodes; not deleted |

## Explicit non-goals (Stage B)

- Snapshot isolation across long-lived readers (single-process CLI; Stage C/P6 harden).
- Automatic re-extract of reverse-dep files on every edit (dirty list is advisory).
- Precise rename / blast-radius claims (P3).
