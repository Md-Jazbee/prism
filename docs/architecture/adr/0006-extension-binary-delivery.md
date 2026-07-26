# ADR-0006: Extension binary delivery — PATH first, verified download fallback

**Date:** 2026-07-26  
**Status:** Accepted  
**Gaps:** G-14  
**Expiry:** none (revisit when Marketplace platform packages ship)

## Context

Phase 8 Stage A requires a binary distribution decision for the VS Code / Cursor extension. Options were: platform-specific VSIX with bundled `prism`/`prismd`, verified download-on-demand, or PATH / workspace-built binary first.

Bundling multi-OS binaries bloats the VSIX and complicates CI for an engine still at `api_version: 0.0.1`. Marketplace-ready platform packages remain a follow-on.

## Decision

1. **Resolve order:** extension setting `prism.binaryPath` → `PATH` (`prism`) → workspace `target/debug/prism` / `target/release/prism` → **verified download-on-demand** (checksum + version handshake).
2. **Thin VSIX:** do not embed native binaries in the default package.
3. **Version skew:** client compares extension `engineMajor` to daemon `/health.api_version` major; mismatched majors refuse with an upgrade action (no silent mismatch).
4. **Platform-specific VSIX** with bundled binaries is deferred until a release train needs offline installs; this ADR documents the intended path.

## Consequences

- Local developers and CI use a built or PATH `prism` with zero download.
- First-run UX must detect missing binary and offer download or build guidance.
- Signing/checksum manifests live under `extensions/vscode/binaries/manifest.json` (placeholder until release artifacts exist).
