# Architecture Decision Records

ADRs capture accepted divergences between documents and the as-built repository.
Opened in **P6 Stage A** (gap register G-03…G-11).

| ID | Title | Status | Expiry |
|---|---|---|---|
| [0001](./0001-wasm-plugin-host-deferred.md) | WASM plugin host deferred | Accepted | P8 |
| [0002](./0002-language-rebaseline-python-rust.md) | Language re-baseline: Python + Rust | Accepted | P9 |
| [0003](./0003-mcp-transport-hand-rolled.md) | MCP transport stays hand-rolled stdio | Accepted | — (reaffirmed; extension cut) |
| [0005](./0005-otlp-exporter-deferred.md) | OTLP exporter deferred (env hook only) | Accepted | P7 |
| ~~0006~~ | Extension binary delivery | **Superseded** by [0007](./0007-extension-cut-cli-mcp.md) | — |
| [0007](./0007-extension-cut-cli-mcp.md) | VS Code extension cut — CLI + MCP product surface | Accepted | — |

**Convention:** one ADR per accepted divergence; every waiver names an expiry phase.
