# First-run onboarding flow (P8)

## Triggers

- Workspace opened and no `.prism/` index artifacts, **or**
- Binary unresolved, **or**
- User runs any Prism command for the first time in the session

## Steps (UI)

1. **Prism: Setup Workspace** (or `prism setup .`) — binary → index → AGENTS.md/rules → MCP → daemon.
2. Open **Graph** panel (architecture map) and/or **Compile Context**.
3. On refusals, use the offered next action (Pick Anchor / Build Index / SCIP).

## Never

- Auto-upload repo content
- Auto-enable telemetry
- Block the editor during index (background + status bar)

See [PRODUCT-SETUP.md](./PRODUCT-SETUP.md).
