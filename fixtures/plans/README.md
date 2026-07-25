# Plan golden fixtures (P2 Stage A)

Deterministic `PlanOutcome` JSON for `prism-plan` conformance.

| Case | Intent / outcome | Input |
|---|---|---|
| `repo_qa` | repo_qa plan | named symbol question |
| `debug` | debug plan (placeholders) | stack + error hints |
| `impact` | impact plan | “impact of …” |
| `refactor` | refactor plan | rename / references |
| `generate` | generate plan | generate near symbol |
| `review` | review plan | PR + changed_paths |
| `architecture` | architecture plan | overview (no anchors) |
| `ambiguous` | `SCOPE_UNRESOLVED` | “tell me about the code” |

Each case has `input.json` + `expected.json`. Crate tests assert equality after `Plan::normalize()`.

Regenerate after intentional planner changes:

```bash
UPDATE_GOLDENS=1 cargo test -p prism-plan write_plan_goldens -- --ignored
cargo test -p prism-plan
```

Do not hand-edit `expected.json` unless you understand plan_id / step shape changes.
