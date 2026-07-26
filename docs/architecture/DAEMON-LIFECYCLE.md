# Daemon lifecycle (`prismd`)

**Phase:** P6 Stage B  
**Hard rule:** the CLI must keep working with **no daemon**. `prismd` is an accelerator for warm caches, file watching, and HTTP/SSE clients.

## Start / stop

```bash
# binary
prismd --bind 127.0.0.1:7420 --token secret /path/to/repo

# or CLI subcommand (same runtime)
prism daemon --bind 127.0.0.1:7420 --token secret /path/to/repo
```

- Prints the bearer token if one was generated.
- Writes `.prism/daemon.lock` with pid + bind (overwrites stale locks).
- Runs an initial index if `graph.sqlite` is missing, otherwise refreshes snapshot id.
- Ctrl-C removes the lock and exits; killing the process leaves the on-disk index intact — CLI continues to work.

## Single instance

One daemon per workspace is the intended mode. A second start warns and overwrites `daemon.lock` (crash recovery). Clients should prefer the lockfile bind address when present.

## Idle shutdown

`--idle-shutdown-secs N` (default `0` = never) exits after N seconds without authenticated requests.

## Multi-workspace

Run one `prismd` per workspace root. Do not point a single process at multiple roots in Stage B.

## Staleness

Every JSON response includes `snapshot_id` (tree fingerprint after the last index). SSE `index.updated` events carry the new snapshot and changed paths. Clients must re-fetch views/packs when the snapshot changes.

## Crash recovery

SQLite WAL + incremental hashes are the source of truth. Losing the daemon process never corrupts the store; the next CLI `prism index` or daemon start reconciles.
