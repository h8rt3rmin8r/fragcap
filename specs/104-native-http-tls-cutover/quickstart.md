# S104 Quickstart Validation

## Prerequisites

- Rust toolchain from `rust-toolchain.toml`.
- No Python, mitmdump, capture driver, game account, remote origin, or real trust-store mutation is required for the controlled scenarios.
- Windows-only trust effects are exercised through injected stores unless a separately marked manual check is run.

## Focused protocol validation

```powershell
cargo test -p fragcap-proxy --test authentication --locked
cargo test -p fragcap-proxy --test http1_proxy --locked
cargo test -p fragcap-proxy --test https_proxy --locked
cargo test -p fragcap-proxy --test lifecycle --locked
```

Expected: standard proxy authentication admits only the matching session; HTTP methods, framing, informational responses, CONNECT, TLS 1.2/1.3, validation failures, timeouts, cancellation, half-close, and ten cleanup cycles pass with reconciled accounting.

## Facade and CLI cutover validation

```powershell
cargo test -p fragcap --test native_proxy --locked
cargo test -p fragcap --test deep_capture_session --locked
cargo test -p fragcap-cli --test cli_deep_capture --locked
cargo test -p fragcap-cli --test cli_doctor --locked
```

Expected: public-library and CLI controlled sessions select `fragcap-native`, child-only routing receives a redacted session credential, trust uses the proxy's exact public authority, and no external proxy child exists.

## Source and dependency gates

```powershell
cargo xtask lint
cargo xtask deps
cargo deny check bans licenses sources
```

Expected: the production source gate rejects external proxy commands, embedded Python, and mitmdump paths; direct `base64` and `httparse` edges add no new lock package or license.

## Full CI parity

```powershell
cargo xtask ci
```

Expected: formatting, clippy, tests, dependency direction, specification lock-step, MSRV, package, documentation, encoding, mojibake, and platform checks all pass.
