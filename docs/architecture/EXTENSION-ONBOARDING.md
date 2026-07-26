# First-run onboarding flow (P8)

## Triggers

- Workspace opened and no `.prism/` index artifacts, **or**
- Binary unresolved, **or**
- User runs any Prism command for the first time in the session

## Steps (UI)

1. **Detect binary** — resolve per ADR-0006; if missing, show “Install / locate Prism CLI”.
2. **Detect index** — `GET /v1/index/status` or `prism index-status`; if empty/missing, offer **Build index**.
3. **Build** — prefer daemon `POST /v1/index` with SSE `index.updated` for completion; CLI fallback `prism index` with progress in the output channel.
4. **Orient** — open Graph panel with `architecture_map` / `repo_map`; offer Compile for a starter question.
5. **Agent (Cursor)** — optionally register MCP (`prism.agent.enableMcp`); generate `AGENTS.md` stub from AGENT-USAGE.

## Never

- Auto-upload repo content
- Auto-enable telemetry
- Block the editor during index (background + status bar)
