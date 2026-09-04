# Quickstart: Validate Native Deep Capture Failure Injection

## Prerequisites

- Run from the repository root.
- Use the pinned Rust toolchain.
- No game, account, Internet connection, capture driver, elevation, or trust mutation is required.

## Validate the closed registry and generated matrix

```sh
cargo xtask failure-matrix
```

Expected: schema version, boundary count, generated scenario count, failure-family count, and executable evidence count are printed with no findings.

## Execute production-coordinator failure scenarios

```sh
cargo test -p fragcap --test deep_capture_session --features deep-capture
cargo test -p fragcap --test deep_capture_journal --features deep-capture
```

Expected: each generated scenario names its injection point on failure; before-side rows call no owned effect, after-side rows retain cleanup or recovery authority, and every outcome dimension matches independently.

## Validate registry rejection behavior

```sh
cargo test -p xtask failure_matrix
```

Expected: controlled malformed registries, missing sides, absent failure families, production drift, incomplete outcomes, and invalid test references are rejected.

## Run the complete merge gate

```sh
cargo xtask ci
```

Expected: formatting, Clippy, workspace tests, lint, dependency, license, wrapper, documentation, specification, threat-model, fuzz, failure-matrix, and conformance gates all pass.
