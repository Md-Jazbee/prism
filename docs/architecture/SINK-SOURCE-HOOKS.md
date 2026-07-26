# Sink / source provider hooks (P4 Stage B)

**Status:** Design locked; adapters optional  
**Audience:** Security / debug intents that need taint-like criteria

---

## Purpose

Let Semgrep-class (or custom) feeds supply **extra slice criteria** without baking a full taint engine into Prism core.

---

## Hook interface (v0)

```text
SourceSinkProvider
  id: string
  version: string
  list_sources(workspace) -> [{ path, line, symbol?, tag }]
  list_sinks(workspace)    -> [{ path, line, symbol?, tag }]
```

Tags examples: `user_input`, `sql_exec`, `http_response`.

Planner (security / future debug enrichment):

1. Resolve primary criterion (stack / symbol).  
2. Optionally union provider sinks/sources near the seed.  
3. Run `Slice` with those criteria (depth-capped).

---

## Storage

Providers write **nothing** into hot `graph.sqlite`. Optional cache:

```text
.prism/semantic/providers/<id>.json
```

Invalidated with content hashes like other semantic artifacts.

---

## Non-goals (Stage B)

- No mandatory Semgrep dependency  
- No claim of sound taint analysis  
- Hooks are opt-in; debug recipe works without them
