# Extension architecture (P8)

**Surface:** `extensions/vscode` — thin TypeScript host; thick Rust daemon/CLI.  
**Binary delivery:** [ADR-0006](./adr/0006-extension-binary-delivery.md)  
**Command contract:** [IDE-INTEGRATION.md](./IDE-INTEGRATION.md)  
**HTTP contract:** [HTTP-API-V1.md](./HTTP-API-V1.md)

## Processes

```text
┌ VS Code / Cursor host ─────────────────────────────────────┐
│  extension.ts (activation)                                  │
│    ├─ lifecycle/  resolve binary, spawn/reuse prismd        │
│    ├─ transport/  DaemonClient → CliFallback                │
│    ├─ panels/     Evidence + Graph webviews                 │
│    ├─ decorations/ ambiguity · hotspot · slice              │
│    └─ agent/      MCP auto-reg · AGENTS.md · refusal UX     │
└──────────────┬──────────────────────────┬───────────────────┘
               │ HTTP + SSE (preferred)   │ spawn CLI
               ▼                          ▼
         prismd :7420                 prism compile|query|…
```

## Transport selection

| Priority | Mode | When |
|---|---|---|
| 1 | Daemon HTTP + SSE | `.prism/daemon.lock` healthy + token + matching API major |
| 2 | CLI fallback | Daemon absent/crashed; UI shows “CLI mode” |
| 3 | MCP stdio | Cursor agent path only (`prism mcp`) — not used for panel RPCs |

Superseded UI requests abort in-flight `fetch` (daemon honors disconnect). Webviews are pure views: the host owns the last pack / last view-model.

## Failure & recovery

| Failure | Recovery |
|---|---|
| No binary | Onboarding: PATH hint → build workspace → download-on-demand |
| No index | Offer `POST /v1/index` or `prism index`; stream invalidation via SSE |
| Daemon crash | Fall back to CLI; status bar states degradation |
| Major version skew | Refuse; show upgrade action |
| `SCOPE_UNRESOLVED` | Anchor picker (do not show raw dump) |
| `PRECISION_REQUIRED` | “Generate / import SCIP” action |
| `VIEW_TOO_LARGE` | Suggest anchors from error payload |

## Security

- Loopback-only daemon; token never leaves the machine.
- Repository content is never transmitted off-machine by the extension.
- “Copy for LLM” applies redaction and emits local `pack_bound_for_llm` audit (opt-in counters only; never content).
- Telemetry: off by default; when enabled, counters only (command ids + refusal codes).

## Activation budget

See [EXTENSION-ACTIVATION-BUDGET.md](./EXTENSION-ACTIVATION-BUDGET.md). Lazy activation on workspace folder + command/view contribution; no work on idle editor start beyond contribution registration.
