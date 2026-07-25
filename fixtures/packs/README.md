# Evidence Pack fixtures (P2 Stage B)

| Case | Outcome | Notes |
|---|---|---|
| `repo_qa_ok` | `ok` pack | synthetic candidates from recipe |
| `budget_drop` | `ok` with drops | optional noise dropped; must-include kept |
| `budget_exceeded` | `BUDGET_EXCEEDED` | must-include > budget |
| `explain_roundtrip` | `ok` with explain | impact synthetic pack |
| `refuse-dump` | `SCOPE_UNRESOLVED` | ambiguous question — no unbounded dump |

```bash
UPDATE_GOLDENS=1 cargo test -p prism-compile write_pack_goldens -- --ignored
cargo test -p prism-compile
```
