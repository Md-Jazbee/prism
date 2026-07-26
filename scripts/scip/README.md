# scripts/scip — precise index helpers (P3)

Stage A uses hand-authored or converted **PreciseIndex JSON** (`schemas/precise-index/v0`).

Full SCIP protobuf → PreciseIndex converters can land here later. Until then:

1. Author `precise-index.json` using [ID-MAPPING.md](../../docs/architecture/ID-MAPPING.md).
2. Or generate SCIP with a language indexer and map fields manually / with a future script.
3. Import: `prism precise import <file>`.

See [SCIP-RUNBOOK.md](../../docs/architecture/SCIP-RUNBOOK.md).
