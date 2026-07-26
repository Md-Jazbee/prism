# Slice fixtures (P4)

Property tests live in `prism-semantic` crate tests:

- criterion line is always covered by the slice
- re-slice is idempotent
- broken syntax does not panic
- inter-proc chain includes callers; memo hits on repeat
- dirty paths invalidate shards

Python sample: `python/sample.py` — criterion around `return y` in `bug`.
