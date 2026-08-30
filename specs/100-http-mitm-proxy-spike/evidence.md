# S100 smaller native proxy fallback evidence

**Run date**: 2026-08-30

**Decision**: Retain external `mitmdump` as the Deep Capture backend. Do not add either native candidate to the product graph and do not open another speculative backend path.

## Environment and boundary

- Windows 11 build 26200.9168, x86_64
- `rustc 1.96.0`, `cargo 1.96.0`, `cargo-deny 0.20.2`
- minimum-toolchain trial: Cargo 1.82.0
- candidate: exact `http-mitm-proxy 0.18.0`, defaults disabled, `native-tls-client`
- comparison evidence: S099 exact `hudsucker 0.23.0` and installed `mitmdump 12.2.3`
- controlled inputs: 25-byte request, 26-byte response, and 22-byte WebSocket message on IPv4 loopback
- no system proxy change, trust-store mutation, remote service, target process, validation bypass, or retained private key

The executable and candidate-only audit are nested workspaces under `spikes/http-mitm-proxy`. HTTPS and HTTP/2 exercise a real client-facing TLS CONNECT session. The local upstream is deliberately cleartext because `DefaultClient` exposes no connector injection for a session-private origin CA; this avoids system trust mutation and is recorded as a limitation rather than hidden.

## Controlled protocol results

| Proof point | Smaller fallback | S099 `hudsucker` | S099 `mitmdump` | Evidence |
| --- | --- | --- | --- | --- |
| Loopback and explicit scope | Pass | Pass | Pass | Harness-owned listeners and clients only. |
| HTTP/1.1 request body | Pass | Pass | Pass | All observed 25 bytes, SHA-256 `07aae526...`. |
| HTTP/1.1 response body | Pass | Pass | Pass | All observed 26 bytes, SHA-256 `f93615f3...`. |
| HTTPS through CONNECT | Pass | Pass | Pass | Fallback client, request, and response rows complete. |
| HTTP/2 through CONNECT | Partial | Pass | Failed measurement | Fallback client-facing request is HTTP/2 and complete; service-to-origin response is HTTP/1.1. S099 retained the same downgrade for `hudsucker`; its forced baseline attempt did not reach the addon. |
| WebSocket handshake | Pass | Partial | Pass | Fallback exposes both empty handshake messages; S099 `hudsucker` missed the response hook. |
| WebSocket messages | Partial | Pass | Pass | Fallback preserves the 22-byte echo but exposes only raw upgraded streams. Message parsing and task ownership would be application code. |
| HAR source | Pass | Pass | Pass | Public messages expose method, URI, versions, headers, status, and complete fixed bodies. |
| HAR output | Unsupported | Unsupported | Pass | Both native candidates require fragcap-owned HAR generation. |
| Client-facing TLS key log | Unsupported | Pass | Pass | Fallback constructs the client-facing rustls server configuration privately and has no public key-log hook. |
| CA and trust separation | Pass | Pass | Pass | The fallback CA was supplied only to controlled clients. |
| Certificate cache | Pass | Partial | Not measured | Caller-owned fallback cache is capacity 32 and reached one observed entry. |
| Shutdown and cleanup | Partial | Pass | Partial | Fallback released its listener in 10 of 10 trials, but internally spawned accepted and CONNECT tasks have no public drain or join handle. |

Missing directional WebSocket rows remain `not-measured` in normalized comparison output. No unsupported, failed, partial, or absent row counts as parity. Complete payload parity requires the same protocol, length, and digest.

## Dependency, license, and toolchain audit

The minimal locked audit graph contains 155 metadata packages and 103 unique active normal-tree renderings, including the local audit package. All sources are crates.io or local. The exact candidate is MIT licensed, declares edition 2024, and declares no `rust-version`.

`cargo deny` reports advisories, licenses, bans, and sources as passing. Duplicate warnings remain for `getrandom` and `syn`; two unused license allowances are warnings. The selected normal and all-target graphs contain no `webpki-roots` package. `native-tls 0.2.18` is active through both the candidate and `tokio-native-tls`.

| Measurement | Result |
| --- | --- |
| Cargo 1.82 metadata/parse | Fail before compilation: `zeroize 1.9.0` requires edition-2024 Cargo support. The candidate manifest itself also declares edition 2024. |
| Cargo 1.82 check/build | Not measured because resolution cannot parse. |
| Rust/Cargo 1.96 check/build | Pass with the exact locked graph. |
| Clean debug build | 15.788 seconds. |
| Warm debug build | 0.249 seconds. |
| Isolated target directory | 501,039,597 bytes. |

The candidate is smaller than S099's 210-package minimal `hudsucker` audit graph, but size does not repair its Rust 1.82 failure or missing public key-log and connection-lifecycle hooks.

## Product isolation

The released root manifest and lock have no diff, root metadata contains no `http-mitm-proxy`, and their SHA-256 values match S099's pre-spike values:

| Artifact | SHA-256 |
| --- | --- |
| `Cargo.toml` | `81bffb391705f2544ed59c5cacfba99be47e1cace6f92f8ca590478fcb0d93db` |
| `Cargo.lock` | `d0b8cb9ad7eadabd2d0f6e76d496b50ebd8fe6040d6cb402a5bd795d1f9fcf9b` |

## Acceptance decision table

| Criterion | Result | Deciding note |
| --- | --- | --- |
| Windows loopback lifecycle | Partial | Listener cancellation passed 10 of 10; active connection tasks cannot be joined through public API. |
| Explicit scoped proxying | Pass | No ambient proxy configuration changed. |
| HTTP/1.1 fidelity | Pass | Request and response lengths and digests match S099. |
| HTTPS CONNECT | Pass | Complete client-facing exchange. |
| HTTP/2 CONNECT | Partial | Client-facing request is HTTP/2; upstream response is HTTP/1.1. |
| WebSocket visibility | Partial | Handshake and client echo pass; message-level proxy observations require a parser and unowned tasks. |
| HAR-source adequacy | Pass | Public service messages contain the required source fields. |
| CA/trust separation | Pass | No operating-system trust mutation. |
| Certificate cache control | Pass | Caller owns a 32-entry cache and can inspect its entry count. |
| Proxy-owned key logging | Fail | Required client-facing configuration is private with no public hook. |
| License, advisory, and source policy | Pass | Exact active graph passes and contains no bundled root-store package. |
| Rust 1.82 and maintenance fit | Fail | Advisory-clean locked resolution is not parseable by Cargo 1.82. |

## Decision rationale

`http-mitm-proxy 0.18.0` is materially smaller than `hudsucker 0.23.0` and demonstrates the principal HTTP, HTTPS, client-facing HTTP/2, bounded-cache, and HAR-source paths. It does not close the S099 blocker. Its current graph is not parseable by Cargo 1.82, and two required ownership surfaces are absent: client-facing TLS key logging and bounded joining of internally spawned connection tasks. WebSocket message inspection would also require fragcap-owned framing over raw streams.

`hudsucker` remains the stronger native functional fit but failed the same repository toolchain policy with a larger graph. Maintaining a fork or compatibility-constrained proxy stack is disproportionate to the current alpha path. The selected outcome is therefore the already shipped external `mitmdump` adapter. S100 closes the speculative backend search; future work implements and hardens the defined external backend rather than opening another candidate spike.

## Reproduction commands

```text
cargo fmt --manifest-path spikes/http-mitm-proxy/Cargo.toml -- --check
cargo clippy --manifest-path spikes/http-mitm-proxy/Cargo.toml --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path spikes/http-mitm-proxy/Cargo.toml --all --locked
cargo run --manifest-path spikes/http-mitm-proxy/Cargo.toml --locked -- candidate
cargo metadata --manifest-path spikes/http-mitm-proxy/audit/Cargo.toml --locked --format-version 1
cargo tree --manifest-path spikes/http-mitm-proxy/audit/Cargo.toml --locked --target all --edges normal
cargo deny --manifest-path spikes/http-mitm-proxy/audit/Cargo.toml --config spikes/http-mitm-proxy/deny.toml check
rustup run 1.82 cargo check --manifest-path spikes/http-mitm-proxy/audit/Cargo.toml --locked
cargo build --manifest-path spikes/http-mitm-proxy/audit/Cargo.toml --locked
```

## Source notes

- Candidate crate metadata and feature definitions: <https://docs.rs/crate/http-mitm-proxy/0.18.0>
- Public proxy and cache API: <https://docs.rs/http-mitm-proxy/0.18.0/http_mitm_proxy/struct.MitmProxy.html>
- Public forwarding and upgraded-stream API: <https://docs.rs/http-mitm-proxy/0.18.0/http_mitm_proxy/default_client/struct.DefaultClient.html>
- Versioned upstream source and examples: <https://github.com/hatoo/http-mitm-proxy/tree/v0.18.0>
