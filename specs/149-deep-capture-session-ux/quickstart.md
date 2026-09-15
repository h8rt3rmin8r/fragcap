# Controlled Validation: S149

## Preconditions

Use the repository Rust toolchain and default offline features. No Npcap installation, elevation, trust mutation, live capture, or game is required. Follow the repository hidden-process execution rules on Windows.

## Validation

```text
cargo test -p fragcap-cli --lib session_ux --locked
cargo test -p fragcap-cli --test cli_deep_capture --locked
cargo test -p fragcap-cli --features etw --lib live_status --locked
cargo xtask ci
```

Pure tests cover exact consequence selection, typed lifecycle, counters, terminal state, retained evidence, residue, output modes, and narrow Unicode text. Controlled command tests exercise the production authorization and event adapters. Existing first-run help and calibration tests supply journey evidence. [acceptance.md](acceptance.md) records exact passing references after verification.
