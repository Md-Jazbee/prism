# Incremental update sequence (P1 Stage B)

A **single-file edit does not require a full-repo rebuild**. Indexing is content-hash skip + per-file subgraph replace.

```mermaid
sequenceDiagram
    participant Edit as Editor
    participant Idx as IncrementalIndexer
    participant Meta as meta.sqlite
    participant Ext as LanguageExtractor
    participant KG as graph.sqlite

    Edit->>Idx: file bytes changed
    Idx->>Idx: discover + content hash (XXH3)
    Idx->>Meta: get prior hash
    alt hash unchanged
        Idx-->>Edit: skip (files_skipped_unchanged++)
    else hash changed
        Idx->>Ext: extract(path, bytes) → FactBundle
        Idx->>KG: BEGIN IMMEDIATE
        Note over KG: DELETE nodes/edges WHERE file_path=path
        Idx->>KG: insert_facts(bundle)
        Idx->>KG: COMMIT (WAL)
        Idx->>Meta: upsert_file_hash
        Idx->>KG: reverse_dep_files(path)
        Note over Idx: Stage B records dirty list;<br/>communities rebuild deferred to Stage D
    end
```

## Dirty-set policy (Stage B)

1. **Must re-extract:** paths whose content hash changed (or were deleted → `invalidate_file_subgraph`).
2. **Reverse-dep list:** `reverse_dep_files(changed)` returns dependents that *reference* symbols defined in the changed file. Today this is advisory for planners / future community refresh; extractors remain pure per-file and do not auto-reparse dependents (same-file CALLS only at T1).
3. **Communities / hubs:** refresh deferred to Phase 1 Stage D (not on every edit).

## Crash mid-transaction

See [KG-FAILURE-MODES.md](./KG-FAILURE-MODES.md). WAL + `BEGIN IMMEDIATE` replace means a crash rolls back to the previous file subgraph; re-run `prism index` to converge.
