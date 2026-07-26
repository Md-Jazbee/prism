# Layout determinism

**Phase:** P6 Stage C  
**Invariant:** identical `(snapshot_id, view_kind, params_key, kept node/edge ids)` ⇒ identical `x`/`y`.

## Seeding

```text
seed = xxh3_64("{snapshot_id}|{view_kind}|{params_key}") as hex
```

`params_key` includes seed id, anchors, and question string.

## Tie-breaking

1. Candidates sorted by `(drop_priority asc, id asc)` before budget cut.
2. Kept nodes sorted by `id` before coordinate assignment.
3. Edges sorted by `id` after endpoint filtering.

## Whitespace-only edits

Whitespace-only source edits that do not change content hashes leave `snapshot_id` unchanged → views are stable. Edits that change hashes bump the snapshot; clients must re-fetch.

## P7 note

ELK/Cytoscape may overwrite coordinates for nicer layouts, but must memoize under `.prism/views/` keyed by the same seed tuple so screenshots remain diffable.
