# Quickstart: Validate Authenticated SOCKS5 TCP Routing

## Prerequisites

- Rust 1.88 or newer
- No capture driver, elevation, game, account, or Internet connection

## Focused Validation

```powershell
cargo test -p fragcap-proxy --test socks5_proxy --locked
cargo test -p fragcap --test deep_capture_routing --locked
cargo test -p fragcap-cli --test cli_deep_capture --locked
```

Expected: the controlled listener passes IPv4, IPv6-available, domain, authentication, malformed, refusal, timeout, half-close, cancellation, backpressure, classification, route, evidence, and conservation cases.

## Full Gate

```powershell
cargo xtask ci
```

Expected: all repository, documentation, dependency, MSRV, platform-neutral, security primitive, protocol, and test gates pass.
