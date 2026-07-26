# IDE integration design (P5 Stage B)

**Status:** Design locked; extension may ship phased  
**Stub predecessor:** [IDE-EVIDENCE-PEEK.md](./IDE-EVIDENCE-PEEK.md)  
**Surface:** VS Code / Cursor TypeScript extension calling `prism` CLI or MCP

---

## Commands

| Command | Behavior |
|---|---|
| `prism.compileContext` | Compile Evidence Pack for selection / symbol / stack; show side panel |
| `prism.evidencePeek` | Jump from citation `C#` → file span / graph node |
| `prism.impact` | Impact cone for symbol under cursor (`require_precise` optional) |
| `prism.slice` | Local/interproc slice for line under cursor |
| `prism.explain` | Toggle EXPLAIN / drops for last pack |

---

## Side panel UX

```text
┌ Evidence Pack ─────────────────────┐
│ Intent · tokens used/budget        │
│ Citations C1…Cn (click → peek)     │
│ Layers: Arch · Mod · Core · …      │
│ Gaps / uncertainty notes           │
│ [EXPLAIN] [Copy for LLM]           │
└────────────────────────────────────┘
```

**Copy for LLM** triggers client-side `pack_bound_for_llm` audit event and applies redaction policy.

---

## Transport

| Mode | Use |
|---|---|
| Daemon HTTP/SSE (`prismd`) | Preferred for IDE panels (P8) |
| `prism compile` / `query` CLI | Extension fallback without daemon |
| MCP stdio | Preferred for agents inside Cursor (auto-registered in P8) |
| `prism lsp` | Native LSP commands (augments; does not replace language servers) |

---

## Non-goals (Stage B)

- Write / apply rename from IDE  
- Hosting a language server that replaces rust-analyzer / pylsp  
- Cloud sync of packs  

## Relationship to native LSP

Prism **augments** go-to-def / find-refs with budgeted evidence. It does not replace the language server for editing intelligence.
