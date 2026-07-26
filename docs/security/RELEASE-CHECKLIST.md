# Security release checklist (P5 Stage B)

**Audience:** maintainers cutting a Prism release  
**Related:** [AUDIT-AND-REDACTION.md](./AUDIT-AND-REDACTION.md) · [IGNORE-POLICY-CHECKLIST.md](../architecture/IGNORE-POLICY-CHECKLIST.md)

---

## Before tag

- [ ] Default mode is **local** — no cloud LLM calls required for index / compile / MCP  
- [ ] Secret-sensitive paths skipped (`is_secret_sensitive`); `.env`, keys, PEM never indexed  
- [ ] MCP allowlist reviewed — no write / apply-rename tools  
- [ ] Safe rename remains dry-run only  
- [ ] Plugin ABI pure-transform rule still documented; no extractor network I/O  
- [ ] Dependency audit (`cargo deny` / `cargo audit` when configured) — no known critical CVEs  
- [ ] Changelog notes security-relevant changes  

## Audit / redaction

- [ ] Pack audit events defined (`pack_bound_for_llm`) — see AUDIT-AND-REDACTION  
- [ ] Redaction policy for secrets in fragment text documented  
- [ ] Plugin review process documented for third-party extractors  

## Post-release

- [ ] Security contact / advisory process noted in README or SECURITY.md  
- [ ] If a CVE: revoke bad plugin versions; bump fact schema only if IR leak  

## Non-goals (this release track)

- Multi-tenant SaaS authz (Phase 6)  
- Signed WASM marketplace (future)
