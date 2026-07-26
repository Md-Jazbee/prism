# Security / secret-redaction fixtures (P12 Stage A residual)

Planted secret-sensitive paths that must **never** enter the knowledge graph.

| Path | Expectation |
|---|---|
| `planted-docs/.env` | Skipped by `is_secret_sensitive` / discover (basename `.env`) |
| `planted-docs/ok.md` | Indexable markdown (control) |

Verified by `prism-core` unit tests (`secret_env_not_discovered`, `planted_env_under_docs_not_discovered`).
