# Quickstart: Native Deep Capture Threat Model

Run the focused validator and its unit tests:

```text
cargo test -p xtask threat_model
cargo xtask threat-model
```

Run focused abuse suites:

```text
cargo test -p fragcap-proxy --test authentication
cargo test -p fragcap-proxy --test upstream
cargo test -p fragcap-proxy --test http1_proxy
cargo test -p fragcap-proxy --test socks5_proxy
cargo test -p fragcap-proxy --test socks5_udp
cargo test -p fragcap-proxy --test quic_http3
cargo test -p fragcap-proxy --test certificates
cargo test -p fragcap-cli --test cli_bundle
cargo test -p fragcap-cli --test cli_doctor
```

Run the complete repository gate:

```text
cargo xtask ci
```

Expected result: the threat registry is complete, all executable references and
review inventories match, focused abuse tests pass, and the full gate is green.
