# Index size budget (N3) — P1 Stage B

**Aim:** syntactic index on disk ≈ **3–10%** of indexed source bytes (ADD N3).

## What counts

| Artifact | Included |
|---|---|
| `.prism/graph.sqlite` | yes (nodes + edges + indexes) |
| `.prism/meta.sqlite` | yes (file hashes + snapshot) |
| `.prism/blobs/` | yes if used (P0 mostly empty) |
| SCIP / semantic shards | no (P3/P4) |

## How to measure

```bash
cargo run -p prism-cli -- index <repo>
cargo run -p prism-cli -- index-status <repo>
# Compare graph_sqlite_bytes + meta_sqlite_bytes to sum of indexed source file sizes.
```

`index-status` prints cardinality (`nodes`, `edges`, `files_indexed`) and sqlite byte sizes. Latency/size NFRs are **tracked** here even when not yet met.

## Expectations at T1

- Small fixtures: ratio may exceed 10% (fixed sqlite header / page overhead).
- Pilot-scale repos (httpx / ripgrep): target band becomes meaningful after warm index.
- If ratio >> 10%: trim `attrs_json` duplication, defer unresolved node GC, or move to Kuzu later.

## Related

- Query latency NFR: [KG-QUERY-API.md](./KG-QUERY-API.md)
- Failure / recovery: [KG-FAILURE-MODES.md](./KG-FAILURE-MODES.md)
