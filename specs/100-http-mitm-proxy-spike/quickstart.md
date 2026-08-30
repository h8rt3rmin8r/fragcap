# Quickstart: Smaller Native Proxy Fallback Spike

Prerequisites are Windows 11 x86_64, Rust 1.82, the repository-pinned Rust toolchain, and `cargo-deny`. No command changes the system proxy or trust store.

From `spikes/http-mitm-proxy`:

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all --locked
cargo run --locked -- candidate --output "$env:TEMP\fragcap-s100-candidate.json"
```

Audit the minimal graph:

```powershell
cargo metadata --manifest-path audit/Cargo.toml --locked --format-version 1
cargo tree --manifest-path audit/Cargo.toml --locked --edges normal
cargo tree --manifest-path audit/Cargo.toml --locked --target all --edges normal
cargo deny --manifest-path audit/Cargo.toml --config deny.toml check
rustup run 1.82 cargo check --manifest-path audit/Cargo.toml --locked
rustup run 1.96 cargo check --manifest-path audit/Cargo.toml --locked
```

From the repository root, run `cargo fmt --all -- --check`, full clippy and tests, then `cargo xtask ci`. The root graph must contain no S100 candidate package.
