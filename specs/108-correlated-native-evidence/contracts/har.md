# Contract: Truthful HAR 1.2

- `log.entries` contains only transactions with every mandatory standard value observed.
- `_fragcapPartialEntries` preserves every other transaction with exact missing, error, truncation, and loss reasons.
- `_fragcap` carries provenance, correlation, and body limitations on standard entries.
- Binary retained content uses base64. Missing bytes are never reconstructed.
- Staging is bounded and atomically published only after source and output validation.
