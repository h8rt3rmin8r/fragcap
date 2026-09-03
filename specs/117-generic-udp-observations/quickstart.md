# Quickstart: Generic UDP Observations

## Focused Verification

```powershell
cargo test -p fragcap-proxy --test socks5_udp
cargo test -p fragcap-proxy generic_udp
cargo test -p fragcap --test deep_capture_application
```

## Full Verification

```powershell
cargo xtask ci
```

The controlled tests use loopback endpoints and synthetic payloads. They require no game, account, Internet access, elevation, capture driver, or trust mutation.
