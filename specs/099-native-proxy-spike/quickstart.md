# Quickstart: Native Proxy Backend Spike

## Preconditions

- Windows 11 x86_64 with Rust 1.96.0 and Rust 1.82 installed.
- `mitmdump` available on `PATH` for baseline comparison.
- `cargo-deny` available for the isolated license audit.
- No game, capture driver, elevation, system proxy change, or trust-store change is required.

## Validate the Isolated Harness

```powershell
cargo test --manifest-path spikes/native-proxy/Cargo.toml --locked
cargo run --manifest-path spikes/native-proxy/Cargo.toml --locked -- compare
```

Expected: both backends receive the same local matrix, an ephemeral directory contains private run material, and stdout contains only sanitized normalized evidence. Missing capabilities appear as explicit negative or inconclusive results.

## Audit the Candidate Graph

```powershell
cargo metadata --manifest-path spikes/native-proxy/Cargo.toml --locked --format-version 1
cargo tree --manifest-path spikes/native-proxy/Cargo.toml --locked --edges normal
cargo tree --manifest-path spikes/native-proxy/Cargo.toml --locked --target all
cargo deny --manifest-path spikes/native-proxy/Cargo.toml check
rustup run 1.82 cargo check --manifest-path spikes/native-proxy/Cargo.toml --locked
```

Expected: the audit accounts for every resolved and target-conditional package. A policy or Rust 1.82 failure is recorded as evidence and is not bypassed.

## Prove Product Isolation

```powershell
cargo metadata --locked --no-deps --format-version 1
git diff -- Cargo.toml Cargo.lock crates
```

Expected: no product manifest or root lock change and no `hudsucker` package in root workspace metadata.

## Full Repository Verification

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all --locked
cargo xtask ci
```

Expected: all product gates remain green after the non-shipping research artifacts are added.
