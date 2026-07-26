# Slice fixtures (P4 Stage A)

Property tests live in `prism-semantic` crate tests:

- criterion line is always covered by the slice
- re-slice is idempotent
- broken syntax does not panic

Python sample: `python/sample.py` — criterion around `return y` in `bug`.
