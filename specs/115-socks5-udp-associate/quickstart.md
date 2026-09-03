# Quickstart: Validate Scoped SOCKS5 UDP Association

```powershell
cargo test -p fragcap-proxy --test socks5_udp
cargo test -p fragcap-proxy --test socks5_proxy
cargo test -p fragcap-proxy
cargo test -p fragcap --features deep-capture
cargo xtask ci
git diff --check
```

Confirm `Cargo.lock` has no S115 delta, every requirements and security checklist item is checked, all task boxes are complete, and UTF-8/mojibake checks pass through the repository gate.
