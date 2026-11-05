# Quickstart: Validate S194

## Prerequisites

- Rust 1.88 toolchain.
- Repository dependencies from the locked workspace.
- No game, capture driver, installed public package, or privileged trust change is required for controlled source checks.

## Focused validation

```powershell
cargo test -p fragcap-targets compatibility
cargo test -p fragcap-targets calibration_workflow
cargo test -p fragcap deep_capture::proposal
cargo test -p fragcap-cli calibrate
cargo test -p fragcap-cli deep_capture
cargo test -p fragcap-cli doctor
```

Expected outcomes:

- Schema 13 fixtures migrate to schema 14 without changing prior values.
- Legacy facts are visible and inapplicable to exact game scope.
- Launcher-mediated `auto` proposes game scope without creating a fact.
- Explicit launch-chain remains selectable.
- Resume preserves scope and fallback intent.
- One eligible linked retry or one typed refusal is produced.
- Packet fallback reports Capture mode and its exact window without positive Deep Capture facts.
- Human, JSON, targets, Doctor, and artifacts agree.

## Repository validation

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all --locked
cargo run --package xtask -- conformance
cargo run --package xtask -- lint
cargo run --package xtask -- spec
cargo run --package xtask -- docs check
cargo run --package xtask -- package-certification
```

The PR must finish exact-head CI with all applicable jobs successful and no unresolved hosted review finding. Installed-product behavior is assessed after release.
