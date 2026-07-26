# P12 adjudication protocol

**Purpose:** Grade narrative / doc-QA answers so a fluent wrong answer with bad citations scores **zero**.

## Rules

1. **Blind grading.** Judges do not see which arm (explore / Prism / Graphify) produced the answer until after scoring.
2. **Citation validity first.** Every claim that cites a path/span must resolve to a real file and roughly matching content. Invalid citation ⇒ answer score = 0 regardless of fluency.
3. **Accepted-answer criteria.** Score against the gold task’s `accepted_answer_criteria` list (0/1 per bullet), then mean.
4. **Tie-break.** Prefer the answer with higher citation validity, then lower token count.
5. **κ reporting.** Dual review on ≥20 packs; report Cohen’s κ. Gate: κ ≥ 0.6 for the sample used in ACC-7.
6. **Proxies are not the gate.** Scripted `quality_proxy` in `five_arm.py` may report progress; the Phase 12 exit requires live-judged runs.

## Score card fields

| Field | Meaning |
|---|---|
| `quality` | Mean of accepted-answer criteria (0–1) |
| `citation_validity` | Fraction of cited spans that exist and match |
| `tokens` | Input tokens for the arm |
| `hops` | Tool hops before answer |

## Invalid-citation examples

- Citing `fixtures/repos/snapshots/...` when gold forbids it
- Citing a section heading that does not exist in the indexed doc
- Claiming `precise` confidence for an `asserted` doc fact
