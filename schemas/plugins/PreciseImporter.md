# Plugin ABI — PreciseImporter (P3 Stage A)

Status: **draft for P3 Stage A** (2026-07-26)  
Interchange: PreciseIndex schema `0.0.1` (`schemas/precise-index/v0`)  
Rust mirror: `prism-precise`

## Role

Map external precise indexes (SCIP / LSP materializations) into Prism `FactBundle` facts at **tier T2**, **confidence precise**, then refine matching heuristic edges in the KG.

## Input

| Field | Type | Notes |
|---|---|---|
| PreciseIndex JSON | document | Must match schema major |
| Workspace snapshot | optional | Bound into `.prism/scip/manifest.json` |

## Output

| Artifact | Notes |
|---|---|
| `FactBundle` nodes/edges | `tier=T2`, `confidence=precise` |
| Overlay manifest | Under `.prism/scip/` |
| Refined edges | Heuristic `CALLS`/`REFERENCES` upgraded when join rules match |

## Hard rules

1. Never claim precise without an imported or LSP-backed fact.
2. Never use opaque integer IDs as Prism node primary keys.
3. Missing precise index ⇒ `PRECISION_REQUIRED` for gated ops; T1 still works.
4. Heuristic edges that do not match stay `heuristic`.

## Versioning

Breaking PreciseIndex shape ⇒ bump major under `schemas/precise-index/` and `PRECISE_INDEX_SCHEMA_VERSION` in `prism-ir`.
