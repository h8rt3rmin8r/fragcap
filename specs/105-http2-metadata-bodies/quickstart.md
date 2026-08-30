# S105 Verification Quickstart

## Portable gate

```powershell
cargo fmt --all -- --check
cargo test -p fragcap-proxy
cargo test -p fragcap
cargo test -p fragcap-cli
cargo xtask lint
cargo xtask deps
cargo xtask docs
```

## Controlled protocol lab

The local lab must demonstrate:

1. One TLS HTTP/2 connection with at least 32 overlapping streams, out-of-order completion, one reset, trailers, and a GOAWAY boundary.
2. Exact client-to-origin stream pairing and exactly one terminal outcome for every accepted stream and connection.
3. HTTP/1.1 exact metadata order and casing plus HTTP/2 typed pseudo-fields, duplicate binary values, and explicit unavailable compressed order.
4. Fixed, chunked, connection-delimited, long-lived, cancelled, gzip, zlib-deflate, Brotli, malformed, and expansion-limited bodies.
5. Byte-identical forwarding for every non-refused exchange while observation retention truncates or drops under its own limits.
6. Metadata-only scope emitting no payload bytes and recording intentional omission.
7. Application records visible before session completion, one reconciling trailer after orderly completion, and a readable incomplete prefix after forced interruption.
8. Queue saturation and injected writer failure that preserve forwarding, retain prior complete lines, count loss, and never produce a complete trailer.
9. Ten repeated start, traffic, stop, and cleanup cycles with no owned task residue.
10. Existing S104 HTTP/1.1 and HTTPS cases unchanged.

No case may require Internet access, a capture driver, elevation, a game account, or a persistent trust-store mutation.
