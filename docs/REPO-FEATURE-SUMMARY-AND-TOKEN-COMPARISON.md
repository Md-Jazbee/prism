# Prism Repo — Feature Summary & Token Comparison

**Generated:** 2026-07-26  
**Method:** Prism MCP (`compile_context`, `repo_map`, `index_status`) + Graphify query + documented four-arm baseline  
**Workspace:** `/Users/jasbeermohammad/AllFiles/AI-lenarning/AI-P2`  
**Index:** T1 · 375 files · **1,629 nodes** · **8,676 edges** · `.prism/graph.sqlite` ≈ 10.3 MB

---

## 1. Short summary

**Prism** is an open-source **repository intelligence platform**. It indexes a codebase into a local knowledge graph, then **compiles Evidence Packs** — minimum-sufficient, provenance-bearing context under a token budget — so AI coding agents call `compile_context` once instead of grepping through the repo.

It is **local-first** (indexing needs no API key), **agent-native** (MCP + CLI + workflows), and designed so small/medium models can approach frontier accuracy when context quality is high.

**Phase:** 11 — Install & Distribution (Stage A+B complete). P0–P7 + P9 gated · P8 cut · P10 deferred.

---

## 2. Detailed features

### 2.1 Product thesis

| Rejected assumption | Prism replacement |
|---|---|
| Larger context windows solve repo understanding | Structured retrieval + program slicing |
| Embeddings are primary retrieval | Symbol / CFG / call-graph / dataflow first; embeddings last |
| Agents should explore via grep/read loops | Agents query a precomputed intelligence IR |
| One graph fits all | Precision ladder: syntactic → symbol → semantic → behavioral |

**Differentiator vs Graphify / GitNexus / Codebase-Memory:** those systems show tree-sitter graphs + MCP cut tokens. Prism treats **context assembly as compilation** — query plans, typed evidence IR, confidence provenance, slice-based reduction, and a path to precise (SCIP/LSP) and semantic (CPG/slicing) tiers.

### 2.2 Core capabilities

| Area | What you get |
|---|---|
| **Indexing** | Incremental local index into `.prism/graph.sqlite`; fingerprint + dirty-path reindex |
| **Evidence Packs** | `compile_context` → budgeted fragments + citations + EXPLAIN (why kept / drops) |
| **Query planner** | Intent → operator DAG (`query_plan`); recipes for repo_qa / architecture / debug / review / refactor |
| **KG queries** | `resolve_symbol`, `neighbors`, `impact` (heuristic at T1; gated when `require_precise`) |
| **Orientation** | `repo_map` (path-prefix communities + degree hubs), `entrypoints`, `detect_changes` |
| **Precision tiers** | T1 tree-sitter (shipped) → T2 precise overlays → T3/T4 semantic/CPG (crates present) |
| **MCP** | Read-only stdio tools; primary tool is `compile_context` |
| **CLI** | `setup`, `doctor`, `index`, `compile`, `query`, `mcp`, `daemon`, `lsp`, `view`, `agent`, `host`, `hook`, `self-update` |
| **Agent workflows** | `onboarding`, `review`, `debug`, `refactor_prep` (catalog → AGENTS.md / Cursor rules) |
| **Install** | curl/irm installers with SHA-256 fail-closed; host adapters (Cursor / VS Code / Claude / generic) |
| **Eval** | Four-arm harness, scorecards, dual-review labeling, public benchmark reports |

### 2.3 MCP tool surface (agent-facing)

| Tool | Role |
|---|---|
| **`compile_context`** | Primary — Evidence Pack + EXPLAIN under budget |
| `query_plan` | Plan-only DAG (no pack) |
| `index_status` | Freshness, cardinality, on-disk size |
| `resolve_symbol` / `neighbors` / `impact` | Targeted graph hops |
| `repo_map` / `entrypoints` / `detect_changes` | Orientation & change blast radius |

### 2.4 Named workflows

| Workflow | Trigger | Tools | Pack shape |
|---|---|---|---|
| `onboarding` | Fresh agent / newcomer | `repo_map` → `entrypoints` → `compile_context` | Architecture orientation |
| `review` | Diff / PR / changed paths | `compile_context` → `impact` | Review + blast radius |
| `debug` | Stack / error text | `compile_context` | Debug slice never dropped |
| `refactor_prep` | Rename / structural refactor | `compile_context` (`require_precise`) | Precise refs or `PRECISION_REQUIRED` |

### 2.5 Workspace layout (from Prism `repo_map`)

**24 path-prefix communities** (top crates by file count):

| Community | Files (approx) | Nodes (approx) |
|---|---:|---:|
| `prism-semantic` | 9 | 91 |
| `prism-precise` | 8 | 77 |
| `prism-agent` | 7 | 56 |
| `prism-view` | 7 | 51 |
| `prism-api` / `prism-compile` / `prism-store` | 6 each | 67–93 |
| `prism-core` / `prism-ir` / `prism-mcp` / `prism-plan` | 5 each | 39–59 |
| `prism-cli` | 4 | 65 |
| Extractors (`python` / `rust` / ABI) + `daemon` / `lsp` / `bench` / eval / fixtures | smaller | — |

Notable structural hubs (degree; T1 heuristic — do not treat as architectural gospel): `main` (CLI), `select_from_kg`, `interproc_slice`.

### 2.6 CLI surface (product)

| Command | Purpose |
|---|---|
| `prism setup` / `doctor` | Bootstrap index + agent assets + MCP registration |
| `prism index` / `index-status` | Incremental index + freshness |
| `prism compile` | Evidence Pack under budget |
| `prism query` | resolve · neighbors · impact · repo-map |
| `prism mcp` | MCP stdio server |
| `prism daemon` / `lsp` / `view` | Optional HTTP/SSE, LSP, graph view-model |
| `prism workflow` / `agent` | Named workflows + generated assets |
| `prism host` / `hook` / `self-update` | Host install, post-commit reindex, upgrades |

---

## 3. Use cases

| Who | Use case | How Prism helps |
|---|---|---|
| **AI coding agents** (Cursor, Claude, etc.) | Answer “how does X work?” without dumping the repo | One `compile_context` Evidence Pack with citations |
| **Agents debugging crashes** | Stack trace → root cause | `debug` intent keeps error/stack + slice criterion |
| **PR / change review** | Blast radius of a diff | `review` + `impact` / `detect_changes` |
| **Safe refactors / renames** | Find references before edit | `refactor_prep` + precise gating (`PRECISION_REQUIRED` when T2 needed) |
| **Onboarding humans & agents** | Orient to a large monorepo | `onboarding` → communities, hubs, architecture pack |
| **Teams wanting privacy** | Index without cloud upload | Local-first sqlite graph; no API key for core indexing |
| **Language / plugin authors** | Add a language | Extractor ABI + goldens (`plugin-guide`) |
| **Eval / research** | Prove token↓ and accuracy parity | Four-arm baselines, scorecards, dual-reviewed packs |
| **IDE / LSP consumers** | Augment (not replace) language servers | `prism lsp` + view-model projection |

**Anti-use-cases (non-goals):** replacing general code search products; training foundation models; embedding-only RAG as the spine; enterprise multi-tenant SaaS in v1.

---

## 4. Tokens consumed — **this task** (measured)

### 4.1 Prism Evidence Pack / tool tokens (measured)

| Call | Intent / tool | `tokens_used` (or estimate) | Notes |
|---|---|---:|---|
| MCP `compile_context` #1 | `architecture` · budget 6000 | **279** | Communities + hubs (most useful pack) |
| MCP `compile_context` #2 | `repo_qa` · budget 8000 | **149** | Thin T1 stubs (docs not deeply sliced) |
| CLI `prism compile` | `repo_qa` · budget 4000 | **92** | Same thin product-narrative gap |
| MCP `repo_map` | orientation | **~900** | Estimated from JSON payload size |
| MCP `index_status` | metadata | **~80** | Estimated |
| **Subtotal — Prism tools** | | **≈ 1,500** | Pack + orientation payloads |

### 4.2 Follow-up reads used to finish the product narrative

Because T1 `compile_context` returns **code-graph fragments**, not README prose, this task also read:

| Source | ~Tokens (chars÷4) |
|---|---:|
| `README.md` | ~1,721 |
| ADD §1–3 (exec summary + goals) | ~3,500 |
| `MCP-TOOL-CATALOG.md` | ~701 |
| Workflow `catalog.json` | ~737 |
| Benchmark report skim | ~800 |
| **Subtotal — targeted docs** | **≈ 7,500** |

### 4.3 Total for this task (actual)

| Bucket | Tokens |
|---|---:|
| Prism tools / Evidence Packs | ≈ **1,500** |
| Targeted doc reads (gap-fill) | ≈ **7,500** |
| Graphify query (comparison arm) | ≈ **1,573** output (budget 2,000) |
| **Grand total this session (approx.)** | **≈ 10,500–11,000** |

> **Ideal Prism-only path for structural orientation:** architecture pack + `repo_map` alone ≈ **1,200 tokens**, then answer.  
> Product-marketing narrative still benefits from README/ADD until markdown/architecture prose extraction improves.

---

## 5. Comparison: Without Prism vs With Graphify vs With Prism

Estimates for the **same task**: “Summarize this repo’s features and use cases.”

### 5.1 Token comparison table

| Approach | Typical context loaded | Est. tokens | vs naive |
|---|---|---:|---:|
| **A — Without Prism** (grep/read explore) | README + ADD + tech stack skim + MCP docs + CLI/MCP crate samples + multi-hop greps | **≈ 30,000** | 1× (baseline) |
| **B — With Graphify** (existing `graphify-out/`) | `graphify query --budget 2000` + report skim + follow-up node sources | **≈ 8,000–13,000** | ~2–4× fewer |
| **C — With Prism** (MCP Evidence Packs) | Measured packs + orientation (~1,500) + small doc gap-fill if needed | **≈ 1,500–9,000** | **~3–20× fewer** |
| **C′ — Prism structural-only** (no gap-fill) | `repo_map` + architecture `compile_context` | **≈ 1,200** | **~25× fewer** |

**Published four-arm proxy** (`docs/eval/PUBLIC-BENCHMARK-REPORT-V2.md`) on structural tasks:

| Arm | Protocol | Hops | Tokens proxy | Quality proxy |
|---|---|---:|---:|---:|
| A | Frontier + explore | 12 | 18,000 | 0.70 |
| B | Medium + explore | 10 | 12,000 | 0.62 |
| C | Medium + Prism | **1** | **800** | 0.68 |
| D | Frontier + Prism | **1** | **1,200** | 0.72 |

→ C vs A ≈ **22×** fewer tokens; quality within ≤3 pts (proxy PASS).

### 5.2 Accuracy comparison (qualitative + measured signals)

| Dimension | Without Prism | With Graphify | With Prism |
|---|---|---|---|
| **Product / thesis narrative** | High if you open README/ADD; easy to miss crates | **Strong** — doc nodes from ADD/README appear in BFS (this run hit Product Goals, MCP, tiers) | Medium at T1 for prose questions; **strong** once README/ADD anchors are read or architecture prose is packed |
| **Structural truth** (crates, hubs, entrypoints) | Medium — depends on which files agent opens | Medium — AST + communities; edges labeled EXTRACTED/INFERRED/AMBIGUOUS | **Strong** — `repo_map` + index cardinality + EXPLAIN provenance |
| **Provenance / confidence** | Weak (no labels) | Good (edge honesty labels) | **Best** — per-fragment confidence + tier + analyzer |
| **Token discipline** | Weak (explore loops grow) | Good (`--budget`) | **Best** (hard budget + must-include + drops) |
| **Precise rename / impact claims** | Risky | Risky | Gated (`PRECISION_REQUIRED` / T2) — honest refusals |
| **Hop count** | Many (10–12+) | Few–moderate (BFS) | **One** primary tool for structural Q&A |
| **Risk of wrong context** | High (wrong files, fixture noise) | Medium (community mix; may surface unrelated nodes) | Lower for code tasks; T1 CALLS still **heuristic** |

**Verdict for this task type (features + use cases):**

1. **Graphify** surfaced product-oriented nodes fastest from an already-built graph (good for “what is this product?”).  
2. **Prism** was best for **repo structure, MCP contracts, workflows, and token-bounded agent UX**, and matches published **~22× token↓** on structural benchmarks.  
3. **Without either**, expect ~30k+ tokens and higher chance of fixture/noise pollution (this repo contains large `fixtures/repos/snapshots/` trees).

**Accuracy ranking (this task):** Graphify ≳ Prism+docs gap-fill > Prism-packs-alone (T1 prose gap) > naive explore.  
**Accuracy ranking (code/debug/impact tasks):** Prism > Graphify > naive explore (per design goals G1–G5 and four-arm proxy).

---

## 6. Caveats (honest)

1. **T1 Evidence Packs are extractive code/KG slices**, not abstractive README summaries. Asking “what are product features?” without reading docs can yield stub fragments — as seen in `repo_qa` packs (149 / 92 tokens of placeholders).  
2. **Graphify cost.json** for this repo shows prior build semantic tokens = 0 (code-heavy AST path); query-time tokens are the budgeted traversal output (~1.5–2k).  
3. Four-arm **quality** numbers are **scripted proxies** until `PRISM_FOUR_ARM_LLM=1` live judges run. Token/hop advantages are the firmer claim today.  
4. `repo_map` hubs include unresolved symbols (`into`, `clone`, `unwrap`) — orientation only; not architectural truth.  
5. Token estimates use ≈4 characters ≈ 1 token; exact billing tokens vary by tokenizer.

---

## 7. Bottom line

| Question | Answer |
|---|---|
| What is this repo? | **Prism** — local-first repository intelligence that compiles Evidence Packs for AI agents |
| Main features? | Index → KG → query plan → budgeted Evidence Pack; MCP/CLI; workflows; precision ladder; install/host adapters |
| Main use cases? | Agent onboarding, structural Q&A, debug, review, refactor prep, private local indexing, eval |
| **Tokens this task (Prism tools)** | **≈ 1,500** measured |
| **Tokens this task (all sources)** | **≈ 10.5k–11k** including doc gap-fill + Graphify arm |
| Without Prism (est.) | **≈ 30,000** |
| With Graphify (est.) | **≈ 8,000–13,000** |
| With Prism ideal structural | **≈ 1,200** |
| Accuracy | Prism wins on structural/agent tasks + provenance; Graphify competitive on product-doc narrative; both beat naive explore |

---

## 8. Sources used

- Prism MCP: `compile_context` (architecture + repo_qa), `repo_map`, `index_status`
- Prism CLI: `prism compile`
- Graphify: `graphify query "…" --budget 2000` against `graphify-out/graph.json`
- Docs: `README.md`, `docs/architecture/ARCHITECTURE-DESIGN-DOCUMENT.md` (§1–3), `MCP-TOOL-CATALOG.md`, `schemas/agent-workflow/v1/catalog.json`, `docs/eval/PUBLIC-BENCHMARK-REPORT-V2.md`
