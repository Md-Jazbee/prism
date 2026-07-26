# T3 analysis design — intra-procedural CFG/DFG

**Phase:** P4 Stage A  
**Crate:** [`prism-semantic`](../../crates/prism-semantic)  
**Language (Stage A):** Python first (tree-sitter); Rust deferred  
**Companion:** [SEMANTIC-ARTIFACTS.md](./SEMANTIC-ARTIFACTS.md) · [SEMANTIC-CRASH-POLICY.md](./SEMANTIC-CRASH-POLICY.md)

---

## Scope

| In | Out (later) |
|---|---|
| Per-function CFG (basic blocks + branch/fallthrough) | Inter-procedural CPG (Stage B / T4) |
| Lightweight DFG (assign → use within function) | Pointer/alias analysis, exceptions as full CFG |
| Local slice: symbol **or** line criterion | Whole-repo shard builds |
| Artifacts under `.prism/semantic/` | Joern/JVM requirement |

Tier = **T3**. Facts stay separate from hot T1/T2 `graph.sqlite`.

---

## CFG model

```text
Function
  blocks[]: { id, start_line, end_line, kind: entry|exit|branch|loop|plain }
  edges[]:  { src, dst, kind: fallthrough|true|false|loop_back }
```

Construction (Python Stage A):

1. Locate `function_definition` / `async` bodies via tree-sitter.  
2. Split into blocks at: `if`/`elif`/`else`, `while`/`for`, `return`/`raise`, `try`/`except`/`finally` (best-effort).  
3. Connect fallthrough and branch edges; loops get `loop_back`.  
4. On parse failure → empty function list + crash note (never panic).

---

## DFG model

Within one function:

| Edge | Meaning |
|---|---|
| `def` | Assignment / parameter introduces a name at a line |
| `use` | Name read at a line |
| `data_dep` | use ← reaching def (same name, last def before use in block order) |

Stage A uses **line-level** reaching defs (not SSA). Good enough for local debug slices.

---

## Local slice operator

**Criterion:** `{ path, line }` **or** `{ path, symbol }` (resolved to def line).

**Direction (Stage A):** backward (default) — blocks + lines that can affect the criterion.

**Algorithm sketch:**

1. Build/load function CFG+DFG containing the criterion.  
2. Seed = block(s) covering criterion line + data_deps into that line.  
3. Walk CFG predecessors + DFG deps until fixed point (intra-proc only).  
4. Emit extractive spans (line ranges) + `cfg_summary` text.

**Properties (tests):**

- **Contains criterion** — slice spans always cover the criterion line.  
- **Idempotent** — same `(path, criterion, algo_version)` → same spans.  
- **Bounded** — max blocks / lines caps; residual noted if truncated.

---

## Limitations (honest)

- No cross-function flow yet.  
- Comprehensions / decorators / `match` best-effort.  
- Dynamic attrs (`getattr`) not modeled.  
- Broken syntax → partial or empty artifact, agent path continues.
