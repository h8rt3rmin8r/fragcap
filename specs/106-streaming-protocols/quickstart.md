# S106 Verification Quickstart

```powershell
cargo test -p fragcap-proxy
cargo test -p fragcap --test application_stream
cargo test -p fragcap --test native_proxy
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo xtask ci
```

All protocol tests use controlled loopback peers and require no Internet access, elevation, trust-store mutation, capture driver, or game account.
