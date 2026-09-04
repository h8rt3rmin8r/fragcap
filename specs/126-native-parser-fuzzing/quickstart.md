# Quickstart: Native Deep Capture Parser Fuzzing

Validate the registry and replay all seeds on stable Rust:

```text
cargo xtask fuzz
cargo test -p fragcap --test fuzz_seeds --features deep-capture
```

Install the exact engine and build every target:

```text
rustup toolchain install nightly-2026-08-25
cargo +1.96.0 install cargo-fuzz --version 0.13.2 --locked
cargo +nightly-2026-08-25 fuzz build
```

Run the bounded smoke profile for one target:

```text
cargo +nightly-2026-08-25 fuzz run TARGET -- -runs=256 -timeout=5 -max_len=65536
```

Run the complete repository gate:

```text
cargo xtask ci
```

Expected result: every registered surface, target, seed, stable replay, and CI
entry agrees; stable replay passes; every libFuzzer target builds and completes
without a finding; and the full repository gate is green.
