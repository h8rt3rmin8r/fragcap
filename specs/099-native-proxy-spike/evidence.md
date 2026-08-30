# S099 native proxy backend spike evidence

**Run date**: 2026-08-29

**Decision**: Evaluate the smaller native fallback. Do not add `hudsucker`
to the product graph.

## Environment and boundary

- Windows 11 build 26200.9168, x86_64
- `rustc 1.96.0`, `cargo 1.96.0`
- minimum-toolchain trial: `rustc 1.82.0`
- candidate: `hudsucker 0.23.0`, exact, with defaults disabled and
  `decoder,http2,native-tls-client,rcgen-ca`
- baseline: installed `mitmdump 12.2.3 binary`
- controlled inputs: 25-byte request, 26-byte response, and 22-byte
  WebSocket message generated on IPv4 loopback
- no system proxy change, operating-system trust mutation, remote service,
  target process, or retained private key was used

The executable harness is a nested workspace under `spikes/native-proxy`.
The candidate-only audit has a second manifest under
`spikes/native-proxy/audit`. This is an intentional correction to the
initial plan: client, server, serialization, and test dependencies needed by
the harness are not part of a product adoption delta.

## Controlled protocol results

| Proof point | Native candidate | External baseline | Evidence |
| --- | --- | --- | --- |
| Loopback and explicit proxy scope | Pass | Pass | Both listeners used IPv4 loopback and only harness-owned clients. |
| HTTP/1.1 request body | Pass | Pass | Both observed all 25 bytes with SHA-256 `07aae526...`. |
| HTTP/1.1 response body | Pass | Pass | Both observed all 26 bytes with SHA-256 `f93615f3...`. |
| HTTPS through CONNECT | Pass | Pass | Request, response, and client result were complete. |
| HTTP/2 through CONNECT | Pass | Fail | Candidate observed client-facing HTTP/2 request and response bodies; its upstream response was HTTP/1.1. The same forced HTTP/2 client attempt failed before the baseline addon received an HTTP/2 flow, so this run does not establish baseline parity. |
| WebSocket handshake | Pass | Pass | Each backend exposed the empty upgrade request and response separately from message bodies. |
| WebSocket messages | Pass | Pass | Both directions and the client echo contained all 22 bytes. |
| HAR authority | Pass | Pass | Candidate public handlers exposed method, URI, versions, headers, status, and complete bodies for fragcap-owned HAR generation. Baseline finalized a HAR from its saved flow. |
| Client-facing TLS key log | Pass | Pass | Candidate public CA wrapper wrote 10 NSS key-log lines; the child-scoped baseline variable wrote 21. |
| CA and trust separation | Pass | Pass | Temporary CA material was supplied directly to controlled clients; neither run installed trust. |
| Certificate cache | Partial | Not measured | Candidate cache was session-owned and bounded at 32 entries. Public APIs expose capacity and tracing, but not enumeration or explicit invalidation. |
| Shutdown and cleanup | Pass | Partial | Candidate completed 10 of 10 bounded graceful shutdown trials. The baseline adapter required bounded forced child termination before offline HAR finalization. |

An empty handshake body is recorded as `empty`, not omitted. A failed or
missing row never counts as parity. The baseline HTTP/2 failure may be a
forced-client interaction rather than a backend limitation; it is retained as
a failed measurement and is not used as a deciding advantage for the
candidate.

## Dependency and toolchain audit

The minimal locked audit graph contains 210 packages: 209 registry packages
and one local audit package. It contains no Git source. `cargo deny` reports
licenses, advisories, bans, and sources as passing for the committed secure
resolution. Duplicate-version warnings remain for `getrandom`, `syn`, and
`windows-sys`.

`webpki-roots 0.26.11` and `1.0.9`, licensed
`CDLA-Permissive-2.0`, appear in all-target metadata, but neither has an
active normal dependency path for the selected Windows native-tls feature
set. They are therefore recorded target-conditional metadata, not treated as
an allowlist pass for a hypothetical rustls-client feature.

The declared candidate minimum is Rust 1.75, but the effective locked graph
does not satisfy fragcap's Rust 1.82 policy:

| Resolution | Rust 1.82 | Advisory audit | Result |
| --- | --- | --- | --- |
| Compatibility-pinned `time 0.3.36` | Pass | Fail | RUSTSEC-2026-0009, fixed in `time >=0.3.47` |
| Secure `time 0.3.47` | Fail before compilation | Pass | `time-core 0.1.8` requires Cargo edition-2024 support |
| Secure `time 0.3.47`, Rust 1.96 | Pass | Pass | Current-toolchain build succeeds |

Reaching the compatibility trial also required pinning eight transitive
selections from the candidate's published lock: `async-compression`,
`async-lock`, `hyper-rustls`, `indexmap` and `hashbrown`,
`jobserver`, `time`, `uuid`, and `zeroize`. A downstream library
consumer does not inherit fragcap's lock file, so a product integration would
need additional direct constraints, a maintained fork, or a toolchain policy
change.

On the secure lock with Rust 1.96, a clean debug build took 16.034 seconds, a
warm build took 0.220 seconds, and the target directory occupied 739,403,571
bytes. Earlier check-only measurements were 15.142 seconds clean and 0.207
seconds warm with a 333,347,128-byte target directory. Timings are one local
Windows run and are comparative evidence, not performance guarantees.

## Product isolation

The root `Cargo.toml` and `Cargo.lock` have no diff. Root Cargo metadata
contains zero `hudsucker` occurrences. Before and after root SHA-256 values
are identical within the measurement ledger:

| Artifact | Before | After |
| --- | --- | --- |
| `Cargo.toml` | `81bffb391705f2544ed59c5cacfba99be47e1cace6f92f8ca590478fcb0d93db` | same |
| `Cargo.lock` | `d0b8cb9ad7eadabd2d0f6e76d496b50ebd8fe6040d6cb402a5bd795d1f9fcf9b` | same |
| HEAD at measurement start | `cde9416ed126f7ece19784eb84bd6e6bf6aa6b11` | same before the S099 commit |

## Twelve-criterion decision table

| Criterion | Result | Deciding note |
| --- | --- | --- |
| Windows loopback lifecycle | Pass | Native graceful cancellation completed 10 of 10 trials. |
| Explicit scoped proxying | Pass | No ambient proxy configuration was changed. |
| HTTP/1.1 fidelity | Pass | Exact request and response lengths and digests matched. |
| HTTPS CONNECT | Pass | Complete controlled exchange. |
| HTTP/2 CONNECT | Pass | Candidate client-facing HTTP/2 was observed; upstream downgraded. |
| WebSocket visibility | Pass | Handshake plus both message directions were observed. |
| HAR-source adequacy | Pass | Public handlers expose required source fields. |
| CA/trust separation | Pass | Session CA never entered an OS trust store. |
| Certificate cache control | Partial | Bounded ownership exists; enumeration and explicit invalidation do not. |
| Proxy-owned key logging | Pass | Implemented through the public CA trait in 10 lines of configuration logic. |
| License and source policy | Pass with caveat | Active Windows graph passes; inactive CDLA root-store metadata is retained. |
| Rust 1.82 and maintenance fit | Fail | No measured resolution is both advisory-clean and parseable by Cargo 1.82; latest `hudsucker 0.25` declares Rust 1.86. |

## Decision rationale

The candidate is functionally capable, including the previously unproven
client-facing key-log path. It is not a sound product dependency under the
current repository policy. The exact older line needs a large graph and
multiple transitive constraints, while its secure resolution misses the Rust
1.82 floor. Selecting a patch or fork would commit fragcap to maintaining a
stale proxy stack before testing the already identified smaller native option.

The one follow-up boundary is therefore
[#274](https://github.com/h8rt3rmin8r/fragcap/issues/274), a non-shipping
`http-mitm-proxy 0.18.0` spike against the same harness and audit contract.
The shipped backend remains external `mitmdump` until that issue reaches its
own evidence-backed decision.

## Reproduction commands

```text
cargo fmt --manifest-path spikes/native-proxy/Cargo.toml -- --check
cargo clippy --manifest-path spikes/native-proxy/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path spikes/native-proxy/Cargo.toml --locked
cargo run --manifest-path spikes/native-proxy/Cargo.toml --locked -- candidate
cargo run --manifest-path spikes/native-proxy/Cargo.toml --locked -- baseline
cargo run --manifest-path spikes/native-proxy/Cargo.toml --locked -- compare
cargo build --manifest-path spikes/native-proxy/audit/Cargo.toml --locked
rustup run 1.82 cargo check --manifest-path spikes/native-proxy/audit/Cargo.toml --locked
cargo deny --manifest-path spikes/native-proxy/audit/Cargo.toml --all-features --locked check
```
