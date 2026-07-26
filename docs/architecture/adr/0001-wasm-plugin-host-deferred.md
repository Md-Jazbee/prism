# ADR-0001: WASM plugin host deferred

**Date:** 2026-07-26  
**Status:** Accepted  
**Gaps:** G-03 · residual risk R11  
**Expiry:** Phase 8 (revisit when external contributed languages need sandboxing)

## Context

The P5 tech-stack view claimed a “WASM host **proven** with one example plugin.” The repository has no `prism-plugin-host`, no wasmtime dependency, and no `plugins/` tree. First-party extractors are native Rust crates that honor the Fact IR ABI and golden fixtures.

## Decision

**Defer** `prism-plugin-host` (wasmtime WIT). Amend the P5 claim to:

> Native plugin ABI documented and conformance-tested; WASM Component Model host deferred.

External contributors prototype against the ABI + goldens; hosting moves to WASM without changing Fact IR when the host ships.

## Consequences

- Plugin guide and tech-stack docs must not say “proven” for WASM until an artifact exists.
- R11 closes as *waived with expiry P8*, not *done*.
- Stage A does **not** build wasmtime scaffolding “for show.”
