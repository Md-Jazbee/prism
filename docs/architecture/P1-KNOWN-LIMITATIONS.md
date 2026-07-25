# Phase 1 known limitations

Wrong or missing callees at T1 **will** poison `impact` and some `neighbors` answers.

| Failure mode | Effect | Mitigation |
|---|---|---|
| Heuristic same-file CALLS miss cross-file callees | Impact under-reports | Escalate to T2 (P3); depth caps |
| Name collision resolves wrong same-file symbol | Wrong neighbor | Prefer unique names / path filter |
| Dynamic Python / Rust macros | Missing CALLS | Documented in extractor design docs; unresolved edges for unknown names |
| `unresolved:*` nodes in impact | Noise in blast radius | Agents must treat as incomplete |
| Path-prefix communities ≠ true modules | Misleading hubs | Notes field on `repo_map`; Leiden later |
| No precise rename | Unsafe to auto-refactor | Explicitly out of P1 claims |

**Product rule:** Never claim precise refactor safety in P1 docs, MCP instructions, or scorecards.
