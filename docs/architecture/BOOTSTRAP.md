# Bootstrap runbook (P11)

Ordered path for humans and agents. Matches the ensure-install skill in generated `AGENTS.md` / `.prism/agent/skills.md`.

1. **Ensure binary** — `command -v prism` or run `scripts/install.sh` / `install.ps1` (set `PRISM_GITHUB_REPO`).
2. **Setup workspace** — `prism setup .` (index + assets + default MCP).
3. **Doctor** — `prism doctor --ready` (also prints host + hook status).
4. **Optional hosts** — `prism host install <cursor|vscode|claude|generic>`.
5. **Optional hook** — `prism hook install` (post-commit incremental `prism index`).
6. **First question** — MCP `compile_context` or `prism workflow run onboarding`.

Never ask for an API key for core indexing. Local-first (G8).

Troubleshooting: [PRODUCT-SETUP.md](./PRODUCT-SETUP.md) · [RELEASE-ARTIFACTS.md](./RELEASE-ARTIFACTS.md) · [HOST-ADAPTERS.md](./HOST-ADAPTERS.md)
