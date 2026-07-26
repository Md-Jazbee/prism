# ADR-0003: MCP transport stays hand-rolled stdio

**Date:** 2026-07-26  
**Status:** Accepted  
**Gaps:** G-05  
**Expiry:** Phase 8 (revisit when IDE extension + multi-transport clients need `rmcp`)

## Context

The tech-stack decision named **`rmcp`** (official Rust MCP SDK). As-built `prism-mcp` is a hand-rolled stdio JSON-RPC 2.0 server (`protocol 2024-11-05`) with nine allowlisted tools. It works against Cursor and other stdio clients today.

## Decision

**Keep** the hand-rolled stdio transport for P6. Externalize tool contracts to `schemas/mcp-tools/v1` and validate the Rust allowlist against those schemas. Revisit `rmcp` migration when:

1. HTTP/SSE (`prismd`) needs a shared request model, or
2. The VS Code / Cursor extension requires SDK features the hand-roll lacks.

## Consequences

- Docs say “stdio JSON-RPC (hand-rolled); `rmcp` migration ADR open until P8.”
- No mid-phase rewrite of a working agent surface.
- Schema drift is caught by the mcp-tools conformance check, not by an SDK.
