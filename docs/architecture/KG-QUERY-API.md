# KG Query API contract (P1 Stage B)

**Status:** Stable for MCP Stage C binding  
**Store:** `graph.sqlite` via [`SqliteKgStore`](../../crates/prism-store/src/kg.rs)  
**CLI:** `prism query …` · **Future HTTP:** ADD §22 (`POST /v1/resolve`, `/neighbors`, `/impact`)

## Design NFR

| Target | Value | Tracking |
|---|---|---|
| Local structural query P95 | &lt;50ms | `query_finished.latency_ms` event |
| Index size (syntactic) | ~3–10% of source | `prism index-status` + [INDEX-SIZE-BUDGET.md](./INDEX-SIZE-BUDGET.md) |

Not yet a hard gate — values are **tracked** even if unmet on large repos.

## Operations

### `resolve_symbol(name, path_contains?, limit)`

Exact name match on `nodes.name`. Optional path substring filter.

**CLI:** `prism query resolve <name> [--file substr] [--limit N] [workspace]`

**Returns:** `GraphNodeView[]` with `id`, `kind`, `name`, `file_path`, `confidence`.

### `neighbors(id, edge_kinds?, direction, limit)`

1-hop expansion. Direction: `outgoing` | `incoming` | `both`.

**CLI:** `prism query neighbors <id> [--kind CALLS,IMPORTS] [--dir outgoing|incoming|both]`

**Returns:** `{ edge, node }[]`. Missing endpoints still surface as stubs.

### `impact(seed_id, max_depth, limit)`

BFS over outgoing `CALLS|IMPORTS|CONTAINS|DEFINES|EXTENDS|IMPLEMENTS`, depth-capped (1–8).

**CLI:** `prism query impact <id> [--depth 2] [--limit 100]`

**Returns:** depth-grouped `ImpactHit[]`. **Always heuristic at T1** — wrong callees possible; do not claim precise refactor safety.

### `reverse_dep_files(changed_path)` / dirty set

Files to consider when `changed_path` edits: the file itself plus any file that has an edge into a node owned by that path.

**CLI:** `prism query dirty <changed_path> [workspace]`

Used for incremental rebuild planning (see [INCREMENTAL-UPDATE.md](./INCREMENTAL-UPDATE.md)).

### `index-status`

Freshness + cardinality + on-disk sqlite bytes.

**CLI:** `prism index-status [workspace]`

## Confidence & provenance

Every edge carries `confidence` (`extracted` | `heuristic` | …). Query results echo node/edge confidence; impact is labeled heuristic in CLI stderr.

## Errors (CLI)

| Condition | Behavior |
|---|---|
| Missing `.prism/graph.sqlite` | Fail with “run `prism index` first” |
| Zero hits | Empty JSON array (not an error) |
| Unknown node id for neighbors/impact | Empty expansion |

`SCOPE_UNRESOLVED` as a first-class product error arrives with MCP Stage C.
