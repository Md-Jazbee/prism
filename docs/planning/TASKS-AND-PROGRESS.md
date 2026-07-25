# Prism — Tasks & Progress Board

**Status date:** 2026-07-25  
**Current phase:** **P1 gate passed** (structural proxies) · **P2 open** · LLM quality baselines still pending  
**Source of truth for design order:** [PLANNING-AND-IMPLEMENTATION.md](./PLANNING-AND-IMPLEMENTATION.md)  
**Source of truth for architecture:** [ARCHITECTURE-DESIGN-DOCUMENT.md](../architecture/ARCHITECTURE-DESIGN-DOCUMENT.md)

Use this file as the living checklist. Update checkbox state and the progress snapshot when a stage exits or a blocker moves.

---

## Progress snapshot

| Phase | Intent | Progress | State |
|---:|---|---:|---|
| **P0** | Foundations (identity, hash, schemas, eval) | ▓▓▓▓▓▓▓▓▓▓ **100%** | ✅ Gate passed 2026-07-25 |
| **P1** | Syntactic KG + MCP | ▓▓▓▓▓▓▓▓▓▓ **100%** | ✅ Gate passed 2026-07-25 (proxies) |
| **P2** | Context Compiler | ░░░░░░░░░░ **0%** | 🟡 Stage A open |
| **P3** | Precise Tier (T2) | ░░░░░░░░░░ **0%** | ⚪ Not started |
| **P4** | Semantic Slicing | ░░░░░░░░░░ **0%** | ⚪ Not started |
| **P5** | Repo Intelligence + Hardening | ░░░░░░░░░░ **0%** | ⚪ Not started |
| **P6** | Team / Distributed (optional) | ░░░░░░░░░░ **0%** | ⚪ Deferred |

**How to read progress:** P1 % ≈ stages done ÷ 4 (A/B/C/D). Later phases stay at 0% until their entry gate passes.

```mermaid
flowchart LR
    P0[P0 Foundations<br/>✅ done] --> P1[P1 Syntactic KG + MCP<br/>✅ done]
    P1 --> P2[P2 Context Compiler<br/>🟡 Stage A]
    P2 --> P3[P3 Precise Tier]
    P3 --> P4[P4 Semantic Slicing]
    P4 --> P5[P5 Intelligence + Eval]
    P5 --> P6[P6 Distributed / Team<br/>optional]

    style P0 fill:#b8e994,stroke:#78e08f,color:#000
    style P1 fill:#b8e994,stroke:#78e08f,color:#000
    style P2 fill:#f6e58d,stroke:#f9ca24,color:#000
    style P3 fill:#dfe6e9,stroke:#b2bec3,color:#000
    style P4 fill:#dfe6e9,stroke:#b2bec3,color:#000
    style P5 fill:#dfe6e9,stroke:#b2bec3,color:#000
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
| Query plan + Evidence Pack | ○ | ○ | ● | ● | ● | ● | ● | ⬜ |
| Precise symbol (T2) | ○ | ○ | ○ | ● | ● | ● | ● | ⬜ |
| Semantic slice (T3/T4) | ○ | ○ | ○ | ○ | ● | ● | ● | ⬜ |
| Architecture intelligence | ○ | ◐ | ◐ | ◐ | ◐ | ● | ● | ◐ path-prefix communities + hubs |
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

## Phase 2 — Context Compiler (outline)

**State:** 🟡 Stage A open (entry: P1 gate passed with noted quality caveat)  
**Duration:** 3–5 weeks  
**Gate:** Context precision ≥60% on labeled sample; refuse unbounded dumps; pack compile P95 design target &lt;300ms (excl. LLM).

| Stage | Tasks (summary) | Status |
|---|---|---|
| **A — Intent + planner** | Intent recipes; operator DAG; cost model v1 | 🟡 open |
| **B — Pack + budget** | Selection/reduction; Evidence Pack IR; EXPLAIN; `BUDGET_EXCEEDED` | ⬜ |
| **C — `compile_context`** | Primary MCP tool; Phase 2 scorecard (precision, tokens, hops, refuse-dump) | ⬜ |

---

## Phase 3 — Precise Tier (outline)

**State:** ⚪ Not started (entry: P2 gate)  
**Duration:** 4–6 weeks  
**Gate:** Call-resolution precision improves on fixtures; safe-rename dry-run demo; tools require T2 when available for accuracy claims.

| Stage | Tasks (summary) | Status |
|---|---|---|
| **A — Precise ingest** | SCIP and/or LSP index import path; tier-tagged edges | ⬜ |
| **B — Hybrid resolve** | Prefer T2 over heuristic CALLS; on-demand upgrade | ⬜ |
| **C — Product behaviors** | Precision-gated impact/rename; Phase 3 scorecard | ⬜ |

---

## Phase 4 — Semantic Slicing (outline)

**State:** ⚪ Not started (entry: P3 gate)  
**Duration:** 5–8 weeks  
**Gate:** Debug tasks token↓ ≥5× with quality within ~5 pts of frontier-explore.

| Stage | Tasks (summary) | Status |
|---|---|---|
| **A — Intra-procedural (T3)** | CFG/DFG shards; criteria for local slices | ⬜ |
| **B — Inter-procedural (T4)** | CPG shards; slice operator; shard budgets | ⬜ |
| **C — Debug recipes + gate** | Wire debug intents; optional runtime enrichment; Phase 4 scorecard | ⬜ |

---

## Phase 5 — Repository Intelligence + Hardening (outline)

**State:** ⚪ Not started  
**Duration:** ~4 weeks  
**Gate:** Published benchmark; medium+Prism ≈ frontier+explore within 3 pts; external plugin SDK usable.

| Stage | Tasks (summary) | Status |
|---|---|---|
| **A — Repo intelligence** | Architecture maps, communities productized, orientation answers | ⬜ |
| **B — Hardening + SDK + IDE** | Security checklist, plugin SDK polish, LSP/IDE commands | ⬜ |
| **C — Public eval** | Published scorecard; release readiness | ⬜ |

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

| ID | Workstream | P1 exit status |
|---|---|---|
| **W-STORE** | Storage & identity | ✅ fact insert + query + dirty lists |
| **W-PLUGIN** | Plugin ABI | ✅ frozen; Python + Rust extractors |
| **W-KG** | Knowledge graph | ✅ query + path-prefix communities |
| **W-PLAN** | Query planning | ⬜ P2 |
| **W-CC** | Context compiler | ⬜ P2 |
| **W-MCP** | Agent surface | ✅ structural MCP tools |
| **W-IDE** | IDE/LSP | ⬜ |
| **W-EVAL** | Evaluation | ✅ tool-hops + P1 scorecard proxies |
| **W-OBS** | Observability | ✅ extract + query_finished |
| **W-SEC** | Security & privacy | 🟡 allowlisted read-only MCP tools |

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
