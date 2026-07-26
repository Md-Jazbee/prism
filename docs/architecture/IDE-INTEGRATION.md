# IDE integration design (P5 Stage B)

**Status:** Design retained; **VS Code extension cut** ([ADR-0007](./adr/0007-extension-cut-cli-mcp.md))  
**Stub predecessor:** [IDE-EVIDENCE-PEEK.md](./IDE-EVIDENCE-PEEK.md)  
**As-built product surface:** CLI + MCP + optional `prismd` / `prism lsp` — see [PRODUCT-SETUP.md](./PRODUCT-SETUP.md)

---

## As-built (post–extension cut)

| Need | Use |
|---|---|
| Agent compile-first | MCP `compile_context` (`prism mcp`) |
| Cold workspace | `prism setup` |
| Orientation / impact / slice | `prism query` / `prism compile` / `prism semantic slice` / `prism view` |
| In-editor LSP augment | `prism lsp` (hover / symbols / codelens) |
| Interactive graph panel | **Out of tree** — `@prism/graph-view` for SVG/Mermaid only |

---

## Historical command surface (design only)

| Command | Behavior |
|---|---|
| `prism.compileContext` | Compile Evidence Pack for selection / symbol / stack; show side panel |
| `prism.evidencePeek` | Jump from citation `C#` → file span / graph node |
| `prism.impact` | Impact cone for symbol under cursor (`require_precise` optional) |
| `prism.slice` | Local/interproc slice for line under cursor |
| `prism.explain` | Toggle EXPLAIN / drops for last pack |

These remain a UX sketch if an IDE host is reconsidered; they are **not** shipped.

---

## Transport

| Mode | Use |
|---|---|
| MCP stdio | Preferred for agents inside Cursor |
| `prism compile` / `query` CLI | Scripts, CI, humans without MCP |
| Daemon HTTP/SSE (`prismd`) | Optional local accelerator |
| `prism lsp` | Native LSP commands (augments; does not replace language servers) |

---

## Non-goals

- Write / apply rename from IDE  
- Hosting a language server that replaces rust-analyzer / pylsp  
- Cloud sync of packs  
- Marketplace VSIX (cut)

## Relationship to native LSP

Prism **augments** go-to-def / find-refs with budgeted evidence. It does not replace the language server for editing intelligence.
