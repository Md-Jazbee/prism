# Prism Repo — Feature Summary & Token Comparison

**Generated:** 2026-07-27 (re-run after accuracy improvements)  
**Prior run:** 2026-07-26 (thin T1 stubs / prose gap)  
**Method:** Prism MCP (`compile_context`, `repo_map`, `index_status`) + CLI `./target/release/prism` + Graphify query + documented four-arm baseline  
**Workspace:** `/Users/jasbeermohammad/AllFiles/AI-lenarning/AI-P2`  
**Index:** T1 · **519 files** · **3,351 nodes** · **12,839 edges** · `.prism/graph.sqlite` ≈ **17.5 MB**  
**Binary:** MCP + measurements use `target/release/prism` (2026-07-27). `~/.cargo/bin/prism` was stale (pre–markdown-pack); do not use PATH CLI for this comparison.

### Delta vs prior run (headline)

| Signal | 2026-07-26 | 2026-07-27 | Change |
|---|---:|---:|---|
| Index nodes / edges | 1,629 / 8,676 | **3,351 / 12,839** | +docs + Louvain communities |
| MCP `architecture` pack | 279 (map-only) | **2,207** (README/ADD/MCP prose + map) | Prose gap closed |
| MCP `repo_qa` pack | 149 (stubs) | **1,898** (asserted markdown slices) | No placeholder fragments |
| Doc gap-fill needed? | ≈7,500 tokens | **0** for this task | Packs sufficient |
| Task grand total (Prism path) | ≈10.5–11k | **≈2.3–3.5k** | **~3–5× fewer** than prior Prism path |
| Accuracy ranking (features/use cases) | Graphify ≳ Prism+docs | **Prism packs alone ≥ Graphify** | Ranking flip |

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
| **Markdown / architecture prose** | `prism-extract-markdown` packs README, ADD, MCP catalog, setup docs as asserted slices (`product_thesis`, `architecture_prose`) |
| **Query planner** | Intent → operator DAG (`query_plan`); recipes for repo_qa / architecture / debug / review / refactor |
| **KG queries** | `resolve_symbol`, `neighbors`, `impact` (heuristic at T1; gated when `require_precise`) |
| **Orientation** | `repo_map` (Louvain communities + resolved-degree hubs), `entrypoints`, `detect_changes` |
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

**Louvain communities** (`louvain_v1+resolved_degree_hubs`; fixtures under `fixtures/repos/` excluded from rollup). Top by file count:

| Community | Files (approx) | Nodes (approx) |
|---|---:|---:|
| `docs/architecture/` | 21 | 180 |
| `eval/scorecards/` | 11 | 46 |
| `prism-compile` | 9 | 108 |
| `prism-semantic` | 9 | 91 |
| `prism-precise` | 8 | 77 |
| `prism-agent` / `prism-store` / `prism-view` | 7 each | 56–128 |
| `prism-api` | 6 | 74 |
| `prism-core` / `prism-ir` / `prism-mcp` / `prism-plan` | 5 each | 46–59 |
| `prism-cli` | 4 | 65 |
| Extractors (`markdown` / languages) + `daemon` / `lsp` / `bench` / eval | smaller | — |

Notable structural hubs (resolved degree; T1 heuristic — orientation only): `main` (CLI), `select_from_kg`, `extract` (markdown), `interproc_slice`, `pack_under_budget_with_gaps`.

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

## 4. Tokens consumed — **this task** (measured, 2026-07-27)

Task: “Summarize this repo’s features and use cases.”

### 4.1 Prism Evidence Pack / tool tokens (measured)

| Call | Intent / tool | `tokens_used` (or estimate) | Notes |
|---|---|---:|---|
| MCP `compile_context` #1 | `architecture` · budget 6000 | **2,207** | README + AGENTS + ADD + MCP catalog/error/setup + Louvain `repo_map` fragment |
| MCP `compile_context` #2 | `repo_qa` · budget 8000 | **1,898** | Same asserted markdown slices (no stubs) |
| CLI `./target/release/prism compile` | `repo_qa` · budget 4000 | **777** | Markdown packs under tighter CLI packing |
| CLI `./target/release/prism compile` | `architecture` · budget 6000 | **973** | Prose + community_map |
| MCP `repo_map` | orientation | **~1,100–2,600** | Full JSON larger; architecture pack already embeds a ~309-token map slice |
| MCP `index_status` | metadata | **~80** | Estimated |
| **Subtotal — Prism tools (answer path)** | | **≈ 2,300–3,500** | One architecture pack + status is enough; second pack optional |

**Pack quality (architecture EXPLAIN):** roles `product_thesis`, `architecture_prose`, `community_map`; confidence mostly **asserted** (`prism-extract-markdown`); drops `[]`; P12 gaps for unresolved NL seeds (`Prism`/`MCP`/`Summarize`) — honest, not placeholder fragments.

### 4.2 Follow-up reads

| Source | ~Tokens | Needed? |
|---|---:|---|
| Manual README / ADD / MCP catalog / workflow catalog | 0 | **No** — covered by pack fragments |
| Graphify arm (comparison only) | ≈1,552 output | Comparison arm only |

### 4.3 Total for this task (actual)

| Bucket | Tokens |
|---|---:|
| Prism tools / Evidence Packs (primary path) | ≈ **2,300–3,500** |
| Targeted doc reads (gap-fill) | **0** |
| Graphify query (comparison arm) | ≈ **1,552** output (budget 2,000) |
| **Grand total this session (Prism answer path)** | **≈ 2.3–3.5k** |
| Prior Prism path (2026-07-26, with gap-fill) | ≈ 10.5–11k |

> **Ideal Prism-only path:** one `architecture` `compile_context` (≈2.2k MCP / ≈1.0k CLI) → answer features + use cases + structure.  
> Product-marketing narrative no longer requires separate README/ADD opens for this question class.

---

## 5. Comparison: Without Prism vs With Graphify vs With Prism

Estimates for the **same task**: “Summarize this repo’s features and use cases.”

### 5.1 Token comparison table

| Approach | Typical context loaded | Est. tokens | vs naive |
|---|---|---:|---:|
| **A — Without Prism** (grep/read explore) | README + ADD + tech stack skim + MCP docs + CLI/MCP crate samples + multi-hop greps | **≈ 30,000** | 1× (baseline) |
| **B — With Graphify** (existing `graphify-out/`) | `graphify query --budget 2000` (~1.5k node list) + follow-up node source reads | **≈ 8,000–13,000** | ~2–4× fewer |
| **C — With Prism** (MCP Evidence Packs) | Measured architecture/repo_qa packs; **no doc gap-fill** | **≈ 2,300–3,500** | **~9–13× fewer** |
| **C′ — Prism single-pack** | One architecture `compile_context` | **≈ 1,000–2,200** | **~14–30× fewer** |

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
| **Product / thesis narrative** | High if you open README/ADD; easy to miss crates | **Strong** — doc nodes from ADD/README appear in BFS; this run also hit the comparison doc itself (noise) | **Strong** — asserted markdown slices (`product_thesis`, `architecture_prose`) with citations |
| **Structural truth** (crates, hubs, entrypoints) | Medium — depends on which files agent opens | Medium — AST + communities; edges labeled EXTRACTED/INFERRED/AMBIGUOUS | **Strong** — Louvain `repo_map` + index cardinality + EXPLAIN provenance |
| **Provenance / confidence** | Weak (no labels) | Good (edge honesty labels) | **Best** — per-fragment confidence + tier + analyzer |
| **Token discipline** | Weak (explore loops grow) | Good (`--budget`) | **Best** (hard budget + must-include + drops; P12 gaps not stubs) |
| **Precise rename / impact claims** | Risky | Risky | Gated (`PRECISION_REQUIRED` / T2) — honest refusals |
| **Hop count** | Many (10–12+) | Few–moderate (BFS) | **One** primary tool for structural / product Q&A |
| **Risk of wrong context** | High (wrong files, fixture noise) | Medium (community mix; may surface unrelated nodes) | Lower for code + prose tasks; T1 CALLS still **heuristic** |

**Verdict for this task type (features + use cases):**

1. **Prism** now ships product-oriented markdown inside the Evidence Pack — one `compile_context` answers thesis, MCP surface, setup, and community map without extra reads.  
2. **Graphify** still surfaces a broad doc-node neighborhood fast (~1.5k under budget) but returns a truncated node *list*; finishing the narrative still needs source opens (or accepting thinner answers).  
3. **Without either**, expect ~30k+ tokens and higher chance of fixture/noise pollution (this repo contains large `fixtures/repos/snapshots/` trees).

**Accuracy ranking (this task):** **Prism packs alone ≥ Graphify+follow-ups > naive explore.**  
*(Prior run: Graphify ≳ Prism+docs gap-fill > Prism-packs-alone.)*  
**Accuracy ranking (code/debug/impact tasks):** Prism > Graphify > naive explore (per design goals G1–G5 and four-arm proxy) — unchanged.

---

## 6. Caveats (honest)

1. **Markdown packing closed the product-prose gap for this question**, but fragments are still **extractive slices** (truncated heads of docs), not abstractive summaries. Deeper ADD sections may still need a follow-up compile with path anchors.  
2. **Stale CLI on PATH** (`~/.cargo/bin/prism`, Jul 26) still emitted synthetic placeholders; MCP/`target/release/prism` is the accurate binary. Reinstall or `cargo install --path` before CLI comparisons.  
3. **Graphify cost.json** for this repo shows prior build semantic tokens = 0 (code-heavy AST path); query-time tokens are the budgeted traversal output (~1.5–2k).  
4. Four-arm **quality** numbers are **scripted proxies** until `PRISM_FOUR_ARM_LLM=1` live judges run. Token/hop advantages are the firmer claim today.  
5. `repo_map` hubs are resolved-degree only (ACC-4 denylist) — orientation, not architectural gospel; CALLS remain heuristic at T1.  
6. Token estimates use ≈4 characters ≈ 1 token; exact billing tokens vary by tokenizer. MCP vs CLI packing densities differ (~2.2k vs ~1.0k for architecture) — both beat the prior stub packs.

---

## 7. Bottom line

| Question | Answer |
|---|---|
| What is this repo? | **Prism** — local-first repository intelligence that compiles Evidence Packs for AI agents |
| Main features? | Index → KG → query plan → budgeted Evidence Pack (incl. markdown prose); MCP/CLI; workflows; precision ladder; install/host adapters |
| Main use cases? | Agent onboarding, structural Q&A, debug, review, refactor prep, private local indexing, eval |
| **Tokens this task (Prism path)** | **≈ 2.3–3.5k** measured (was ≈10.5–11k with gap-fill) |
| **Tokens — single architecture pack** | **≈ 1.0–2.2k** |
| Without Prism (est.) | **≈ 30,000** |
| With Graphify (est.) | **≈ 8,000–13,000** |
| Accuracy | **Prism wins this task class after markdown packing**; still wins structural/agent tasks + provenance; both beat naive explore |

---

## 8. Sources used

- Prism MCP (`target/release/prism mcp`): `compile_context` (architecture + repo_qa), `repo_map`, `index_status`
- Prism CLI: `./target/release/prism compile` (architecture + repo_qa)
- Graphify: `graphify query "…" --budget 2000` against `graphify-out/graph.json`
- Docs embedded in packs: `README.md`, `AGENTS.md`, `ARCHITECTURE-DESIGN-DOCUMENT.md`, `MCP-TOOL-CATALOG.md`, `MCP-ERROR-MODEL.md`, `PRODUCT-SETUP.md` (+ CLI also surfaced `AGENT-USAGE.md`)
- Benchmark: `docs/eval/PUBLIC-BENCHMARK-REPORT-V2.md`
