<!-- prism:generated from schemas/agent-workflow/v1 — regenerate via `prism agent generate-assets` -->
# AGENTS.md — Prism guidance (generated)

> Prefer `compile_context` (or a named workflow) before explore loops.

## Primary path

1. Call **compile_context** or `prism workflow run <id>`.
2. Answer from Evidence Pack citations; inspect gaps / EXPLAIN drops.
3. Use micro-tools only for targeted follow-ups.

## Ensure install (P11)

If `prism` is missing from PATH, install before exploring the repo. **Do not ask for an API key** for core indexing — local-first.

1. macOS/Linux: `curl -fsSL "https://raw.githubusercontent.com/${PRISM_GITHUB_REPO:-example/prism}/main/scripts/install.sh" | bash`
2. Windows (PowerShell): `irm "https://raw.githubusercontent.com/$env:PRISM_GITHUB_REPO/main/scripts/install.ps1" | iex` (set `PRISM_GITHUB_REPO` first)
3. Re-resolve the binary (`command -v prism`) and run `prism setup .`
4. Confirm with `prism doctor --ready`, then call MCP `compile_context`
5. Optional: `prism host install <cursor|vscode|claude|generic>` · `prism hook install` for post-commit reindex

Contributor fallback (this monorepo only): `cargo build -p prism-cli --release` then use `./target/release/prism`.

## Workflows

### `onboarding` — Repo onboarding / orientation

- Trigger: Newcomer or fresh agent needs orientation
- Tools: repo_map → entrypoints → compile_context
- Pack: architecture orientation pack with communities/hubs citations

### `review` — Change review

- Trigger: Review a diff / PR / changed paths
- Tools: compile_context → impact
- Pack: review pack citing changed paths + blast radius

### `debug` — Debug / crash investigation

- Trigger: Stack trace or error text available
- Tools: compile_context
- Pack: debug pack with error/stack + slice criterion never dropped

### `refactor_prep` — Refactor preparation

- Trigger: Rename or structural refactor about to start
- Tools: compile_context
- Pack: refactor pack with precise references or PRECISION_REQUIRED repair

## Refusals → next action

| Code | Do this |
|---|---|
| SCOPE_UNRESOLVED | Pick a symbol / path / stack frame |
| BUDGET_EXCEEDED | Raise `remaining_context_tokens` / narrow anchors |
| INDEX_UNAVAILABLE | `prism setup .` or `prism index .` (ensure install first) |
| PRECISION_REQUIRED | `prism precise import` or continue labeled heuristic |
| VIEW_TOO_LARGE | Narrow seeds / anchors |

## Anti-patterns

- Do not open dozens of files via grep/read when compile_context can answer.
- Do not claim rename safety from unlabeled impact.
- Do not block on API keys for indexing or MCP setup.
