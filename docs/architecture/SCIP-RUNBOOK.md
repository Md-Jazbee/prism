# Precise index prerequisites runbook

**Phase:** P3 Stage A  
**Goal:** Produce a PreciseIndex (or SCIP that maps to it) and attach it with `prism precise import`.

---

## Mental model

```text
Language indexer / LSP  →  PreciseIndex JSON (v0)  →  prism precise import  →  .prism/scip/ + graph overlay
```

Prism does **not** require SCIP at runtime for T1. Missing precise artifacts ⇒ T1 continues; precision-gated paths return `PRECISION_REQUIRED`.

---

## Python (Stage A primary)

### Option A — Fixture / hand-authored PreciseIndex

For tests and small demos, author `fixtures/precise/**/precise-index.json` directly (schema: `schemas/precise-index/v0`).

```bash
cargo run -p prism-cli -- index .
cargo run -p prism-cli -- precise import fixtures/precise/oracle/python/precise-index.json
cargo run -p prism-cli -- precise status
```

### Option B — SCIP Python indexer → JSON map

1. Install a SCIP-capable Python indexer (e.g. Sourcegraph `scip-python` or equivalent) in the target repo.
2. Generate an index at a **frozen commit** matching the Prism snapshot.
3. Convert SCIP protobuf → PreciseIndex JSON (Stage A: use the documented field map in [ID-MAPPING.md](./ID-MAPPING.md); a thin converter may live under `scripts/scip/` later).
4. Import:

```bash
prism precise import path/to/precise-index.json --workspace .
```

### Option C — LSP (Stage B)

Interactive resolve via pylsp / basedpyright. Stage A does not ship the client; design expects the same PreciseIndex edge shapes when LSP results are materialized.

---

## Snapshot binding

Every import records:

| Field | Why |
|---|---|
| `git_commit` (if any) | Reproducible eval |
| `tree_fingerprint` | Dirty-tree safety |
| `analyzer` | Provenance |
| `language` | Join rules |

If the workspace fingerprint diverges from the manifest, `precise status` reports **stale**; refinement still present until re-import.

---

## Failure modes

| Situation | Behavior |
|---|---|
| No `.prism/scip/` / empty overlay | T1 only; `PRECISION_REQUIRED` on gated ops |
| Import JSON schema mismatch | Reject with clear error; no partial write |
| Symbol not in T1 graph | Insert T2 node/edge; optional join later |
| Heuristic call already correct | Upgrade confidence in place |

---

## Eval (oracle P/R)

```bash
cargo test -p prism-precise
# or
cargo run -p prism-cli -- precise score \
  --t1 fixtures/precise/oracle/python/t1-calls.json \
  --oracle fixtures/precise/oracle/python/oracle-calls.json \
  --t2 fixtures/precise/oracle/python/precise-index.json
```

Precision/recall compare callee resolution of `CALLS` edges against the oracle list.
