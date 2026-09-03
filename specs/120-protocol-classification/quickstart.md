# Quickstart: Validate Exhaustive Protocol Classification

## Prerequisites

- Work on `codex/120-protocol-classification`.
- `.specify/feature.json` points to `specs/120-protocol-classification` and remains unstaged.
- Use only controlled loopback protocol evidence and synthetic failure injection.

## Focused Validation

```sh
cargo test -p fragcap --test protocol_classification
cargo test -p fragcap --test application_stream
cargo test -p fragcap --test deep_capture_session
cargo test -p fragcap-cli --test cli_deep_capture
```

Expected results:

- Every published traffic family maps to one valid schema version 1 classification.
- Unknown, unsupported, and failed remain distinct.
- Parser, retention, and writer failures retain separate authority.
- Compatibility policy proposes no positive fact from insufficient evidence.
- Application, manifest, human, and JSON summaries reconcile exactly.

## Full Gate

```sh
cargo xtask ci
```

Review the final diff for issue #316 only. Confirm no routing or calibration expansion, process access, target key extraction, pinning bypass, global proxy mutation, invented observation, Unicode dash punctuation, BOM, mojibake, or staged `.specify/feature.json`.
