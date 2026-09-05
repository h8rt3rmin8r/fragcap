# Quickstart: Native Deep Capture Performance Envelope

## Prerequisites

- Rust 1.96.0 selected by `rust-toolchain.toml`.
- No game, capture driver, elevation, trust mutation, or external network.
- Loopback TCP and UDP sockets available.

## Validate the frozen contract

```text
cargo xtask performance
cargo test -p xtask performance
```

Expected: schema version 1, fourteen unique cases, both campaign profiles, attributed executable evidence, and workflow coverage pass.

## Run focused runtime accounting tests

```text
cargo test -p fragcap-proxy runtime::tests
cargo test -p fragcap-proxy certificate::tests
cargo test -p fragcap --features deep-capture deep_capture::application::tests
```

Expected: failure detail, task, queue, and cache gauges remain within declared capacity, every overflow is counted, and terminal ownership reaches zero.

## Run the short performance campaign

```text
cargo run --release --locked --manifest-path performance/native-proxy/Cargo.toml -- --profile short --output target/performance/short.jsonl
```

Expected: every protocol and retention row completes through the production proxy, all hard invariants pass, timing rows meet their predeclared thresholds or use at most one guard-band retry, and one complete campaign terminal is written.

## Validate a generated report

```text
cargo xtask performance --report target/performance/short.jsonl
cargo xtask docs check
```

Expected: the report digest, environment class, sequences, case inventory, metrics, and terminal reconciliation validate.

## Run the genuine soak profile

```text
cargo run --release --locked --manifest-path performance/native-proxy/Cargo.toml -- --profile soak --output target/performance/soak.jsonl
```

Expected after at least two wall-clock hours: periodic samples cover repeated complete protocol and lifecycle churn; memory span, task/cache/queue bounds, conservation, disk growth, and every shutdown pass; one complete soak terminal is written. Deterministic product tests separately exercise overload and saturation paths without turning timing noise into a loss-accounting oracle.

## Full repository gate

```text
cargo xtask ci
git diff --check
```

The full gate includes static performance-contract validation. The dedicated performance workflow owns release-mode short measurement, while scheduled or explicitly dispatched automation owns multi-hour evidence.
