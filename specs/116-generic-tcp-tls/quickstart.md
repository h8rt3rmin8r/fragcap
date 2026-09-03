# Quickstart: Generic TCP And Non-HTTP TLS Evidence

## Verification

```sh
cargo test -p fragcap-proxy generic_stream
cargo test -p fragcap-proxy --test socks5_proxy
cargo test -p fragcap --test application_stream
cargo xtask ci
```

## Expected Evidence

- Plain SOCKS5 TCP emits `generic.stream_chunk` with `tcp-plaintext` provenance.
- SOCKS5 TLS remains byte-transparent and emits `tls-encrypted` provenance.
- Trusted no-ALPN CONNECT TLS emits `tls-decrypted` provenance after both TLS negotiation records.
- HTTP ALPN and recognizable no-ALPN HTTP retain their existing HTTP records.
- Disabled or exhausted retention emits lengths and outcomes without payload fields.
