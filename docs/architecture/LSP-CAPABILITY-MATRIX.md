# LSP capability matrix

**Phase:** P6 Stage C  
**Binary:** `prism-lsp` / `prism lsp`  
**Transport:** stdio (`lsp-server` + `lsp-types`)

Prism **augments** the editor. It does **not** replace rust-analyzer or pylsp for editing intelligence.

| Capability | Prism LSP | Deferred to language server |
|---|---|---|
| Hover (evidence summary) | ✅ budgeted pack snippet | Full type hover / docs |
| Go to definition | stub / null (use native) | ✅ rust-analyzer / pylsp |
| Find references | — | ✅ native |
| Workspace symbol | ✅ KG `resolve_symbol` | ✅ native (richer) |
| Code lens | ✅ impact + compile entry | — |
| `prism.compileContext` | ✅ executeCommand | — |
| `prism.impact` | ✅ executeCommand | — |
| `prism.slice` | ✅ view projection | — |
| `prism.evidencePeek` / `explain` | ✅ stub note | Panel UX in P8 |
| Completions / rename / format | ❌ | ✅ native |
| Diagnostics | ❌ | ✅ native |

## Commands (IDE-INTEGRATION.md)

| Command | Stage C |
|---|---|
| `prism.compileContext` | ✅ |
| `prism.impact` | ✅ |
| `prism.slice` | ✅ |
| `prism.evidencePeek` | stub |
| `prism.explain` | stub |

## Startup

```bash
prism lsp --workspace /path/to/repo
# or
prism-lsp --workspace /path/to/repo
```

Requires `.prism/graph.sqlite` for useful results (`prism index` first).
