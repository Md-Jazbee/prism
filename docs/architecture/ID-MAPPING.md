# Precise ID mapping rules

**Phase:** P3 Stage A  
**Status:** Locked for PreciseIndex v0  
**See also:** [PRECISE-TIER.md](./PRECISE-TIER.md)

---

## Principles

1. **One graph identity.** Nodes in `graph.sqlite` use Prism IDs (`file:…`, `sym:…`, `unresolved:…`, `module:…`). SCIP strings are attributes, never opaque integer keys as primary IDs.
2. **Prefer SCIP-compatible readable strings when present.** Store the SCIP symbol on the node/edge as `attrs.scip_symbol`.
3. **Stable syntactic IDs remain valid.** T1 `sym:{path}:{kind}:{name}:{start_byte}` continues as the default Prism ID so overlays can join without rewriting the whole graph.
4. **Reject opaque integer graphs.** LSIF-style numeric IDs are not accepted as Prism node IDs.

---

## Prism ID forms (unchanged from T1)

| Kind | Form | Example |
|---|---|---|
| File | `file:{path}` | `file:pkg/app.py` |
| Symbol | `sym:{path}:{symbol_kind}:{name}:{start_byte}` | `sym:pkg/app.py:function:greet:0` |
| Unresolved | `unresolved:{name}` | `unresolved:print` |
| Module | `module:{dotted}` | `module:httpx` |

Paths are repo-relative with forward slashes.

---

## SCIP → Prism mapping

| SCIP field | Prism field |
|---|---|
| `Document.relative_path` | `file_path` / `file:{path}` |
| `SymbolInformation.symbol` | `attrs.scip_symbol` (string) |
| Occurrence / symbol display name | `name` |
| Definition occurrence range | `span` → also feeds `start_byte` in Prism ID when joining T1 |
| `SymbolRole.Definition` | `DEFINES` / symbol node |
| `SymbolRole.Reference` (+ call-ish) | `REFERENCES` or `CALLS` when role/attrs say call |

### Join strategy (overlay → existing T1)

Match order when refining a heuristic edge:

1. Exact Prism `src` + `dst` IDs if the PreciseIndex already uses them.
2. Same `file_path` + overlapping `span` (byte range intersection > 0) + same edge `kind`.
3. Same `file_path` + same caller symbol name + same callee **name** (upgrade `unresolved:{name}` → precise def id).

If no match: **insert** the precise edge; do not delete the heuristic edge (Stage B may reconcile). Never rewrite confidence without a match rule firing.

---

## Ambiguity

- Multiple SCIP symbols map to one T1 name → keep separate Prism nodes keyed by distinct `start_byte` / SCIP symbol; do not collapse.
- One SCIP symbol, multiple T1 candidates → prefer span overlap; else leave heuristic untouched and attach precise edge with `attrs.join=unmatched`.

---

## Versioning

Breaking ID rules ⇒ bump PreciseIndex schema major (`schemas/precise-index/`) and document migration. Additive attrs are non-breaking.
