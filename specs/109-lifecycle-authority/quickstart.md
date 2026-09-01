# Quickstart: Validate Crash-Safe Lifecycle Authority

## Prerequisites

- Rust 1.88 or newer.
- No game, account, Internet access, capture driver, or elevation for portable tests.
- A supported Windows host for the platform recovery and restart tier.

## Focused Validation

```sh
cargo test -p fragcap --test deep_capture_routing
cargo test -p fragcap --test deep_capture_journal
cargo test -p fragcap --test deep_capture_lifecycle
cargo test -p fragcap-cli --test cli_deep_capture
```

Expected outcomes:

- Only the child-environment strategy applies; every future strategy refuses before effects.
- Every controlled kill boundary leaves a parseable obligation or a safe explicit refusal.
- Repeated recovery changes no already-terminal resource.
- Proxy and cleanup prefixes remain readable without trailers.
- Final sidecars, journal, summary, manifest, application stream, and terminal report reconcile.
- Identity churn reaches bounded overflow accounting without increasing the localized map.

## Repository Gates

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all --locked
cargo xtask ci
cargo xtask msrv
cargo xtask deps
```

On Windows, also run the platform workflow-equivalent tests that exercise current-user trust identity checks and machine-restart journal replay.
