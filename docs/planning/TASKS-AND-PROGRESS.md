# Prism — Tasks & Progress Board

**Status date:** 2026-07-26  
**Current phase:** **P5 Stage C open** · Stage B (hardening/SDK/IDE) exited · LLM quality / dual-review labels still pending  
**Source of truth for design order:** [PLANNING-AND-IMPLEMENTATION.md](./PLANNING-AND-IMPLEMENTATION.md)  
**Source of truth for architecture:** [ARCHITECTURE-DESIGN-DOCUMENT.md](../architecture/ARCHITECTURE-DESIGN-DOCUMENT.md)

Use this file as the living checklist. Update checkbox state and the progress snapshot when a stage exits or a blocker moves.

---

## Progress snapshot

| Phase | Intent | Progress | State |
|---:|---|---:|---|
| **P0** | Foundations (identity, hash, schemas, eval) | ▓▓▓▓▓▓▓▓▓▓ **100%** | ✅ Gate passed 2026-07-25 |
| **P1** | Syntactic KG + MCP | ▓▓▓▓▓▓▓▓▓▓ **100%** | ✅ Gate passed 2026-07-25 (proxies) |
| **P2** | Context Compiler | ▓▓▓▓▓▓▓▓▓▓ **100%** | ✅ Gate passed 2026-07-26 (proxies) |
| **P3** | Precise Tier (T2) | ▓▓▓▓▓▓▓▓▓▓ **100%** | ✅ Gate passed 2026-07-26 |
| **P4** | Semantic Slicing | ▓▓▓▓▓▓▓▓▓▓ **100%** | ✅ Gate passed 2026-07-26 |
| **P5** | Repo Intelligence + Hardening | ▓▓▓▓▓▓░░░░ **67%** | 🟡 Stage C open |
| **P6** | Team / Distributed (optional) | ░░░░░░░░░░ **0%** | ⚪ Deferred |

**How to read progress:** P5 % ≈ stages done ÷ 3 (A/B/C). Later phases stay at 0% until their entry gate passes.

```mermaid
flowchart LR
    P0[P0 Foundations<br/>✅ done] --> P1[P1 Syntactic KG + MCP<br/>✅ done]
    P1 --> P2[P2 Context Compiler<br/>✅ done]
    P2 --> P3[P3 Precise Tier<br/>✅ done]
    P3 --> P4[P4 Semantic Slicing<br/>✅ done]
    P4 --> P5[P5 Intelligence + Eval<br/>🟡 Stage C]
    P5 --> P6[P6 Distributed / Team<br/>optional]

    style P0 fill:#b8e994,stroke:#78e08f,color:#000
    style P1 fill:#b8e994,stroke:#78e08f,color:#000
    style P2 fill:#b8e994,stroke:#78e08f,color:#000
    style P3 fill:#b8e994,stroke:#78e08f,color:#000
    style P4 fill:#b8e994,stroke:#78e08f,color:#000
    style P5 fill:#f6e58d,stroke:#f9ca24,color:#000
    style P6 fill:#dfe6e9,stroke:#b2bec3,color:#000
```

### Legend

| Mark | Meaning |
|---|---|
| ✅ | Done / accepted |
| 🟡 | In progress / stub exists |
| ⬜ | Not started |
| 🚫 | Blocked (see notes) |
| ⚪ | Phase not open yet |

---

## Capability maturity (today)

| Capability | P0 | P1 | P2 | P3 | P4 | P5 | P6 | Today |
|---|---|---|---|---|---|---|---|---|
| Content-hash incremental store | ● | ● | ● | ● | ● | ● | ● | ✅ live; measured on pilots |
| Syntactic facts (T1) | ○ | ● | ● | ● | ● | ● | ● | ✅ Python + Rust extractors + goldens |
| MCP graph tools | ○ | ● | ● | ● | ● | ● | ● | ✅ prism-mcp stdio tools |
| Query plan + Evidence Pack | ○ | ○ | ● | ● | ● | ● | ● | ✅ plan + pack + EXPLAIN + MCP `compile_context` |
| Precise symbol (T2) | ○ | ○ | ○ | ● | ● | ● | ● | ✅ gated product path |
| Semantic slice (T3/T4) | ○ | ○ | ○ | ○ | ● | ● | ● | ✅ debug packs slice-minimal (gate proxies) |
| Architecture intelligence | ○ | ◐ | ◐ | ◐ | ◐ | ● | ● | 🟡 communities+hubs+entrypoints+hotspots |
| Team/shared index | ○ | ○ | ○ | ○ | ○ | ○ | ● | ⬜ |

● required · ◐ partial · ○ not yet

---

## Open blockers (P0 → P1)

| # | Item | Blocks | Owner / notes |
|---|---|---|---|
| 1 | ~~Freeze pilot repo commit SHAs~~ | — | ✅ 2026-07-25 · httpx `b5addb6`, ripgrep `f9c05a9`; all 22 tasks pinned |
| 2 | ~~Measure index size on pilot repos after cold walk~~ | — | ✅ httpx 124 files / 88K; ripgrep 236 files / 136K (see checklist) |
| 3 | ~~Architect sign-off on schema/ABI~~ | — | ✅ recorded via P0-exit commit review (solo mode) |
| 4 | Fill gold hints / necessary spans on tasks | Eval scoring quality (non-blocking) | 🟡 first batch T001–T011 filled; T012–T022 remain |

---

## Phase 0 — Foundations (detailed)

**Goal:** Workspace identity, incremental hashing, durable schemas, plugin contracts, and a measurable eval path — *before* intelligence features.  
**Duration:** 2–3 weeks  
**Phase gate:** Incremental re-index path prototype-validatable; metrics pipeline exists; ≥20 gold tasks versioned to commit SHAs.

```mermaid
flowchart LR
    A[Stage A<br/>Workspace + Fingerprint] --> B[Stage B<br/>Schema + Plugin ABI]
    B --> C[Stage C<br/>Eval + Observability]
    C --> G[[P0 Gate → P1]]
```

| Stage | Focus | Progress | State |
|---|---|---:|---|
| **A** | Workspace identity & fingerprinting | 100% | ✅ Exited 2026-07-25 |
| **B** | Durable schema & plugin ABI | 100% | ✅ Exited 2026-07-25 |
| **C** | Eval skeleton & observability | 100% | ✅ Exited 2026-07-25 |

### Stage A — Workspace identity & content fingerprinting

#### Deliverables

- [x] Workspace Manager (`crates/prism-core` / `workspace.rs`) — roots, git SHA, dirty stamp
- [x] Fingerprint algorithm note — [FINGERPRINT.md](../architecture/FINGERPRINT.md) (XXH3-128 + tree Merkle)
- [x] `.prism/` layout (meta, graph, blobs, logs) created by indexer
- [x] Incremental invalidation design (per-file skip when hash matches)
- [x] Ignore + secret policy (`ignore_policy.rs`) + [IGNORE-POLICY-CHECKLIST.md](../architecture/IGNORE-POLICY-CHECKLIST.md)
- [x] Pilot repos listed — [fixtures/repos/README.md](../../fixtures/repos/README.md) (httpx, ripgrep)
- [x] CLI `doctor` + `index` / `index --dry-run`

#### Exit / acceptance

- [x] Documented algorithm classifies edit as changed/unchanged given hashes
- [x] Dirty worktree vs clean commit identities distinguishable
- [x] Ignore policy review checklist exists
- [x] Pilot repos listed with approx LOC and languages
- [x] Ignore checklist fully ticked (index size measured on pilots 2026-07-25)

#### Key code / docs

| Artifact | Path |
|---|---|
| Workspace + identity | `crates/prism-core/src/workspace.rs` |
| Fingerprint | `crates/prism-core/src/fingerprint.rs` |
| Ignore / secrets | `crates/prism-core/src/ignore_policy.rs` |
| CLI | `crates/prism-cli/src/main.rs` |

---

### Stage B — Durable schema & plugin ABI draft

#### Deliverables

- [x] Schema v0 — `schemas/meta/v0/meta.schema.json`
- [x] Fact schema v0 — `schemas/fact-schema/v0/fact.schema.json`
- [x] Events schema v0 — `schemas/events/v0/events.schema.json`
- [x] Plugin ABI draft — `schemas/plugins/LanguageExtractor.md`
- [x] IR types — confidence, IDs, schema version constants (`prism-ir`)
- [x] `meta.sqlite` store (WAL upsert / skip-unchanged) — `prism-store`
- [x] `graph.sqlite` adjacency stub + replace/invalidate — `prism-store`
- [x] Formal architect “schema review signed” note — recorded in P0-exit commit (solo mode)

#### Exit / acceptance

- [x] Crash-safe write strategy chosen (SQLite WAL transactional replaces) — implemented + unit tested
- [x] Confidence values include `extracted` / `heuristic` / `precise` / `observed`
- [x] Plugin ABI reviewable without reading the whole ADD
- [x] Migration policy: breaking fact schema bumps major version (documented in schemas / IR versions)
- [x] Explicit sign-off recorded (P0-exit commit, 2026-07-25)

#### Key code / docs

| Artifact | Path |
|---|---|
| Meta store | `crates/prism-store/src/meta.rs` |
| KG stub | `crates/prism-store/src/kg.rs` |
| Confidence / IDs | `crates/prism-ir/src/` |
| Plugin contract | `schemas/plugins/LanguageExtractor.md` |

---

### Stage C — Evaluation skeleton & observability baseline

#### Deliverables

- [x] Eval harness package — `eval/` (`uv` + `prism-eval smoke`)
- [x] Gold task pack v0 — **22** tasks in `eval/tasks/T001.json` … `T022.json` (≥20 required)
- [x] Scorecard template — `eval/scorecards/templates/scorecard.md`
- [x] Baseline runbook notes — `eval/baselines/` + eval README (“How we know P1 saved tokens”)
- [x] Named event schema + emit path — `prism-obs` + `schemas/events/v0`
- [x] Incremental path end-to-end stub: discover → hash → parse-hook → txn → invalidate
- [x] Pilot SHAs frozen — httpx `b5addb64…`, ripgrep `f9c05a94…` (all 22 tasks)
- [x] Gold hints first batch (T001–T011) filled from frozen snapshots · 🟡 T012–T022 pending (non-blocking)

#### Exit / acceptance (Phase 0 gate)

- [x] Incremental re-index path specified & implemented end-to-end (parsers stubbed)
- [x] Metrics pipeline exists with named event schema
- [x] ≥20 gold tasks versioned (files present)
- [x] Tasks tied to **real** commit SHAs (pinned 2026-07-25)
- [x] Written procedure for “How will we know P1 saved tokens?” — `eval/README.md`

#### Key code / docs

| Artifact | Path |
|---|---|
| Incremental indexer | `crates/prism-core/src/incremental.rs` |
| Index events | `crates/prism-obs/src/events.rs` |
| Harness | `eval/harness/runner.py` |
| Tasks | `eval/tasks/` |
| Fixtures | `fixtures/repos/` |

---

### Phase 0 — punch list (✅ completed 2026-07-25)

1. [x] Clone httpx + ripgrep at known-good commits; SHA/date/license recorded in `fixtures/repos/*.md` (snapshots gitignored, reproducible by SHA)
2. [x] Replace `PIN_ME` across `eval/tasks/*.json` (22/22 pinned)
3. [x] Cold-walk pilots — httpx 124 files 91ms/88K; ripgrep 236 files 110ms/136K; warm walk skips 100%
4. [x] [IGNORE-POLICY-CHECKLIST.md](../architecture/IGNORE-POLICY-CHECKLIST.md) fully ticked with measurements
5. [x] Gold hints first batch T001–T011 (real symbols/spans from frozen SHAs; T008 question corrected `GrepWalker` → ignore crate `Walk`/`WalkBuilder`)
6. [x] Stage A/B/C exits marked; Phase 1 Stage A opened below

**Suggested P0 exit command smoke:**

```bash
cargo test --workspace
cargo run -p prism-cli -- doctor .
cargo run -p prism-cli -- index . --dry-run
cd eval && uv sync && uv run prism-eval smoke
```

---

## Phase 1 — Syntactic Knowledge Graph + MCP

**State:** ✅ **Gate passed 2026-07-25** (structural hop/token proxies; LLM quality pending)  
**Duration:** 4–6 weeks · **Languages:** Python + Rust  
**Gate evidence:** [eval/scorecards/p1-phase-gate.md](../../eval/scorecards/p1-phase-gate.md)

| Stage | Tasks (summary) | Status |
|---|---|---|
| **A — T1 extractors** | Per-language design docs; symbols/imports/heuristic CALLS; golden fact fixtures; unresolved edges first-class | ✅ exited 2026-07-25 |
| **B — KG persist + query** | Fact persist; neighbors/resolve/impact API; reverse-dep dirty lists; size budget + failure docs | ✅ exited 2026-07-25 |
| **C — MCP tools** | `index_status`, `resolve_symbol`, `neighbors`, `impact`, `repo_map`; safety + error model | ✅ exited 2026-07-25 |
| **D — Communities + gate** | Path-prefix communities/hubs; Phase 1 scorecard; limitations register | ✅ exited 2026-07-25 |

**Phase 1 kickoff checklist:**

- [x] Freeze LanguageExtractor ABI + fact schema versions used by extractors (ABI frozen; `FACT_SCHEMA_VERSION` 0.0.1)
- [x] Pick first 2 languages + fixture repos — **Python (httpx `b5addb6`) + Rust (ripgrep `f9c05a9`)**
- [x] Stand up golden-fixture conformance harness (`fixtures/languages/` + crate tests)
- [x] Define MCP tool JSON schemas + refusal behaviors (Stage C)

### Stage A — Language extractors (T1) ✅

#### Deliverables

- [x] Extractor design docs — [python.md](../architecture/extractors/python.md), [rust.md](../architecture/extractors/rust.md)
- [x] Golden fact fixtures — `fixtures/languages/python/`, `fixtures/languages/rust/`
- [x] Resolution-cheap policy — same-file + unresolved first-class (ABI + extractors)
- [x] Crates — `prism-extract`, `prism-extract-python`, `prism-extract-rust`
- [x] Fact IR — `prism-ir::facts` (`FactBundle`, kinds, spans)
- [x] Indexer wired — parse-hook → extract → `KgStore::insert_facts` + `FileExtracted` events

#### Exit / acceptance

- [x] Each language has golden fixtures passing conformance tests
- [x] Unresolved edges are first-class, not silent deletes
- [x] Extractor docs state known failure modes (dynamic imports, macros, generics)

#### Handoff to Stage B

Fact producers live; Stage B owns query API (`neighbors` / `resolve`), reverse-dep dirty lists, and index-size budget note. Thin persist already writes nodes/edges during replace.

### Stage B — KG persistence & query API ✅

#### Deliverables

- [x] KG query API — [KG-QUERY-API.md](../architecture/KG-QUERY-API.md); `resolve` / `neighbors` / `impact` / `dirty` in `prism-store` + CLI
- [x] Incremental update sequence — [INCREMENTAL-UPDATE.md](../architecture/INCREMENTAL-UPDATE.md)
- [x] Index size budget note — [INDEX-SIZE-BUDGET.md](../architecture/INDEX-SIZE-BUDGET.md); `prism index-status`
- [x] Failure modes — [KG-FAILURE-MODES.md](../architecture/KG-FAILURE-MODES.md)
- [x] Reverse-dep dirty lists — `SqliteKgStore::reverse_dep_files` / `dirty_set_for_paths`
- [x] Query latency tracking — `query_finished` obs event

#### Exit / acceptance

- [x] Documented that a single-file edit does not require full rebuild
- [x] Query API can express: symbol lookup, 1-hop neighbors, depth-limited impact candidates
- [x] Latency/size NFRs are tracked (even if not yet met)

#### Handoff to Stage C

CLI query surface is the contract MCP tools should mirror (`index_status`, `resolve_symbol`, `neighbors`, `impact`). Bind via MCP stdio next; keep confidence fields and heuristic labeling.

### Stage C — MCP structural tools ✅

#### Deliverables

- [x] MCP tool catalog — [MCP-TOOL-CATALOG.md](../architecture/MCP-TOOL-CATALOG.md)
- [x] Agent usage guide — [AGENT-USAGE.md](../architecture/AGENT-USAGE.md)
- [x] Error model — [MCP-ERROR-MODEL.md](../architecture/MCP-ERROR-MODEL.md) (`SCOPE_UNRESOLVED`)
- [x] Crate `prism-mcp` + CLI `prism mcp` (stdio JSON-RPC)
- [x] Tools: `index_status`, `resolve_symbol`, `neighbors`, `impact`, `repo_map`
- [x] Eval tool-hop recording — `prism-eval tool-hops`

#### Exit / acceptance

- [x] Tool catalog reviewed against ADD §25 subset for P1
- [x] Every tool return includes provenance/confidence or marks heuristics
- [x] Eval harness can record tool hops per task

### Stage D — Communities + Phase 1 gate ✅

#### Deliverables

- [x] Community design — [COMMUNITIES.md](../architecture/COMMUNITIES.md) (path-prefix + degree hubs)
- [x] Phase 1 scorecard — [p1-phase-gate.md](../../eval/scorecards/p1-phase-gate.md)
- [x] Known limitations — [P1-KNOWN-LIMITATIONS.md](../architecture/P1-KNOWN-LIMITATIONS.md)
- [x] `repo_map` MCP/CLI wired to communities

#### Exit / acceptance (Phase 1 gate)

- [x] ≥5× token reduction on structural subset (**proxy** 21.7×; replace with live baselines when ready)
- [x] ≥5× hop reduction proxy (5.42×)
- [ ] Quality within ~10 pts of explore — **PENDING** LLM explore baselines
- [x] Incremental edit path demonstrated (hash skip + file subgraph replace; unit-tested)
- [x] No narrative claiming precise refactor safety

#### Handoff to Phase 2

MCP structural tools are live. Next: intent recipes + `compile_context` Evidence Packs (P2). Treat quality gate as open until measured LLM scorecards land.

---

## Phase 2 — Context Compiler

**State:** ✅ **Gate passed 2026-07-26** (precision proxies; dual-review labels pending)  
**Duration:** 3–5 weeks  
**Gate evidence:** [eval/scorecards/p2-phase-gate.md](../../eval/scorecards/p2-phase-gate.md)

| Stage | Tasks (summary) | Status |
|---|---|---|
| **A — Intent + planner** | Intent recipes; operator DAG; cost model v1; plan-only API | ✅ exited 2026-07-26 |
| **B — Pack + budget** | Selection/reduction; Evidence Pack IR; EXPLAIN; `BUDGET_EXCEEDED` | ✅ exited 2026-07-26 |
| **C — `compile_context`** | Primary MCP tool; Phase 2 scorecard (precision, tokens, hops, refuse-dump) | ✅ exited 2026-07-26 |

### Stage A — Intent classification & query planner ✅

#### Deliverables

- [x] Intent recipe catalog v1 — [INTENT-RECIPES.md](../architecture/INTENT-RECIPES.md) + `schemas/plugins/IntentRecipe.md`
- [x] Planner design — [QUERY-PLANNER.md](../architecture/QUERY-PLANNER.md); operator catalog + cost sketch
- [x] Plan IR schema — `schemas/plan/v0/plan.schema.json` + `PLAN_SCHEMA_VERSION`
- [x] Crate `prism-plan` — classify → recipe → `Plan` / `SCOPE_UNRESOLVED` (no LLM)
- [x] Example plans — fixtures `fixtures/plans/{debug,impact,repo_qa,…}/`
- [x] Plan-only API — CLI `prism query plan` (HTTP `POST /v1/query/plan` contracted)

#### Exit / acceptance

- [x] For each intent, a recipe produces a plan without executing LLM
- [x] Fixtures cover ambiguous queries → `SCOPE_UNRESOLVED`
- [x] Plan-only API (`/query/plan`) contract documented

#### Handoff to Stage B

Plans are stable JSON. Stage B owns Evidence Pack IR, must-include enforcement under budget, reduction techniques, and EXPLAIN reason codes. Do not promote `compile_context` MCP until Stage C.

### Stage B — Selection, reduction & Evidence Pack ✅

#### Deliverables

- [x] Evidence Pack schema v0 — `schemas/evidence-pack/v0` + [EVIDENCE-PACK.md](../architecture/EVIDENCE-PACK.md)
- [x] Selection priority — [SELECTION-PRIORITY.md](../architecture/SELECTION-PRIORITY.md)
- [x] Reduction catalog — [REDUCTION.md](../architecture/REDUCTION.md)
- [x] Crate `prism-compile` — select → budget pack → EXPLAIN; `BUDGET_EXCEEDED`
- [x] Must-include invariant tests + `fixtures/packs/`
- [x] Labeling process — [eval/labeling/README.md](../../eval/labeling/README.md)
- [x] CLI `prism compile` (+ `--synthetic`)

#### Exit / acceptance

- [x] Pack schema round-trips through EXPLAIN report
- [x] Written proof must-include cannot be budget-evicted (test + budget_drop fixture)
- [x] Labeled sample process documented

#### Handoff to Stage C

`prism compile` produces packs. Stage C promotes MCP `compile_context`, agent guidance (“call compile first”), Phase 2 scorecard (precision ≥60%, refuse-dump, latency tracking).

### Stage C — `compile_context` primary + Phase 2 gate ✅

#### Deliverables

- [x] MCP `compile_context` + `query_plan` — `prism-mcp` allowlist; server instructions
- [x] Primary-path guide — [AGENT-USAGE.md](../architecture/AGENT-USAGE.md)
- [x] EXPLAIN format — [EXPLAIN.md](../architecture/EXPLAIN.md)
- [x] IDE peek stub — [IDE-EVIDENCE-PEEK.md](../architecture/IDE-EVIDENCE-PEEK.md)
- [x] Phase 2 scorecard — [p2-phase-gate.md](../../eval/scorecards/p2-phase-gate.md) + `prism-eval p2-scorecard`
- [x] Refuse-dump fixture — `fixtures/packs/refuse-dump/`
- [x] Precision sample labels — `eval/labeling/packs/` (proxy-v0)
- [x] `BUDGET_EXCEEDED` in MCP error model

#### Exit / acceptance (Phase 2 gate)

- [x] Context precision ≥60% on labeled sample (**proxy** labels; dual review pending)
- [x] Unresolved scope → refuse unbounded dump (fixtures + MCP test)
- [x] `compile_context` documented as preferred tool over ten reads
- [x] Pack compile latency budget tracked toward &lt;300ms P95 (`mcp:compile_context` / `compile` events)
- [x] Provenance present on every fragment (enforced in MCP tool)

#### Handoff to Phase 3

Evidence Packs are the primary agent path. Next: precise tier (SCIP/LSP) so high-stakes impact/refactor can upgrade heuristic `CALLS`. Treat precision gate as open until dual-reviewed labels land.

---

## Phase 3 — Precise Tier

**State:** ✅ **Gate passed 2026-07-26** (oracle P/R uplift; dry-run rename; accuracy gating)  
**Duration:** 4–6 weeks  
**Gate evidence:** [eval/scorecards/p3-phase-gate.md](../../eval/scorecards/p3-phase-gate.md)

| Stage | Tasks (summary) | Status |
|---|---|---|
| **A — Precise ingest** | SCIP/PreciseIndex import; tier-tagged edges; oracle P/R; `PRECISION_REQUIRED` | ✅ exited 2026-07-26 |
| **B — Hybrid resolve** | Prefer T2 over heuristic CALLS; on-demand upgrade | ✅ exited 2026-07-26 |
| **C — Product behaviors** | Precision-gated impact/rename; Phase 3 scorecard | ✅ exited 2026-07-26 |

### Stage A — Precise index ingest ✅

#### Deliverables

- [x] Precise tier integration design — [PRECISE-TIER.md](../architecture/PRECISE-TIER.md)
- [x] ID mapping rules — [ID-MAPPING.md](../architecture/ID-MAPPING.md)
- [x] Build/index prerequisites runbook — [SCIP-RUNBOOK.md](../architecture/SCIP-RUNBOOK.md) + `scripts/scip/`
- [x] PreciseIndex schema v0 — `schemas/precise-index/v0` + plugin card `PreciseImporter.md`
- [x] Crate `prism-precise` — import → T2 facts → edge refine → P/R score
- [x] Store overlay upserts — `upsert_overlay_*` / refine heuristic CALLS
- [x] CLI `prism precise import|status|score`
- [x] Oracle fixtures — `fixtures/precise/oracle/python/` (T2 precision↑ vs T1)
- [x] `PRECISION_REQUIRED` — MCP error model + `precise status`

#### Exit / acceptance

- [x] Import path attaches precise defs/refs for Python end-to-end (PreciseIndex → KG)
- [x] Fixtures define precision/recall measurement vs oracle
- [x] Failure mode when SCIP/overlay missing is clear (`PRECISION_REQUIRED`)

#### Handoff to Stage B

Precise overlays attach optionally under `.prism/scip/`. Next: hybrid resolver + planner `UpgradePrecision` so high-stakes intents prefer T2 on critical edges without always paying for full indexes.

### Stage B — Hybrid resolve & on-demand upgrade ✅

#### Deliverables

- [x] Hybrid resolution algorithm — [HYBRID-RESOLVE.md](../architecture/HYBRID-RESOLVE.md) + `prism-precise::hybrid_resolve`
- [x] Planner upgrade policies — [UPGRADE-POLICY.md](../architecture/UPGRADE-POLICY.md); `UpgradePrecision` executable
- [x] Ambiguity index — [AMBIGUITY-INDEX.md](../architecture/AMBIGUITY-INDEX.md); CLI `prism precise ambiguity`
- [x] Impact recipe inserts optional upgrade; refactor/debug mandatory
- [x] Compiler prefers precise fragments; dual-candidate uncertainty notes
- [x] Obs `precision_upgrade` event (confirmed / deferred / latency)

#### Exit / acceptance

- [x] Documented policy: high-stakes intents prefer T2 on critical path
- [x] Latency cost of upgrade bounded (≤200ms / 32 edges; excess deferred)

#### Handoff to Stage C

Hybrid resolve runs on the compile path. Next: precision-gated product behaviors (safe-rename dry-run, gating matrix, Phase 3 scorecard) so accuracy claims require T2 when available.

### Stage C — Precision-gated product behaviors + Phase 3 gate ✅

#### Deliverables

- [x] Gating matrix — [PRECISION-GATING.md](../architecture/PRECISION-GATING.md)
- [x] Safe rename dry-run — [SAFE-RENAME-DRY-RUN.md](../architecture/SAFE-RENAME-DRY-RUN.md) + `scripts/precise/safe-rename-dry-run.sh` + CLI
- [x] Phase 3 scorecard — [p3-phase-gate.md](../../eval/scorecards/p3-phase-gate.md) + `prism-eval p3-scorecard`
- [x] `require_precise` on MCP/CLI impact + compile_context accuracy claims
- [x] Heuristic answers remain labeled (never silent upgrade)

#### Exit / acceptance (Phase 3 gate)

- [x] Call resolution precision↑ ≥20pp vs T1 on oracle (Δ=+50pp; T1 0.50 → T2 1.00)
- [x] Refactor/impact paths document T2 requirement when available
- [x] Dry-run rename demo exists (read-only)
- [x] Heuristic answers remain labeled; never silently upgraded

#### Handoff to Phase 4

Precise IDs and gated claims are live. Next: semantic slicing (T3 CFG/DFG) for debug minimum-sufficient slices.

---

## Phase 4 — Semantic Slicing

**State:** ✅ **Gate passed 2026-07-26** (debug token↓ proxy; pack gates; runtime optional)  
**Duration:** 5–8 weeks  
**Gate evidence:** [eval/scorecards/p4-phase-gate.md](../../eval/scorecards/p4-phase-gate.md)

| Stage | Tasks (summary) | Status |
|---|---|---|
| **A — Intra-procedural (T3)** | CFG/DFG shards; criteria for local slices | ✅ exited 2026-07-26 |
| **B — Inter-procedural (T4)** | CPG shards; slice operator; shard budgets | ✅ exited 2026-07-26 |
| **C — Debug recipes + gate** | Wire debug intents; optional runtime enrichment; Phase 4 scorecard | ✅ exited 2026-07-26 |

### Stage A — Intra-procedural control/data flow (T3) ✅

#### Deliverables

- [x] T3 analysis design — [T3-ANALYSIS.md](../architecture/T3-ANALYSIS.md)
- [x] Semantic artifact layout — [SEMANTIC-ARTIFACTS.md](../architecture/SEMANTIC-ARTIFACTS.md) (`.prism/semantic/`)
- [x] Crash policy — [SEMANTIC-CRASH-POLICY.md](../architecture/SEMANTIC-CRASH-POLICY.md)
- [x] Schema v0 — `schemas/semantic-artifact/v0`
- [x] Crate `prism-semantic` — Python CFG/DFG + `local_slice` + store
- [x] CLI `prism semantic build|slice|status`
- [x] Fixture — `fixtures/slices/python/sample.py`
- [x] Property tests — criterion-in-slice + idempotent re-slice; no panic on broken source

#### Exit / acceptance

- [x] Local slice operator specified for symbol/line criteria (`local_slice` / CLI `--file` + `--line`)
- [x] Property-based acceptance tests defined (and green in `prism-semantic`)

#### Handoff to Stage B

Intra-proc Python slices land under `.prism/semantic/` (not hot `graph.sqlite`). Next: inter-procedural shards, first-class `Slice` planner operator, depth caps + residual expand.

### Stage B — Inter-procedural CPG shards & slice operator (T4) ✅

#### Deliverables

- [x] Lazy sharding strategy — [T4-SHARDING.md](../architecture/T4-SHARDING.md)
- [x] Slice operator contract — [SLICE-OPERATOR.md](../architecture/SLICE-OPERATOR.md)
- [x] Sink/source provider hooks — [SINK-SOURCE-HOOKS.md](../architecture/SINK-SOURCE-HOOKS.md)
- [x] Overlay `CALLS` / `DATA_FLOW` / `CONTROL_DEP` in semantic shards
- [x] Executable planner `Slice` + compile selection + `slice_finished` obs
- [x] Memo keys `(snapshot_id, algorithm_version, params_hash)`
- [x] CLI `prism semantic slice` (interproc) + `shard-build`
- [x] Dirty-path shard invalidation + property tests

#### Exit / acceptance

- [x] Shard rebuild is on-demand / dirty subsets only
- [x] Slice returns minimal spans with provenance
- [x] Memoization keys include `(snapshot_id, algorithm_version, params_hash)`

#### Handoff to Stage C

`Slice` is live on the debug recipe path. Next: tighten debug/security recipes, optional runtime enrichment design, and Phase 4 scorecard (≥5× debug token↓).

### Stage C — Debug recipes + Phase 4 gate ✅

#### Deliverables

- [x] Debug / security recipes — [DEBUG-RECIPES.md](../architecture/DEBUG-RECIPES.md)
- [x] Debug pack quality gates — [DEBUG-PACK-GATES.md](../architecture/DEBUG-PACK-GATES.md) + budget protected roles
- [x] Optional runtime enrichment design — [RUNTIME-ENRICHMENT.md](../architecture/RUNTIME-ENRICHMENT.md)
- [x] Debug gold tasks — `eval/tasks/T013.json`, `T022.json`
- [x] Phase 4 scorecard — [p4-phase-gate.md](../../eval/scorecards/p4-phase-gate.md) + `prism-eval p4-scorecard`
- [x] AGENT-USAGE updated for debug/slice primary path

#### Exit / acceptance (phase gate)

- [x] Debug token↓ ≥5× vs explore (proxy **40×**)
- [x] Quality proxy (necessary_spans); LLM within-5pts pending
- [x] Slice + error/stack never dropped under budget pressure
- [x] Runtime not required

#### Handoff to Phase 5

Debug packs are slice-minimal on the agent path. Next: repository intelligence products (hubs, entrypoints, hotspots) and hardening/SDK.

---

## Phase 5 — Repository Intelligence + Hardening

**State:** 🟡 **Stage C open** (Stage B exited 2026-07-26)  
**Duration:** ~4 weeks  
**Gate:** Published benchmark; medium+Prism ≈ frontier+explore within 3 pts; external plugin SDK usable.

| Stage | Tasks (summary) | Status |
|---|---|---|
| **A — Repo intelligence** | Architecture maps, communities productized, orientation answers | ✅ exited 2026-07-26 |
| **B — Hardening + SDK + IDE** | Security checklist, plugin SDK polish, LSP/IDE commands | ✅ exited 2026-07-26 |
| **C — Public eval** | Published scorecard; release readiness | 🟡 open |

### Stage A — Repository intelligence products ✅

#### Deliverables

- [x] Derived intelligence catalog — [REPO-INTELLIGENCE.md](../architecture/REPO-INTELLIGENCE.md)
- [x] Refresh/invalidation rules — [INTEL-REFRESH.md](../architecture/INTEL-REFRESH.md)
- [x] Ambiguity auto-T2 usage — [AMBIGUITY-INDEX.md](../architecture/AMBIGUITY-INDEX.md)
- [x] Entrypoints / layering / hotspots / contracts — `prism-store::intel`
- [x] MCP `entrypoints`, `detect_changes`, `repo_map.full_intel`
- [x] CLI `prism query intel|entrypoints|detect-changes|repo-map --full`
- [x] Architecture packs include tiny entrypoint list

#### Exit / acceptance

- [x] Each derived product has method + confidence notes
- [x] LLM naming of communities not required (path-prefix labels)

#### Handoff to Stage B

Orientation intel is productized. Next: hardening, plugin SDK contributor path, IDE commands, security checklist.

### Stage B — Hardening, plugin SDK, IDE ✅

#### Deliverables

- [x] Contributor plugin guide — [docs/contributing/plugin-guide.md](../contributing/plugin-guide.md)
- [x] Security release checklist — [docs/security/RELEASE-CHECKLIST.md](../security/RELEASE-CHECKLIST.md)
- [x] Audit + redaction — [docs/security/AUDIT-AND-REDACTION.md](../security/AUDIT-AND-REDACTION.md) + `SECURITY.md`
- [x] IDE integration design — [IDE-INTEGRATION.md](../architecture/IDE-INTEGRATION.md)
- [x] Test matrix — [TEST-MATRIX.md](../architecture/TEST-MATRIX.md)
- [x] Pack stability property — [PACK-STABILITY.md](../architecture/PACK-STABILITY.md) + unit test
- [x] Conformance script — `scripts/plugins/conformance-check.sh` (CI)
- [x] OTel span design + `pack_bound_for_llm` / `token_savings_shadow` events

#### Exit / acceptance

- [x] Documented path to add a language via ABI + goldens (no core engine changes)
- [x] Audit + redaction policies written
- [x] Pack stability specified and tested

#### Handoff to Stage C

Hardening docs and SDK path are ready. Next: public eval report, release readiness, Phase 5 gate.

---

## Phase 6 — Team / Distributed — optional (outline)

**State:** ⚪ Deferred  
**Gate:** Two developers share an index safely; CI freshness SLA defined and met.

| Stage | Tasks (summary) | Status |
|---|---|---|
| **A — Shared index server** | Read-mostly shared store; authz baseline | ⬜ |
| **B — CI publishers** | Freshness SLAs; publish jobs | ⬜ |
| **C — Certified caches** | Optional memoization **with** dependency certificates only | ⬜ |

---

## Cross-cutting workstreams (always on)

Track these every phase; each phase exit must refresh **W-EVAL** and **W-OBS**.

| ID | Workstream | P4 gate exit status |
|---|---|---|
| **W-STORE** | Storage & identity | ✅ + `.prism/semantic/` shards/memo |
| **W-PLUGIN** | Plugin ABI | ✅ + contributor plugin guide + conformance CI |
| **W-KG** | Knowledge graph | ✅ + overlay DATA_FLOW/CONTROL_DEP |
| **W-PLAN** | Query planning | ✅ executable `Slice` on debug |
| **W-CC** | Context compiler | ✅ debug pack gates (never drop error/slice) |
| **W-MCP** | Agent surface | ✅ AGENT-USAGE debug path |
| **W-IDE** | IDE/LSP | ✅ IDE-INTEGRATION design (extension phased) |
| **W-EVAL** | Evaluation | ✅ P4 scorecard; P5 public report pending |
| **W-OBS** | Observability | ✅ `pack_bound_for_llm` + OTel span design |
| **W-SEC** | Security & privacy | ✅ release checklist + audit/redaction |

---

## Definition of Done reminders

**Stage exit:** entry criteria met, deliverables reviewed, exit checkboxes ticked, handoff written in this file.  
**Phase exit:** phase gate metrics measured (or design-validated for P0), risks waived in writing if skipped.  
**Program (P0–P5):** north-star claims G1–G9 evidenced; see planning doc §15.

---

## Changelog (keep short)

| Date | Change |
|---|---|
| 2026-07-19 | Initial board created from plan + repo inventory; P0 ~85% |
| 2026-07-25 | P0 punch list completed: SHAs frozen (httpx `b5addb6`, ripgrep `f9c05a9`), pilots cold-walked, checklist ticked, gold hints T001–T011. **P0 gate passed**; P1 Stage A opened (Python + Rust extractors) |
| 2026-07-25 | **P1 Stage A exited:** Fact IR, tree-sitter Python/Rust extractors, golden fixtures, design docs, indexer extract path + `insert_facts`. Stage B opened |
| 2026-07-25 | **P1 Stage B exited:** KG query API (resolve/neighbors/impact/dirty), `index-status`, reverse-dep lists, size/failure docs, `query_finished` metrics. Stage C opened |
| 2026-07-25 | **P1 Stages C+D exited / gate passed (proxies):** `prism-mcp` tools, communities/`repo_map`, scorecard ≥5× hop+token proxies; quality LLM baseline still pending. **P2 Stage A opened** |
| 2026-07-26 | **P2 Stage A exited:** `prism-plan` intent recipes + plan IR, `SCOPE_UNRESOLVED` fixtures, `prism query plan`, QUERY-PLANNER / INTENT-RECIPES docs. **Stage B opened** (Evidence Pack + budget) |
| 2026-07-26 | **P2 Stage B exited:** `prism-compile` Evidence Pack IR, must-include budget invariant, EXPLAIN, `BUDGET_EXCEEDED`, `prism compile`, labeling process. **Stage C opened** (`compile_context` MCP + scorecard) |
| 2026-07-26 | **P2 Stage C exited / gate passed (proxies):** MCP `compile_context` + `query_plan`, AGENT-USAGE primary path, precision ≥60% proxy labels, refuse-dump fixture, p2-scorecard. **P3 Stage A opened** |
| 2026-07-26 | **P3 Stage A exited:** PreciseIndex ingest (`prism-precise`), ID mapping + SCIP runbook, oracle P/R fixtures (Python), `PRECISION_REQUIRED`, CLI `prism precise`. **Stage B opened** (hybrid resolve) |
| 2026-07-26 | **P3 Stage B exited:** hybrid resolve + ambiguity index, executable `UpgradePrecision` (mandatory refactor/debug; optional impact), prefer-precise packs, `precision_upgrade` obs. **Stage C opened** |
| 2026-07-26 | **P3 Stage C exited / gate passed:** gating matrix, safe-rename dry-run, `require_precise`, p3-scorecard (+50pp call precision). **P4 Stage A opened** |
| 2026-07-26 | **P4 Stage A exited:** T3 Python CFG/DFG (`prism-semantic`), `.prism/semantic/` artifacts, crash policy, local slice CLI + property tests. **Stage B opened** (inter-proc / Slice operator) |
| 2026-07-26 | **P4 Stage B exited:** T4 shards + executable `Slice`, memo keys, overlay DATA_FLOW/CONTROL_DEP, compile/obs wiring, sink/source hooks design. **Stage C opened** (debug recipes + P4 gate) |
| 2026-07-26 | **P4 Stage C exited / gate passed:** debug recipes + pack gates, runtime enrichment design-only, p4-scorecard (40× debug token proxy). **P5 Stage A opened** |
| 2026-07-26 | **P5 Stage A exited:** repo intel catalog (entrypoints, layering, hotspots, contracts), MCP/CLI surfaces, ambiguity→T2 hints. **Stage B opened** (hardening/SDK/IDE) |
| 2026-07-26 | **P5 Stage B exited:** plugin guide, security/audit policies, IDE design, test matrix, pack-stability test, conformance CI. **Stage C opened** (public eval) |
