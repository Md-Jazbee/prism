# Language golden fixtures (P1 Stage A)

Small hand-authored snippets with expected `FactBundle` JSON for extractor conformance.

| Language | Source | Expected |
|---|---|---|
| Python | `python/simple_module.py` | `python/expected.json` |
| Rust | `rust/simple_mod.rs` | `rust/expected.json` |
| Java | `java/simple_class.java` | `java/expected.json` |
| Perl | `perl/simple_module.pl` | `perl/expected.json` |

Each extractor crate runs a unit test that extracts the source and asserts equality against the golden JSON (after `FactBundle::normalize()`).

To regenerate after intentional extractor changes:

```bash
cargo test -p prism-extract-python golden_simple_module -- --nocapture
# or re-run the extract helpers and overwrite expected.json
```

Do not edit `expected.json` by hand unless you understand span/byte shifts.
