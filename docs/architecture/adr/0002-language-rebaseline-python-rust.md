# ADR-0002: Language re-baseline — Python + Rust first

**Date:** 2026-07-26  
**Status:** Accepted  
**Gaps:** G-04 · residual risk R5  
**Expiry:** Phase 9 (language expansion track review)

## Context

Early planning named TypeScript and Go among first extractors. As-built P1 delivered **Python** and **Rust** only (`prism-extract-python`, `prism-extract-rust`). The change was never written down as a waiver.

## Decision

Accept **Python + Rust** as the P0–P5 language baseline. Treat TS/Go (and further languages) as a post-MVP expansion track via the documented native ABI + golden fixtures, not as silent scope creep back into the engine half.

## Consequences

- Planning and tech-stack language lists match the crates that exist.
- R5 remains *Accepted* with this dated ADR as the waiver artifact.
- New languages land without core engine redesign (plugin guide path).
