# Release readiness checklist (Phase 5)

**Use before tagging a Prism release that claims P0–P5 gates.**

## Product / docs

- [ ] [PUBLIC-BENCHMARK-REPORT.md](../eval/PUBLIC-BENCHMARK-REPORT.md) current  
- [ ] [plugin-guide.md](../contributing/plugin-guide.md) linked from README  
- [ ] [AGENT-USAGE.md](../architecture/AGENT-USAGE.md) primary path documented  
- [ ] [SECURITY.md](../../SECURITY.md) + [RELEASE-CHECKLIST.md](../security/RELEASE-CHECKLIST.md)  
- [ ] Planning board shows P5 gate status  

## Engineering

- [ ] `cargo test --workspace` green  
- [ ] `./scripts/plugins/conformance-check.sh` green  
- [ ] `uv run prism-eval smoke` + `p5-scorecard` green  
- [ ] No write/rename apply in MCP allowlist  
- [ ] Secret paths still skipped at index  

## Eval honesty

- [ ] Proxy vs LLM claims clearly labeled in the public report  
- [ ] Precision interim (&lt;70%) has a written close plan if not met  
- [ ] Suite version frozen in `eval/SUITE-VERSION.md`  

## Optional defer

- [ ] Phase 6 team/shared index — deferred unless product need  
