# ADR-0004: Crate consolidation — graph + intel in `prism-store`

**Date:** 2026-07-26  
**Status:** Accepted  
**Gaps:** as-built inventory (planned `prism-graph` / `prism-intel`)  
**Expiry:** none (structural; revisit only if a split is forced by binary size or ABI)

## Context

The monorepo layout planned separate `prism-graph` and `prism-intel` crates. Implementation folded KG persistence/query and repository intelligence into **`prism-store`** (`query.rs`, `intel.rs`). Docs still mentioned the split crates.

## Decision

**Accept** the consolidation. `prism-store` owns:

- SQLite KG adjacency + query API
- Meta store / fingerprints
- Derived intel (communities, hubs, entrypoints, hotspots, contracts)

Do not create empty stub crates to match old diagrams.

## Consequences

- Tech-stack §3 layout lists `prism-store` as the home for graph + intel.
- Future HTTP/daemon layers depend on `prism-store` traits, not phantom crates.
