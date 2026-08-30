# S102 Research: Native Deep Capture Proxy Foundation

## Decision 1: Resolve five ordered foundation issues together

S102 resolves #279, #280, #281, #282, and #291. The architecture contract unlocks the toolchain policy; that policy unlocks the crate; the crate unlocks bounded runtime ownership; and the public status must change with the architecture. Splitting these across separate CI and review cycles would create intermediate states that either select an unused graph, expose a backend with unsafe lifecycle ownership, or continue misleading public documentation.

Protocol forwarding, TLS interception, certificate lifecycle, trust changes, application events, and CLI cutover remain deferred to their dependent issues. The native listener in S102 proves resource ownership only and never reports application inspectability.

## Decision 2: Supersede S100's end state

S100 correctly concluded that neither measured turnkey candidate was acceptable under the then-current Rust 1.82 and dependency constraints. It also closed further speculative candidate work and retained external mitmdump as the shipped backend.

The operator subsequently made native Rust completion an explicit product requirement and created #278 as the completion authority. S102 therefore supersedes only S100's end-state decision. It preserves the measurements and rejects the same unsafe unbounded-task shape, but replaces candidate shopping with an owned custom stack and a higher measured MSRV.

## Decision 3: Raise the workspace MSRV to Rust 1.88

The native graph is part of the workspace claim, not an exception hidden behind a feature. `rcgen` 0.14.10 and its selected `time` graph require Rust 1.88. Maintaining Rust 1.82 would require old certificate and transitive pins, recreating the narrow compatibility-patch burden #280 exists to eliminate.

Rust 1.88 parses the committed version 4 lockfile and builds the full workspace graph. The repository development toolchain remains pinned to Rust 1.96.0. CI and local `xtask msrv` checks must use 1.88.0.

## Decision 4: Own an exact, minimal-feature protocol stack

The selected direct graph for `fragcap-proxy` is:

| Dependency | Exact version | Features | Purpose |
| --- | --- | --- | --- |
| `tokio` | 1.53.1 | `net`, `rt-multi-thread`, `io-util`, `macros`, `sync`, `time` | Bounded listener, task, cancellation, and deadline ownership |
| `hyper` | 1.11.1 | `client`, `server`, `http1`, `http2` | Future HTTP protocol engine |
| `hyper-util` | 0.1.20 | `tokio`, `server-auto` | Future Tokio/Hyper adapters |
| `http-body-util` | 0.1.5 | none | Future bounded HTTP body composition |
| `rustls` | 0.23.43 | `std`, `ring`, `tls12` | Future TLS engine with an explicit provider |
| `tokio-rustls` | 0.26.4 | `ring`, `tls12` | Future async TLS adapter |
| `rcgen` | 0.14.10 | `ring` | Future in-process CA and leaf issuance |
| `rustls-native-certs` | 0.8.4 | none | Future Windows root loading through Schannel |

Every entry is exact-pinned with default features disabled. S102 uses Tokio for the foundation runtime. The remaining packages are locked and compiled at the product feature boundary so Windows release artifacts exercise the selected graph before later protocol work relies on it.

## Decision 5: Select one cryptography provider

Rustls, Tokio Rustls, and rcgen use `ring` only. Default features are disabled to prevent AWS-LC and rustls's default post-quantum preference from silently creating a second provider. Ring builds bundled C and pregenerated assembly with the MSVC toolchain and requires no OpenSSL DLL, CMake, Perl, NASM, or runtime redistributable.

FIPS and post-quantum modes are not Deep Capture requirements. Changing provider later requires a new decision record and full dependency, license, advisory, MSRV, and Windows packaging audit.

## Decision 6: Load Windows roots without bundled root data

Future upstream TLS verification will load Windows roots with `rustls-native-certs`, which uses Schannel on Windows, then apply rustls/webpki verification semantics to that snapshot. This does not promise the Windows chain engine's complete policy behavior.

`webpki-roots` and `rustls-platform-verifier` were rejected because their all-target graphs include root data under CDLA-Permissive-2.0, outside the repository allowlist. This choice avoids OpenSSL and bundled root stores.

## Decision 7: Accept measured multi-version platform bindings

The graph contains multiple `windows-sys` lines: the existing 0.36 platform/pcap binding, ring's 0.52 line, and Tokio/Schannel's 0.61 line. Cargo deny reports this as a warning, not a violation. Forcing ABI-binding upgrades into S102 would enlarge scope without reducing runtime duplication that independent upstream crates control.

## Decision 8: Exact pins and deliberate upgrades

The direct graph is exact-pinned and `Cargo.lock` remains committed. Upgrade one direct package at a time. Each upgrade must run Rust 1.88 locked metadata/build, all-feature clippy and tests, dependency direction, license and advisory checks, package verification, and the Windows release build. RustSec advisories are reviewed immediately; otherwise the graph receives a quarterly review.

## Decision 9: No hidden routing or premature protocol claims

The foundation binds only an explicit IPv4 or IPv6 loopback address. It does not mutate system proxy settings, inspect target memory, install a driver, issue certificates, change trust, parse HTTP, forward bytes upstream, or claim traffic inspectability. Accepted sockets exist only to exercise finite ownership and are closed under the runtime's bounded policy.

## Evidence

- Official crate metadata reports Tokio 1.53.1 MSRV 1.71/MIT, Hyper 1.11.1 MSRV 1.63/MIT, rustls 0.23.43 MSRV 1.71/Apache-2.0 OR ISC OR MIT, Tokio Rustls 0.26.4 MSRV 1.71/MIT OR Apache-2.0, rcgen 0.14.10 MSRV 1.88/MIT OR Apache-2.0, and rustls-native-certs 0.8.4 MSRV 1.71/Apache-2.0 OR ISC OR MIT.
- The exact locked graph built on `x86_64-pc-windows-msvc` with Rust 1.88.0 and 1.96.0.
- Cargo deny found no advisory, ban, source, or dependency-license violation in the graph.
- The isolated graph contained 102 registry packages across all target and optional lock entries; 54 were reachable in the Windows normal/build graph.
