# Audit log & redaction policies (P5 Stage B)

**Status:** Written / enforceable stubs in `prism-obs`  
**Goal:** Know what Evidence Pack content could leave the machine; never leak secrets into indexes or LLM prompts.

---

## Audit events

| Event | When | Fields |
|---|---|---|
| `pack_bound_for_llm` | Agent/IDE marks a pack as sent (or about to send) to an LLM | `plan_id`, `token_estimate`, `fragment_count`, `redacted`, `workspace_fingerprint` |
| `query_finished` | Structural / compile ops | existing |
| `file_skipped_secret` | Indexer skip | path |

Emission today: `tracing` via `emit_index_event`. OTel exporters later ([OTEL-SPANS.md](../architecture/OTEL-SPANS.md)).

Agents **should** call / log `pack_bound_for_llm` when exporting pack JSON to a model. MCP `compile_context` does not auto-forward to an LLM — the client owns the boundary.

---

## Redaction policy

1. **At index time:** secret-sensitive paths never enter `graph.sqlite` or semantic artifacts.  
2. **At pack time:** if fragment text matches secret patterns (API key regex, `-----BEGIN`, `.env` assignments), replace body with `[REDACTED]` and set provenance note `redacted=true`.  
3. **Never** strip provenance node ids for non-secret code — audits need them.  
4. Error/stack verbatim may contain secrets from the user's runtime — agents must scrub before `pack_bound_for_llm` if policy requires.

---

## Plugin review process

| Step | Owner |
|---|---|
| ABI conformance + golden fixtures | Contributor |
| Code review: no network/FS side effects | Maintainer |
| Run `scripts/plugins/conformance-check.sh` | CI |
| Document analyzer string + limitations | Contributor |
| Optional: security note in extractor doc | Maintainer |

Rejected plugins: process spawn, credential reads, unbounded temp writes.
