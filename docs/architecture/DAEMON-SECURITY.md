# Local security posture (`prismd`)

**Phase:** P6 Stage B

| Control | Default |
|---|---|
| Bind address | `127.0.0.1:7420` only; **non-loopback binds are refused** |
| Auth | Bearer / `X-Prism-Token` required for all `/v1/*` except public health |
| Token source | `--token`, `PRISM_TOKEN`, or generated ephemeral token printed at start |
| Remote origin | Not supported in Stage B — no CORS wildcards for remote hosts |
| Pack audit | Existing `pack_bound_for_llm` events still apply when packs are compiled |
| Secrets | Indexer ignore policy unchanged; daemon does not relax secret skipping |

## Operator checklist

1. Prefer generated tokens for local IDE sessions; do not commit tokens.
2. Do not tunnel `prismd` to the public internet without an explicit future design (authz, TLS).
3. Treat Evidence Packs as sensitive — they contain source spans.
