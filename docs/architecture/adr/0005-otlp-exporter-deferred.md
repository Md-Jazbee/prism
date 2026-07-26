# ADR-0005: OTLP exporter deferred to collector wiring

**Date:** 2026-07-26  
**Status:** Accepted  
**Gaps:** G-09 · residual risk R-obs  
**Expiry:** Phase 7

## Context

Stage B requires a real OTLP path behind an opt-in flag. Pulling the full `opentelemetry-otlp` + tonic stack in the first daemon cut risks license/deny churn and a large dependency graph before clients exist.

## Decision

Ship an **opt-in hook**: when `PRISM_OTLP_ENDPOINT` is set, `prismd` logs the endpoint and emits a tracing/obs event. Full OTLP SDK export lands when a local collector is part of the supported operator path (target P7).

## Consequences

- G-09 is waived with expiry P7, not claimed “done”.
- Span design in `OTEL-SPANS.md` remains the contract the exporter must honor.
