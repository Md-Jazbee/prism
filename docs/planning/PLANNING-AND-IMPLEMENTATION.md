# Prism — Planning & Implementation Document

**Project working name:** Prism — Repository Intelligence Platform  
**Document type:** Detailed Planning & Implementation Guide (phase → stage)  
**Status:** Active — P0–P7 + P9 delivered; P8 cut (CLI+MCP); P10 deferred; P11 Stage C pending; **P12 Accuracy & Grounding gated**; **P13–P15 engineering half opened** (refactor · performance · reliability)  
**Date:** 2026-07-19 · **Revised:** 2026-07-27 (P13–P15 engineering half; **Java + Perl T1 extractors** delivered on the P1 language-expansion track)  
**Governs:** Execution of the [Architecture Design Document](../architecture/ARCHITECTURE-DESIGN-DOCUMENT.md)  
**Audience:** Project leads, architects, implementers, evaluators, open-source contributors  

---

## How to use this document

This document turns the ADD’s high-level roadmap into an executable plan. It does **not** invent new architecture. Where this plan and the ADD conflict, the **ADD wins** on design; this document wins on **sequence, gates, and work packaging**.

| Concept | Meaning in this plan |
|---|---|
| **Phase** | A major product capability boundary with a named exit gate |
| **Stage** | A sequenced work package inside a phase; each has entry/exit criteria |
| **Workstream** | Parallel concern inside a stage (e.g., storage vs. extractors vs. eval) |
| **Deliverable** | Concrete artifact (schema, tool surface, harness, report) — not source trees |
| **Gate** | Must-pass checks before the next stage or phase may start |

**Out of scope for this document:** writing application code. Technology choices live in [TECH-STACK-AND-PROJECT-STRUCTURE.md](../architecture/TECH-STACK-AND-PROJECT-STRUCTURE.md), which was promoted from guidance to decisions at implementation kickoff. Day-to-day checkbox state lives in [TASKS-AND-PROGRESS.md](./TASKS-AND-PROGRESS.md).

---

## Table of Contents

1. [Document map & relationships](#1-document-map--relationships)
2. [Product thesis (planning brief)](#2-product-thesis-planning-brief)
3. [Program overview](#3-program-overview)
4. [Cross-cutting workstreams](#4-cross-cutting-workstreams)
5. [Master timeline & dependencies](#5-master-timeline--dependencies)
6. [Phase 0 — Foundations](#6-phase-0--foundations)
7. [Phase 1 — Syntactic Knowledge Graph + MCP](#7-phase-1--syntactic-knowledge-graph--mcp)
8. [Phase 2 — Context Compiler](#8-phase-2--context-compiler)
9. [Phase 3 — Precise Tier](#9-phase-3--precise-tier)
10. [Phase 4 — Semantic Slicing](#10-phase-4--semantic-slicing)
11. [Phase 5 — Repository Intelligence + Hardening](#11-phase-5--repository-intelligence--hardening)
12. [Post-Phase-5 repository re-analysis & gap register](#12-post-phase-5-repository-re-analysis--gap-register)
13. [Phase 6 — Consolidation & Interaction Substrate](#13-phase-6--consolidation--interaction-substrate)
14. [Phase 7 — Visual Repository Intelligence](#14-phase-7--visual-repository-intelligence)
15. [Phase 8 — IDE Extension (VS Code / Cursor)](#15-phase-8--ide-extension-vs-code--cursor)
16. [Phase 9 — Agent Experience & Workflows](#16-phase-9--agent-experience--workflows)
17. [Phase 10 — Team / Distributed (optional)](#17-phase-10--team--distributed-optional)
18. [Phase 11 — Install & Distribution (any system)](#18-phase-11--install--distribution-any-system)
19. [Phase 12 — Accuracy & Grounding (doc-aware evidence)](#19-phase-12--accuracy--grounding-doc-aware-evidence)
20. [Phase 13 — Core Refactor & Boundary Hardening](#20-phase-13--core-refactor--boundary-hardening)
21. [Phase 14 — Performance & Scale Proof](#21-phase-14--performance--scale-proof)
22. [Phase 15 — Reliability, Governance & Release Trust](#22-phase-15--reliability-governance--release-trust)
23. [Evaluation program (runs across phases)](#23-evaluation-program-runs-across-phases)
24. [Risk register & guardrails](#24-risk-register--guardrails)
25. [Definition of Done (program-level)](#25-definition-of-done-program-level)
26. [Appendix — Checklists & templates](#26-appendix--checklists--templates)

> **Phase renumbering note (2026-07-26).** The former *Phase 6 — Team / Distributed* is now **Phase 10** and stays optional/deferred. Phases 6–9 cover the *interaction* half: service surfaces, graph rendering, IDE extension (later cut), and agent experience. Historical references to “P6 Stage C certified caches” now mean **P10 Stage C**.
>
> **P11 note (2026-07-26).** **Phase 11 — Install & Distribution** is planned next and does **not** depend on Phase 10. P10 remains skipped. Inspiration: Graphify’s ensure-installed skill step, host adapters (`claude install` / hooks), and one-shot project bootstrap — adapted to Prism’s single Rust binary + MCP surface ([PRODUCT-SETUP.md](../architecture/PRODUCT-SETUP.md), ADR-0007).
>
> **P12 note (2026-07-26).** **Phase 12 — Accuracy & Grounding** is opened and runs **parallel to P11 Stage C**. It answers a measured failure: on a product/architecture narrative question, Prism returned tiny packs made of *role placeholders* while a doc-aware graph (Graphify) answered from README/ADD concept nodes. P12 adds a documentation layer, kills synthetic fragments in favour of honest `gaps[]`, replaces directory “communities” with seeded semantic clustering, and settles accuracy with a five-arm benchmark. Token efficiency is already won; **sufficiency** is the open goal.
>
> **P13–P15 note (2026-07-27).** Phases 0–12 grew capability; a measured re-analysis of the as-built codebase (19,960 LOC / 20 crates / 103 test fns) found the next constraint is no longer capability but **changeability, proven performance, and maintained claims**. Three engineering-half phases open together: **P13 Core Refactor** (god files, four duplicated tool surfaces, stringly-typed refusals), **P14 Performance & Scale Proof** (NFRs still measured on a 16-file synthetic fixture while pilot corpora sit unused), **P15 Reliability & Governance** (two expired waivers, a stale risk register, the accuracy harness outside CI). These phases ship **no new user-visible capability** — their gates are parity, ceilings, and evidence.
>
> **Language expansion (2026-07-27).** **Java** and **Perl** T1 extractors landed on the ongoing P1 expansion track (not a new phase): `prism-extract-java`, `prism-extract-perl`, golden fixtures, conformance CI, and `tree-sitter` bumped **0.24 → 0.26** (required by `tree-sitter-perl`). Details: [§7.5 Language expansion — Java & Perl](#75-language-expansion--java--perl-delivered-2026-07-27).

---

## 1. Document map & relationships

```mermaid
flowchart TB
    ADD[Architecture Design Document]
    PLAN[This Planning & Implementation Doc]
    EVAL[Eval artifacts & scorecards]
    IMPL[Future implementation repos]

    ADD -->|design truth| PLAN
    PLAN -->|phase gates + stage packs| IMPL
    PLAN -->|metrics targets| EVAL
    EVAL -->|gate evidence| PLAN
```

| Document | Role |
|---|---|
| `docs/architecture/ARCHITECTURE-DESIGN-DOCUMENT.md` | **What** to build and **why** (components, IR, pipelines, NFRs) |
| This document | **When / in what order / how we know a stage is done** |
| Future eval reports | **Evidence** against gates (tokens, quality, latency) |

**Supersession rule:** Prior AOE cache-first planning ideas remain historical notes. Prism’s spine is **pre-LLM repository understanding + context compilation**. Answer caches, if any, appear only in Phase 6 as optional memoization with certificates.

---

## 2. Product thesis (planning brief)

### 2.1 One-sentence bet

Frontier-model accuracy on large codebases is mostly a **context quality** problem; Prism compiles minimum sufficient, provenance-bearing evidence so **medium models** can approach frontier quality at a fraction of the token cost.

### 2.2 What every phase must protect

| Principle | Planning implication |
|---|---|
| Understand before prompting | No phase ships “LLM-first” without indexing + KG behind it |
| Structure before similarity | Embeddings are last-resort; never the retrieval spine |
| Cache is memoization only | Do not plan answer-cache work before Evidence Packs are solid |
| Progressive precision | T1 everywhere first; pay for T2/T4 only when plans need them |
| Provenance always | Schema and APIs must carry source range, tier, confidence from Day 1 of packing |
| Refuse unbounded dumps | Scope-unresolved is a valid product behavior, not a failure |

### 2.3 Success north star (12 months)

Align with ADD Goals G1–G9:

| ID | Target |
|---|---|
| G1 | Medium + Prism ≥ frontier + naive explore within ≤3 pts answer quality |
| G2 | ≥10× fewer input tokens (structural); ≥5× (debug/refactor) |
| G3 | ≥70% context precision on labeled samples |
| G4 | Typical edit re-index <2s with dirty-set invalidation |
| G5 | Every fragment has provenance |
| G6 | New language = grammar + extractor plugin |
| G7 | MCP/LSP first-class; one-shot structural answers |
| G8 | Local-first; indexing does not require network |
| G9 | Clear IR schemas, plugin contracts, reproducible eval |

---

## 3. Program overview

### 3.1 Phase sequence

The program has five halves — the arithmetic is wrong and the history is honest. **P0–P5 (engine half, delivered)** made repository understanding correct, budgeted, and measurable. **P6–P9 (interaction half, delivered)** made that understanding usable — service surfaces, rendered graphs, and agent workflows (P8 IDE extension was **cut** in favor of CLI + MCP). **P11 (distribution half)** makes Prism installable without a Rust toolchain. **P12 (accuracy half, gated)** made packs sufficient rather than merely small. **P13–P15 (engineering half, opened 2026-07-27)** ship no new capability: they make the code changeable, prove the performance NFRs at real scale, and bring the program’s claims back under maintenance. **P10** remains an optional scale-out and is **skipped for now**.

```mermaid
flowchart LR
    subgraph Engine["Engine half — delivered"]
      P0[P0 Foundations] --> P1[P1 Syntactic KG + MCP]
      P1 --> P2[P2 Context Compiler]
      P2 --> P3[P3 Precise Tier]
      P3 --> P4[P4 Semantic Slicing]
      P4 --> P5[P5 Intelligence + Eval]
    end
    subgraph Interaction["Interaction half — delivered / cut"]
      P6[P6 Consolidation + Interaction Substrate]
      P7[P7 Visual Repo Intelligence]
      P8[P8 IDE Extension — cut]
      P9[P9 Agent Experience]
    end
    subgraph Distribution["Distribution half — planned"]
      P11[P11 Install & Distribution]
    end
    subgraph Accuracy["Accuracy half — gated"]
      P12[P12 Accuracy & Grounding]
    end
    subgraph Engineering["Engineering half — opened 2026-07-27"]
      P13[P13 Core Refactor]
      P14[P14 Performance & Scale Proof]
      P15[P15 Reliability & Governance]
    end
    P5 --> P6 --> P7 --> P8
    P7 --> P9
    P9 --> P11
    P9 --> P12
    P12 --> P13 --> P14 --> P15
    P11 -.co-gates P11 Stage C.-> P15
    P9 -.skipped for now.-> P10[P10 Distributed / Team]
    P11 -.optional later.-> P10
```

**P12 is parallel, not sequential:** it depends on the P9 agent surface and the P2 compiler, not on install work. P11 makes Prism *reachable*; P12 makes its answers *sufficient*.

**P13 → P14 → P15 is sequential on purpose.** P14 optimizes the boundary P13 creates (one engine, one store owner) rather than four call paths; P15 audits claims that only become checkable once parity harnesses and perf ceilings exist. P15 Stage C co-gates with the still-open **P11 Stage C** cold-VM run.

| Phase | Intent | Duration (calendar) | Primary user-visible outcome |
|---|---|---|---|
| **P0** | Substrate that makes later phases measurable and safe | 2–3 weeks | Workspace identity, hash index, schema draft, eval skeleton |
| **P1** | Agents can query structure without grepping | 4–6 weeks | Syntactic KG + MCP tools; impact/heuristic neighbors |
| **P2** | Agents stop multi-hop explore for most tasks | 3–5 weeks | `compile_context` Evidence Packs with budgets + EXPLAIN |
| **P3** | High-stakes nav/refactor becomes trustworthy | 4–6 weeks | SCIP/LSP hybrid resolution; precision-gated tools |
| **P4** | Debug/security packs become slice-minimal | 5–8 weeks | CFG/DFG/CPG shards + slice operator + debug recipes |
| **P5** | Product hardening + published proof | ~4 weeks | Repo intelligence + public eval; plugin SDK polish |
| **P6** | Close as-built drift; build the surfaces a UI needs | 3–5 weeks | `prismd` daemon, HTTP/SSE API, LSP, Graph View-Model contract |
| **P7** | Make the graph *seeable* without dumping it | 4–6 weeks | Budgeted, provenance-bearing interactive graph views |
| **P8** | ~~A developer never needs the terminal~~ | — | **Cut** — CLI + MCP supersedes VSIX (ADR-0007) |
| **P9** | Agents choose Prism first, and we can prove it | ~4 weeks | Agent workflows, rules/skills assets, closed-loop eval |
| **P10** | Team/CI scale (optional) | TBD | Shared index, authz, optional certified caches — **deferred / skipped for now** |
| **P11** | Anyone can install Prism on any common OS | 3–5 weeks | One-shot installers + Graphify-like agent bootstrap; cold machine → MCP-ready |
| **P12** | Packs are sufficient, not merely small | 5–7 weeks | Doc-aware evidence, honest gaps instead of placeholders, semantic communities, five-arm accuracy report |
| **P13** | The codebase becomes safe to change | 4–6 weeks | *(no user-visible change by design)* one service layer, decomposed god files, typed refusals, size budgets in CI |
| **P14** | Performance stops being a claim | 4–6 weeks | Pilot-scale N1/N2 numbers with enforced CI ceilings; warm orientation; published memory + index-size budgets |
| **P15** | Every claim has a live owner and artifact | 3–5 weeks | Refreshed waivers/risk register, three-OS + eval-harness CI, release provenance (SBOM/signing) |

### 3.2 Capability maturity ladder

| Capability | P0 | P1 | P2 | P3 | P4 | P5 | P6 | P7 | P8 | P9 | P10 | P11 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Content-hash incremental store | ● | ● | ● | ● | ● | ● | ● | ● | ● | ● | ● | ● |
| Syntactic facts (T1) | ○ | ● | ● | ● | ● | ● | ● | ● | ● | ● | ● | ● |
| MCP graph tools | ○ | ● | ● | ● | ● | ● | ● | ● | ● | ● | ● | ● |
| Query plan + Evidence Pack | ○ | ○ | ● | ● | ● | ● | ● | ● | ● | ● | ● | ● |
| Precise symbol (T2) | ○ | ○ | ○ | ● | ● | ● | ● | ● | ● | ● | ● | ● |
| Semantic slice (T3/T4) | ○ | ○ | ○ | ○ | ● | ● | ● | ● | ● | ● | ● | ● |
| Architecture intelligence | ○ | ◐ | ◐ | ◐ | ◐ | ● | ● | ● | ● | ● | ● | ● |
| Long-lived daemon + HTTP/SSE API | ○ | ○ | ○ | ○ | ○ | ○ | ● | ● | ● | ● | ● | ● |
| LSP surface | ○ | ○ | ○ | ○ | ○ | ○ | ● | ● | ● | ● | ● | ● |
| Graph View-Model contract | ○ | ○ | ○ | ○ | ○ | ○ | ● | ● | ● | ● | ● | ● |
| Interactive graph rendering | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ● | ● | ● | ● | ● |
| IDE extension (VS Code / Cursor) | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ◐ | ✂ | — | — | — |
| Agent workflows + rules assets | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ◐ | ● | ● | ● |
| Cross-platform binary install | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ◐ | ○ | ● |
| Agent ensure-install + host adapters | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ◐ | ○ | ● |
| Team/shared index | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ● | ○ |

● = required deliverable · ◐ = partial / heuristic · ○ = not yet · ✂ = cut (ADR-0007)

#### 3.2.1 Accuracy-half capability ladder (P12)

Kept as a separate table so the P0–P11 ladder above stays a historical record.

| Capability | Today (post-P11 Stage B) | P12 Stage A | Stage B | Stage C | Stage D |
|---|---|---|---|---|---|
| Documentation / prose nodes (`Doc`, `Section`) | ○ | ● | ● | ● | ● |
| Doc↔code binding (`DESCRIBES` / `MENTIONS`) | ○ | ◐ | ● | ● | ● |
| `asserted` confidence for doc-sourced claims | ○ | ● | ● | ● | ● |
| Fragments always cite a real node/edge | ○ (placeholders emitted) | ○ | ● | ● | ● |
| `gaps[]` with `why_absent` + repair action | ◐ (field exists, unused) | ◐ | ● | ● | ● |
| Grounded seed resolution + candidate refusal | ○ | ○ | ● | ● | ● |
| Lexical seed index (structure-first fallback) | ○ | ○ | ● | ● | ● |
| First-party vs vendored/fixture scoping | ○ | ○ | ● | ● | ● |
| Semantic communities (seeded Leiden) + labels | ○ (path prefixes) | ○ | ○ | ● | ● |
| Centrality hubs excluding language builtins | ○ (degree, noisy) | ○ | ○ | ● | ● |
| Live-judged accuracy vs doc-aware baseline | ○ (proxies only) | ○ | ○ | ○ | ● |

#### 3.2.2 Engineering-half capability ladder (P13–P15)

These are **internal** capabilities: none of them changes what a user sees, and that is the point. Baseline column measured 2026-07-27.

| Capability | Today (post-P12) | P13 | P14 | P15 |
|---|---|---|---|---|
| Pack-output parity harness (frozen question set) | ○ | ● | ● | ● |
| Typed refusal codes in IR (no hardcoded strings) | ○ (12 files) | ● | ● | ● |
| Single service layer behind all four surfaces | ○ (4 adapters) | ● | ● | ● |
| Enforced file/function size budgets | ○ (max 1,549 / ~949) | ● | ● | ● |
| Architecture fitness rules on crate edges | ○ | ● | ● | ● |
| One source of truth for MCP tool schemas | ○ (schema + 168-line JSON builder) | ● | ● | ● |
| Store/session ownership behind one boundary | ○ (8+ ad-hoc `open` sites) | ◐ | ● | ● |
| Pilot-scale N1/N2 measurements | ○ (16-file fixture) | ○ | ● | ● |
| Hard perf ceilings + regression guard in CI | ○ (smoke only) | ○ | ● | ● |
| Snapshot-keyed orientation cache | ○ (recomputed per call) | ○ | ● | ● |
| Memory + index-size budgets published | ○ | ○ | ● | ● |
| Waivers with live expiry + owner | ◐ (2 expired) | ◐ | ◐ | ● |
| Risk register generated from gate artifacts | ○ (hand-maintained) | ○ | ○ | ● |
| Coverage measured + ratcheted | ○ | ◐ | ◐ | ● |
| CI matrix across macOS / Linux / Windows | ○ (Linux only) | ○ | ○ | ● |
| Accuracy/parity harness executed by CI | ○ | ◐ | ◐ | ● |
| Release provenance (SBOM / signatures) | ○ (checksums only) | ○ | ○ | ● |

#### 3.2.3 Language-expansion track (P1 continuation — ongoing)

First-party T1 extractors beyond the original P1 gate (Python + Rust). Markdown joined in P12; Java and Perl in 2026-07-27.

| Language | Crate | Extensions | Analyzer | Golden fixture | T2/T3/T4 |
|---|---|---|---|---|---|
| Python | `prism-extract-python` | `.py`, `.pyi` | `tree-sitter-python@0.23` | `fixtures/languages/python/` | T3/T4 (Python) |
| Rust | `prism-extract-rust` | `.rs` | `tree-sitter-rust@0.23` | `fixtures/languages/rust/` | — |
| Markdown | `prism-extract-markdown` | `.md`, `.markdown`, `.mdx`, … | extractive (no tree-sitter) | `fixtures/languages/markdown/` | — |
| **Java** | **`prism-extract-java`** | **`.java`** | **`tree-sitter-java@0.23`** | **`fixtures/languages/java/`** | **— (T1 only)** |
| **Perl** | **`prism-extract-perl`** | **`.pl`, `.pm`, `.perl`** | **`tree-sitter-perl@1.1`** | **`fixtures/languages/perl/`** | **— (T1 only)** |

**Dispatch:** all languages route through `prism-extract::detect_language` → `extract_file`. **Conformance:** `scripts/plugins/conformance-check.sh` runs every golden. **Re-index trigger:** `ANALYZER_PIPELINE_VERSION` bumped to `p12-doc-v2-perl-java` when Java/Perl landed.

### 3.3 What “implementation” means here

Each stage specifies:

1. **Purpose** — why this stage exists  
2. **Entry criteria** — what must already be true  
3. **Workstreams** — parallel tracks  
4. **Deliverables** — documents, schemas, contracts, harnesses, runbooks  
5. **Dependencies** — upstream stages / external systems  
6. **Risks** — local to the stage  
7. **Exit / acceptance** — measurable checks  
8. **Handoff** — what the next stage inherits  

No stage requires creating application source trees as part of *this planning effort*.

---

## 4. Cross-cutting workstreams

These run in every phase. Stage plans reference them by ID.

| ID | Workstream | Owns | Never owns |
|---|---|---|---|
| **W-STORE** | Storage & identity | `.prism` layout, SQLite schemas, snapshot IDs, transactions | Prompt formatting |
| **W-PLUGIN** | Plugin ABI | Extractor/resolver/semantic contracts, versioning, golden fixtures | Product UX |
| **W-KG** | Knowledge graph service | Node/edge kinds, query API shape, confidence tagging | LLM calls |
| **W-PLAN** | Query planning | Intent recipes, operator DAG, cost model | Indexing |
| **W-CC** | Context compiler | Selection, reduction, budgets, Evidence Pack IR, EXPLAIN | Full-repo CPG |
| **W-MCP** | Agent surface | MCP tool contracts, safety rules, one-shot guidance | Graph algorithms |
| **W-IDE** | IDE/LSP | Commands, peek UX, edit-time incremental policy | Replacing language servers |
| **W-EVAL** | Evaluation | Gold tasks, baselines, scorecard automation | Feature shipping |
| **W-OBS** | Observability | Metrics, traces, audit logs of packs sent to LLMs | Business logic |
| **W-SEC** | Security & privacy | Local default, secret redaction, plugin sandbox policy | Multi-tenant SaaS (until P10) |

### 4.1 Workstreams added for the interaction half (P6+)

| ID | Workstream | Owns | Never owns |
|---|---|---|---|
| **W-SVC** | Local service layer | `prismd` lifecycle, HTTP/SSE contracts, session + cancellation, warm caches | Graph algorithms, pixels |
| **W-VIZ** | Visualization | Graph View-Model schema, level-of-detail, layout determinism, render budgets, visual encoding of tier/confidence | Selecting evidence (that is W-CC) |
| **W-AX** | Agent experience | Tool ergonomics, refusal-repair loops, workflow recipes, rules/skills assets, trace capture | Model hosting, prompt magic |
| **W-DEBT** | As-built reconciliation | Drift register between docs and repo, waivers, deprecations | New features |

### 4.2 Workstreams added for the distribution half (P11)

| ID | Workstream | Owns | Never owns |
|---|---|---|---|
| **W-DIST** | Install & distribution | Release artifacts, installers, package managers, checksum/signing, PATH placement, upgrade/uninstall | Graph algorithms, pack quality, team/shared indexes (P10) |

### 4.3 Workstreams added for the accuracy half (P12)

| ID | Workstream | Owns | Never owns |
|---|---|---|---|
| **W-DOC** | Documentation intelligence | Doc/section extraction, link & mention resolution, doc↔code binding, `asserted` confidence, doc goldens | Writing or rewriting the repo’s docs; abstractive summaries |
| **W-ACC** | Accuracy program | Doc-QA gold sets, adjudication protocol, citation-validity scoring, cross-tool baseline arms, ablation discipline | Shipping features; choosing retrieval algorithms |

### 4.4 Workstreams added for the engineering half (P13–P15)

| ID | Workstream | Owns | Never owns |
|---|---|---|---|
| **W-REFAC** | Structural refactor | Service-layer boundary, module decomposition, typed error/refusal IR, size & complexity budgets, architecture fitness rules, parity harness | Behavior changes, new capabilities, performance tuning |
| **W-PERF** | Performance & scale | Pilot-scale benches, profiling, caching and pooling strategy, pipeline parallelism, NFR ceilings and regression guards | Pack semantics, accuracy trade-offs, answer caching (P10) |
| **W-REL** | Reliability & governance | Waiver expiry enforcement, risk-register generation, CI matrix and coverage policy, fuzz/property invariants, release provenance | Feature scope, roadmap negotiation |

**Planning rules:**

1. Every phase exit must update **W-EVAL** and **W-OBS**, even if the product surface barely changed.
2. From P6 onward, every phase exit must also update **W-DEBT** — either close a drift item or record a written waiver.
3. **W-VIZ never invents evidence.** A view may only render facts the KG or a pack already contains, with the same provenance and confidence labels.
4. **W-DIST never requires a network after install** for the core index + MCP path (N5 / G8). Installers may download the binary once; day-2 usage stays local-first.
5. **W-DOC never invents evidence, and W-ACC never ships features.** Documentation facts are extractive and labeled `asserted`; the accuracy program grades what exists rather than negotiating the gate. From P12 onward, no fragment may reach a pack without a citation to a real node or edge.
6. **W-REFAC never changes behavior.** Any behavior difference discovered or intentionally chosen during a refactor requires an ADR before it merges; the parity harness is the arbiter, not reviewer intuition.
7. **W-PERF never trades accuracy for latency.** Every optimization cites a profile, and the P12 accuracy checklist is re-run at each performance gate.
8. **W-REL enforces expiry.** From P15 onward, a waiver whose expiry phase has shipped blocks the next phase gate until it is re-dated or closed — waivers age out loudly rather than silently.

---

## 5. Master timeline & dependencies

### 5.1 Critical path

```mermaid
flowchart TD
    P0S1[P0 Stage A: Workspace + hash] --> P0S2[P0 Stage B: Schema + ABI]
    P0S2 --> P0S3[P0 Stage C: Eval skeleton]
    P0S3 --> P1S1[P1 Stage A: Extractors T1]
    P1S1 --> P1S2[P1 Stage B: KG write/query]
    P1S2 --> P1S3[P1 Stage C: MCP graph tools]
    P1S3 --> P1S4[P1 Stage D: Token gate]
    P1S4 --> P2S1[P2 Stage A: Intent + planner]
    P2S1 --> P2S2[P2 Stage B: Pack + budget]
    P2S2 --> P2S3[P2 Stage C: compile_context]
    P2S3 --> P3S1[P3 Stage A: SCIP/LSP ingest]
    P3S1 --> P3S2[P3 Stage B: Hybrid refine]
    P3S2 --> P3S3[P3 Stage C: Gated high-stakes tools]
    P3S3 --> P4S1[P4 Stage A: CFG/DFG]
    P4S1 --> P4S2[P4 Stage B: Slice + shards]
    P4S2 --> P4S3[P4 Stage C: Debug recipes gate]
    P4S3 --> P5S1[P5 Stage A: Repo intelligence]
    P5S1 --> P5S2[P5 Stage B: Hardening + SDK]
    P5S2 --> P5S3[P5 Stage C: Public eval]
    P5S3 --> P6S1[P6 Stage A: Debt paydown]
    P6S1 --> P6S2[P6 Stage B: Daemon + HTTP/SSE]
    P6S2 --> P6S3[P6 Stage C: LSP + Graph View-Model]
    P6S3 --> P7S1[P7 Stage A: View-model + layout]
    P7S1 --> P7S2[P7 Stage B: Renderer + interaction]
    P7S2 --> P7S3[P7 Stage C: Evidence overlays gate]
    P7S3 --> P8S1[P8 Stage A: Extension skeleton]
    P8S1 --> P8S2[P8 Stage B: Commands + panels]
    P8S2 --> P8S3[P8 Stage C: Cursor + release]
    P8S3 -->     P9S1[P9 Stage A: Agent contract hardening]
    P9S1 --> P9S2[P9 Stage B: Workflows + rules assets]
    P9S2 --> P9S3[P9 Stage C: Closed-loop eval gate]
    P9S3 --> P11S1[P11 Stage A: Binary release matrix]
    P11S1 --> P11S2[P11 Stage B: Agent bootstrap + hosts]
    P11S2 --> P11S3[P11 Stage C: Install gate]
    P11S3 --> P12S1[P12 Stage A: Documentation layer]
    P12S1 --> P12S4[P12 Stage D: Accuracy gate]
    P12S4 --> P13S1[P13 Stage A: Seams + characterization]
    P13S1 --> P13S2[P13 Stage B: Service layer]
    P13S2 --> P13S3[P13 Stage C: God-file decomposition]
    P13S3 --> P13S4[P13 Stage D: Parity gate]
    P13S4 --> P14S1[P14 Stage A: Pilot harness + profiles]
    P14S1 --> P14S2[P14 Stage B: Store & cache optimization]
    P14S2 --> P14S3[P14 Stage C: Pipeline parallelism]
    P14S3 --> P14S4[P14 Stage D: NFR gate]
    P14S4 --> P15S1[P15 Stage A: Governance sweep]
    P15S1 --> P15S2[P15 Stage B: Test & CI hardening]
    P15S2 --> P15S3[P15 Stage C: Release trust]
    P9S3 -.-> P10[P10 Optional team mode — skipped for now]
```

### 5.2 Hard dependency rules

1. **No MCP “explore replacement” claims** before P1 Stage D gate.  
2. **No `compile_context` as primary tool** before P2 Stage C gate.  
3. **No safe rename / precise impact claims** before P3 Stage C gate.  
4. **No “slice-based debug” claims** before P4 Stage C gate.  
5. **No shared index / answer cache** before P9 Stage C succeeds (was: P5 Stage C — moved because the interaction half now precedes scale-out).  
6. **Embedding work** may appear as a tiny fallback prototype in P2+, but must be flagged `low_confidence` and excluded from success narratives.  
7. **No pixels before contracts.** No renderer work before the Graph View-Model schema is frozen in P6 Stage C.  
8. **No extension release** before P7 Stage C — an extension without views is a CLI with extra steps. *(P8 later cut; rule retained for history.)*  
9. **No “agents prefer Prism” claim** before P9 Stage C measures it on real traces.  
10. **No visualization that invents structure.** Every rendered node/edge maps to a KG node/edge or pack fragment with its original tier and confidence.  
11. **No “install anywhere” claim** before P11 Stage C proves cold-machine → MCP-ready on macOS, Linux, and Windows without a Rust toolchain.  
12. **P11 does not wait on P10.** Distribution is a solo-developer adoption gate; team/shared index stays optional and deferred.  
13. **No refactor without a parity net.** P13 Stage B may not move code before Stage A’s golden corpus reproduces deterministically (added 2026-07-27).  
14. **No optimization without a profile.** P14 changes cite a Stage A profile; “obvious” wins that no profile supports do not merge.  
15. **No perf claim from the mini fixture.** N1/N2 claims require pilot-repo numbers with named hardware and snapshot id.  
16. **No new capability inside P13–P15.** A feature idea discovered during this half becomes a backlog item, not a stage deliverable.

### 5.3 Approximate effort envelope

| Phase | Weeks | Relative effort |
|---|---|---|
| P0 | 2–3 | Foundation (small but blocking) |
| P1 | 4–6 | Largest early build |
| P2 | 3–5 | Highest product leverage |
| P3 | 4–6 | Integration-heavy |
| P4 | 5–8 | Deepest analysis risk |
| P5 | ~4 | Proof + polish |
| P6 | 3–5 | Plumbing + debt (unglamorous, unblocking) |
| P7 | 4–6 | New discipline (front-end + layout determinism) |
| P8 | — | Cut (ADR-0007) |
| P9 | ~4 | Proof of the agent thesis |
| P10 | optional / skipped for now | Ops/product expansion |
| P11 | 3–5 | Release engineering + Graphify-like bootstrap |
| P12 | 5–7 | Accuracy program + doc layer |
| P13 | 4–6 | Refactor discipline (no new features; parity is the hard part) |
| P14 | 4–6 | Profiling + systems tuning at pilot scale |
| P15 | 3–5 | Governance + CI/release engineering |

Total critical path (P0–P5): roughly **22–36 weeks**. Interaction half (P6–P9): roughly **15–20 weeks** on top. Distribution half (P11): roughly **3–5 weeks** after P9, and it is the first block that needs **release-engineering** skill (CI matrix, signing, package taps) alongside Rust. Engineering half (P13–P15): roughly **11–17 weeks**, and it is the first block whose success is measured by what *does not* change.

---

## 6. Phase 0 — Foundations

**Phase goal:** Make repository identity, incremental hashing, durable schemas, plugin contracts, and evaluation measurable *before* inventing intelligence features.

**Phase duration:** 2–3 weeks  
**Phase gate (summary):** Incremental re-index path is designed and prototype-validatable; metrics pipeline exists for later phases.

```mermaid
flowchart LR
    A[Stage A Workspace + Fingerprint] --> B[Stage B Schema + Plugin ABI]
    B --> C[Stage C Eval Skeleton + Observability]
```

---

### Stage A — Workspace identity & content fingerprinting

#### Purpose

Define how Prism knows *which repository, which snapshot, which files changed*, without depending on language analysis.

#### Entry criteria

- ADD §12 (indexing pipeline) and §21 (storage layout) accepted as design baseline  
- Target pilot repos identified (1–2 real codebases for later gold tasks)

#### Workstreams

| Workstream | Activities |
|---|---|
| W-STORE | Specify workspace roots, ignore rules (`.gitignore` + vendoring heuristics), content hash per file, Merkle/dir fingerprint for skip |
| W-OBS | Define counters: files discovered, files skipped unchanged, indexing wall time |
| W-SEC | Specify secret-sensitive path defaults (e.g., never index `.env` by default) |

#### Deliverables

1. **Workspace Manager specification** — roots, VCS identity (git commit SHA + dirty stamp), ignore policy  
2. **Fingerprint algorithm note** — file hash, tree Merkle, “unchanged subtree skip” contract  
3. **`.prism/` layout draft** — folders for meta, graph, blobs, logs (names freezeable in Stage B)  
4. **Incremental invalidation design** — unit of change = file subgraph (+ later reverse deps)

#### Dependencies

- None technical beyond access to pilot repos and git metadata conventions

#### Risks

| Risk | Mitigation |
|---|---|
| Over-engineering distributed identity | Stay solo-local; git SHA + worktree stamp only |
| Ignore rules miss huge vendor trees | Explicit vendoring heuristics; measure index size early |

#### Exit / acceptance

- [ ] Documented algorithm can classify a file edit as changed/unchanged given hashes  
- [ ] Dirty worktree vs clean commit identities are distinguishable  
- [ ] Ignore policy review checklist exists  
- [ ] Pilot repos listed with approximate LOC and languages

#### Handoff to Stage B

Stable definitions of `Repository`, `Snapshot`, `File` identity fields.

---

### Stage B — Durable schema & plugin ABI draft

#### Purpose

Lock the *shapes* of facts and plugins so Phase 1 extractors do not thrash the store.

#### Entry criteria

- Stage A exit complete  
- Node/edge kinds from ADD §11 agreed at least for T1 subset

#### Workstreams

| Workstream | Activities |
|---|---|
| W-STORE | Draft `meta.sqlite` tables (snapshots, files, hashes, jobs); draft graph tables or embedded-graph mapping |
| W-PLUGIN | Draft `LanguageExtractor` contract: bytes → versioned facts; declare confidence enums |
| W-KG | Freeze T1 node/edge subset for P1 (File, Symbol, IMPORTS, CALLS heuristic, CONTAINS, etc.) |
| W-SEC | Policy stub for plugin side effects / sandbox expectations |

#### Deliverables

1. **Schema v0 document** — tables/collections, indexes, transaction boundary (“replace file subgraph”)  
2. **Fact schema v0** — attributes required for symbols, spans, edges, confidence, analyzer, tier  
3. **Plugin ABI draft** — input/output, versioning, pure-transform rule, golden-fixture expectation  
4. **IR layer cheat-sheet** — L0–L5 from ADD §9.3, what P0/P1 populate now vs later  

#### Dependencies

- Stage A identities  
- ADD precision ladder (tiers labeled even if only T1 implemented later)

#### Risks

| Risk | Mitigation |
|---|---|
| Schema trying to encode T4 CPG early | Explicit “semantic layer lazy / separate artifacts” |
| Opaque integer IDs (LSIF trap) | Prefer readable SCIP-compatible or deterministic syntactic IDs |

#### Exit / acceptance

- [ ] Schema review signed by architect: crash-safe write strategy chosen (e.g., SQLite WAL transactional replaces)  
- [ ] Confidence values include at least `extracted` / `heuristic` / `precise` / `observed` (observed may be unused until later)  
- [ ] Plugin ABI reviewable by a future language contributor without reading the whole ADD  
- [ ] Migration policy: “breaking fact schema bumps major version”

#### Handoff to Stage C

Stable IDs and schemas that eval fixtures will cite.

---

### Stage C — Evaluation skeleton & observability baseline

#### Purpose

Ensure **every later gate has a measurement path**. Without this, P1/P2 “token wins” become anecdotes.

#### Entry criteria

- Stage B schema/ABI drafts accepted  
- Pilot repos frozen enough to attach gold tasks

#### Workstreams

| Workstream | Activities |
|---|---|
| W-EVAL | Define 20 gold tasks across 1–2 repos; task types: symbol explain, impact, architecture orientation (debug/refactor can be stubs) |
| W-EVAL | Specify baseline protocols: frontier+explore, medium+explore (explore = grep/read/glob loops) |
| W-OBS | Metrics pipeline design: index latency, query latency placeholders, tokens/task, tool calls/task |
| W-MCP | Note which future tools each gold task will prefer (for later checklists) |

#### Deliverables

1. **Eval harness design** — inputs, outputs, judges (human and/or LLM-judge with fixed rubric), reproducibility (pinned snapshots)  
2. **Gold task pack v0** — ≥20 tasks with gold hints / answers / necessary-span notes where known  
3. **Scorecard template** — columns aligned to ADD §32  
4. **Baseline measurement runbook** — how to measure explore-token usage without Prism  

#### Dependencies

- Pilot repo snapshots (content-addressed)  
- Access assumptions for hosted/local judges documented (privacy)

#### Risks

| Risk | Mitigation |
|---|---|
| Tasks too vague to score | Every task needs accepted answer criteria or graded checklist |
| Eval becomes blocked on LLM API | Allow offline structural metrics first (tokens to pack; hop counts) |

#### Exit / acceptance (Phase 0 gate)

- [ ] Incremental re-index path is specified end-to-end (discover → hash → parse-hook → txn → invalidate) even if parsers are stubs  
- [ ] Metrics pipeline exists on paper with named event schema  
- [ ] ≥20 gold tasks versioned and tied to commit SHAs  
- [ ] Team can answer: “How will we know P1 saved tokens?” with a written procedure  

#### Phase 0 phase-level risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Building code before contracts | High | High | Treat Stage B/C as blockers |
| Eval forever “coming later” | High | High | No Phase 1 gate without harness design |

---

## 7. Phase 1 — Syntactic Knowledge Graph + MCP

**Phase goal:** Index repositories into a durable syntactic knowledge graph and expose agent-usable structural tools, proving large token reduction on structural tasks *without* a full context compiler yet.

**Phase duration:** 4–6 weeks  
**Languages (recommended):** start with 2–3 (e.g., Python, TypeScript/JavaScript, Go)  
**Phase gate (summary):** ≥5× token reduction on structural tasks vs explore; quality within ~10 pts of explore on the gold subset that structural tools can answer.

```mermaid
flowchart LR
    A[Stage A T1 Extractors] --> B[Stage B KG Persist + Query]
    B --> C[Stage C MCP Structural Tools]
    C --> D[Stage D Communities + Token Gate]
```

---

### Stage A — Language extractors (T1)

#### Purpose

Turn source files into **typed facts** with honest confidence labels using tree-sitter (or equivalent) CST/AST extraction.

#### Entry criteria

- Phase 0 complete  
- Plugin ABI draft frozen enough for first languages  
- Language priority list agreed

#### Workstreams

| Workstream | Activities |
|---|---|
| W-PLUGIN | Per-language extractor design: symbols, imports, call sites, inheritance/implements where cheap, routes/tests heuristics |
| W-KG | Map extractor outputs to edges: `CONTAINS`, `IMPORTS`, `CALLS` (heuristic), `EXTENDS`/`IMPLEMENTS` (best-effort) |
| W-EVAL | Golden fixture files per language (small repos or snippets) with expected fact dumps |
| W-OBS | Unresolved call rate metric definition |

#### Deliverables

1. **Extractor design docs** (one per language) — what is extracted, what is explicitly *not* claimed precise  
2. **Golden fact fixtures** — deterministic expected outputs for conformance  
3. **Resolution-cheap policy** — same-file + import-graph heuristics only; label unresolved  

#### Dependencies

- Schema v0  
- Tree-sitter grammars availability for chosen languages  

#### Risks

| Risk | Mitigation |
|---|---|
| Pretending textual calls are precise | Mandatory `heuristic` / `best_effort` tags |
| Too many languages at once | Cap at 2–3 until fixtures green |

#### Exit / acceptance

- [ ] Each language has golden fixtures passing the *design* of conformance tests  
- [ ] Unresolved edges are first-class, not silent deletes  
- [ ] Extractor docs state known failure modes (dynamic imports, macros, generics)

#### Handoff

Fact producers ready for Stage B persistence contracts.

---

### Stage B — Knowledge graph persistence & query API

#### Purpose

Make facts durable, incrementally replaceable, and queryable with low latency targets in mind (local P95 structural queries &lt;50ms as *design* NFR).

#### Entry criteria

- Stage A extractor contracts + fixtures exist  
- Transaction model from P0 Stage B agreed

#### Workstreams

| Workstream | Activities |
|---|---|
| W-STORE | Specify per-file subgraph replace; reverse dependency dirty lists; snapshot isolation |
| W-KG | Query shapes: resolve by name/path, neighbors by edge type, depth-limited expansion |
| W-OBS | Index files/s and incremental latency measurement plan against NFR N1 |
| W-SEC | Path isolation; no unexpected blob retention of secrets |

#### Deliverables

1. **KG query API contract** (CLI/HTTP conceptual — ADD §22 subset needed for P1)  
2. **Incremental update sequence diagram** — edit → dirty set → rebuild dependents (communities may be deferred to Stage D)  
3. **Index size budget note** — aim ~3–10% of source for syntactic index (N3)  
4. **Failure modes** — crash mid-txn recovery expectations |

#### Dependencies

- Extractor fact schema  
- Fingerprint invalidation from P0  

#### Risks

| Risk | Mitigation |
|---|---|
| In-memory-only graph (NetworkX trap) | Persist from day one of Stage B |
| Whole-repo rewrite on any edit | File subgraph replace + reverse dep propagation |

#### Exit / acceptance

- [ ] Documented that a single-file edit does not require full rebuild  
- [ ] Query API can express: symbol lookup, 1-hop neighbors, depth-limited impact candidates  
- [ ] Latency/size NFRs are tracked (even if not yet met)

---

### Stage C — MCP structural tool surface

#### Purpose

Give agents first-class tools so they stop rediscovering structure via grep/read. This is **not** yet the full context compiler; it is the structural substrate MCP (GitNexus/Graphify lesson).

#### Entry criteria

- Stage B query API contract stable  
- Safety defaults for local MCP agreed (W-SEC)

#### Workstreams

| Workstream | Activities |
|---|---|
| W-MCP | Specify tools: `index_status`, `resolve_symbol`, `neighbors`, `impact` (heuristic), `repo_map` stub if communities not ready |
| W-OBS | Tool-call auditing; optional “pack sent to LLM” log stub |
| W-EVAL | Map gold structural tasks to tool traces (expected fewer hops) |
| W-SEC | Allowlist; citation + confidence in returns; no write/rename tools yet |

#### Deliverables

1. **MCP tool catalog v1** — inputs, outputs, errors, confidence fields  
2. **Agent usage guide draft** — prefer structural tools over read/grep loops  
3. **Error model draft** — at least `SCOPE_UNRESOLVED` placeholder behavior  

#### Dependencies

- KG query API  
- Index freshness (`index_status`)

#### Risks

| Risk | Mitigation |
|---|---|
| Too many micro-tools → agent still thrash | Keep surface small; prepare for P2 one-shot |
| Overclaiming impact accuracy | Label impact confidence; depth caps |

#### Exit / acceptance

- [ ] Tool catalog reviewed against ADD §25 subset for P1  
- [ ] Every tool return includes provenance/confidence or explicitly marks heuristics  
- [ ] Eval harness can record tool hops per task |

---

### Stage D — Communities, orientation & Phase 1 gate

#### Purpose

Add lightweight architecture orientation (communities/hubs) and **prove** the token thesis on structural tasks.

#### Entry criteria

- Stages A–C complete  
- Gold structural subset ready for scoring  

#### Workstreams

| Workstream | Activities |
|---|---|
| W-KG / Architecture miner | Leiden/Louvain (or similar) on import+call graph; path-prefix labels first |
| W-MCP | Flesh out `repo_map` / hubs exposure |
| W-EVAL | Run explore vs Prism structural comparison; compute token and quality deltas |
| W-OBS | Unresolved call rate dashboards per language |

#### Deliverables

1. **Community detection design** — algorithm, refresh triggers, labeling policy  
2. **Phase 1 scorecard report** — tokens/task, quality vs explore, unresolved edge rates  
3. **Known limitations register** — what wrong callees do to impact answers  

#### Exit / acceptance (Phase 1 gate)

- [ ] ≥5× token reduction on **structural** gold tasks vs explore baseline  
- [ ] Quality within ~10 points of explore on those tasks (not debug-hard tasks)  
- [ ] Incremental edit path demonstrated on a fixture scenario (&lt;2s as stretch target; must at least show no full rebuild)  
- [ ] No narrative claiming precise refactor safety yet  

#### Phase 1 phase-level risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Heuristic CALLS poison impact | High | High | Confidence + depth caps; escalate to P3 |
| Scope creep into search UI | Medium | High | MCP tools only; context compiler deferred to P2 |
| Quality gap vs explore ≥10 pts | Medium | High | Improve recipes and resolution; delay P2 marketing claims |

---

### 7.5 Language expansion — Java & Perl (delivered 2026-07-27)

> **Track, not a phase.** Extends P1 Stage A (T1 extractors) after the P12 gate. Does **not** reopen P1–P12 gates; adds first-party languages using the frozen LanguageExtractor ABI + golden conformance pattern.

**Goal:** Index `.java` and Perl (`.pl`/`.pm`) sources into the syntactic KG with the same T1 fact shapes as Python/Rust — symbols, imports/`use`, heuristic `CALLS`, and `EXTENDS` where cheap.

**Depends on:** P1 gate (ABI + dispatch) · P12 doc layer (shared `prism-extract` registry)

| ID | Deliverable | Status |
|---|---|---|
| **LANG-1** | `prism-extract-java` — classes, methods, imports, extends, calls | ✅ delivered |
| **LANG-2** | `prism-extract-perl` — package, `use`, subs, calls | ✅ delivered |
| **LANG-3** | Dispatch in `prism-extract` for `.java`, `.pl`, `.pm`, `.perl` | ✅ delivered |
| **LANG-4** | Golden fixtures + conformance tests in CI | ✅ delivered |
| **LANG-5** | `tree-sitter` workspace bump **0.24 → 0.26** (Perl grammar requires 0.26) | ✅ delivered |
| **LANG-6** | `ANALYZER_PIPELINE_VERSION` → `p12-doc-v2-perl-java` (forces re-index) | ✅ delivered |

**Extracted fact shapes (T1, heuristic where noted):**

| Language | Definitions | Imports / modules | Calls | Inheritance |
|---|---|---|---|---|
| Java | class, interface, enum, method, constructor | `import` → `module:` nodes | `method_invocation` → `CALLS` | `extends` → `EXTENDS` |
| Perl | `sub` → function symbols | `package` + `use` → `module:` nodes | bareword calls → `CALLS` | — (not in v1) |

**Non-goals (explicit):**

- T2 precise tier, T3/T4 semantic slices, or SCIP for Java/Perl (remain Python-first per ADR-0002 trajectory)
- New MCP tools — existing structural tools consume the new facts automatically after re-index
- TypeScript, Go, or other languages (future expansion entries follow the same LANG-* checklist)

**Verification:**

```bash
./scripts/plugins/conformance-check.sh   # includes java + perl goldens
cargo test -p prism-extract-java -p prism-extract-perl -p prism-extract
prism index .                            # pick up new extensions after pipeline version bump
```

**Residual / follow-ons:**

- Extractor design docs under `docs/architecture/extractors/` (java.md, perl.md) — optional polish
- Pilot-repo eval on a Java or Perl corpus — not run yet; structural proxies unchanged
- P13 refactor should treat new crates like existing extractors (no special-case god files)

---

## 8. Phase 2 — Context Compiler

**Phase goal:** Make **context compilation** the product: intent → query plan → Evidence Pack under token budget, with EXPLAIN and refuse-to-dump behavior. Elevate `compile_context` to the primary agent tool.

**Phase duration:** 3–5 weeks  
**Phase gate (summary):** Context precision ≥60% on labeled sample; unresolved scopes refuse unbounded dumps; pack compile latency design targets P95 &lt;300ms excluding LLM.

```mermaid
flowchart LR
    A[Stage A Intent + Planner] --> B[Stage B Selection Reduction Pack]
    B --> C[Stage C compile_context + EXPLAIN Gate]
```

---

### Stage A — Intent classification & query planner v1

#### Purpose

Turn natural-language / agent intents into an **operator DAG** instead of ad-hoc tool chains.

#### Entry criteria

- Phase 1 gate passed  
- Operator catalog from ADD §19 accepted as v1 scope  

#### Workstreams

| Workstream | Activities |
|---|---|
| W-PLAN | Intent set: repo-QA, debug (lightweight), impact, refactor (warn if no T2), generate, review, architecture |
| W-PLAN | Recipes: seed rules + expand rules per intent (ADD §17) |
| W-PLAN | Cost model sketch: prefer cheap T1 ops; insert `UpgradePrecision` only as placeholder until P3 |
| W-EVAL | Intent fixtures → expected operator shapes |

#### Deliverables

1. **Intent recipe catalog v1** — seeds, expands, must-include, drop priorities  
2. **Planner design** — plan IR JSON/YAML shape; operator list for v1  
3. **Example plans** — at least debug/impact/repo-QA walkthroughs  

#### Dependencies

- P1 graph operators (`Resolve`, `Expand`, `Impact`)  
- Diff/worktree detection design (can be shallow in v1)

#### Risks

| Risk | Mitigation |
|---|---|
| Planner becomes another LLM prompt | Keep deterministic recipes first; LLM optional for intent only if needed |
| Debug recipes overpromise before slicer | Mark debug plans as “best-effort until P4” |

#### Exit / acceptance

- [ ] For each intent, a recipe produces a plan without executing LLM  
- [ ] Fixtures cover ambiguous queries → `SCOPE_UNRESOLVED` paths  
- [ ] Plan-only API (`/query/plan`) contract documented  

---

### Stage B — Selection, reduction & Evidence Pack IR

#### Purpose

Implement (as design + contracts) the compiler backend: materialize fragments, rank, budget, never drop must-includes.

#### Entry criteria

- Stage A recipes stable  
- Evidence fragment schema (ADD §11.4) accepted  

#### Workstreams

| Workstream | Activities |
|---|---|
| W-CC | Hierarchical pack layout: arch → module signatures → core slices → neighbor signatures → diffs → optional run |
| W-CC | Drop order under budget (ADD §18.1)  
| W-CC | Extractive default for code; abstractive only for docs/ADR with links  
| W-EVAL | Start labeling necessary vs unnecessary fragments on a sample of packs |
| W-OBS | Drop counts, tokens/pack, reason codes |

#### Deliverables

1. **Evidence Pack schema v1** — meta, hierarchy, citations, gaps  
2. **Selection priority document** — deterministic ordering  
3. **Reduction techniques catalog** — when to use signature skeleton vs span slice  
4. **Quality gates** — must-include checks; `BUDGET_EXCEEDED` behavior  

#### Dependencies

- Planner plan IR  
- Source span materialization rules (line/byte ranges)

#### Risks

| Risk | Mitigation |
|---|---|
| LLM summaries as default | Ban abstractive code summaries in v1 |
| Soft budgets silently truncate truth | Hard refuse if must-include cannot fit |

#### Exit / acceptance

- [ ] Pack schema round-trips through an example EXPLAIN report  
- [ ] Written proof that must-include cannot be budget-evicted  
- [ ] Labeled sample process documented (≥N packs for precision measurement) |

---

### Stage C — `compile_context` as primary tool + Phase 2 gate

#### Purpose

Ship the agent UX change: **one** high-value call returns the substrate for answering; EXPLAIN enables trust/debugging.

#### Entry criteria

- Stages A–B complete  
- MCP catalog ready to promote `compile_context`

#### Workstreams

| Workstream | Activities |
|---|---|
| W-MCP | Specify `compile_context` inputs (question, repo ref, budget, hints); deprecate multi-hop as primary guidance |
| W-CC | EXPLAIN CONTEXT — per-fragment `why_included` reason codes |
| W-IDE | Optional: `prism.compileContext` / evidence peek design (can be stub UX) |
| W-EVAL | Measure context precision ≥60%; token and hop reductions vs P1 multi-tool baseline |

#### Deliverables

1. **MCP primary-path guide** — “call `compile_context` first”  
2. **EXPLAIN report format**  
3. **Phase 2 scorecard** — precision, tokens, hops, refuse-dump cases  
4. **Gap communication** — pack `gaps` for missing precise tier / unresolved symbols |

#### Exit / acceptance (Phase 2 gate)

- [ ] Context precision ≥60% on labeled sample  
- [ ] Unresolved scope → refuse unbounded dump (ask for anchors) demonstrated in fixtures  
- [ ] `compile_context` documented as preferred tool over ten reads  
- [ ] Pack compile latency budget tracked toward &lt;300ms P95 (excluding LLM)  
- [ ] Provenance present on every fragment |

#### Phase 2 phase-level risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Agents ignore one-shot tool | Medium | High | IDE/MCP guidance; hooks where available |
| Precision &lt;60% | Medium | High | Tighten recipes; improve must-include labeling |
| Feature creep into answer cache | Medium | Medium | Explicit non-goal until P10 |

---

## 9. Phase 3 — Precise Tier

**Phase goal:** Overlay compiler-grade or LSP-grade symbol intelligence (SCIP import and/or Hybrid LSP resolvers) so high-stakes impact/refactor paths stop trusting heuristic `CALLS` when better data exists.

**Phase duration:** 4–6 weeks  
**Phase gate (summary):** Call resolution precision materially improves on fixtures; safe rename dry-run demo exists; tools require T2 when available for refactor/impact accuracy claims.

```mermaid
flowchart LR
    A[Stage A SCIP/LSP Ingest] --> B[Stage B Hybrid Refinement]
    B --> C[Stage C Precision-Gated Tools + Gate]
```

---

### Stage A — Precise index ingest

#### Purpose

Bring external precise indexes into Prism’s identity system without requiring a Google-scale Kythe platform.

#### Entry criteria

- Phase 2 gate passed  
- Primary languages for T2 chosen (subset of P1 languages)  

#### Workstreams

| Workstream | Activities |
|---|---|
| W-PLUGIN | SCIP import mapping to Prism symbol IDs; LSP client adapter design |
| W-STORE | `scip/` artifact storage; link precise symbols to file hashes/snapshots  
| W-KG | Edge refinement API: upgrade `CALLS`/`REFERENCES` confidence to precise when matched  
| W-EVAL | Oracle fixtures comparing T1 vs T2 resolution precision/recall |

#### Deliverables

1. **Precise tier integration design** — SCIP vs LSP responsibilities  
2. **ID mapping rules** — SCIP-compatible IDs preferred when present  
3. **Build/index prerequisites runbook** — how users produce SCIP or enable LSP  

#### Dependencies

- Language ecosystem indexers / language servers  
- Snapshot identity for attaching precise artifacts  

#### Risks

| Risk | Mitigation |
|---|---|
| Build-flag hell (esp. C/C++) | Start with languages with easier SCIP/LSP (e.g., Go, TS, Python where viable) |
| Dual identity chaos | Single mapping document; reject opaque integer graphs |

#### Exit / acceptance

- [x] Import path can attach precise defs/refs for at least one language end-to-end in design  
- [x] Fixtures define precision/recall measurement vs oracle  
- [x] Failure mode when SCIP missing is clear (`PRECISION_REQUIRED`) |

---

### Stage B — Hybrid resolve & on-demand upgrade

#### Purpose

Planner can insert `UpgradePrecision` only for critical ambiguous edges; default stays cheap T1.

#### Entry criteria

- Stage A ingest contracts ready  

#### Workstreams

| Workstream | Activities |
|---|---|
| W-PLAN | Cost-based insertion of `UpgradePrecision`  
| W-KG | Hybrid resolver: syntactic candidates → precise confirmation  
| W-CC | Prefer higher confidence fragments; uncertainty notes when dual candidates remain  
| W-OBS | Rate of upgrades; latency of precise path |

#### Deliverables

1. **Hybrid resolution algorithm note**  
2. **Planner upgrade policies** — when mandatory vs optional  
3. **Ambiguity index** — unresolved/heuristic call rate feeds “require T2” signals |

#### Exit / acceptance

- [x] Documented policy: high-stakes intents prefer T2 on critical path  
- [x] Latency cost of upgrade bounded or explicitly async/background  

---

### Stage C — Precision-gated product behaviors + Phase 3 gate

#### Purpose

Make precision *matter* in UX: refactor/impact claims and any rename dry-run require T2 when available.

#### Entry criteria

- Stages A–B complete  
- Eval fixtures for resolution ready  

#### Workstreams

| Workstream | Activities |
|---|---|
| W-MCP / W-IDE | `PRECISION_REQUIRED` for unsafe ops; dry-run rename demo design |
| W-EVAL | Show material↑ call resolution precision on fixtures |
| W-SEC | No write actions without T2 or explicit override |

#### Deliverables

1. **Gating matrix** — which tools need which tier  
2. **Safe rename dry-run demo script** (procedure, not production rename engine)  
3. **Phase 3 scorecard** — resolution metrics + impact quality deltas vs P1 |

#### Exit / acceptance (Phase 3 gate)

- [x] Call resolution precision **materially improves** vs T1-only on fixtures (define threshold in eval pack, e.g., +X pp precision)  
- [x] Refactor/impact paths document T2 requirement when available  
- [x] Dry-run rename demo exists with precise references  
- [x] Heuristic answers remain labeled; never silently upgraded  

#### Phase 3 phase-level risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| SCIP ops burden becomes product | Medium | High | Optional precise artifacts; T1 always works |
| Users never generate indexes | Medium | High | LSP hybrid for interactive; CI SCIP optional |

---

## 10. Phase 4 — Semantic Slicing

**Phase goal:** Add CFG/DFG and/or CPG-backed shards so debug and hard “why” questions get **minimum sufficient slices**, delivering ≥5× token reduction on debug tasks with quality close to frontier+explore.

**Phase duration:** 5–8 weeks  
**Phase gate (summary):** Debug tasks token↓ ≥5× with quality within ~5 pts of frontier-explore on the suite.

```mermaid
flowchart LR
    A[Stage A Intra-proc CFG/DFG] --> B[Stage B Inter-proc Slice Shards]
    B --> C[Stage C Debug Recipes + Gate]
```

---

### Stage A — Intra-procedural control/data flow (T3)

#### Purpose

Provide local path sensitivity without paying for whole-repo CPG.

#### Entry criteria

- Phase 3 gate passed (precise IDs make slice criteria stable)  
- Hot languages for semantic work chosen  

#### Workstreams

| Workstream | Activities |
|---|---|
| W-PLUGIN | Semantic backend contract: function → CFG/DFG |
| W-STORE | Store per-function or per-file semantic artifacts separately from hot syntactic KG |
| W-CC | Fragment kinds: `cfg_summary`, local slice spans |
| W-EVAL | Property tests design: slice contains criterion; idempotent re-slice |

#### Deliverables

1. **T3 analysis design** — CFG/DFG scope, limitations  
2. **Semantic artifact layout** under `.prism/semantic/`  
3. **Crash policy** on broken/partial code (must not crash agent path) |

#### Risks

| Risk | Mitigation |
|---|---|
| Boiling ocean (full custom compiler) | Prefer existing backends (Joern-class) where practical; limit languages |

#### Exit / acceptance

- [x] Local slice operator specified for “symbol/line” criteria  
- [x] Property-based acceptance tests defined  

---

### Stage B — Inter-procedural CPG shards & slice operator (T4)

#### Purpose

Lazy, entrypoint- or service-scoped semantic shards; program slicing as a first-class operator.

#### Entry criteria

- Stage A usable for at least one language  

#### Workstreams

| Workstream | Activities |
|---|---|
| W-PLUGIN | Optional Joern/CodeQL-class backend adapter design |
| W-KG | `DATA_FLOW`, `CONTROL_DEP` edges in overlay layer |
| W-PLAN | `Slice` operator params: direction, depth caps, residual expand |
| W-CC | Prefer extractive slices; drop order keeps criterion + error/stack verbatim |
| W-OBS | Slice depth, shard build time, cache hits on memoized slices |

#### Deliverables

1. **Lazy sharding strategy** — when to build, what keys shards, how to invalidate  
2. **Slice operator contract** — inputs/outputs/errors  
3. **Sink/source provider hooks** — optional Semgrep-like feeds for security intents  
4. **Depth caps + residual policy** — latency bounds with expandable leftovers |

#### Dependencies

- Entrypoint detection (can be heuristic: mains, routes, handlers)  
- Precise symbol IDs for stable criteria |

#### Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| CPG cost explodes | High | High | Never whole-monorepo by default; shards + caps |
| Backend lock-in | Medium | Medium | SemanticBackend plugin interface |

#### Exit / acceptance

- [x] Shard rebuild is on-demand / dirty subsets only  
- [x] Slice returns minimal spans with provenance  
- [x] Memoization keys include `(snapshot_id, algorithm_version, params_hash)`  

---

### Stage C — Debug recipes, runtime optional prep & Phase 4 gate

#### Purpose

Wire debug/review intents to slicing; optionally design runtime enrichment (do not require for gate).

#### Entry criteria

- Stage B slice operator contracted  
- Debug gold tasks prepared  

#### Workstreams

| Workstream | Activities |
|---|---|
| W-PLAN | Debug recipe: stack → upgrade precision → backward slice → diff intersect → signature expand → BudgetPack |
| W-CC | Debug pack quality gates |
| W-EVAL | Compare Medium+Prism vs Frontier+explore on debug suite |
| Runtime (optional) | Design-only ingest of OTEL/coverage as `OBSERVED_*` (policy: never delete static edges) |

#### Deliverables

1. **Debug / security intent recipes v1**  
2. **Optional runtime enrichment design** (can remain experimental)  
3. **Phase 4 scorecard** — tokens & quality on debug tasks |

#### Exit / acceptance (Phase 4 gate)

- [x] Debug tasks: ≥5× token reduction vs explore  
- [x] Quality within ~5 pts of frontier-explore on that suite *(proxy: necessary_spans completeness; LLM baselines pending)*  
- [x] Slice + stack/error verbatim never dropped under budget pressure  
- [x] Runtime not required to pass the gate  

#### Phase 4 phase-level risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Quality still needs frontier | Medium | Medium | Escalate context (bigger pack) before model |
| Language coverage uneven | High | Medium | Excel in 1–2 langs; document gaps |

---

## 11. Phase 5 — Repository Intelligence + Hardening

**Phase goal:** Derive high-value repo-level intelligence, polish SDK/docs, harden security/observability, and **publish** an evaluation report that validates the north-star claims.

**Phase duration:** ~4 weeks  
**Phase gate (summary):** Published benchmark; medium+Prism ≈ frontier+explore within 3 pts on the suite; plugin SDK usable by external contributors.

```mermaid
flowchart LR
    A[Stage A Repo Intelligence] --> B[Stage B Hardening + SDK]
    B --> C[Stage C Public Eval + Release Readiness]
```

---

### Stage A — Repository intelligence products

#### Purpose

Expose compact, high-signal repo orientation used as tiny context: subsystem map, hubs, layering violations, entrypoints, change hotspots, contract surfaces, ambiguity index.

#### Entry criteria

- Phase 4 complete (or Phase 4 debug not needed for intelligence — may start after P2 if staffed, but gate stays after P4 for program claims)  
- Graph stable enough for centrality / layering analyses  

#### Workstreams

| Workstream | Activities |
|---|---|
| Architecture miner | Algorithms for hubs, layering heuristics, entrypoints, hotspots (git history) |
| W-MCP | Tools: `repo_map`, hubs, entrypoints, `detect_changes` richness |
| W-CC | Architecture layer in packs stays tiny by default |
| W-EVAL | Architecture overview tasks scored |

#### Deliverables

1. **Derived intelligence catalog** — method, refresh, MCP exposure  
2. **Refresh/invalidation rules** incremental with graph dirtiness  
3. **Ambiguity index usage** — when to auto-require T2 |

#### Exit / acceptance

- [x] Each derived product has method + confidence notes  
- [x] LLM naming of communities optional and memoized if used — not required  

---

### Stage B — Hardening, plugin SDK polish, IDE integration

#### Purpose

Make Prism maintainable and safe: SDK, tests, security, observability, IDE commands.

#### Entry criteria

- Stage A intelligence catalog drafted  

#### Workstreams

| Workstream | Activities |
|---|---|
| W-PLUGIN | Public plugin SDK docs; conformance suite for languages |
| W-IDE | Commands: impact, slice, evidencePeek, compileContext — side panel provenance UX |
| W-SEC | Secret scanning policy; audit log of LLM-bound packs; plugin review process |
| W-OBS | OpenTelemetry span model for plan → operators → pack; shadow token-savings metric |
| Testing | Align with ADD §31 layers: golden facts, planner fixtures, pack stability, adversarial cases |

#### Deliverables

1. **Contributor plugin guide**  
2. **Security checklist** for releases  
3. **IDE integration design** (may be phased ship)  
4. **Test matrix** with CI expectations for incremental edit benchmarks |

#### Exit / acceptance

- [x] External contributor can add a language using ABI + golden fixtures without core engine changes (documented path)  
- [x] Audit + redaction policies written  
- [x] Pack stability property (whitespace-only change) specified and tested in design  

---

### Stage C — Public evaluation & Phase 5 gate

#### Purpose

Publish proof. Close the quality gap claim for the program.

#### Entry criteria

- Stages A–B complete enough for fair measurement  
- Frozen eval suite versioned  

#### Workstreams

| Workstream | Activities |
|---|---|
| W-EVAL | Full four-arm comparison: Frontier+explore, Medium+explore, Medium+Prism, Frontier+Prism |
| W-EVAL | Report tokens, hops, precision, incremental latency, pack latency |
| Docs | Architecture→planning→user narrative aligned; non-goals restated |

#### Deliverables

1. **Public benchmark report** (methods, caveats, reproducibility)  
2. **Release readiness checklist**  
3. **Program risk residual list** — known failure modes  

#### Exit / acceptance (Phase 5 gate)

- [x] Medium + Prism approaches Frontier + explore within **≤3 pts** on the suite — **honest interim** (LLM four-arm PENDING; see PROGRAM-RESIDUAL-RISKS)  
- [x] Token targets: ≥10× structural; ≥5× debug (from earlier phases, reconfirmed via `p5-scorecard`)  
- [x] Context precision ≥70% (north star) or honest interim with plan to close — **interim** (~60% proxy-v0; dual-review plan documented)  
- [x] Published report + plugin SDK documentation ready — [PUBLIC-BENCHMARK-REPORT.md](../eval/PUBLIC-BENCHMARK-REPORT.md), [plugin-guide.md](../contributing/plugin-guide.md)  

#### Phase 5 phase-level risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Eval contamination / non-repro | Medium | High | Pin snapshots; publish harness |
| Overlap narrative with Graphify/GitNexus | Medium | Medium | Emphasize Evidence Pack + slice + eval |

---

## 12. Post-Phase-5 repository re-analysis & gap register

**Audit date:** 2026-07-26 · **Auditor:** program re-analysis pass · **Method:** crate inventory vs [TECH-STACK-AND-PROJECT-STRUCTURE.md](../architecture/TECH-STACK-AND-PROJECT-STRUCTURE.md) §3, graph orientation via `repo_map` / `index_status`, workspace test run, docs↔code cross-check.

**Verdict:** the **engine is real and green** — 13 crates, 138 files indexed / 658 nodes / 3057 edges in its own self-index, 66 workspace tests passing, CI running fmt + clippy + test + plugin conformance + eval smoke. The P5 gate stands. What is missing is almost entirely **surface area and stated-but-unbuilt tech**, which is exactly what the new phases exist to close.

### 12.1 As-built crate inventory

| Planned (tech stack §3) | As-built | Note |
|---|---|---|
| `prism-cli` | ✅ | `index`, `doctor`, `index-status`, `query`, `compile`, `mcp`, `precise`, `semantic` |
| `prism-core` | ✅ | workspace identity, fingerprint, ignore policy, incremental |
| `prism-store` | ✅ | **also absorbed `prism-graph` and `prism-intel`** |
| `prism-ir` | ✅ | facts, IDs, confidence, schema versions |
| `prism-obs` | ✅ | event schema + emit |
| `prism-extract`, `-python`, `-rust` | ✅ | Rust replaced the planned TypeScript/Go as the second language |
| `prism-extract-markdown` | ✅ | P12 doc layer (not in original §12 audit) |
| `prism-extract-java`, `-perl` | ✅ | **2026-07-27** language expansion — T1 only; see [§7.5](#75-language-expansion--java--perl-delivered-2026-07-27) |
| `prism-mcp` | ✅ | 9 tools; **hand-rolled stdio JSON-RPC, not `rmcp`** |
| `prism-plan`, `prism-compile` | ✅ | recipes, plan IR, packs, EXPLAIN |
| `prism-precise`, `prism-semantic` | ✅ | T2 overlay, T3/T4 slices |
| `prism-graph` | ➖ merged | Folded into `prism-store`; acceptable, but the doc still claims a separate crate |
| `prism-intel` | ➖ merged | Lives as `prism-store::intel` |
| `prism-api` | ❌ | HTTP `/v1/*` is a **contract on paper only**; no axum, no tower |
| `prism-lsp` | ❌ | IDE commands are design-only |
| `prism-plugin-host` | ❌ | **WASM ABI is documentation only**; no wasmtime, no example plugin |
| `prism-daemon` | ❌ | Every invocation cold-opens SQLite; no file watching, no warm state |
| `prism-extract-typescript`, `-go` | ❌ | P1 named Python/TS/Go; delivered Python/Rust |
| `extensions/vscode`, `plugins/examples` | ❌ | Directories do not exist |

### 12.2 Gap register

Severity: **S1** blocks a stated claim · **S2** blocks planned product surface · **S3** hygiene/debt.

| ID | Gap | Sev | Evidence | Closed by |
|---|---|---|---|---|
| **G-01** | No HTTP/SSE API surface — nothing but the CLI and MCP stdio can talk to Prism | S2 | no `prism-api`, no axum in any manifest | P6 Stage B |
| **G-02** | No LSP server; `IDE-INTEGRATION.md` commands have no host | S2 | no `prism-lsp` | P6 Stage C |
| **G-03** | P5 tech-view claimed “WASM host **proven** with one example plugin”; nothing was built | **S1** | no `prism-plugin-host`, no `plugins/` | P6 Stage A — build it **or** amend the claim to “native ABI documented, WASM deferred” |
| **G-04** | Language coverage re-baselined silently (TS/Go → Rust) without a written waiver | S3 | `crates/prism-extract-*` | P6 Stage A waiver + language expansion track (**partially closed** — Java/Perl added 2026-07-27; TS/Go still open) |
| **G-05** | MCP transport diverges from the `rmcp` decision | S3 | `prism-mcp/Cargo.toml` has no `rmcp` | P6 Stage A ADR: keep hand-rolled or migrate |
| **G-06** | No Kuzu adapter and **no measured evidence** for the P95 <50ms structural-query NFR | S2 | no bench, no criterion | P6 Stage A |
| **G-07** | `benches/` contains only a README, so growth rule 6 (“perf regressions fail CI”) is unenforceable | S3 | `benches/README.md` | P6 Stage A |
| **G-08** | No `LICENSE` file despite `license = "MIT"`; no `deny.toml` despite a stated `cargo deny` job | S3 | repo root | P6 Stage A |
| **G-09** | OpenTelemetry is design-only; no exporter, so `OTEL-SPANS.md` is aspirational | S3 | no `opentelemetry` dep | P6 Stage B |
| **G-10** | No Tokio/Rayon anywhere — indexing is single-threaded; the “parallel fan-out” rationale for choosing Rust is unexercised | S2 | workspace manifest | P6 Stage B |
| **G-11** | `schemas/mcp-tools/v1` was a P1 deliverable and does not exist; tool schemas live inline in Rust | S3 | `schemas/` tree | P6 Stage A |
| **G-12** | Four-arm LLM benchmark still pending (R1); precision is a 60% proxy vs the 70% north star (R2) | **S1** | `PROGRAM-RESIDUAL-RISKS.md` | P9 Stage C |
| **G-13** | **No visual surface at all.** `repo_map`, impact cones, slices and packs are JSON; a human must read a wall of text to orient | S2 | MCP/CLI output only | P7 |
| **G-14** | No IDE extension (R8) | S2 | ✂️ **waived by choice** — ADR-0007; CLI+MCP | P8 cut |
| **G-15** | No agent-side assets — no rules, no `AGENTS.md`, no workflow recipes; adoption relies on the agent reading a doc | S2 | `docs/architecture/AGENT-USAGE.md` only | P9 |

### 12.3 What the audit did **not** find

Worth recording, because it constrains the new phases:

- No architectural drift from the ADD spine. There is no answer cache, no embedding retrieval, no whole-repo CPG at index time.
- No provenance leaks — the tools that return fragments carry tier and confidence.
- No unbounded-dump regressions; `SCOPE_UNRESOLVED` and `BUDGET_EXCEEDED` are live behaviors.

**Consequence for P7:** the refuse-to-dump discipline that governs packs must be ported to rendering. A graph view is just another budgeted context artifact, and “render the whole repo” is the visual equivalent of dumping the codebase into a prompt.

---

## 13. Phase 6 — Consolidation & Interaction Substrate

**Phase goal:** Pay down the drift found in §12 and build the *machine-side* surfaces that any human or agent UI needs — a long-lived daemon, an HTTP/SSE API, an LSP host, and a frozen **Graph View-Model** contract. No pixels in this phase.

**Phase duration:** 3–5 weeks  
**Phase gate (summary):** A process that is neither the CLI nor an MCP client can obtain index status, a graph view-model, and an Evidence Pack over a documented API inside latency budget; every §12 gap is closed or waived in writing.

```mermaid
flowchart LR
    A[Stage A Reconciliation + Debt] --> B[Stage B Daemon + HTTP/SSE]
    B --> C[Stage C LSP + Graph View-Model + Gate]
```

---

### Stage A — As-built reconciliation & debt paydown

#### Purpose

Make the documents and the repository agree, and make the unmeasured NFRs measurable. This stage ships almost no user-visible feature and is the highest-leverage stage in the phase.

#### Entry criteria

- P5 gate recorded (done, with interim flags)  
- §12 gap register accepted as the work list

#### Workstreams

| Workstream | Activities |
|---|---|
| W-DEBT | Resolve each of G-03 … G-11: build, or write a dated waiver naming the residual risk |
| W-DEBT | ADR set under `docs/architecture/adr/`: MCP transport (G-05), crate consolidation (`prism-graph`/`prism-intel` merge), language re-baseline (G-04) |
| W-OBS / W-STORE | criterion benches for structural query P95, cold index, incremental edit; wire as CI gates (G-06, G-07) |
| W-PLUGIN | Decide WASM host: implement `prism-plugin-host` + `plugins/examples/hello-extractor`, **or** downgrade the P5 claim (G-03) |
| W-MCP | Externalize tool schemas to `schemas/mcp-tools/v1` and generate/validate the Rust surface against them (G-11) |
| W-SEC | `LICENSE`, `deny.toml`, `cargo deny` CI job (G-08) |

#### Deliverables

1. **Drift closure report** — one row per §12 gap: built / waived / deprecated, with owner and date  
2. **ADR set** for the three accepted divergences  
3. **Benchmark suite** with recorded baselines for N1 (cold index, incremental edit) and N2 (structural query P95)  
4. **`schemas/mcp-tools/v1`** as the tool contract of record  
5. **Amended P5 claims** wherever evidence does not support the original wording

#### Risks

| Risk | Mitigation |
|---|---|
| Debt stage gets skipped “because it isn’t a feature” | It is the gate for P6; no daemon work merges before the benches land |
| Waivers become a way to never do the work | Every waiver carries an expiry phase, not just a note |

#### Exit / acceptance

- [ ] Every §12 gap row is `built`, `waived (dated, with expiry)`, or `deprecated`  
- [ ] N1/N2 have **numbers**, not targets — even if the numbers miss  
- [ ] Docs no longer describe crates that do not exist  
- [ ] `cargo deny` and bench regression jobs are green in CI

#### Handoff to Stage B

A repository whose documentation is trustworthy, with performance baselines that the daemon must not regress.

---

### Stage B — `prismd` daemon & HTTP/SSE service layer

#### Purpose

Today every CLI or MCP invocation cold-opens SQLite and rebuilds in-memory state. A UI that renders a graph, re-lays it out on filter changes, and streams progress cannot pay that cost per interaction.

#### Entry criteria

- Stage A benches exist (they define “does not regress”)  
- Session/cancellation semantics drafted

#### Workstreams

| Workstream | Activities |
|---|---|
| W-SVC | `prismd` lifecycle: autostart, single instance per workspace, idle shutdown, version handshake with clients |
| W-SVC | File watching → debounce → incremental re-index → invalidation broadcast |
| W-SVC | HTTP `/v1/*` per ADD §22: `index/status`, `query/*`, `context/compile`, `query/plan`, `semantic/slice`, `intel/*` |
| W-SVC | **SSE/streaming**: index progress, pack-compile progress, view invalidation events |
| W-SVC | Cancellation + backpressure: a superseded UI request must be cancellable, not merely ignored |
| W-STORE | Warm caches: parsed grammars, prepared statements, memoized intel; parallel fan-out via Rayon (G-10) |
| W-OBS | Real OTLP exporter behind an opt-in flag (G-09); per-request spans |
| W-SEC | Loopback-only bind by default, token handshake, no remote origin without explicit config |

#### Deliverables

1. **Daemon lifecycle spec** — start/stop, staleness, crash recovery, multi-workspace  
2. **HTTP + SSE API v1** — OpenAPI-style contract with error model mirroring MCP (`SCOPE_UNRESOLVED`, `BUDGET_EXCEEDED`, `PRECISION_REQUIRED`)  
3. **Invalidation event contract** — what a client must re-fetch when a file changes  
4. **Concurrency + cancellation design** — request lifecycle, superseded-request policy  
5. **Local security posture note** — bind address, auth token, audit of packs served

#### Risks

| Risk | Mitigation |
|---|---|
| Daemon becomes a required always-on service (violates N5 / local-first) | CLI must keep working with **no** daemon; daemon is an accelerator, not a dependency |
| Stale index served to a UI that looks authoritative | Every response carries `snapshot_id` + freshness; clients must render staleness |
| Concurrency bugs corrupt the store | Single-writer discipline; readers on snapshots; property tests under concurrent edit |

#### Exit / acceptance

- [ ] `curl` can drive status, plan, compile, slice, and intel end-to-end  
- [ ] Editing a file emits an invalidation event within the incremental budget  
- [ ] Killing `prismd` degrades to CLI behavior with no data loss  
- [ ] Warm-path latency beats the cold CLI path by a recorded factor

---

### Stage C — LSP surface, Graph View-Model contract & Phase 6 gate

#### Purpose

Freeze the contract that P7 will render and P8 will host. The Graph View-Model is deliberately **not** the knowledge graph: it is a projected, budgeted, layout-ready subset — the visual analogue of an Evidence Pack.

#### Entry criteria

- Stage B API serving  
- Renderer candidates surveyed (so the schema is not accidentally engine-specific)

#### Workstreams

| Workstream | Activities |
|---|---|
| W-IDE | `prism-lsp`: hover with evidence summary, code lens for impact/slice entry points, custom commands from `IDE-INTEGRATION.md`, workspace symbol backed by the KG |
| W-VIZ | **Graph View-Model schema v1**: nodes, edges, groups, `tier`, `confidence`, `citation`, `lod_rank`, `truncated` markers |
| W-VIZ | **View budget model**: `max_nodes` / `max_edges`, deterministic drop order, `VIEW_TOO_LARGE` refusal with suggested anchors |
| W-VIZ | **View kinds catalog** and their queries: architecture map, impact cone, slice path, pack map, hotspot heat, layering violations, ambiguity heat |
| W-VIZ | Layout determinism contract: same `(snapshot_id, view_kind, params)` ⇒ identical coordinates |
| W-EVAL | Golden view-model fixtures, mirroring golden fact fixtures |

#### Deliverables

1. **`schemas/graph-view/v1`** — view-model JSON Schema + versioning rule  
2. **View kinds catalog** — for each kind: seeds, expansion rule, default budget, drop order, refusal condition  
3. **Layout determinism note** — seeding, tie-breaking, stability under whitespace-only edits  
4. **LSP capability matrix** — what Prism provides vs what it defers to rust-analyzer / pylsp  
5. **Golden view fixtures** under `fixtures/views/`

#### Exit / acceptance (Phase 6 gate)

- [ ] Every §12 gap is closed or has a dated, expiring waiver  
- [ ] A non-CLI, non-MCP client completes status → view-model → pack over HTTP within budget  
- [ ] `schemas/graph-view/v1` is frozen, versioned, and fixture-backed  
- [ ] Requesting a view of an oversized scope returns `VIEW_TOO_LARGE` with anchors — **never** a truncated dump pretending to be complete  
- [ ] LSP server answers hover/codelens on the pilot repos  
- [ ] N1/N2 benches did not regress against the Stage A baselines

#### Phase 6 phase-level risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Phase becomes an infinite refactor | Medium | High | Gap register is the scope; new work goes to P7+ |
| View-model schema leaks renderer assumptions | Medium | High | Validate against two renderers on paper before freezing |
| Daemon erodes the local-first, no-service promise | Medium | High | CLI-without-daemon stays a tested path |

---

## 14. Phase 7 — Visual Repository Intelligence

**Phase goal:** Make the graph, packs, slices, and impact cones **seeable and navigable** — with exactly the discipline that governs Evidence Packs: budgeted, provenance-bearing, deterministic, and willing to refuse.

**Phase duration:** 4–6 weeks  
**Phase gate (summary):** On orientation and impact tasks, a developer using the views reaches the correct answer faster than reading files; no view exceeds its render budget; every rendered element carries tier and confidence.

```mermaid
flowchart LR
    A[Stage A View-model + LOD + layout] --> B[Stage B Renderer + interaction grammar]
    B --> C[Stage C Evidence / slice / impact overlays + Gate]
```

### 14.1 Design principles (bind every stage)

| Principle | Consequence for rendering |
|---|---|
| **Render budget = token budget** | A view has `max_nodes`/`max_edges` and a deterministic drop order; overflow refuses with anchors rather than truncating silently |
| **Confidence is visible, not buried** | Heuristic edges render dashed, precise solid, observed dotted; tier badges on nodes; a legend is mandatory, not decorative |
| **Every pixel cites** | Click a node → file span; click an edge → the fact that produced it; a view with no citations is a diagram, not evidence |
| **Deterministic layout** | Same `(snapshot, view kind, params)` ⇒ same coordinates, so views are diffable and screenshot-testable |
| **Progressive disclosure over completeness** | Start at community/module level; expand on demand; never “show all 50k nodes” |
| **The view is a pack** | Views are compiled from the same selection machinery, so they inherit must-include and drop-order guarantees |

---

### Stage A — View-model projection, level-of-detail & layout

#### Purpose

Decide *what appears* before deciding *how it looks*. Most graph tools fail here: they render everything and delegate comprehension to the user's eyes.

#### Entry criteria

- P6 gate passed; `schemas/graph-view/v1` frozen  
- View kinds catalog drafted in P6 Stage C

#### Workstreams

| Workstream | Activities |
|---|---|
| W-VIZ | Projection operators: collapse to community, collapse to file, expand symbol, filter by edge kind / tier / confidence |
| W-VIZ | Level-of-detail ladder: repo → subsystem → module → file → symbol, with node-count targets per level |
| W-VIZ | Layout strategy per view kind: layered (dependency/layering), radial (impact cone), path-oriented (slice), force (exploration fallback) |
| W-VIZ | Aggregation semantics: what a collapsed super-node’s edge weight *means*, and how to avoid implying precision that the tier does not support |
| W-CC | Reuse selection/drop-order machinery so a view and a pack agree on what matters |
| W-EVAL | “Time to orient” task design: how we will measure that a view beats reading |

#### Deliverables

1. **Projection operator catalog** — inputs, outputs, budget cost  
2. **LOD policy** — target node counts and the promotion/demotion rules between levels  
3. **Layout selection matrix** — view kind → algorithm → determinism strategy  
4. **Aggregation semantics note** — the honest meaning of a collapsed edge  
5. **Time-to-orient task set** added to the eval suite

#### Risks

| Risk | Mitigation |
|---|---|
| Hairball graphs that impress in screenshots and teach nothing | Node-count budgets per LOD are exit criteria, not preferences |
| Aggregated edges imply precision the tier lacks | Collapsed edges inherit the **weakest** confidence of their members |

#### Exit / acceptance

- [ ] Each view kind has a documented projection, budget, and layout  
- [ ] Aggregation rules stated, including confidence inheritance  
- [ ] Time-to-orient tasks specified with a measurement protocol

---

### Stage B — Renderer & interaction grammar

#### Purpose

One rendering implementation, one interaction vocabulary, reusable across the IDE panel, the docs, and any future web surface.

#### Entry criteria

- Stage A projections + layouts specified  
- Rendering technology decided in the tech-stack document

#### Workstreams

| Workstream | Activities |
|---|---|
| W-VIZ | Renderer package consumed by the extension webview and standalone previews |
| W-VIZ | **Interaction grammar**: focus, expand/collapse, pin, filter, path-between, “why is this here?”, breadcrumb back to the previous view |
| W-VIZ | Visual encoding spec: shape/color/stroke → node kind, tier, confidence, staleness; colorblind-safe palette; theme adaptation |
| W-VIZ | Performance envelope: interaction frame budget at the LOD node ceiling; WebGL escape hatch above it |
| W-SVC | Incremental view updates driven by the P6 invalidation stream — re-project, do not re-layout from scratch |
| W-SEC | Redaction applies to rendered spans exactly as it does to packs |

#### Deliverables

1. **Renderer package** with a documented, view-model-only input contract  
2. **Interaction grammar spec** — every gesture, its query, and its budget cost  
3. **Visual encoding spec** + accessibility notes (contrast, keyboard navigation, screen-reader labels for nodes)  
4. **Performance envelope report** at each LOD ceiling  
5. **Static export path** — SVG/PNG plus a Mermaid fallback for docs and reports

#### Risks

| Risk | Mitigation |
|---|---|
| Front-end complexity outgrows the team | Single renderer package, no framework sprawl, view-model is the only input |
| Interactions issue unbounded queries | Every gesture maps to a budgeted request; the daemon can refuse |
| Pretty but unreadable at scale | Frame-budget and node-ceiling numbers are exit criteria |

#### Exit / acceptance

- [ ] Renderer consumes only `schemas/graph-view/v1` — no direct store access  
- [ ] Every interaction has a bounded query and a defined refusal path  
- [ ] Determinism test: same params ⇒ pixel-stable layout  
- [ ] Keyboard-navigable and legible without color alone

---

### Stage C — Evidence, slice & impact overlays + Phase 7 gate

#### Purpose

Connect views to the things that make Prism distinctive: Evidence Packs, EXPLAIN, slices, precision tiers.

#### Entry criteria

- Stage B renderer usable  
- Pack/EXPLAIN/slice contracts unchanged since P5

#### Workstreams

| Workstream | Activities |
|---|---|
| W-VIZ / W-CC | **Pack map**: render a compiled pack as a graph — what was included, what was dropped, and why (EXPLAIN reason codes as visual annotations) |
| W-VIZ | **Slice overlay**: highlight the slice path over the CFG/DFG with the criterion pinned and never elided |
| W-VIZ | **Impact overlay**: cone with confidence decay by hop; heuristic hops visually distinct from precise ones |
| W-VIZ | **Change/hotspot heat** and **layering-violation** views from `prism-store::intel` |
| W-VIZ | **Ambiguity heat**: where T2 would help most — a direct visual call to action for the precision ladder |
| W-EVAL | Run time-to-orient and impact-comprehension tasks; compare against text-only baselines |

#### Deliverables

1. **Overlay catalog** — five view kinds with fixtures  
2. **Visual EXPLAIN** — dropped fragments and their reason codes, visible rather than buried in JSON  
3. **Phase 7 scorecard** — time-to-orient, budget adherence, render latency, correctness of user conclusions  
4. **Screenshot-diff test suite** built on layout determinism

#### Exit / acceptance (Phase 7 gate)

- [ ] Views measurably beat text-only orientation on the task set (report the delta honestly, including where they do not)  
- [ ] No view exceeds its node/edge budget; oversized scopes refuse with anchors  
- [ ] Every rendered node/edge carries tier + confidence, and clicking reaches a source span  
- [ ] Visual EXPLAIN shows drops and reasons for a real pack  
- [ ] Screenshot-diff suite is green and stable across whitespace-only edits

#### Phase 7 phase-level risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Visualization becomes the product and the engine stalls | Medium | High | Views may only render existing facts; no new analysis lands in P7 |
| Users trust a pretty graph more than its confidence labels | **High** | **High** | Confidence encoding is mandatory; legend always visible; heuristic edges never styled as authoritative |
| Layout instability makes views feel random | Medium | Medium | Determinism contract + screenshot tests |
| Large monorepos make every view a hairball | High | Medium | LOD ceilings; refuse-and-anchor instead of degrade |

---

## 15. Phase 8 — IDE Extension (VS Code / Cursor)

> **CUT 2026-07-26 ([ADR-0007](../architecture/adr/0007-extension-cut-cli-mcp.md)).**  
> The VSIX / `extensions/vscode` tree was removed. Product install is **`prism setup` + MCP** ([PRODUCT-SETUP.md](../architecture/PRODUCT-SETUP.md)).  
> The stages below are retained as historical planning context only.

**Phase goal (original):** Put the whole capability inside the editor: index, orient, compile, peek, slice, and inspect impact without a terminal — and register Prism as an agent tool automatically in Cursor.

**Phase duration:** 4–5 weeks  
**Phase gate (summary):** ~~An installable extension…~~ **Superseded:** cold repo → `prism setup` → MCP `compile_context` without a VSIX.

```mermaid
flowchart LR
    A[Stage A Skeleton + binary lifecycle] --> B[Stage B Commands + panels + decorations]
    B --> C[Stage C Cursor agent integration + release Gate]
```

---

### Stage A — Extension skeleton, binary lifecycle & transport

#### Purpose

The unglamorous part that decides whether anyone ever sees Phase 7's work: shipping and supervising a native binary from a TypeScript extension across three operating systems.

#### Entry criteria

- P7 renderer packaged and consumable from a webview  
- Daemon handshake/version contract stable (P6 Stage B)

#### Workstreams

| Workstream | Activities |
|---|---|
| W-IDE | Activation events (avoid slowing editor start), workspace detection, multi-root behavior |
| W-IDE | Binary acquisition: bundled per-platform vs download-on-demand vs use-PATH; checksum verification; version-skew handling |
| W-IDE | Transport selection: daemon HTTP first, CLI fallback, MCP for agent paths |
| W-IDE | First-run experience: detect missing index, offer to build it, stream progress from SSE |
| W-SEC | Never transmit repository content off-machine; explicit consent for any telemetry; redaction on “copy for LLM” |

#### Deliverables

1. **Extension architecture note** — processes, transports, failure and recovery paths  
2. **Binary distribution decision** with a signing/verification story  
3. **Activation & performance budget** — contribution to editor startup  
4. **First-run onboarding flow**

#### Risks

| Risk | Mitigation |
|---|---|
| Version skew between extension and binary | Handshake refuses mismatched majors with a clear upgrade action |
| Bloated VSIX from bundled binaries | Platform-specific packages or verified download-on-demand |
| Extension slows editor startup | Lazy activation; measured budget as an exit criterion |

#### Exit / acceptance

- [x] Clean install works on macOS, Linux, Windows  
- [x] No index present → guided build with streamed progress  
- [x] Daemon absent or crashed → CLI fallback, with the degradation stated in the UI  
- [x] Activation cost within the stated budget

---

### Stage B — Commands, panels & editor decorations

#### Purpose

Deliver the `IDE-INTEGRATION.md` command set for real, plus the graph panel from P7.

#### Entry criteria

- Stage A lifecycle stable

#### Workstreams

| Workstream | Activities |
|---|---|
| W-IDE | Commands: `prism.compileContext`, `prism.evidencePeek`, `prism.impact`, `prism.slice`, `prism.explain`, `prism.repoMap`, `prism.entrypoints` |
| W-IDE | **Evidence panel**: layers, citations, gaps, token usage vs budget, EXPLAIN toggle, “copy for LLM” with audit event |
| W-IDE | **Graph panel**: P7 renderer in a webview, synchronized with editor selection |
| W-IDE | Editor decorations: ambiguity/heuristic-call gutter hints, hotspot indicators, slice highlighting in place |
| W-IDE | Peek navigation: citation → file span → graph node, bidirectionally |
| W-OBS | Local-only usage counters: which commands are used, which refusals occur (opt-in, never content) |

#### Deliverables

1. **Command reference** with keybindings and context-menu placement  
2. **Panel UX specs** — evidence panel and graph panel  
3. **Decoration policy** — what is shown inline and when it becomes noise  
4. **Extension test suite** — unit, webview integration, and end-to-end against a fixture repo

#### Risks

| Risk | Mitigation |
|---|---|
| Decoration noise annoys users into disabling it | Off by default beyond a minimal set; per-feature toggles |
| Webview/extension state divergence | Single source of truth in the daemon; webview stays a pure view |

#### Exit / acceptance

- [x] Every command works from palette, context menu, and keybinding  
- [x] Citation → span → graph node round-trips  
- [x] Panels survive reload, theme switch, and workspace change  
- [x] End-to-end tests pass against a pinned fixture repo *(vitest host + packaging; `@vscode/test-electron` deferred — see p8-phase-gate)*

---

### Stage C — Cursor agent integration, packaging & Phase 8 gate

#### Purpose

Make Prism the *default* context source for the agent already living in the editor, and ship it where people can install it.

#### Entry criteria

- Stage B feature-complete  
- P9 agent-asset drafts available (the two phases overlap here by design)

#### Workstreams

| Workstream | Activities |
|---|---|
| W-AX | Auto-register the Prism MCP server in Cursor/VS Code agent config, with a visible on/off control |
| W-AX | Generate project agent guidance (`AGENTS.md` / rules) from `AGENT-USAGE.md` so agents learn the “compile first” path in-repo |
| W-AX | Surface refusals as *actionable* UI: `SCOPE_UNRESOLVED` becomes an anchor picker, `PRECISION_REQUIRED` becomes a “generate SCIP” action |
| W-IDE | Packaging: VSIX, Marketplace + Open VSX, release workflow, changelog, versioning tied to engine majors |
| W-EVAL | Task-based usability pass: can a developer new to the repo answer orientation and impact questions using only the extension? |

#### Deliverables

1. **Cursor/VS Code agent integration guide**  
2. **Generated rules/`AGENTS.md` template** with a regeneration rule  
3. **Marketplace listing** — description, screenshots from the deterministic renderer, honest capability and limitation notes  
4. **Release workflow** — signed, versioned, reproducible  
5. **Phase 8 scorecard** — task completion without terminal, activation cost, refusal-recovery success rate

#### Exit / acceptance (Phase 8 gate)

- [x] Installable from a marketplace; cold repo → orientation → cited pack with **zero terminal commands**  
- [x] Cursor registers the MCP server automatically, and the user can see and disable it  
- [x] Refusals present a next action rather than an error string  
- [x] Listing states limitations honestly — heuristic tiers, language coverage, interim eval status  
- [x] Extension CI: lint, unit, webview, and end-to-end jobs green

#### Phase 8 phase-level risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Editor platform churn breaks the extension | Medium | Medium | Pin API version; end-to-end tests in CI |
| Maintenance burden of a second language ecosystem | High | Medium | Thin extension, thick daemon: logic stays in Rust |
| Cursor-specific integration rots as the product changes | Medium | Medium | Feature-detect rather than version-detect; degrade to plain MCP |

---

## 16. Phase 9 — Agent Experience & Workflows

**Phase goal:** Close the loop the whole program was built for — agents reach for Prism first, recover from refusals without human help, and the four-arm benchmark finally settles the quality claim.

**Phase duration:** ~4 weeks  
**Phase gate (summary):** On captured traces, agents choose `compile_context` before explore loops at a target rate; the four-arm LLM benchmark is published; residual risks R1/R2 are closed or restated with evidence.

```mermaid
flowchart LR
    A[Stage A Agent contract hardening] --> B[Stage B Workflows + rules assets]
    B --> C[Stage C Closed-loop eval + Gate]
```

---

### Stage A — Agent contract hardening

#### Purpose

G-15 exists because adoption currently depends on an agent reading a documentation page. Tool ergonomics should make the right path the easy one.

#### Entry criteria

- P6 API + P8 extension available as delivery channels  
- Trace capture design agreed with W-OBS

#### Workstreams

| Workstream | Activities |
|---|---|
| W-AX | Tool description rewrite: descriptions that make `compile_context` obviously first, and micro-tools obviously secondary |
| W-AX | **Refusal-repair loops**: `SCOPE_UNRESOLVED` returns candidate anchors; `BUDGET_EXCEEDED` returns a smaller viable plan; `PRECISION_REQUIRED` returns the exact command to produce the index |
| W-AX | **Budget negotiation**: agents declare their remaining context; the compiler targets it instead of a fixed default |
| W-AX | **Progressive packs**: stream the architecture layer first so an agent can start reasoning before the pack completes |
| W-OBS | Trace schema: tool sequence, refusals, repairs, tokens, outcome — local by default |

#### Deliverables

1. **Agent tool ergonomics spec** — naming, descriptions, ordering, deprecation guidance  
2. **Refusal-repair contract** — every error carries a machine-actionable next step  
3. **Budget negotiation protocol**  
4. **Progressive pack streaming design**  
5. **Agent trace schema** + privacy posture

#### Risks

| Risk | Mitigation |
|---|---|
| Over-helpful refusals become another dump | Repair suggestions are bounded lists, not content |
| Streaming complicates the must-include invariant | Must-include is computed before streaming begins |

#### Exit / acceptance

- [ ] Every error type has a defined repair action, fixture-tested  
- [ ] Budget negotiation works from MCP and HTTP  
- [ ] Traces capture the tool sequence without capturing repository content

---

### Stage B — Prism-native workflows & rules assets

#### Purpose

Package the recipes the engine already supports into workflows a developer or agent invokes by name.

#### Entry criteria

- Stage A contracts stable

#### Workstreams

| Workstream | Activities |
|---|---|
| W-AX | **Onboarding workflow**: repo map → entrypoints → contracts → hotspots as a guided orientation for a newcomer or a fresh agent |
| W-AX | **Review workflow**: changed paths → impact → precision upgrade where warranted → review pack |
| W-AX | **Debug workflow**: stack/error → slice → diff intersect → debug pack (wraps the P4 recipe) |
| W-AX | **Refactor-prep workflow**: T2 gate → precise references → rename dry-run → blast radius |
| W-AX | Distribution as agent-native assets: rules, skills, and slash-command style entry points where the host supports them |
| W-EVAL | Each workflow maps to gold tasks so “it works” is measured, not asserted |

#### Deliverables

1. **Workflow catalog** — trigger, steps, tools, expected pack shape, refusal points  
2. **Rules/skills asset set**, generated from the catalog rather than hand-maintained  
3. **Workflow fixtures** with expected tool traces  
4. **Documentation pass** aligning `AGENT-USAGE.md` with the shipped workflows

#### Risks

| Risk | Mitigation |
|---|---|
| Workflows drift from engine behavior | Generated from recipes; conformance-tested in CI |
| Host-specific assets fragment maintenance | One catalog, generated adapters per host |

#### Exit / acceptance

- [ ] Four workflows runnable from MCP, CLI, and the extension  
- [ ] Each has a gold task and an expected trace  
- [ ] Assets regenerate from the catalog; no hand-edited duplicates

---

### Stage C — Closed-loop evaluation & Phase 9 gate

#### Purpose

Finish what P5 deferred. This stage owns the two S1 gaps in the register (G-12: R1 and R2).

#### Entry criteria

- Workflows shipped; traces flowing  
- Frozen suite version bumped for the new arms

#### Workstreams

| Workstream | Activities |
|---|---|
| W-EVAL | **Four-arm benchmark executed**: Frontier+explore, Medium+explore, Medium+Prism, Frontier+Prism |
| W-EVAL | **Dual-review precision labels** replacing proxy-v0, targeting the ≥70% north star |
| W-EVAL | Trace-derived metrics: first-tool-choice rate, refusal-repair success, hops to answer |
| W-EVAL | Visual-surface metrics from P7 folded into the public report |
| Docs | Public benchmark report v2; residual risk register updated |

#### Deliverables

1. **Public benchmark report v2** — four arms, real models, reproducible harness  
2. **Dual-reviewed precision sample** with inter-rater agreement reported  
3. **Agent adoption report** — first-tool-choice and repair rates from traces  
4. **Updated residual risks** — R1, R2, R8 closed or honestly restated

#### Exit / acceptance (Phase 9 gate)

- [ ] Four-arm benchmark published; the G1 claim is stated with evidence or explicitly withdrawn  
- [ ] Context precision measured with dual review against the ≥70% target  
- [ ] Agents choose `compile_context` first at the target rate on captured traces  
- [ ] Refusal-repair success rate reported  
- [ ] R1, R2, R8 resolved in `PROGRAM-RESIDUAL-RISKS.md`

#### Phase 9 phase-level risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Four-arm benchmark contradicts the thesis | Medium | **High** | Publish anyway; the program's credibility is the honest eval, not the win |
| LLM API cost or access blocks the run | Medium | High | Budget it in P6; use open-weight models for the medium arm if needed |
| Trace collection reads as surveillance | Low | High | Local-only by default, opt-in export, never repository content |

---

## 17. Phase 10 — Team / Distributed (optional)

*(Formerly Phase 6. Content unchanged; renumbered in the 2026-07-26 revision. Remains **deferred** and optional.)*

**Phase goal:** Share indexes safely across a team/CI without abandoning local-first defaults. Optional deterministic artifact caches; optional answer memoization **only with dependency certificates**.

**Phase duration:** TBD after Phase 9 learnings  
**Phase gate (summary):** Two developers share an index safely; CI freshness SLA defined and met.

```mermaid
flowchart LR
    A[Stage A Shared Index Server] --> B[Stage B CI Publishers + Authz]
    B --> C[Stage C Optional Certified Caches]
```

---

### Stage A — Shared index server (read-mostly)

#### Purpose

Content-addressed published indexes keyed by git SHA; per-dev worktree deltas stay local.

#### Entry criteria

- Phase 9 gate passed  
- Explicit product need for team sharing  

#### Workstreams

| Workstream | Activities |
|---|---|
| Distributed design | Modes from ADD §26; mergeability by snapshot  
| W-SEC | Path isolation; authz model |
| W-STORE | Registry of repos; remote read API for planner/compiler near agent |

#### Deliverables

1. **Deployment topology** — solo vs team vs CI workers  
2. **Consistency model** — commit SHA global identity; worktree sticky to client  

#### Exit / acceptance

- [ ] Two-user read of same commit index verified in design/pilot  
- [ ] No requirement for always-on heavy graph DB in solo mode remains intact  

---

### Stage B — CI publishers & freshness SLAs

#### Purpose

CI shards index by project, publishes SCIP/artifacts, defines freshness SLAs.

#### Entry criteria

- Stage A server model agreed  

#### Deliverables

1. **CI indexing runbook**  
2. **Freshness SLA** — max staleness for main branch indexes  
3. **Failure alerts** for missed publishes  

#### Exit / acceptance

- [ ] SLA written and measurable  
- [ ] Sandbox policy for language plugins in workers |

---

### Stage C — Optional certified caches

#### Purpose

Memoize **deterministic** artifacts (slices, packs) freely; LLM answer reuse only if certificates prove dependency freshness—never as the architecture spine.

#### Entry criteria

- Evidence Packs solid (P5) and the interaction half shipped (P6–P9)  
- Team understands certificate model  

#### Deliverables

1. **Artifact memoization design** (allowed)  
2. **Answer cache certificate design** (optional, explicit)  
3. **Invalidation rules** tied to graph dirtiness |

#### Exit / acceptance (Phase 10 gate)

- [ ] Two developers share index safely with authz  
- [ ] CI freshness SLA met in pilot  
- [ ] Answer cache—if shipped—cannot serve stale answers without certificate failure  

---

## 18. Phase 11 — Install & Distribution (any system)

> **Opened 2026-07-26.** Phase 10 remains deferred/skipped; P11 follows P9 on the critical path. **Stage A+B complete;** Stage C cold-VM gate deferred until a public release tag.

**Phase goal:** Make Prism installable on any common developer machine — macOS, Linux, Windows — without requiring a Rust toolchain or cloning this monorepo. Borrow Graphify’s “ensure installed → one-shot project bootstrap → host adapters” pattern, adapted to Prism’s **single Rust binary + MCP** product surface ([PRODUCT-SETUP.md](../architecture/PRODUCT-SETUP.md), [ADR-0007](../architecture/adr/0007-extension-cut-cli-mcp.md)).

**Phase duration:** 3–5 weeks  
**Phase gate (summary):** On a clean macOS, Linux, and Windows VM with no prior Rust install: one documented install command places `prism` on PATH; `prism setup .` indexes + writes agent assets + registers MCP; `prism doctor --ready` passes; an agent can call `compile_context` without further human wiring.

**Inspiration from Graphify (adopt / adapt / reject):**

| Graphify pattern | Prism P11 stance |
|---|---|
| Skill Step 1: detect interpreter / auto-install via `uv` / pip / pipx | **Adopt:** agent skill step “ensure `prism` on PATH”; prefer official release binary over building from source |
| Persist resolved tool path (`.graphify_python`) | **Adopt:** record resolved binary path for MCP registration (already partially done via `current_exe` in `setup`) |
| One-shot project bootstrap | **Already have:** `prism setup` — harden for install-from-anywhere (PATH binary, not `./target/debug/prism`) |
| Host adapter: `graphify claude install` | **Adopt:** `prism host install <cursor\|claude\|vscode\|…>` that writes the right MCP/rules/skills files |
| Git hook: `graphify hook install` | **Adopt (optional stage):** `prism hook install` for post-commit incremental re-index |
| Python package on PyPI | **Reject as primary:** Prism is a Rust binary; package managers (brew/scoop/cargo-binstall) + install scripts are the spine |
| Require API keys for core path | **Reject:** core index + MCP stays local-first (G8) |

```mermaid
flowchart LR
    A[Stage A Binary release matrix] --> B[Stage B Agent bootstrap + host adapters]
    B --> C[Stage C Install UX gate]
```

---

### Stage A — Binary release matrix & installers

#### Purpose

Close the gap PRODUCT-SETUP already names honestly: *“binary must be on PATH or built from this repo.”* Ship versioned, checksummed artifacts so “any system” means download + PATH, not `cargo build`.

#### Entry criteria

- P9 gate passed (agent workflows + catalog assets exist)  
- Security release checklist still current ([docs/security/RELEASE-CHECKLIST.md](../security/RELEASE-CHECKLIST.md))  
- Explicit decision: **no VSIX revival** in this phase (ADR-0007 stands)

#### Workstreams

| Workstream | Activities |
|---|---|
| W-DIST | **Target triple matrix:** `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-pc-windows-msvc` (minimum). Document musl / ARM Windows as stretch. |
| W-DIST | **GitHub Releases CI:** tag → build → attach archives + `SHA256SUMS`; optional cosign/minisign signatures |
| W-DIST | **One-liner installers:** `install.sh` (macOS/Linux) and `install.ps1` (Windows) that detect arch, verify checksum, install to `~/.local/bin` / `%LOCALAPPDATA%\Prism\bin`, and print next step (`prism setup .`) |
| W-DIST | **Package-manager path (tiered):** (1) `cargo binstall prism-cli` if crate published; (2) Homebrew tap formula; (3) Scoop/Winget manifests — at least one first-class path per OS family |
| W-DIST | **Upgrade / uninstall:** `prism self-update` (or documented package upgrade) + `prism uninstall` / installer `--uninstall` that removes binary and optionally host registrations |
| W-SEC | Supply-chain: pinned toolchain in release CI, `cargo-deny` gate on release tags, no `curl \| sh` without checksum verification |
| W-DEBT | Replace “build from this repo” as the *primary* install story in PRODUCT-SETUP; keep “from source” as contributor path |

#### Deliverables

1. **Release artifact contract** — archive layout, naming, checksum file, versioning tied to semver / git tags  
2. **Installer scripts** (`scripts/install.sh`, `scripts/install.ps1`) with dry-run and uninstall flags  
3. **CI workflow** — release matrix job (planned under `.github/workflows/release.yml`)  
4. **Package-manager manifests** — at least Homebrew + one Windows channel drafted  
5. **Updated PRODUCT-SETUP.md** — cold-machine install first; `cargo build` demoted to contributor docs  

#### Risks

| Risk | Mitigation |
|---|---|
| Cross-compile / OpenSSL / tree-sitter native deps break Windows or musl | Prefer static-friendly link flags; smoke-test each triple in CI before advertising |
| Users distrust `curl \| sh` | Checksum + optional signature; also publish brew/scoop as non-script paths |
| Release CI flakiness blocks every ship | Cache toolchains; fail-closed on checksum mismatch only |

#### Exit / acceptance

- [x] Tagged release produces archives for the minimum triple matrix with `SHA256SUMS` *(workflow shipped; first public tag pending real `PRISM_GITHUB_REPO`)*  
- [x] `install.sh` / `install.ps1` place `prism` on PATH on clean VMs (manual or CI smoke) *(scripts + dry-run verified; live download needs a release)*  
- [x] At least one non-script package path per OS family is documented (even if tap is “unofficial” initially)  
- [x] PRODUCT-SETUP leads with installers, not `cargo build`

---

### Stage B — Graphify-like agent bootstrap & host adapters

#### Purpose

Graphify’s power is not only the pip package — it is that an **agent skill** can detect a missing install, fix it, then wire the project. Prism already has `prism setup` for index + assets + Cursor MCP; Stage B generalizes that into a portable bootstrap contract any agent host can follow.

#### Entry criteria

- Stage A artifacts installable on PATH  
- Workflow catalog + `prism-agent` generators stable (P9)

#### Workstreams

| Workstream | Activities |
|---|---|
| W-AX / W-DIST | **Ensure-installed skill step** (Graphify Step 1 analogue): detect `prism` on PATH → if missing, run platform installer (or print the one-liner) → re-resolve binary → persist path used for MCP `command` |
| W-AX | **Host adapter catalog:** Cursor (`.cursor/mcp.json` + rules — exists), VS Code (`.vscode/mcp.json` — partial), Claude Code (`CLAUDE.md` / mcp config), and a generic “stdio MCP snippet” for other hosts |
| W-DIST | CLI surface: `prism host install <host>`, `prism host uninstall <host>`, `prism host status` — idempotent merges, never clobber unrelated MCP servers |
| W-DIST | Optional: `prism hook install` / `uninstall` / `status` — post-commit incremental re-index (Graphify hook analogue); append-only if a hook already exists |
| W-AX | Ship the ensure-install + setup sequence inside generated skills / AGENTS.md so a fresh agent session can bootstrap a cold machine without a human README |
| W-MCP | Doctor readiness expands: binary version, PATH vs absolute MCP command, index freshness, host registration targets, hook status |
| W-OBS | Install/setup telemetry: local-only counters for install path used (brew vs script vs cargo), setup step failures — never repo content |

#### Deliverables

1. **Bootstrap runbook** — ordered steps an agent or human follows (install → setup → doctor → first `compile_context`)  
2. **Host adapter matrix** — per-host files touched, merge rules, uninstall behavior  
3. **CLI help / man-page drafts** for `host` and optional `hook` subcommands  
4. **Skill/AGENTS fragment** — “If `prism` missing, install thus; never ask for API keys for core path”  
5. **Doctor checklist v2** schema (JSON) covering install + host + index  

#### Risks

| Risk | Mitigation |
|---|---|
| Host config formats churn (Cursor / Claude / VS Code) | Feature-detect files; keep a generic stdio snippet as fallback |
| Auto-writing MCP configs surprises teams | `setup --no-mcp` already exists; default stays opt-in merge with clear report |
| Hooks fight with existing git tooling | Append-only; `hook status`; never replace a foreign hook wholesale |

#### Exit / acceptance

- [x] Documented ensure-install sequence works without a Rust toolchain *(documented; live path needs release)*  
- [x] ≥3 host adapters specified; Cursor + one other implemented-in-plan with merge/uninstall rules *(cursor/vscode/claude/generic)*  
- [x] Generated agent assets mention install bootstrap (catalog-driven, not hand-edited forever)  
- [x] `doctor --ready --json` reports install + host + index readiness

---

### Stage C — Install UX gate & Phase 11 exit

#### Purpose

Prove the north-star user journey: **cold machine → indexed workspace → agent `compile_context`**, on three OS families, without cloning the Prism source tree.

#### Entry criteria

- Stages A and B deliverables reviewed  
- Fixture “customer repo” (small public sample) pinned for smoke tests  

#### Workstreams

| Workstream | Activities |
|---|---|
| W-EVAL | **Cold-machine matrix:** macOS arm64, Linux x86_64, Windows x86_64 — install → setup → doctor → MCP smoke (`compile_context` on fixture) |
| W-EVAL | Time-to-ready metric: wall clock from zero to first successful pack (install + first index) |
| W-DIST | Upgrade path smoke: install N → release N+1 → self-update or package upgrade → doctor still green |
| W-SEC | Release checklist executed on the gate tag; SBOM or license summary attached if already required by P5 checklist |
| Docs | Contributor vs end-user install split; troubleshooting (PATH, quarantine on macOS, execution policy on Windows) |
| W-DEBT | Close PRODUCT-SETUP “must build from repo” as primary; record any remaining platform gaps as explicit limitations |

#### Deliverables

1. **P11 scorecard** — cold-machine pass/fail per OS, time-to-ready, upgrade pass  
2. **Public INSTALL.md** (or PRODUCT-SETUP rewrite) — end-user canonical path  
3. **Troubleshooting appendix** — top failure modes with repair actions (mirror refusal-repair style)  
4. **Phase 11 gate evidence pack** archived under eval/docs  

#### Exit / acceptance (Phase 11 gate)

- [ ] Clean macOS, Linux, and Windows VMs: install without Rust → `prism setup .` → `doctor --ready` → MCP `compile_context` succeeds on the fixture repo  
- [ ] Checksums verified by the installer path used in the gate  
- [ ] Host registration works for Cursor and at least one additional host from the matrix  
- [ ] Uninstall (or documented package remove) leaves no broken MCP entries claiming a missing binary  
- [ ] PRODUCT-SETUP / INSTALL docs match the as-built path (W-DEBT clean for install claims)  
- [ ] Limitations stated honestly (unsupported triples, unsigned builds if applicable, no team shared-index — that is P10)

#### Phase 11 phase-level risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Native deps make “single static binary” false advertising | Medium | High | Define supported glibc/MSVC baselines; test them in CI |
| Package-manager lag behind GitHub Releases | High | Medium | Install scripts + releases are the SLA; brew/scoop are convenience |
| Agents auto-install into surprising locations | Medium | Medium | Prefer `~/.local/bin`; never write into system dirs without elevation; report path in setup |
| Scope creep into P10 team distribution | Medium | High | P11 ships **solo local install only**; shared indexes explicitly out of scope |
| Reopening a VSIX under “install” pressure | Low | Medium | ADR-0007; P11 gate is CLI+MCP only |

#### Non-goals (this phase)

- Team / shared index servers (P10)  
- Marketplace VSIX / IDE extension revival  
- Publishing to every package ecosystem on day one (Winget/AUR/Nix can be follow-ons)  
- Changing Evidence Pack or compiler semantics  

#### Handoff

- End-users and agents install from releases; contributors still build from source  
- P10 (when opened) can assume a standard binary layout and version handshake for CI workers  
- Eval program gains a recurring “install smoke” job on every release tag  

---

## 19. Phase 12 — Accuracy & Grounding (doc-aware evidence)

> **Opened 2026-07-26.** Runs **parallel to P11 Stage C**; depends on P9, not on P10/P11. Motivation: a head-to-head against Graphify on a *product/architecture narrative* question — “summarize this repo’s features and use cases” — where Prism packs were token-cheap but **content-poor**, while Graphify’s graph answered from README/ADD concept nodes. See [`docs/REPO-FEATURE-SUMMARY-AND-TOKEN-COMPARISON.md`](../REPO-FEATURE-SUMMARY-AND-TOKEN-COMPARISON.md).

**Phase goal:** Make Evidence Packs *sufficient*, not merely small. Prism must answer documentation-grounded and architecture-narrative questions from compiled evidence with real quoted spans — matching or beating a doc-aware graph baseline (Graphify) — while keeping the token advantage, the extractive default, and local-first indexing.

**Phase duration:** 5–7 weeks  
**Phase gate (summary):** On a pinned doc-QA + architecture gold set, a medium model answering **only** from one `compile_context` pack matches or beats the Graphify arm on quality at ≤½ its tokens; **zero** packs contain role-placeholder text; unmet roles appear as `gaps[]` with repair actions.

### 19.1 Measured baseline (why this phase exists)

Evidence collected 2026-07-26 on this workspace (T1 · 375 files · 1,629 nodes · 8,676 edges):

| Observation | Measurement / location | Consequence |
|---|---|---|
| Doc/prose questions return **role stubs, not content** | `repo_qa` pack `tokens_used=149`; `prism compile` `tokens_used=92`; fragment text `// must-include \`primary_symbol_definition\` locus near README.md` and `[optional:architecture_prose] related context for README.md` | Pack looks successful, teaches the model nothing |
| Placeholders are generated by design | `role_template()` in `crates/prism-compile/src/select.rs` returns synthetic text for any role the KG cannot fill; `fragment.rs` stamps `synthetic:<label>` provenance | Unfillable roles silently become fake evidence |
| **Markdown is not extracted** | `detect_language("readme.md") == None` (`crates/prism-extract/src/lib.rs`); `NodeKind = File \| Symbol \| Module \| Package` (`crates/prism-ir/src/facts.rs`) | README/ADD/docs have no queryable nodes at all |
| Seed grounding picks the wrong anchor | Question anchored on `README.md` resolved its primary symbol to `module:json` | Pack is *about* the wrong thing |
| Communities are directories, not subsystems | `"Communities are directory prefixes (deterministic); seeded Leiden deferred."` (`crates/prism-store/src/communities.rs`) | Orientation packs restate the folder tree; hub denylist landed in Stage C interim |
| Hub ranking is dominated by language noise | `into` 456 · `clone` 364 · `Some` 293 · `unwrap` 235 (top hubs by degree) | Budget spent on `unwrap` instead of architecture |
| Fixture corpora rank alongside first-party code | `fixtures/repos/snapshots/{ripgrep,httpx}` communities indexed | Evidence can come from vendored sample repos |
| Graphify comparison | BFS query (~1.5k tokens) surfaced `Product Goals & Thesis`, `MCP & Precision Tiers`, `T2 Precise Symbol Tier` concept nodes with `source_location` | A doc-aware graph beats us on narrative questions today |

**Reframing:** G2 (token reduction) is met; G3 (context precision) is measured on a thin sample and **G1 quality is unproven for prose intents**. Phase 12 moves the program from *cheap* to *cheap and sufficient*.

### 19.2 Inspiration from Graphify (adopt / adapt / reject)

| Graphify pattern | Prism P12 stance |
|---|---|
| Extract doc/markdown entities as first-class nodes with `source_location` | **Adopt:** `Doc` / `Section` node kinds, extractive spans, provenance per heading |
| Cross-document + doc→code edges (concept ↔ implementation) | **Adopt:** `DESCRIBES` / `MENTIONS` edges, confidence-labeled |
| Community detection (Louvain/Leiden) + human-readable community labels | **Adopt:** seeded Leiden on import+call+doc graph; labels synthesized **extractively** from headings/paths |
| God nodes + “surprising connections” (bridge report) | **Adapt:** centrality on *resolved* nodes only; bridges reported as orientation fragments, labeled heuristic |
| Question vocabulary expansion before traversal | **Adopt:** expand question terms against graph vocabulary before seeding; record expansion in EXPLAIN |
| Honest audit labels (`EXTRACTED` / `INFERRED` / `AMBIGUOUS`) | **Already have** confidence IR; **extend** with `asserted` for doc-sourced claims |
| LLM semantic extraction at index time (Gemini / host agent subagents) | **Reject as default:** index path stays deterministic, offline, key-free (G8); structural doc extraction only |
| Answering from a rebuilt graph per query | **Reject:** incremental index + compiled packs remain the spine |
| HTML / wiki / Obsidian output | **Reject here:** rendering belongs to P7 (W-VIZ), not to an accuracy phase |

```mermaid
flowchart LR
    A[Stage A Documentation layer] --> B[Stage B Grounded selection + honest gaps]
    B --> C[Stage C Semantic communities + orientation]
    C --> D[Stage D Accuracy gate: five-arm eval]
```

### 19.3 Accuracy targets (new, measurable)

| ID | Target | Baseline today |
|---|---|---|
| **ACC-1** | ≥80% of doc-QA gold tasks answerable from **one** pack with quoted doc spans | ~0% (no doc nodes) |
| **ACC-2** | **Zero** placeholder fragments in any pack; unfilled roles → `gaps[]` + repair action | placeholders are the norm for prose intents |
| **ACC-3** | Seed-grounding precision ≥90%; wrong-seed rate ≤5%; otherwise `SCOPE_UNRESOLVED` with ranked candidate anchors | wrong seed observed on first prose attempt |
| **ACC-4** | ≥70% of semantic community labels accepted by dual review; **0** language-builtin symbols in top-10 hubs | labels = directory names; 4 of top-5 hubs are builtins |
| **ACC-5** | Prism ≥ the doc-aware graph arm (A5) on a shared 20-task product/architecture set at ≤½ tokens | Graphify currently better on narrative |
| **ACC-6** | 0 vendored/fixture fragments in packs unless the question anchors there | fixtures indexed indistinguishably |
| **ACC-7** | G3 precision sample extended to n≥20 dual-reviewed, still ≥70% | n=10 (T001), κ=0.78 |

> **Naming:** `ACC-n` are P12 accuracy targets; `A1`–`A5` remain **benchmark arms** (§23.1). Arm A5 is the doc-aware graph baseline that target ACC-5 measures against.

---

### Stage A — Documentation & narrative layer

#### Purpose

Give the KG the nodes it is missing. Repository *intent* lives in markdown; today Prism indexes only code, so any “what is this / why does it exist / how do I use it” question has nothing to retrieve and falls through to placeholders.

#### Entry criteria

- P9 gate passed (workflows + assets stable)  
- Schema versioning path available for a node/edge-kind extension (`prism-ir`)  
- Written decision that doc extraction is **structural and offline** — no LLM in the index path

#### Workstreams

| Workstream | Activities |
|---|---|
| **W-DOC** (new) | **Markdown/doc extractor:** headings → `Section` nodes with byte ranges; front-matter; lists/tables preserved as extractive spans; fenced code blocks tagged with language |
| **W-DOC** | **Doc graph edges:** `CONTAINS` (doc→section), `REFERENCES` (relative links / anchors), `MENTIONS` (identifier or path occurrences in prose) |
| **W-DOC / W-KG** | **Doc↔code binding:** bind `MENTIONS` to real symbols/files when the token resolves; label unresolved mentions `ambiguous` rather than dropping them |
| **W-KG / W-IR** | **IR extension:** add `Doc` / `Section` node kinds and `DESCRIBES` edge kind; add confidence value **`asserted`** (documentation claims it) as distinct from `extracted` (code proves it) |
| **W-STORE** | Storage + query support for doc nodes; keep them out of code-only queries by default (`kind` filters) |
| **W-PLUGIN** | Golden fixtures: a doc corpus fixture with expected sections/links/mentions, mirroring language-extractor goldens |
| **W-SEC** | Redaction pass on docs (secrets/tokens pasted in markdown must not enter the graph) |
| **W-DEBT** | Record that doc-sourced facts can be **stale by nature**; `asserted` never satisfies a precision gate |

#### Deliverables

1. **Doc extraction spec** — what becomes a node, span rules, link/mention resolution, determinism guarantees  
2. **IR schema delta** — `Doc`/`Section`/`DESCRIBES`/`asserted`, with schema version bump and migration note  
3. **Doc golden fixture set** + expected-facts JSON  
4. **Indexing policy** — which paths count as documentation, cost/size caps, incremental behavior on doc edits  
5. **Honesty rule** — `asserted` facts must be visually/semantically distinguishable in packs, views, and reports  

#### Risks

| Risk | Mitigation |
|---|---|
| Doc layer drifts into abstractive summarization | Extractive spans only; guardrail added to §24.4 |
| Docs contradict code and the model believes the doc | `asserted` confidence + never precision-gated; review sample includes contradiction cases |
| Markdown-heavy monorepos blow up index size | Section-level granularity + size caps + `index_status` reporting doc share |
| Doc nodes pollute code queries | Default `kind` filtering; opt-in inclusion by recipe |

#### Exit / acceptance

- [x] `docs/` appears as indexed content with section-level nodes and byte ranges *(markdown extractor + `Doc`/`Section` nodes; verified end-to-end in a temp workspace)*  
- [x] Doc goldens pass deterministically *(`fixtures/languages/markdown` + `golden_sample_conformance`; regenerate via `cargo run -p prism-extract-markdown --example gen_golden`)*  
- [x] `asserted` confidence flows store-side *(IR `Confidence::Asserted`; MENTIONS persisted with `confidence=asserted`)* — pack/EXPLAIN surfacing lands with Stage B  
- [x] Secret-redaction verified on a planted-secret fixture *( `fixtures/security/planted-docs/.env` + `planted_env_under_docs_not_discovered` )*  
- [x] Doc-edit re-index still meets the N-series incremental target (G4) *(`g4_doc_edit_reindex_is_incremental`: one-file extract + &lt;5s on fixture)*

> **Delivered (2026-07-26):** IR extension (`Doc`/`Section`/`Describes`/`Mentions`/`asserted`), `prism-extract-markdown` crate, dispatch + `detect_language` for `.md/.markdown/.mdx`, golden fixture, planted-secret docs skip, G4 incremental doc-edit test.

#### Handoff

Stage B gains real fragments to select for prose roles (`architecture_prose`, `primary_symbol_definition` on doc anchors) instead of templates.

---

### Stage B — Grounded selection & honest gaps

#### Purpose

Stop manufacturing evidence. Today an unfillable role becomes synthetic text; a mis-parsed question becomes a confident pack about the wrong symbol. Both are accuracy failures that a token metric rewards.

#### Entry criteria

- Stage A doc nodes queryable  
- Agreement that a smaller-but-honest pack is a **pass**, not a regression  

#### Workstreams

| Workstream | Activities |
|---|---|
| **W-CC** | **Delete the placeholder path:** every fragment must cite ≥1 real node/edge with a source range; `role_template()`-style synthesis is removed, not merely relabeled |
| **W-CC** | **`gaps[]` semantics:** unfilled role → gap entry with `role`, `why_absent` (no such node / below tier / budget), and a repair action mirroring the refusal-repair style |
| **W-PLAN** | **Seed grounding:** score candidate anchors (exact symbol > path > heading > lexical); require a minimum score; below it, refuse `SCOPE_UNRESOLVED` **with ranked candidates** instead of guessing |
| **W-PLAN** | **Vocabulary expansion** (Graphify analogue): expand question terms against graph vocabulary (identifiers, paths, headings) before planning; record expansion + rejected terms in EXPLAIN |
| **W-KG** | **Lexical seed index** (trigram/BM25 over identifiers, paths, headings) as the structure-first fallback; embeddings remain out of scope for this phase |
| **W-CC** | **First-party scoping:** classify vendored / fixture / generated paths; exclude from evidence and hubs unless the question anchors there; report the policy in EXPLAIN |
| **W-CC** | **Prose recipes:** `repo_qa` / `architecture` gain doc-backed roles (`product_thesis`, `usage_surface`, `architecture_prose`) bound to real sections with never-drop rules |
| **W-MCP / W-AX** | Tool docs + generated assets state that a pack may legitimately return gaps, and what the agent should do next |
| **W-OBS** | New counters: placeholder attempts (must trend to 0), gap rate by role, seed-refusal rate, lexical-fallback share |

#### Deliverables

1. **Fragment integrity rule** — “no fragment without a citation”, written as a testable invariant  
2. **Gap & refusal spec update** — `gaps[]` shape, `why_absent` taxonomy, repair actions  
3. **Seed-grounding design** — scoring, thresholds, candidate ranking, refusal copy  
4. **Vocabulary-expansion design** — term sources, expansion caps, EXPLAIN disclosure  
5. **Path-class policy** — first-party vs vendored/fixture/generated, with per-repo overrides  
6. **Updated intent recipe cards** for `repo_qa` and `architecture` (template §26.3)  

#### Risks

| Risk | Mitigation |
|---|---|
| Packs get smaller and look “worse” to existing token dashboards | Report answerability alongside tokens; ACC-1/ACC-2 are the gate, not pack size |
| Lexical fallback becomes the retrieval spine | Lexical may only *seed* anchors, never emit fragments; EXPLAIN records seed origin; guardrail in §24.4 |
| More refusals annoy agents | Refusals must carry ranked candidates; measure repair success as in P9 |
| Path classification hides real first-party code | Overrides + `index_status` reporting of excluded classes |

#### Exit / acceptance

- [x] Invariant test: no pack fragment lacks a node/edge citation with a source range *(live `compile_context` strips `synthetic:*` provenance and `EvidencePack::assert_no_placeholder_fragments`; offline `compile_synthetic` remains fixture-only)*  
- [x] Prose questions either quote real doc/code spans or return explicit gaps — never templates *(Doc/Section selection + structured `gaps[]` with repair; recipes gain `product_thesis` / doc-backed roles)*  
- [x] Seed-grounding precision ≥90% on the gold anchor set (**ACC-3**) *(lexical `score_anchor` / `lexical_seed_search`; live `ground_plan_seeds` refuses below score 70 with ranked `candidates[]` — measured precision sample still open for Stage D)*  
- [x] Wrong-seed cases refuse with ranked candidates and recover on the second call *(unit-tested; MCP surfaces `candidates` on SCOPE_UNRESOLVED)*  
- [x] Fixture/vendored fragments absent unless anchored (**ACC-6**) *(path-class filter in selection + hub communities exclude `fixtures/repos/`)*

> **Delivered (2026-07-26):** `EvidenceGap` / `WhyAbsent`, `pack_under_budget_with_gaps`, path-class policy, live selection without `role_template` prepend, doc-prose roles, analyzer-pipeline version forcing re-extract (`p12-doc-v1`), **ACC-3 lexical seed grounding**.

#### Handoff

Stage C can rank orientation content knowing hubs/communities feed *real* fragments and that noise paths are already classified.

---

### Stage C — Semantic communities & orientation quality

#### Purpose

Make architecture packs describe subsystems rather than folders, and hubs describe design rather than `unwrap`. This is where Prism should decisively beat a syntactic graph baseline: same community idea, but with precise-tier edges and budgeted, provenance-bearing output.

#### Entry criteria

- Stages A and B delivered (doc nodes + honest fragments)  
- Path-class policy active so clustering is not dominated by fixtures  

#### Workstreams

| Workstream | Activities |
|---|---|
| **W-KG** | **Seeded Leiden/Louvain** over import + call + `DESCRIBES` edges; deterministic seed, versioned algorithm id, snapshot-tested membership |
| **W-KG** | **Extractive community labels:** derive from dominant doc headings, module names, and shared path segments — no LLM required; keep directory label as fallback |
| **W-KG** | **Hub ranking v2:** centrality (PageRank/betweenness) over **resolved** nodes; language-builtin/stdlib denylist; unresolved symbols reported separately as an index-quality metric, not as architecture |
| **W-KG** | **Bridge report** (“surprising connections” analogue): cross-community edges ranked by betweenness, labeled heuristic |
| **W-CC** | Architecture recipe consumes v2 communities/hubs/bridges; never-drop the community map; cap bridge fragments |
| **W-VIZ** | Graph View-Model consumes semantic communities so views and packs agree (no second clustering) |
| **W-AX** | `repo_map` / `entrypoints` notes updated: honesty text must change when the algorithm changes (the “not Leiden yet” note is a contract, not decoration) |
| **W-OBS** | Metrics: unresolved-hub share, community stability across commits, label acceptance rate |

#### Deliverables

1. **Clustering design** — algorithm, seeding, determinism, versioning, incremental strategy  
2. **Label synthesis spec** — sources, precedence, fallback, review protocol  
3. **Hub/centrality spec** — measure choice, denylist policy, resolved-only rule  
4. **Bridge report spec** — ranking, caps, confidence labeling  
5. **Updated orientation notes** for `repo_map` / architecture packs (docs + tool `confidence_note`)  

#### Risks

| Risk | Mitigation |
|---|---|
| Clustering nondeterminism breaks reproducible packs | Fixed seed + snapshot tests + algorithm id in provenance |
| Community labels become marketing prose | Extractive only; dual review; fallback to path label |
| Incremental clustering cost per commit | Cluster on demand + cache by snapshot id; recluster budget documented |
| Doc edges distort code communities | Weight `DESCRIBES` below structural edges; ablation recorded in the report |

#### Exit / acceptance

- [x] Communities are semantic and stable across two adjacent commits (documented drift bound) *(deterministic Louvain `louvain_v1+resolved_degree_hubs`; unit-tested membership stability; path-prefix fallback when graph sparse)*  
- [x] Top-10 hubs contain **0** language builtins (**ACC-4**) *(resolved-degree hubs + builtin/unresolved denylist; unit-tested)*  
- [ ] ≥70% of labels accepted in dual review (**ACC-4**) *(extractive leaf labels shipped; dual-review measurement pending live Stage D)*  
- [x] Architecture pack for this repo names subsystems via Louvain community labels + hubs *(views/packs share `comm:louvain:*` ids from `repo_map`)*  
- [x] Views and packs cite the **same** community ids *(`prism-view` and `CommunityOf` both call `kg.repo_map`)*

> **Delivered (2026-07-26):** Deterministic Louvain on file-level IMPORTS/CALLS/DESCRIBES, bridge report, hub ranking v2, extractive labels. Dual-review label acceptance remains a Stage D live measurement.

#### Handoff

Stage D can benchmark orientation against a doc-aware baseline with a stable, versioned clustering to cite.

---

### Stage D — Accuracy gate (five-arm eval & adjudication)

#### Purpose

Settle the accuracy claim the same way P9 settled tool choice: with a published, reproducible head-to-head — including an external doc-aware graph baseline that currently wins on narrative questions.

#### Entry criteria

- Stages A–C deliverables reviewed  
- Pinned repo SHAs + pinned Graphify graph build for reproducibility  
- Dual-review labeling capacity available (mandatory since P9 Stage C)  

#### Workstreams

| Workstream | Activities |
|---|---|
| **W-EVAL** | **Doc-QA / narrative gold set** (≥25 tasks): product thesis, feature inventory, use cases, install/bootstrap, workflow semantics, non-goals — each with accepted-answer criteria and necessary doc spans |
| **W-EVAL** | **Five-arm harness:** extend four-arm with **arm E = doc-aware graph baseline (Graphify)**; record tokens, hops, quality, and citation validity per arm |
| **W-ACC** (new) | **Adjudication protocol:** blind grading, tie-breaks, κ reporting, and a rule that a *fluent answer with invalid citations* scores zero |
| **W-EVAL** | Extend precision sample to n≥20 dual-reviewed (**ACC-7**); include prose fragments, not only code |
| **W-EVAL** | Ablations: docs off/on, semantic communities off/on, lexical seeds off/on — so the accuracy delta is attributable |
| **W-DEBT** | Reconcile every published accuracy claim with the as-built repo; retire the “thin pack looks like success” failure mode from docs |
| **W-OBS** | Ship the accuracy counters (ACC-2 / ACC-3 rates) as part of the standard scorecard |

#### Deliverables

1. **Doc-QA gold suite v1** (task cards per template §26.9)  
2. **Five-arm accuracy report v1** — per-arm tokens/quality/citation validity + ablation table  
3. **Adjudication protocol** doc + κ results  
4. **P12 scorecard** — ACC-1…ACC-7 pass/fail with artifacts  
5. **Phase 12 gate evidence pack** archived under `docs/eval/`  

#### Risks

| Risk | Mitigation |
|---|---|
| Baseline comparison is unfair or unstable | Pin baseline version + graph build; publish its exact invocation; let it use its best settings |
| Scripted proxies substituted for real judging again | Arm quality must come from live judged runs; proxies may not satisfy the P12 gate |
| Gold set written to flatter Prism | Author tasks from user questions and README/ADD claims *before* seeing packs; external review of the set |
| Accuracy work quietly regresses tokens | Token budget adherence stays a co-gate: quality wins must hold at ≤½ baseline tokens |

#### Exit / acceptance (Phase 12 gate)

- [x] ACC-1…ACC-7 met or explicitly waived with residual risk recorded *(live-judged PASS 2026-07-27 — [p12-phase-gate.md](../../eval/scorecards/p12-phase-gate.md))*  
- [x] Five-arm report published (live-judged quality + ablations) *([P12-FIVE-ARM-REPORT.md](../eval/P12-FIVE-ARM-REPORT.md))*  
- [x] Prism ≥ arm A5 (doc-aware graph) on the shared narrative set at ≤½ tokens (**ACC-5**) *(Δq=+43.3 pts, token ratio 0.461)*  
- [x] Ablations show which capability bought which accuracy points *(docs / louvain / lexical rows in five-arm report)*  
- [x] No published claim without an archived artifact (guardrail §24.6 rule 7)

> **Closeout (2026-07-27):** Stage D live adjudication complete (1A agent dual-pass + 2A Graphify). **Phase 12 gate PASS.** Residual: optional human 1C spot-check; explore arms A/B remain scripted placeholders.

#### Non-goals (this phase)

- LLM-based extraction or summarization in the index path (stays offline / key-free — G8)  
- Embedding-first retrieval (lexical seeds only; embeddings remain a later, labeled fallback)  
- HTML/wiki/vault generation (P7 owns rendering)  
- Answer caching (blocked until P10 Stage C)  
- Auto-rewriting the repository’s documentation  
- New precision tiers (T2/T4 semantics unchanged)

#### Handoff

- P7 views inherit semantic communities; P10 (if opened) inherits a stable accuracy scorecard  
- Agent assets inherit “packs may return gaps” guidance  
- The program can finally state G1/G3 for prose intents with evidence rather than proxies

---

## 20. Phase 13 — Core Refactor & Boundary Hardening

> **Opened 2026-07-27.** First phase whose product outcome is **zero new user-visible features**. P0–P12 grew capability fast; the engine now carries god files, four duplicated tool surfaces, and a stringly-typed error model. P13 pays that down **without changing behavior**.

**Phase goal:** Make the codebase safe to change. Extract one service layer that every surface calls, decompose the three god files, replace duplicated refusal strings with a typed IR, and put size/complexity budgets under CI — with pack-output parity as the proof that nothing moved.

**Phase duration:** 4–6 weeks  
**Phase gate (summary):** On a frozen question set, packs produced before and after the refactor are semantically identical; every surface (MCP/CLI/HTTP/LSP) routes through one service crate; no file >600 LOC and no function >120 LOC without a recorded waiver.

### 20.1 Measured baseline (why this phase exists)

Measured 2026-07-27 — **19,960 LOC** of Rust across **20 crates**, **103 test functions**:

| Signal | Measurement | Consequence |
|---|---|---|
| CLI god file | `prism-cli/src/main.rs` **1,549 LOC**; `main()` ≈ **949 lines**; graph hub degree **298** | Every CLI change edits one function; zero in-file tests |
| Selection god function | `prism-compile/src/select.rs` **1,099 LOC**; `select_from_kg()` ≈ **516 lines**; **no `#[cfg(test)]` module** | The accuracy-critical core (P12) is the least unit-tested code we own |
| Agent contract has two sources of truth | `prism-mcp/src/tools.rs` **874 LOC**, `list_tools_schema()` **168 lines** of hand-built JSON, while `schemas/mcp-tools/v1/` is the declared contract of record | Schema drift between the file agents read and the code agents call |
| No shared service layer | Tool-name references: MCP **88** · CLI **34** · HTTP **23** · LSP **12**; `prism-cli` depends on **14** workspace crates (incl. `prism-api`, `prism-daemon`, `prism-lsp`, `prism-view`) | Four adapters re-implement dispatch; the CLI has become a fan-in god crate |
| Refusal model is stringly typed | Refusal codes appear in **12 files** (`mcp/errors.rs` 14, `agent/repair.rs` 6, `agent/assets.rs` 6, `precise/require.rs` 5, `cli/main.rs` 5, `api/error.rs` 5) | “Every surface shares one error model” (§25.1 item 2) is a convention, not a type |
| Store handles opened ad hoc | `SqliteKgStore::open(...)` at 8+ call sites outside `prism-store` — including per-request in `prism-api/src/routes.rs` | No ownership boundary; the performance half is P14 |
| Test net is thin where it matters | 103 test fns / ~20k LOC; `main.rs` and `select.rs` have **0** in-file tests; `communities.rs` is ~27% tests | Refactoring without characterization tests is a gamble |
| Orientation clustering is fragmenting | `repo_map`: **40** communities, **10 singleton** `docs/architecture/` communities, `bridges=0`, largest community is docs (21 files / 180 nodes) | Louvain quality degrades as the doc layer grows (P12 Stage C follow-through) |

**What is already healthy — do not “fix” it:** `unwrap()`/`expect()` in the four highest-count files (`incremental.rs`, `intel.rs`, `compile/lib.rs`, `workspace.rs`) are **entirely inside `#[cfg(test)]` modules** — production paths return `Result`. Zero `TODO`/`FIXME`/`HACK` markers. MSRV pinned (`rust-version = 1.85`). Clippy runs `-D warnings` on all targets in CI.

### 20.2 Refactor targets (measurable)

| ID | Target | Baseline today |
|---|---|---|
| **REF-1** | Pack **parity**: frozen question set produces semantically identical packs pre/post refactor | no parity harness exists |
| **REF-2** | One service crate owns capability execution; MCP/CLI/HTTP/LSP contain adapter code only | 4 duplicated dispatch surfaces |
| **REF-3** | No file >600 LOC, no function >120 LOC (waiver requires an ADR) | max file 1,549; max fn ~949 |
| **REF-4** | Refusal codes are a typed enum in `prism-ir`; no surface hardcodes the strings | codes in 12 files |
| **REF-5** | MCP tool schemas generated from `schemas/mcp-tools/v1` — one source of truth | 168-line hand-built JSON |
| **REF-6** | `prism-cli` depends on ≤6 workspace crates | 14 |
| **REF-7** | Characterization tests cover CLI dispatch + selection roles before decomposition | 0 in-file tests in both |

```mermaid
flowchart LR
    A[Stage A Seams + characterization] --> B[Stage B Service layer extraction]
    B --> C[Stage C God-file decomposition]
    C --> D[Stage D Parity gate + budgets in CI]
```

---

### Stage A — Seams before surgery

#### Purpose

Buy the safety net first. Nothing is moved in this stage: we add types, tests, and measurements that make the later stages verifiable.

#### Entry criteria

- P12 gate passed (pack semantics stable enough to freeze)  
- Agreement that P13 ships **no behavior change** — parity is the gate

#### Workstreams

| Workstream | Activities |
|---|---|
| **W-REFAC** (new) | **Characterization tests:** freeze a question/command set; record current pack + CLI JSON output as golden artifacts |
| **W-REFAC / W-IR** | **Typed refusal IR:** `RefusalCode` enum + `PrismError` in `prism-ir`, with `Display`/serde mapping to today’s exact strings (wire-compatible) |
| **W-REFAC** | **Complexity census:** per-file/per-function size report committed as a baseline artifact; budgets defined but **not yet enforced** |
| **W-CC** | Extract pure helpers out of `select_from_kg` where provably side-effect free, guarded by the new goldens |
| **W-DEBT** | Record which oversized units get waivers vs decomposition, before anyone starts moving code |

#### Deliverables

1. **Parity harness spec + golden corpus** — frozen questions, commands, and expected pack/CLI shapes  
2. **Refusal IR design** — enum, wire mapping, migration order per surface  
3. **Complexity baseline report** — file/function size census with budget thresholds proposed  
4. **Refactor waiver list** — units allowed to stay large, each with a reason and expiry  

#### Risks

| Risk | Mitigation |
|---|---|
| Goldens encode current bugs as “correct” | Parity is scoped to *structure and citations*, not to placeholder-era text; P12 invariants still apply |
| Typed errors change wire strings and break agents | Serde mapping is byte-compatible; conformance test asserts old strings |
| Census becomes a vanity metric | Budgets are enforced only in Stage D, after decomposition proves them reachable |

#### Exit / acceptance

- [ ] Golden corpus reproducible on two machines (deterministic)  
- [ ] `RefusalCode` exists and round-trips to the exact current strings  
- [ ] Complexity baseline committed with proposed budgets (REF-3)  
- [ ] No production behavior change (parity harness green against itself)

---

### Stage B — Service layer extraction

#### Purpose

Create the missing middle. Today four adapters each know how to *execute* capabilities; they should only know how to *speak a protocol*.

#### Entry criteria

- Stage A goldens green  
- Capability inventory agreed (compile, plan, resolve, neighbors, impact, repo_map, entrypoints, detect_changes, view, slice, workflow)

#### Workstreams

| Workstream | Activities |
|---|---|
| **W-REFAC** | **`prism-engine` crate:** one typed entry per capability — request struct in, typed result or `PrismError` out; owns store/session handles |
| **W-MCP / W-SVC** | MCP + HTTP become adapters: parse → call engine → serialize; no graph or budget logic left in either |
| **W-IDE** | LSP adapter follows the same contract (it already has the thinnest surface — do it first as the pilot) |
| **W-REFAC** | **CLI dependency diet:** CLI talks to the engine, not to `prism-api`/`prism-daemon`/`prism-lsp`/`prism-view` directly (REF-6) |
| **W-STORE** | Session/handle ownership moves behind the engine — no `SqliteKgStore::open` outside `prism-store`/engine (sets up P14) |
| **W-OBS** | One instrumentation point per capability instead of four partial ones |

#### Deliverables

1. **Capability contract** — request/response types, error taxonomy, cancellation and budget parameters  
2. **Adapter migration order** — LSP → MCP → HTTP → CLI, with parity checked after each  
3. **Dependency-graph target** — allowed edges between crates (an architecture fitness rule)  
4. **Deprecation notes** for any direct-store call sites removed  

#### Risks

| Risk | Mitigation |
|---|---|
| Engine becomes a second god crate | Capability modules, not one `engine.rs`; Stage D budgets apply to it too |
| Migration drags and the repo lives in two worlds | Adapter-by-adapter with parity gates; no long-lived branch |
| Hidden behavior differences between surfaces surface as “regressions” | They are **findings** — record them; the engine picks one correct behavior with an ADR |

#### Exit / acceptance

- [ ] All four surfaces route through `prism-engine` (REF-2)  
- [ ] No `SqliteKgStore::open` outside `prism-store` / engine  
- [ ] `prism-cli` workspace dependencies ≤6 (REF-6)  
- [ ] Parity green after each adapter migration  
- [ ] Divergences found between surfaces are recorded as ADRs, not silently unified

---

### Stage C — God-file decomposition

#### Purpose

Break the three files that concentrate change risk, now that behavior is pinned by goldens and routed through one engine.

#### Entry criteria

- Stage B complete; parity harness reliable  
- Waiver list agreed (what may stay big)

#### Workstreams

| Workstream | Activities |
|---|---|
| **W-REFAC** | `main.rs` → one module per command group (`index`, `compile`, `query`, `host`, `hook`, `agent`, `self-update`); `main()` becomes routing only |
| **W-CC** | `select.rs` → role resolvers (doc/prose, code definition, neighbors, community) + a thin orchestrator; each resolver independently unit-tested |
| **W-MCP** | `tools.rs` → schema-driven dispatch generated from `schemas/mcp-tools/v1` (REF-5); handlers become one function each |
| **W-KG** | Split `communities.rs` (818 LOC) into clustering, labeling, and hub-ranking modules — prerequisite for the P12-C quality follow-ups (singleton communities, `bridges=0`) |
| **W-EVAL** | Unit tests land **with** each extracted module; coverage of `select.rs` roles becomes a tracked number |

#### Deliverables

1. **Module maps** for `prism-cli`, `prism-compile`, `prism-mcp`, `prism-store::communities`  
2. **Schema-generated MCP dispatch** design (codegen or build script, with conformance test)  
3. **Role-resolver test matrix** for selection  
4. **Community-quality follow-up note** — singleton communities and inert bridges triaged (fix here or defer to an accuracy follow-on with an ID)

#### Risks

| Risk | Mitigation |
|---|---|
| Decomposition changes pack ordering subtly | Parity harness compares citations and roles, not just token counts |
| Codegen adds build complexity | Prefer a checked-in generated file with a CI freshness test over build-time magic |
| Splitting `communities.rs` disturbs P12 gate results | Re-run the P12 ACC checklist as part of Stage C exit |

#### Exit / acceptance

- [ ] `main()` is routing only; command modules own their logic  
- [ ] No selection role resolver exceeds the function budget; each has tests (REF-7)  
- [ ] MCP schema has exactly one source of truth, verified by conformance test (REF-5)  
- [ ] P12 ACC-1…ACC-7 re-run and still pass  
- [ ] Parity green

---

### Stage D — Parity gate & enforced budgets

#### Purpose

Turn the one-off cleanup into a property the repository keeps.

#### Entry criteria

- Stages A–C delivered; waiver list final

#### Workstreams

| Workstream | Activities |
|---|---|
| **W-REFAC** | Enforce REF-3 budgets in CI (file/function size), with the waiver list as the only escape hatch |
| **W-REFAC** | **Architecture fitness test:** forbidden crate edges fail the build (e.g. CLI → daemon, adapters → store) |
| **W-EVAL** | Parity harness runs in CI on every PR touching engine/compile/store |
| **W-DEBT** | Close the refactor drift items; every remaining oversized unit has an ADR with an expiry phase |
| **W-OBS** | Publish the P13 scorecard: LOC distribution, duplication counts, dependency fan-in before/after |

#### Deliverables

1. **P13 scorecard** — REF-1…REF-7 pass/fail with before/after numbers  
2. **CI policy** — size budgets, fitness rules, parity job  
3. **ADR set** for accepted divergences and remaining waivers  
4. **Phase 13 gate evidence pack**  

#### Exit / acceptance (Phase 13 gate)

- [ ] REF-1…REF-7 met or waived with a dated ADR  
- [ ] CI fails a PR that reintroduces a god file or a forbidden crate edge  
- [ ] Pack parity job green on the frozen set (REF-1)  
- [ ] No user-visible behavior change shipped by this phase without an ADR

#### Non-goals (this phase)

- New capabilities, tiers, or languages  
- Performance work (that is P14 — measure, do not optimize here)  
- Changing Evidence Pack semantics or P12 accuracy behavior  
- Rewriting extractors or the storage engine

#### Handoff

P14 inherits a single place to optimize (engine + store boundary) instead of four call paths.

---

## 21. Phase 14 — Performance & Scale Proof

> **Opened 2026-07-27.** Prism’s NFRs (N1 cold index, N2 P95 query) are still **unproven at real scale**: benches run against a 16-file synthetic fixture while two pinned pilot corpora sit unused for performance work.

**Phase goal:** Prove — and then defend in CI — that indexing and structural queries meet their NFRs on real repositories, by fixing the measured hot spots: per-call store opens, uncached orientation, and a mostly single-threaded pipeline.

**Phase duration:** 4–6 weeks  
**Phase gate (summary):** N1 and N2 targets met on pinned pilot repos (httpx, ripgrep) with hard numeric ceilings enforced in CI, plus published memory and index-size budgets.

### 21.1 Measured baseline (why this phase exists)

| Signal | Measurement | Consequence |
|---|---|---|
| Benchmarks measure a toy | N1/N2 baselines use **8 Python + 8 Rust synthetic modules**; means: cold index 6.52 ms, incremental 3.43 ms, queries 12.8–31.8 µs | Numbers are a regression slope, not NFR proof — the baseline doc says so honestly |
| CI bench is a smoke, not a gate | `--sample-size 10 --warm-up-time 1 --measurement-time 1`; “hard P95 fail gates wait until pilot-repo numbers exist” | A 10× slowdown could merge unnoticed |
| Pilot corpora exist but are unused for perf | `fixtures/repos/snapshots/httpx` **12 MB**, `ripgrep` **10 MB** — pinned for eval only | The hard part is already vendored; nobody points the benches at it |
| Store opened per call | `SqliteKgStore::open(...)` per request in `prism-api/src/routes.rs`, plus LSP/compile/precise call sites | PRAGMA + schema probe per request; no pooling or warm handle |
| Orientation recomputed every time | `repo_map` runs `file_adjacency()` + `louvain_cluster()` on each call (`communities.rs`, 818 LOC); no snapshot-keyed cache | Louvain cost is paid per agent orientation call |
| Parallelism is shallow | Exactly **one** `par_iter()` in the workspace (`prism-core/src/incremental.rs`); writes serialize through a single connection | Extraction scales; persistence does not |
| Index growth is real | This workspace: **519 files → 3,351 nodes / 12,839 edges / 17.5 MB** (doc layer ~doubled node count) | Size/memory budgets need numbers before the next layer lands |

### 21.2 Performance targets (measurable)

| ID | Target | Baseline today |
|---|---|---|
| **PERF-1** | N1 cold index meets the ADD target on **pilot repos**, published per repo | only mini-fixture numbers |
| **PERF-2** | N1 incremental single-file edit <2 s on pilot repos | mini-fixture 3.43 ms (not comparable) |
| **PERF-3** | N2 structural query **P95 <50 ms on pilot repos**, enforced as a CI ceiling | ceiling explicitly deferred |
| **PERF-4** | Warm `repo_map` served from a snapshot-keyed cache; cold recompute only on index change | recomputed every call |
| **PERF-5** | Peak RSS + on-disk index size budgets published per KLOC and enforced | unmeasured |
| **PERF-6** | Perf regression guard: CI fails on >20% slowdown vs recorded baseline | none |

```mermaid
flowchart LR
    A[Stage A Pilot harness + profiles] --> B[Stage B Store & query optimization]
    B --> C[Stage C Pipeline parallelism]
    C --> D[Stage D NFR gate]
```

---

### Stage A — Pilot-scale harness & profiling

#### Purpose

Measure the real thing before changing it. Every optimization in this phase must cite a profile.

#### Entry criteria

- Pilot SHAs pinned (already true: httpx `b5addb6`, ripgrep `f9c05a9`)  
- P13 Stage B engine boundary available, or a documented plan to measure both paths

#### Workstreams

| Workstream | Activities |
|---|---|
| **W-PERF** (new) | Pilot-scale bench suite: cold index, incremental edit, and the N2 query set against httpx + ripgrep |
| **W-PERF** | Phase timing breakdown: walk → extract → resolve → persist → cluster, reported per repo |
| **W-PERF** | Profiles: CPU flamegraph + `EXPLAIN QUERY PLAN` for the hot SQL in `query.rs` / `communities.rs` / `intel.rs` |
| **W-OBS** | Perf events emitted with snapshot id so numbers are attributable to an index version |
| **W-EVAL** | Baseline scorecard format that can be diffed release over release |

#### Deliverables

1. **Pilot bench suite** design + fixture policy (vendored snapshots, not network clones)  
2. **Phase-timing report** for both pilots  
3. **Query plan audit** — indexes used/missed, N+1 patterns, prepared-statement reuse opportunities  
4. **Perf baseline v1** scorecard  

#### Risks

| Risk | Mitigation |
|---|---|
| CI runners too slow/noisy for absolute ceilings | Ceilings set from CI-runner baselines with variance bands; local numbers reported separately |
| Vendored pilots bloat the repo further | Already vendored (22 MB total); no new corpora without a size review |
| Profiling becomes a one-off | Profiles are archived artifacts referenced by the gate |

#### Exit / acceptance

- [ ] Benches run against both pilots and publish per-phase timings  
- [ ] Query plan audit lists every full-table scan on a hot path  
- [ ] Baseline v1 archived and diffable

---

### Stage B — Store, query & cache optimization

#### Purpose

Fix the two structural costs the audit already predicts: opening a store per call, and recomputing orientation per call.

#### Entry criteria

- Stage A profiles exist; optimizations must reference them  
- P13 Stage B ownership boundary in place (no ad-hoc `open` call sites)

#### Workstreams

| Workstream | Activities |
|---|---|
| **W-PERF / W-STORE** | Warm handle / connection pool owned by the engine; PRAGMA + schema probe once per process, not per request |
| **W-PERF** | Prepared-statement reuse and batched reads on `query.rs` hot paths |
| **W-PERF** | **Snapshot-keyed caches** for orientation (communities, hubs, entrypoints) and, if profiles justify, compiled packs — invalidated by snapshot id, never by TTL |
| **W-STORE** | Index/schema tuning driven by the query-plan audit; write batching + transaction sizing |
| **W-SEC** | Cache lives under `.prism/`, respects redaction, and never persists secrets from doc nodes |
| **W-DEBT** | Any memoization must satisfy the program rule: **cache is memoization only**, dependency-keyed, never an answer cache (that stays P10 Stage C) |

#### Deliverables

1. **Session/pool design** — lifetime, concurrency, WAL implications  
2. **Cache design** — keys (snapshot id + query shape), invalidation, size caps, disable switch  
3. **Schema/index change set** with before/after query plans  
4. **Updated perf scorecard** showing the delta attributable to each change  

#### Risks

| Risk | Mitigation |
|---|---|
| Caching serves stale orientation after an edit | Key on snapshot id; incremental index bumps it; test edit→query→fresh result |
| Pooling introduces lock contention under the daemon | Load test the daemon path; WAL + read-only connections for query traffic |
| “Cache” quietly becomes an answer cache | Explicit non-goal; guardrail in §24.6 |

#### Exit / acceptance

- [ ] No per-request store open on any surface  
- [ ] Warm `repo_map` served from cache; correctness test proves invalidation on edit (PERF-4)  
- [ ] Hot-path query plans use indexes; no full scans on N2 paths  
- [ ] Measured improvement attributed change-by-change

---

### Stage C — Pipeline parallelism & incremental correctness

#### Purpose

Make indexing scale past one `par_iter()` without breaking the incremental guarantees P0–P1 established.

#### Entry criteria

- Stage B landed (store boundary stable under concurrency)  
- Incremental correctness tests exist and pass

#### Workstreams

| Workstream | Activities |
|---|---|
| **W-PERF / W-STORE** | Pipeline extract → persist with bounded channels; batch writes instead of per-file transactions |
| **W-PERF** | Parallelize markdown + code extraction paths; measure whether tree-sitter parsing or SQLite writes dominate |
| **W-CORE** | Deterministic output under parallelism (stable node/edge ordering) so packs and goldens stay reproducible |
| **W-PERF** | Analyzer-version reindex cost: P12 introduced `ANALYZER_PIPELINE_VERSION` invalidation — measure and bound a full re-extract |
| **W-EVAL** | Concurrency stress: index while querying; assert no torn reads and no corrupted snapshot |

#### Deliverables

1. **Pipeline design** — stages, backpressure, failure isolation per file  
2. **Determinism proof** — same repo indexed N times yields identical graph bytes or a documented canonical ordering  
3. **Reindex-cost note** — what a pipeline-version bump costs on pilots  
4. **Concurrency test suite**  

#### Risks

| Risk | Mitigation |
|---|---|
| Parallel writes corrupt or serialize anyway | Single writer + batched transactions; parallelism stays on the CPU-bound side |
| Nondeterministic ordering breaks pack parity (REF-1) | Canonical sort before persist; parity harness is the guard |
| Speedups only appear on huge repos | Report both pilots and the mini fixture; no cherry-picking |

#### Exit / acceptance

- [ ] Measurable cold-index speedup on pilots with determinism preserved  
- [ ] Index-while-query stress passes with no stale/torn results  
- [ ] Analyzer-version reindex cost documented (PERF-1 context)

---

### Stage D — NFR gate

#### Purpose

Convert measurements into enforced ceilings, so performance stops being a claim.

#### Entry criteria

- Stages A–C delivered; CI-runner variance characterized

#### Workstreams

| Workstream | Activities |
|---|---|
| **W-PERF / W-EVAL** | Hard ceilings in CI for N1/N2 on pilot repos with variance bands (PERF-3, PERF-6) |
| **W-PERF** | Memory + index-size budgets published per KLOC (PERF-5) |
| **W-OBS** | Perf scorecard published alongside the accuracy scorecard — tokens, accuracy, and latency in one place |
| **W-DEBT** | Retire the P6 Stage A note that hard gates “wait until pilot numbers exist” — the numbers now exist or the gate fails honestly |

#### Deliverables

1. **P14 scorecard** — PERF-1…PERF-6 pass/fail per pilot  
2. **CI perf policy** — ceilings, bands, and the escape procedure for legitimate regressions  
3. **Capacity note** — expected behavior on a 100k-LOC repository, stated with evidence or stated as unknown  
4. **Phase 14 gate evidence pack**  

#### Exit / acceptance (Phase 14 gate)

- [ ] PERF-1…PERF-6 met or waived with dated ADRs  
- [ ] CI fails a >20% regression on the pilot benches  
- [ ] Published numbers name the hardware and the snapshot they came from  
- [ ] No accuracy regression: P12 ACC checklist re-run green

#### Non-goals (this phase)

- Shared/team index or remote workers (P10)  
- LLM answer caching (P10 Stage C)  
- Swapping SQLite for another engine without an ADR  
- New analysis tiers or languages

#### Handoff

P15 inherits trustworthy perf artifacts to include in release evidence.

---

## 22. Phase 15 — Reliability, Governance & Release Trust

> **Opened 2026-07-27.** Two waivers have outlived their expiry phases, the residual-risk register predates P12, and the harness that gates accuracy is not run by CI. This phase makes the program’s **claims** as maintained as its code.

**Phase goal:** Close governance drift, extend the test and CI safety net to the places that gate releases, and finish the release-trust story P11 started.

**Phase duration:** 3–5 weeks  
**Phase gate (summary):** Every waiver has a live expiry, the residual-risk register is generated from evidence rather than memory, CI exercises all three OS families plus the eval harness, and release artifacts carry provenance.

### 22.1 Measured baseline (why this phase exists)

| Signal | Measurement | Consequence |
|---|---|---|
| Expired / orphaned waivers | ADR-0005 (OTLP exporter) expiry **Phase 7** — passed. ADR-0001 (WASM plugin host) expiry **Phase 8** — a phase that was **cut** (ADR-0007) | Two waivers are live with no enforceable review point |
| Residual-risk register is stale | R4 still says communities are path-prefix with “optional Leiden later” — Louvain shipped in P12 Stage C; R2/R13 predate the P12 numbers | The public risk list understates *and* overstates reality |
| Gate-critical harness is outside CI | `eval/baselines/p12_live_adjudication.py` is a **top-5 graph hub (degree 136)** and produces the ACC verdicts, but no CI job runs Python eval | The accuracy gate can silently rot |
| CI is single-OS | `ci.yml` runs `ubuntu-latest` only, while P11 claims macOS/Linux/Windows support | Cross-platform claims rest on the install-smoke job alone |
| No coverage signal | No `tarpaulin`/`llvm-cov`; 103 test fns for ~20k LOC | “Well tested” is unmeasured; P13 makes it measurable |
| No MSRV verification job | `rust-version = 1.85` declared; CI builds with `stable` | MSRV can break without anyone noticing |
| P11 Stage C still open | Cold-VM gate pending a public release tag | The distribution claim remains partially unproven |

### 22.2 Reliability targets (measurable)

| ID | Target | Baseline today |
|---|---|---|
| **REL-1** | Every ADR waiver has a **live, future** expiry phase and a named owner | 2 expired/orphaned |
| **REL-2** | Residual-risk register generated from gate artifacts, regenerated at every phase exit | hand-maintained, stale |
| **REL-3** | Eval harness (accuracy + parity) runs in CI on a schedule and on relevant PRs | not run |
| **REL-4** | CI matrix covers macOS, Linux, Windows for build + test | Linux only |
| **REL-5** | Coverage measured and ratcheted (no decrease), with selection/engine paths reported separately | unmeasured |
| **REL-6** | MSRV job proves `1.85` still builds | absent |
| **REL-7** | Release artifacts carry provenance (SBOM and/or signatures) beyond SHA-256 | checksums only |

```mermaid
flowchart LR
    A[Stage A Governance sweep] --> B[Stage B Test & CI hardening]
    B --> C[Stage C Release trust]
```

---

### Stage A — Governance sweep

#### Purpose

Make the paper trail true again. This is cheap, high-trust work that the program’s own guardrails already demand (“no claim without an artifact”).

#### Entry criteria

- P12 gate artifacts available as evidence inputs

#### Workstreams

| Workstream | Activities |
|---|---|
| **W-REL** (new) | **Waiver audit:** re-date ADR-0001 and ADR-0005 to live expiry phases, or close them; add an owner field |
| **W-REL / W-DEBT** | **Risk register refresh:** regenerate `PROGRAM-RESIDUAL-RISKS.md` from phase gate outputs; mark R4/R2/R13 with their P12 reality |
| **W-DEBT** | Drift sweep across README / ADD / TASKS for post-P12 statements (e.g. README still says “Phase 11”) |
| **W-REL** | Define the **expiry enforcement rule**: a waiver whose phase has shipped blocks the next phase gate until re-dated |

#### Deliverables

1. **Waiver audit report** + updated ADRs  
2. **Regenerated residual-risk register** with provenance links  
3. **Doc drift closure list**  
4. **Waiver expiry policy** (enforcement point + owner)

#### Exit / acceptance

- [ ] Zero waivers with past or orphaned expiries (REL-1)  
- [ ] Risk register entries each cite an artifact (REL-2)  
- [ ] Phase-status statements agree across README, ADD, planning, and TASKS

---

### Stage B — Test & CI hardening

#### Purpose

Extend CI to the surfaces that actually gate releases: other operating systems, the eval harness, coverage, and MSRV.

#### Entry criteria

- Stage A complete (so CI is not enforcing stale claims)  
- P13 parity harness available to run in CI

#### Workstreams

| Workstream | Activities |
|---|---|
| **W-REL** | OS matrix for build + test (macOS, Windows, Linux) with clear skip rules for platform-specific tests |
| **W-REL / W-ACC** | Eval harness job: parity + ACC checklist on a schedule; failures open an issue rather than silently passing |
| **W-REL** | Coverage baseline + ratchet; report `prism-compile` / `prism-engine` separately (they carry accuracy) |
| **W-REL** | MSRV job pinned to `rust-version` |
| **W-EVAL** | Property/fuzz tests for extractors (malformed source, huge files, mixed encodings) and pack invariants (never drop must-include; no fragment without citation) |
| **W-SEC** | Secret-redaction regression test kept in the matrix (planted-secret fixture already exists) |

#### Deliverables

1. **CI matrix design** + runtime budget (keep PR feedback under a stated ceiling)  
2. **Coverage policy** — baseline, ratchet rule, exclusions  
3. **Property/fuzz test plan** for extractor and pack invariants  
4. **Harness-in-CI runbook** (what to do when accuracy drifts)

#### Exit / acceptance

- [ ] Three-OS matrix green (REL-4)  
- [ ] Coverage measured, published, and ratcheted (REL-5)  
- [ ] MSRV job green (REL-6)  
- [ ] Eval harness runs unattended and reports (REL-3)  
- [ ] Fuzz corpus finds no panics in extractor paths

---

### Stage C — Release trust & Phase 15 exit

#### Purpose

Finish what P11 started: an install users can verify, on a release that can be traced.

#### Entry criteria

- Stages A–B delivered  
- A public release tag exists (also unblocks the open **P11 Stage C** cold-VM gate)

#### Workstreams

| Workstream | Activities |
|---|---|
| **W-REL / W-DIST** | Provenance: SBOM and/or signature (cosign/minisign) attached to release artifacts (REL-7) |
| **W-DIST** | Upgrade/rollback smoke: N → N+1 → back to N, with host adapters intact |
| **W-SEC** | Release checklist executed against the gate tag; supply-chain review of new dependencies added since P11 |
| **W-EVAL** | Co-gate with P11 Stage C: cold-VM matrix run on the same tag |
| **W-DEBT** | Final reconciliation: phase status in every top-level document matches the gate evidence |

#### Deliverables

1. **P15 scorecard** — REL-1…REL-7  
2. **Provenance documentation** — what is signed, how to verify  
3. **Upgrade/rollback report**  
4. **Phase 15 gate evidence pack**

#### Exit / acceptance (Phase 15 gate)

- [ ] REL-1…REL-7 met or waived with dated ADRs  
- [ ] A user can verify a release artifact’s provenance from published instructions  
- [ ] Upgrade and rollback both proven on a real tag  
- [ ] P11 Stage C cold-VM gate closed or its residual risk restated with a date

#### Non-goals (this phase)

- New product capability of any kind  
- Team/shared index (P10)  
- Re-opening the IDE extension (ADR-0007)  
- Performance work (P14 owns ceilings)

#### Handoff

The program reaches a state where **capability, accuracy, performance, and claims** are each backed by a maintained artifact — the precondition for opening P10 or a public 1.0.

---

## 23. Evaluation program (runs across phases)

### 23.1 Benchmark arms (always)

| Arm | Description |
|---|---|
| A1 | Frontier model + explore tools |
| A2 | Medium model + explore tools |
| A3 | Medium model + Prism |
| A4 | Frontier model + Prism (optional best) |
| **A5** | **Medium model + doc-aware graph baseline (Graphify)** — added P12 Stage D |

**Program success:** A3 approaches A1; A4 optional ceiling. The four-arm run with real models is **executed in P9 Stage C** — until then, published numbers are structural proxies and must say so. **From P12 Stage D the suite is five-arm:** A5 exists so “we beat naive explore” is never mistaken for “we beat a doc-aware graph”, which measurement on 2026-07-26 showed we did not on narrative questions.

### 23.2 Task categories by phase emphasis

| Category | Introduce | Primary phase gate |
|---|---|---|
| Symbol explain / neighbors | P0/P1 | P1 |
| Impact (heuristic then precise) | P1 / P3 | P1 then P3 |
| Architecture overview | P1/P5 | P5 |
| Repo-QA / generate / review packs | P2 | P2 |
| Refactor prep | P3 | P3 |
| Bug localization / debug | P4 | P4 |
| **Time-to-orient (visual vs text)** | P7 | P7 |
| **Terminal-free task completion** | P8 | P8 *(cut — CLI+MCP path)* |
| **Agent tool-choice & refusal repair** | P9 | P9 |
| **Cold-machine install → MCP-ready** | P11 | P11 |
| **Doc-grounded product / narrative QA** | P12 | P12 |
| **Citation validity (does the answer’s evidence exist?)** | P12 | P12 |
| **Cross-tool parity vs doc-aware graph** | P12 | P12 |
| **Pack parity across a refactor** (same question, same evidence) | P13 | P13 |
| **Pilot-scale index & query performance** | P14 | P14 |
| **Claim freshness audit** (waivers, risk register, docs vs gates) | P15 | P15 |

### 23.3 Metrics ownership

| Metric | Owner workstream | First measured |
|---|---|---|
| Tokens / task | W-EVAL | P1 |
| Tool hops / task | W-EVAL | P1 |
| Context precision | W-EVAL + W-CC | P2 |
| Unresolved edge rate | W-OBS + W-KG | P1 |
| Call resolution P/R | W-EVAL | P3 |
| Incremental latency | W-OBS | P0 design / P1 measure |
| Pack compile latency | W-OBS + W-CC | P2 |
| Answer quality | W-EVAL | P1 onward; hard gate P9 (four-arm) |
| **Structural query P95 (N2)** | W-OBS + W-STORE | **P6 Stage A** (never measured before) |
| **Warm vs cold path latency** | W-SVC | P6 Stage B |
| **View render latency + budget adherence** | W-VIZ | P7 |
| **Time to orient** | W-EVAL + W-VIZ | P7 |
| **Extension activation cost** | W-IDE | P8 |
| **First-tool-choice rate** | W-AX + W-EVAL | P9 |
| **Refusal-repair success rate** | W-AX | P9 |
| **Cold-machine time-to-ready** | W-DIST + W-EVAL | P11 |
| **Install path success rate** (script / brew / scoop / …) | W-DIST | P11 |
| **Doc-QA answerability from one pack** (ACC-1) | W-ACC + W-CC | P12 |
| **Placeholder-fragment rate** (must reach 0 — ACC-2) | W-CC + W-OBS | P12 |
| **Seed-grounding precision / wrong-seed rate** (ACC-3) | W-PLAN + W-ACC | P12 |
| **Community label acceptance + unresolved-hub share** (ACC-4) | W-KG + W-ACC | P12 |
| **Parity delta vs doc-aware baseline arm A5** (ACC-5) | W-ACC + W-EVAL | P12 |
| **Pack parity rate across refactors** (REF-1) | W-REFAC + W-EVAL | P13 |
| **Max file / function size + waiver count** (REF-3) | W-REFAC | P13 |
| **Surface duplication & crate fan-in** (REF-2, REF-6) | W-REFAC | P13 |
| **N1 cold/incremental on pilot repos** (PERF-1, PERF-2) | W-PERF + W-STORE | P14 |
| **N2 P95 on pilot repos, enforced** (PERF-3) | W-PERF + W-OBS | P14 |
| **Warm orientation hit rate + invalidation correctness** (PERF-4) | W-PERF | P14 |
| **Peak memory + index size per KLOC** (PERF-5) | W-PERF | P14 |
| **Perf regression detection rate** (PERF-6) | W-PERF + W-EVAL | P14 |
| **Waiver freshness (expired waivers = 0)** (REL-1) | W-REL + W-DEBT | P15 |
| **Test coverage, ratcheted** (REL-5) | W-REL + W-EVAL | P15 |
| **CI matrix breadth + harness execution** (REL-3, REL-4) | W-REL | P15 |

### 23.4 Labeling discipline

- Necessary-span labels are **versioned** with pack algorithm version.  
- Prefer dual review on precision samples; dual review becomes **mandatory** from P9 Stage C.  
- Never change gold answers silently after a published report—cut a new suite version.  
- Visual-surface metrics (P7) are reported alongside token metrics, never instead of them: a faster-to-read wrong answer is still wrong.
- **Token metrics are never reported without an answerability metric (added P12):** a 149-token pack that teaches the model nothing is a failure, not a 22× win.
- **Performance numbers name their fixture, hardware, and snapshot (added P14):** a mini-fixture microsecond and a pilot-repo millisecond are different claims and are never averaged together.

---

## 24. Risk register & guardrails

### 24.1 Program risks (from ADD, planning actions)

| Risk | Planning guardrail |
|---|---|
| Syntactic call graphs too wrong | P3 mandatory before “safe refactor” marketing |
| Scope creep into search SaaS | Every phase asks: “Does this improve Evidence Packs?” |
| CPG cost explodes | P4 Stage B sharding + depth caps are exit criteria |
| Quality still needs frontier | Escalate context before model; accept hybrid |
| Plugin ecosystem stagnation | Ship 3–5 languages excellently; SDK in P5 |
| Users bypass Prism | P2 one-shot `compile_context` is the UX bet; P9 measures whether the bet paid |
| AOE cache bet returns | Answer cache blocked until P10 Stage C |
| Prism only usable by Rust contributors | P11 cold-machine install gate; PRODUCT-SETUP must not lead with `cargo build` |

### 24.2 Interaction-half risks (added 2026-07-26)

| Risk | Planning guardrail |
|---|---|
| Documentation describes systems that were never built | W-DEBT reconciliation is a gate item in every phase from P6 |
| Visualization becomes the product; the engine stagnates | P7 may only render existing facts — no new analysis lands in a rendering phase |
| A pretty graph is trusted more than its confidence labels | Confidence/tier encoding and a visible legend are exit criteria, not polish |
| Rendering degenerates into hairballs on real monorepos | LOD ceilings + `VIEW_TOO_LARGE` refusal; the same refuse-to-dump rule as packs |
| Daemon quietly becomes mandatory, breaking local-first | CLI-without-daemon stays a tested, supported path |
| TypeScript surface area outgrows the maintainers | Thin extension, thick daemon; one renderer package; no framework sprawl |
| Editor/agent platform churn | Feature-detect, degrade to plain MCP, pin API versions in CI |
| The four-arm benchmark never happens (again) | It is the P9 gate; the phase cannot exit without it |

### 24.3 Distribution-half risks (added 2026-07-26 — P11)

| Risk | Planning guardrail |
|---|---|
| “Install anywhere” claimed while still requiring `cargo build` | P11 Stage C cold-VM matrix is the gate |
| Unsigned / unverified `curl \| sh` | Checksums mandatory; signatures preferred; package managers as alternate trust path |
| Host adapters become a second product | One host-adapter catalog; generated snippets; generic stdio fallback |
| P11 quietly absorbs P10 shared-index work | Explicit non-goal; shared index stays deferred |
| VSIX revived under distribution pressure | ADR-0007; CLI + MCP only |

### 24.4 Accuracy-half risks (added 2026-07-26 — P12)

| Risk | Planning guardrail |
|---|---|
| Placeholder fragments make empty packs look successful | P12 Stage B invariant: no fragment without a citation; placeholder rate is a gated metric (ACC-2) |
| Token efficiency is optimized at the cost of sufficiency | Every token claim ships beside an answerability claim (§23.4); ACC-5 requires quality parity *and* ≤½ tokens |
| The doc layer becomes an LLM summarizer | Extractive spans only; no model in the index path (G8); `asserted` confidence, never precision-gated |
| Documentation lies and Prism launders the lie | `asserted` ≠ `extracted`; contradiction cases are part of the review sample; W-DEBT owns drift |
| Lexical seeding slides into embedding-first retrieval | Lexical may seed anchors only, never emit fragments; “structure before similarity” stays a release-notes guardrail |
| Semantic clustering becomes nondeterministic | Fixed seed, versioned algorithm id in provenance, snapshot-tested membership |
| Honesty notes outlive the code they describe | If clustering or hub ranking changes, the `confidence_note` / `repo_map` notes change in the same stage |
| Accuracy phase quietly reopens tiers or caching | T2/T4 semantics and answer caching are explicit P12 non-goals |
| Baseline arm is dropped when it wins | Arm A5 stays in the suite regardless of outcome; removing an arm requires a written waiver |

### 24.5 Engineering-half risks (added 2026-07-27 — P13–P15)

| Risk | Planning guardrail |
|---|---|
| A refactor phase quietly ships behavior changes | Parity harness is the gate (REF-1); any intentional divergence needs an ADR before merge (planning rule 6) |
| The refactor never ends — “just one more cleanup” | Stage D enforces budgets and closes the phase; remaining oversized units get dated waivers, not open-ended intent |
| The new service layer becomes the next god crate | Capability modules with the same size budgets; architecture fitness rules apply to the engine too |
| Typed refusals break the agent wire contract | Serde mapping is byte-compatible with today’s strings; a conformance test asserts it |
| Optimization work degrades accuracy | P12 ACC checklist re-runs at every P14 gate; caches are dependency-keyed memoization only |
| Snapshot-keyed caches serve stale evidence after an edit | Invalidation keyed on snapshot id (never TTL); edit→query freshness test is an exit criterion |
| Parallel indexing makes packs nondeterministic | Canonical ordering before persist; the P13 parity harness is the detector |
| Pilot benches get tuned into a benchmark game | Both pilots plus the mini fixture are reported; no cherry-picking a corpus |
| CI runner variance turns hard ceilings into flake | Ceilings derived from runner baselines with variance bands and a documented escape procedure |
| Growing CI matrix slows PR feedback until it is bypassed | Stated runtime budget; heavy jobs (eval harness, pilots) run scheduled, not per-PR |
| Governance sweep becomes a documentation-only ritual | REL-2 requires the register to be *generated* from gate artifacts; unsupported entries fail the sweep |
| Waivers accumulate silently again | Planning rule 8: an expired waiver blocks the next phase gate |
| The engineering half is skipped under feature pressure | Its own gate evidence packs; P10 / 1.0 may not open until P15 exits or its residual risk is restated with a date |

### 24.6 Stage churn guardrails

1. **No skipping phase gates** without a written waiver listing residual risk.  
2. **No embedding-centric retrieval narrative** in release notes.  
3. **No whole-repo CPG** as default indexing.  
4. **No abstractive code summaries** as default packing.  
5. **Vertical first:** correctness + tokens on one large repo before distributed work.  
6. **No unbounded rendering** — views obey budgets and refuse, exactly as packs do.  
7. **No claim without an artifact.** If a gate says “proven”, the repository must contain the thing that proves it (this rule exists because of gap G-03).  
8. **No “works on my machine” install story.** If a gate claims cross-platform install, evidence is cold VMs or CI matrix artifacts — not a maintainer laptop.  
9. **No synthesized evidence.** A fragment either cites a real node/edge with a source range or it does not ship; absence is expressed as a gap, never as plausible-looking filler (added P12).  
10. **No accuracy claim from proxies alone.** Structural proxies may report progress; a *quality* gate requires live-judged runs against the baseline arms (added P12).  
11. **No silent behavior change during a refactor.** Parity is the arbiter; divergences ship as ADRs (added P13).  
12. **No performance claim without fixture, hardware, and snapshot id** (added P14).  
13. **No expired waiver.** A waiver past its expiry phase blocks the next gate until re-dated, re-owned, or closed (added P15).

---

## 25. Definition of Done (program-level)

Prism’s planning program (P0–P5) is done when all are true:

1. **Architecture fidelity:** Delivered capabilities map cleanly to ADD components without elevating cache/RAG as spine.  
2. **Evidence Packs are primary:** Agents can answer most structural/debug intents via compiled packs with provenance.  
3. **Precision ladder is real:** T1 always; T2/T4 available where invested; confidence is honest.  
4. **Eval is public and reproducible:** Scorecard meets or honestly reports progress against G1–G4.  
5. **Extensibility:** Plugin ABI + golden fixtures allow a new language without core redesign.  
6. **Local-first privacy:** Default indexing path never requires network.  
7. **Operational clarity:** Incremental invalidation, observability, and security checklists exist.

**Status:** items 1–3 and 5–7 are met as of the P5 gate. Item 4 is **interim** — the four-arm benchmark lands in P9 Stage C.

### 25.1 Definition of Done — interaction half (P6–P9)

The interaction program is done when all are true:

1. **Documents match the repository.** No planning or tech-stack claim describes an artifact that does not exist; every accepted divergence has an ADR.
2. **Surfaces are real.** A daemon, an HTTP/SSE API, an LSP host, and an MCP server expose the same capabilities with the same error model.
3. **The graph is seeable without being dumped.** Every view is budgeted, deterministic, provenance-bearing, and refuses oversized scopes with anchors.
4. **The editor is sufficient.** ~~A developer completes orientation, impact, and debug tasks without a terminal.~~ **Restated after ADR-0007:** CLI + MCP complete orientation/impact/debug without a VSIX; terminal is an acceptable host for `prism setup`.
5. **Agents choose Prism unprompted,** measured on traces rather than asserted in a guide.
6. **The quality claim is settled.** The four-arm benchmark is published, and G1 is either evidenced or withdrawn.
7. **Local-first survived.** No surface added in P6–P9 requires network access or an always-on service.

Phase 10 is an **optional expansion**, not required for MVP product identity — and is **skipped for now**.

### 25.2 Definition of Done — distribution half (P11)

The distribution program is done when all are true:

1. **Cold machine works.** macOS, Linux, and Windows users install without a Rust toolchain.
2. **One-shot bootstrap works.** `prism setup` (or the documented agent skill sequence) yields index + agent assets + MCP registration.
3. **Trust path exists.** Installers verify checksums; release artifacts are reproducible enough for the security checklist.
4. **Hosts are covered.** Cursor plus at least one additional agent host have documented, uninstallable adapters.
5. **Docs match reality.** PRODUCT-SETUP / INSTALL lead with the release path; from-source is contributor-only.
6. **Local-first survived install.** After the one-time binary download, indexing and MCP do not require network.

### 25.3 Definition of Done — accuracy half (P12)

The accuracy program is done when all are true:

1. **Packs are sufficient.** A medium model answers doc-grounded and architecture questions from one pack, quoting real spans — no placeholders anywhere in the surface.
2. **Absence is honest.** When evidence does not exist, the pack says so in `gaps[]` with a repair action, and the agent recovers on the next call.
3. **Seeds are grounded.** Questions resolve to the right anchors or refuse with ranked candidates; “confidently about the wrong symbol” is a tracked defect class with a target of ≤5%.
4. **The repository is described in its own terms.** Communities and hubs name subsystems and design, not directories and `unwrap`.
5. **Documentation is a first-class, clearly-labeled source.** Doc claims carry `asserted` confidence and never satisfy precision gates.
6. **The comparison is public.** A five-arm, live-judged report — including a doc-aware graph baseline — shows Prism at parity or better on narrative tasks at ≤½ the tokens, with ablations attributing the gains.
7. **Local-first survived accuracy.** No LLM, key, or network entered the index path to achieve any of the above.

### 25.4 Definition of Done — engineering half (P13–P15)

The engineering program is done when all are true:

1. **The code is changeable.** One service layer owns capability execution, no file or function exceeds its budget without a dated ADR, and CI rejects reintroductions.
2. **Nothing moved that should not have.** A frozen question set produces the same evidence before and after the refactor; every intentional difference has an ADR.
3. **The error model is a type, not a convention.** Refusal codes live in the IR and every surface renders the same code the same way.
4. **Performance is proven where users live.** N1 and N2 targets are met on pinned pilot repositories with hard CI ceilings, published memory and index-size budgets, and a regression guard.
5. **Orientation is warm.** Repeated `repo_map` / community work is served from a snapshot-keyed cache whose invalidation is tested, not assumed.
6. **Claims are maintained.** Every waiver has a live expiry and an owner; the residual-risk register is generated from gate artifacts; documents agree on phase status.
7. **CI covers what gates releases.** Three OS families, MSRV, coverage with a ratchet, and the accuracy/parity harness all run without a human remembering to run them.
8. **Releases are verifiable.** A user can check provenance beyond a checksum, and upgrade/rollback are proven on a real tag.
9. **Nothing new shipped.** The engineering half added no user-visible capability — by design, and the changelog shows it.

---

## 26. Appendix — Checklists & templates

### 26.1 Stage kickoff checklist

- [ ] Re-read relevant ADD sections for this stage  
- [ ] Confirm entry criteria  
- [ ] Assign workstream owners (W-* IDs)  
- [ ] Name deliverables and review date  
- [ ] Identify eval measurement (even qualitative)  
- [ ] List non-goals for this stage (what we refuse to build now)  

### 26.2 Stage exit review template

| Field | Content |
|---|---|
| Stage | e.g., P2 Stage B |
| Deliverables attached | links to designs/fixtures/reports |
| Metrics | numbers or “N/A — design-only” |
| Open risks | residual |
| Waiver? | none / signed waiver |
| Next stage entry | confirmed / blocked by X |

### 26.3 Intent recipe card (template)

```text
Intent: <name>
Seeds: <anchors>
Expand: <operators + depth>
Must-include: <fragments>
Drop order exceptions: <never drop …>
Min tier: T1 | T2 | T3 | T4
Refuse when: <SCOPE_UNRESOLVED conditions>
Eval tasks: <IDs>
```

### 26.4 Gold task card (template)

```text
Task ID:
Repo + commit:
Category:
Prompt:
Accepted answer criteria:
Explore baseline notes:
Prism preferred tools / pack expectations:
Labels (necessary spans): 
Scoring method:
```

### 26.5 Phase gate evidence pack (required artifacts)

| Phase | Evidence to archive |
|---|---|
| P0 | Schema, ABI, 20 tasks, metrics event schema |
| P1 | Token/quality scorecard, unresolved rates, MCP catalog |
| P2 | Precision sample, EXPLAIN examples, refuse-dump fixtures |
| P3 | Resolution P/R, gating matrix, rename dry-run script |
| P4 | Debug scorecard, shard policy, slice properties |
| P5 | Public report, SDK docs, security checklist |
| P6 | Drift closure report, ADR set, N1/N2 benchmark baselines, `graph-view/v1` schema + fixtures |
| P7 | View kinds catalog, render budget report, time-to-orient scorecard, screenshot-diff suite |
| P8 | ~~VSIX artifact…~~ **Cut (ADR-0007)** — retain CLI+MCP setup evidence instead |
| P9 | Four-arm benchmark report v2, dual-reviewed precision sample, agent trace metrics |
| P10 | Authz pilot notes, SLA, cache certificate design *(deferred)* |
| P11 | Release archives + `SHA256SUMS`, installer smoke logs (3 OS), host-adapter matrix, P11 scorecard, INSTALL/PRODUCT-SETUP as-built |
| P12 | Doc extraction spec + goldens, IR schema delta (`Doc`/`Section`/`asserted`), fragment-integrity invariant test, doc-QA gold suite, five-arm accuracy report v1 + ablations, adjudication protocol + κ, P12 scorecard (ACC-1…ACC-7) |
| **Lang expansion** | Java + Perl extractor crates, golden fixtures, conformance CI update, `tree-sitter` 0.26 bump, pipeline version `p12-doc-v2-perl-java` | **2026-07-27** |
| P13 | Complexity census (before/after), golden parity corpus + harness run, capability contract for the service layer, crate dependency-graph diff, MCP schema conformance test, ADR set for waivers/divergences, P13 scorecard (REF-1…REF-7) |
| P14 | Pilot bench suite + per-phase timings, CPU profiles and query-plan audit, cache design + invalidation test, determinism proof under parallel indexing, perf scorecard per pilot with hardware + snapshot ids, CI ceiling policy, P14 scorecard (PERF-1…PERF-6) |
| P15 | Waiver audit + re-dated ADRs, generated residual-risk register with artifact links, CI matrix + coverage + MSRV run logs, fuzz/property corpus results, provenance (SBOM/signature) verification transcript, upgrade/rollback report, P15 scorecard (REL-1…REL-7) |

### 26.6 View kind card (template — P7)

```text
View kind: <name>
Purpose: <what question it answers in one sentence>
Seeds: <anchors / query>
Projection: <collapse + expand rules>
Default LOD: repo | subsystem | module | file | symbol
Budget: max_nodes / max_edges
Drop order: <what disappears first>
Never drop: <criterion / must-show elements>
Layout: <algorithm + determinism seed>
Confidence encoding: <how tier + confidence are shown>
Refuse when: <VIEW_TOO_LARGE conditions + suggested anchors>
Eval tasks: <IDs>
```

### 26.7 Glossary (planning-oriented)

| Term | Meaning |
|---|---|
| Evidence Pack | Budgeted hierarchical context with citations |
| Precision ladder | T0–T4 analysis tiers |
| Query plan | Operator DAG for assembling evidence |
| Gate | Must-pass phase/stage exit condition |
| Certified cache | Memoization allowed only with dependency freshness proof |
| **Graph View-Model** | Projected, budgeted, layout-ready subset of the KG — the visual analogue of an Evidence Pack |
| **Render budget** | `max_nodes`/`max_edges` ceiling with a deterministic drop order; overflow refuses |
| **LOD** | Level of detail: repo → subsystem → module → file → symbol |
| **Refusal repair** | A machine-actionable next step returned with every error, so agents recover unaided |
| **Drift register** | The W-DEBT list of divergences between documentation and the repository |
| **Ensure-install** | Graphify-inspired agent step: detect missing `prism`, install via approved path, persist resolved binary for MCP |
| **Host adapter** | Idempotent writer for a specific agent host’s MCP/rules/skills config (`cursor`, `claude`, `vscode`, …) |
| **Cold-machine path** | Install + setup + doctor on a VM with no Rust toolchain and no prior Prism state |
| **Time-to-ready** | Wall clock from zero install to first successful `compile_context` on a fixture repo |
| **Placeholder fragment** | Role-shaped text with no cited node/edge behind it — the P12 defect class being eliminated |
| **Gap (`gaps[]`)** | Declared absence of evidence for a role, with a reason and a repair action; the honest alternative to a placeholder |
| **Seed grounding** | Resolving a question to the anchors it is actually about, or refusing with ranked candidates |
| **Vocabulary expansion** | Expanding question terms against the graph’s own identifiers/paths/headings before planning |
| **`asserted` confidence** | A fact the documentation claims but code does not prove; never satisfies a precision gate |
| **Doc / Section node** | Documentation file and heading-scoped span, indexed extractively with byte ranges |
| **Semantic community** | Cluster from seeded Leiden over import+call+`DESCRIBES` edges, labeled extractively — successor to path-prefix communities |
| **Citation validity** | Whether the evidence an answer cites actually exists and says what the answer claims |
| **Answerability** | Whether one pack is sufficient to answer a gold task — reported beside every token metric |
| **Characterization test** | A test that records current behavior (not desired behavior) so a refactor can prove nothing moved |
| **Parity harness** | The P13 runner that compiles packs for a frozen question set before and after a change and diffs citations, roles, and structure |
| **Service layer (`prism-engine`)** | The single owner of capability execution; MCP/CLI/HTTP/LSP become protocol adapters over it |
| **Architecture fitness rule** | A build-enforced constraint on which crates may depend on which — the automated version of a boxes-and-arrows diagram |
| **Complexity budget** | Enforced ceilings on file and function size; exceeding one requires a dated ADR waiver |
| **Snapshot-keyed cache** | Memoization invalidated by index snapshot id rather than by time; the only caching P14 permits |
| **Pilot corpus** | A pinned real repository (httpx, ripgrep) used for scale claims — as opposed to the mini synthetic fixture |
| **Regression guard** | CI check that fails a change exceeding a recorded performance ceiling by a stated margin |
| **Waiver expiry sweep** | The P15 audit that re-dates, re-owns, or closes every ADR waiver whose expiry phase has shipped |

### 26.8 Install journey card (template — P11)

```text
OS / arch: <macos-arm64 | linux-x86_64 | windows-x86_64 | …>
Install path: <install.sh | install.ps1 | brew | scoop | cargo-binstall | …>
Binary location after install:
Checksum verified: yes/no
Setup command: prism setup <repo>
Hosts registered: <cursor | claude | vscode | …>
doctor --ready: pass/fail (JSON attached)
First compile_context: pass/fail + latency
Upgrade from previous tag: pass/fail / N/A
Uninstall clean: pass/fail
Notes / limitations:
```

### 26.9 Doc-QA gold task card (template — P12)

```text
Task ID:
Repo + commit (pinned SHA):
Category: product thesis | feature inventory | use cases | install/bootstrap | workflow semantics | non-goals | contradiction
Prompt (as a real user would ask it):
Accepted answer criteria (facts that must appear):
Necessary spans (doc sections and/or code ranges, with paths + line ranges):
Forbidden sources (vendored/fixture paths that must not be cited):
Expected pack shape: <roles that must be filled | roles allowed to be gaps>
Expected refusal (if under-specified): SCOPE_UNRESOLVED + candidate anchors
Baseline arm notes (explore / doc-aware graph):
Scoring: quality rubric + citation validity (invalid citation ⇒ 0)
```

### 26.10 Accuracy defect card (template — P12)

```text
Defect ID:
Class: placeholder fragment | wrong seed | missing doc node | noisy hub | vendored leak | stale asserted fact | invalid citation
Question that exposed it:
Observed pack (tokens, fragment ids, texts):
Why it is wrong (what a correct pack would have contained):
Root cause (component + file):
Metric affected: ACC-1 | ACC-2 | ACC-3 | ACC-4 | ACC-5 | ACC-6 | ACC-7
Fix stage: P12 Stage A | B | C | D
Regression artifact added: <fixture / invariant test / gold task id>
```

### 26.11 Refactor parity card (template — P13)

```text
Refactor ID:
Unit moved: <file / function / crate boundary>
Before: <LOC, fn length, dependents, hub degree>
After: <LOC, fn length, dependents>
Behavior intent: none (parity) | intentional change (ADR: ____)
Parity run: <corpus version, questions compared, diff result>
Differences found: <none | list, each with ADR or fix>
Tests added with the move: <ids>
Budget compliance: file ≤600 / fn ≤120 — pass | waiver (ADR: ____, expires P__)
```

### 26.12 Performance budget card (template — P14)

```text
Measurement ID:
Target: PERF-1 | PERF-2 | PERF-3 | PERF-4 | PERF-5 | PERF-6
Fixture: mini | httpx@<sha> | ripgrep@<sha>
Hardware: <CI runner class | local machine spec>
Index snapshot id:
Metric + value: <p50 / p95 / mean, with units>
Profile reference: <flamegraph / query plan artifact>
Change attributed: <what code change moved this number>
Accuracy re-check: P12 ACC checklist pass/fail
Ceiling proposed: <value + variance band> | not yet
```

---

## Related documents

- [Architecture Design Document](../architecture/ARCHITECTURE-DESIGN-DOCUMENT.md) — design authority  
- [Tech Stack & Project Structure](../architecture/TECH-STACK-AND-PROJECT-STRUCTURE.md) — how it is built, per phase  
- [Product setup](../architecture/PRODUCT-SETUP.md) — current CLI+MCP bootstrap (P11 expands this to any system)  
- [Tasks & Progress](./TASKS-AND-PROGRESS.md) — living checklist and phase state  
- [Program residual risks](../eval/PROGRAM-RESIDUAL-RISKS.md) — R1/R2/R8 are the P9 targets  
- [Repo feature summary & token comparison](../REPO-FEATURE-SUMMARY-AND-TOKEN-COMPARISON.md) — the 2026-07-26 measurement that motivated P12  
- [Public benchmark report v2](../eval/PUBLIC-BENCHMARK-REPORT-V2.md) — four-arm proxies that P12 Stage D replaces with live-judged five-arm results  
- [Why agents grepped instead of using Prism](../eval/WHY-AGENTS-GREP-INSTEAD-OF-PRISM.md) — the sufficiency diagnosis behind P12  
- [P6 Stage A performance baselines](../../eval/scorecards/p6-stage-a-baselines.md) — the mini-fixture numbers P14 replaces with pilot-scale proof  
- ADD §36 Phased Implementation Roadmap — phase durations and high-level gates (expanded here)

---

*End of Planning & Implementation Document. P0–P7 + P9 delivered; P8 cut; P10 optional/deferred (skipped for now); P11 Stage A+B complete with Stage C pending a public release; P12 Accuracy & Grounding gated; **Java + Perl T1 extractors delivered 2026-07-27** ([§7.5](#75-language-expansion--java--perl-delivered-2026-07-27)); P13 Core Refactor, P14 Performance & Scale Proof, and P15 Reliability & Governance opened 2026-07-27 — planned, not yet implemented.*
