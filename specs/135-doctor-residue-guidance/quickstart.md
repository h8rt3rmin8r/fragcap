# Quickstart: Validate Doctor Residue Guidance

## Prerequisites

- Rust 1.88 toolchain from `rust-toolchain.toml`.
- Repository dependencies available from the locked graph.
- No game, capture driver, elevation, real trust mutation, or cleanup effect is required.

## 1. Run focused Doctor tests

```sh
cargo test -p fragcap-cli doctor
cargo test -p fragcap-cli --test cli_doctor
```

Expected: every residue health class has truthful plain-language guidance, the abandoned owner regression is explicit, and action selection matches the existing recoverability facts.

## 2. Inspect human layouts

Run the injected-width report tests for 80 and 40 display columns.

Expected: aligned output is stable at the default width, compact output keeps the status attached at the narrow boundary, ordinary lines fit, exact tokens are not truncated, and plain versus color output has identical visible layout.

## 3. Inspect structured output

```sh
cargo test -p fragcap-cli --test cli_doctor deep_capture_residue
```

Expected: native check records retain all common fields and gain the seven-field `native_resource` object; verdict records remain separate; escaped Unicode and punctuation parse as one JSON record per line.

## 4. Prove cleanup compatibility

```sh
cargo test -p fragcap-cli --test cli_doctor cleanup
```

Expected: the same exact findings offer cleanup, active resources remain protected, and read-only Doctor creates no effects.

## 5. Run repository gates

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all --locked
cargo xtask ci
```

Expected: every command succeeds, Cargo.lock adds no package, privacy and prohibited-capability checks remain green, and specification and workspace versions stay synchronized.
