# Deep Capture proxy backend research

**Status:** research record for issue #214.\
**Date:** 2026-08-24.\
**Audience:** maintainers, reviewers, future Deep Capture slice authors.

This document evaluates proxy backend options for Deep Capture. It does not add a dependency, ship a proxy, or define the final feature specification. Its job is to keep the backend decision from being made by convenience alone.

## Recommendation

Use a staged strategy.

The first native spike should target `hudsucker` rather than bundling Python `mitmproxy` or starting from a lower-level proxy framework. `hudsucker` is shaped like the thing fragcap needs: an embeddable Rust HTTP/S MITM proxy with request, response, CONNECT, TLS interception, certificate authority, HTTP/2, and WebSocket hooks. That makes it the best candidate for a native MVP.

Do not add it to the workspace yet. The spike must first prove five items in isolation:

1. A Windows build passes under the toolchain policy selected for the Deep Capture feature.
2. The selected feature set satisfies the license allowlist without target-conditional root-store surprises.
3. A generated or imported fragcap CA can be used without silent trust changes.
4. HTTP/1.1, HTTP/2, and WebSocket events can be converted into fragcap's structured event stream and later HAR output without changing traffic.
5. Proxy-owned TLS key-log export is possible, or is explicitly deferred from the MVP.

Use external `mitmdump` as the baseline and fallback during research. It is mature, well documented, and already supports the comparison artifacts Deep Capture cares about, including HAR export and TLS key logging for analyzer workflows. It should not become the product path unless the native spike fails on a hard blocker, because freezing Python tooling into the Windows distribution would add packaging and lifecycle complexity that does not fit the rest of fragcap.

The immediate implementation sequence should therefore be:

1. Open a PR-only spike for `hudsucker` in a disposable crate or example outside the shipped workspace graph.
2. Measure Windows behavior, event shape, CA lifecycle control, key-log feasibility, and license/MSRV impact.
3. Keep an external `mitmdump` harness as an oracle for expected HTTP, HTTPS, WebSocket, HAR, and Wireshark-key-log behavior.
4. Decide whether the MVP uses native `hudsucker`, a patched/forked `hudsucker`, a smaller native backend, or external orchestration.

## Decision table

| Candidate | Classification | Why |
| --- | --- | --- |
| `hudsucker` | Viable with caveats | Best API fit for native Deep Capture. Current latest release is Rust 1.86, so either pin an older line, gate the feature outside MSRV, or wait for a policy decision. HAR is not built in. Key-log export requires proof. License audit needs care around root-store packages. |
| `http-mitm-proxy` | Viable with major caveats | Smaller graph and MIT license, with a simple embeddable MITM API. It appears less complete for Deep Capture because WebSocket and richer protocol/event handling are not first-class in the public docs. Keep as a fallback if `hudsucker` fails. |
| `mitmdump` external process | Baseline and fallback | Mature HTTP/S inspection tool, MIT licensed, supports HTTP/1, HTTP/2, WebSockets, HAR export, and SSL key logging. Packaging a Python proxy into fragcap remains a product and distribution compromise. |
| Pingora | Deferred, not an MVP backend | Strong proxy framework, but not a turnkey MITM proxy. Windows support is preliminary, MSRV is currently 1.85, and the graph is large for a first Deep Capture path. Revisit only if fragcap needs a custom high-performance proxy framework after MVP. |
| `mitmproxy_rs` | Rejected for the default design | It is mitmproxy's Rust support code, not the Python product as an embeddable Rust MITM library. Its Windows redirector path is based on WinDivert, which is the wrong posture for fragcap's default target-scoped proxy design. |
| `soth-mitm` | Rejected | The public API shape is attractive, but MPL-2.0 is outside fragcap's allowlist and the crate declares Rust 1.88. |
| `slinger-mitm` | Rejected | GPL-3.0-only is outside fragcap's allowlist. |

## Evaluation criteria

The backend decision is constrained by both product fit and repository posture:

- License compatibility with Apache-2.0 publication and the repository allowlist.
- Rust MSRV and edition compatibility.
- Windows support.
- HTTP CONNECT and explicit-proxy behavior.
- HTTP/1.1, HTTP/2, WebSocket, and future gRPC/SSE headroom.
- Certificate generation, certificate cache behavior, and local CA lifecycle hooks.
- TLS key-log export for analyzer integration.
- Structured transaction events suitable for session, process, flow, and artifact correlation.
- Dependency graph size and auditability.
- No packet interception drivers, system-wide redirect mechanisms by default, code injection, hooks, process memory reads, Winsock catalog changes, executable modification, target TLS key extraction, or certificate pinning bypass.
- Failure reporting and cleanup behavior.

## Candidate findings

### `hudsucker`

`hudsucker` is the closest native Rust match. Its README describes a MITM HTTP/S proxy that can modify HTTP/S requests, HTTP/S responses, and WebSocket messages. The public docs expose `HttpHandler` hooks for request and response handling, CONNECT interception, and TLS interception, plus `WebSocketHandler` hooks for WebSocket streams and messages. The builder accepts a certificate authority, custom client/server builders, graceful shutdown, HTTP handlers, WebSocket handlers, native-tls or rustls upstream connectors, and a custom connector.

That shape maps well to Deep Capture:

- The proxy is an ordinary local process or in-process server, not a packet interception driver.
- Request, response, error, CONNECT, TLS, and WebSocket hooks can become structured fragcap events.
- `RcgenAuthority` issues and caches forged server certificates from an imported CA issuer.
- HTTP/2 is a feature flag.
- Native-tls can use the Windows trust store for upstream connections.

The blockers are solvable but must be measured before adoption.

First, the latest `hudsucker` 0.25.0 declares `rust-version = "1.86.0"`. The older 0.23.0 line declares `rust-version = "1.75.0"` and is the better spike target under fragcap's current MSRV. A production dependency on a stale line needs an explicit maintenance decision.

Second, the dependency graph is large. A temp project with `hudsucker = "=0.23.0"` and `rcgen-ca`, `native-tls-client`, `decoder`, and `http2` resolved 216 unique packages. The equivalent rustls-client measurement resolved 198 unique packages. The graph is not disqualifying, but it is a feature-scale dependency addition and must live behind a Deep Capture feature gate.

Third, license posture needs a precise audit. The rustls-client path pulls `webpki-roots` 1.0.9, which is CDLA-Permissive-2.0 and outside the current allowlist. That should be avoided. The native-tls path is preferable on Windows because it uses the platform trust store, but metadata for the measured graph still included `webpki-roots`; the spike must prove whether this is an inactive target-conditional package, an avoidable feature edge, or an actual lockfile/audit blocker.

Fourth, HAR export is not a built-in `hudsucker` output. That is acceptable. fragcap already wants HAR to be a utility-wide output shape, so HAR should be generated from fragcap-owned event records rather than delegated to a proxy library.

Fifth, proxy-owned TLS key-log export is not proven from the public builder surface. Rustls has a `KeyLog` trait and `KeyLogFile`, but Deep Capture needs key material from the proxy-owned side of the inspected session and must not extract target process keys. The spike must prove whether `hudsucker` exposes enough rustls configuration to attach key logging, whether a custom connector is enough for upstream only, or whether key-log support requires an upstream patch or MVP deferral.

Classification: viable with caveats, recommended first spike target.

### `http-mitm-proxy`

`http-mitm-proxy` is an embeddable MITM proxy library with an API that binds a local proxy, signs generated certificates from a root issuer, forwards through a default client, and lets the caller inspect or modify request and response objects.

It is attractive because it is smaller and simpler than `hudsucker`. A temp project with `http-mitm-proxy = "=0.18.0"`, default features disabled, and `native-tls-client` resolved 155 unique packages. Its direct license is MIT, and the native-tls measurement did not print a CDLA package from metadata.

The caveat is product fit. Its public docs show the core request/response MITM path, but do not show first-class WebSocket hooks, protocol event richness, or an obvious key-log path. For fragcap, missing those hooks means the product would have to rebuild more behavior around the library, which reduces the advantage of taking it.

Classification: viable with major caveats, fallback native candidate.

### External `mitmdump`

The Python `mitmproxy` project remains the baseline. Its README describes an interactive SSL/TLS-capable proxy for HTTP/1, HTTP/2, and WebSockets, with `mitmdump` as the command-line form. The official docs state regular proxy mode is the simplest and most robust setup when the target can be configured to use an HTTP proxy. Its HAR support includes `mitmdump --set hardump=...` for saving HAR on exit, and its Wireshark TLS guide documents `SSLKEYLOGFILE` key logging.

That gives fragcap a known-good comparison point for:

- HTTP/1.1 behavior.
- HTTP/2 behavior.
- WebSocket behavior.
- HAR output.
- Analyzer key-log workflow.
- User-facing diagnostics for proxy and certificate problems.

The cost is distribution and lifecycle quality. Shipping Python plus mitmproxy inside a Rust Windows tool would be operationally janky: large artifacts, separate dependency update cadence, process orchestration, log translation, shutdown behavior, artifact cleanup, and support burden. It is useful as a baseline and fallback, not as the preferred product path.

Classification: baseline and fallback.

### Pingora

Pingora is a serious Rust proxy framework, not a Deep Capture MITM backend out of the box. Its README describes a framework for fast, reliable programmable network services, HTTP 1/2 end-to-end proxying, TLS via several stacks, gRPC and WebSocket proxying, and observability support.

The mismatch is scope. Deep Capture needs a local explicit MITM proxy with generated certificates, operator-visible trust lifecycle, HTTP transaction events, HAR-ready event data, and analyzer key-log hooks. Pingora would make fragcap build most of that layer itself.

The repository posture is also weak for MVP. Pingora's README says Linux is tier 1, Windows support is preliminary by community effort, and current MSRV is 1.85. A temp project with `pingora = "=0.8.1"` plus `proxy` and `rustls` resolved 260 unique packages. That is a large framework graph before certificate authority and HAR work.

Classification: deferred, not an MVP backend.

### `mitmproxy_rs`

`mitmproxy_rs` is useful context but not the default backend. Its README says it contains mitmproxy's Rust bits, especially WireGuard mode and local redirect mode. It also documents a Windows traffic redirector based on WinDivert.

That is the wrong default posture for fragcap. Deep Capture is meant to start with selected targets that can be explicitly configured for a local proxy. A Windows redirector based on packet diversion or local redirection may be valid research context, but it is not the simple target-scoped proxy path and risks dragging the design toward interception mechanisms the constitution deliberately avoids.

Classification: rejected for the default design.

### `soth-mitm`

The public docs are attractive. They describe HTTP/1.1, HTTP/2, WebSocket, gRPC, and SSE interception, deterministic handler/event contracts, local process metadata, CA generation, system trust install/uninstall, and trust checks.

The blockers are hard. `cargo info soth-mitm` reports MPL-2.0 and `rust-version = "1.88"`. Both conflict with fragcap's current policy.

Classification: rejected.

### `slinger-mitm`

`cargo info slinger-mitm` reports GPL-3.0-only. That is outside fragcap's allowlist.

Classification: rejected.

## Dependency measurements

Measurements were taken on 2026-08-24 with `rustc 1.96.0` and `cargo 1.96.0`, using isolated temp projects and `cargo metadata`.

| Candidate and feature set | Unique packages | Direct license | Declared Rust version | Notes |
| --- | ---: | --- | --- | --- |
| `hudsucker = "=0.25.0"` metadata | Not measured as graph | MIT OR Apache-2.0 | 1.86.0 | Latest line misses fragcap's current MSRV. |
| `hudsucker = "=0.23.0"`, `rcgen-ca`, `native-tls-client`, `decoder`, `http2` | 216 | MIT OR Apache-2.0 | 1.75.0 | Best spike shape, but license audit must resolve `webpki-roots` metadata. |
| `hudsucker = "=0.23.0"`, `rcgen-ca`, `rustls-client`, `decoder`, `http2` | 198 | MIT OR Apache-2.0 | 1.75.0 | Pulls `webpki-roots` CDLA root store, likely unacceptable. |
| `http-mitm-proxy = "=0.18.0"`, `native-tls-client` | 155 | MIT | Unknown | Smaller graph, less complete public API for Deep Capture. |
| `http-mitm-proxy = "=0.18.0"`, `rustls-client` | 137 | MIT | Unknown | Pulls `webpki-roots` CDLA root store, likely unacceptable. |
| `pingora = "=0.8.1"`, `proxy`, `rustls` | 260 | Apache-2.0 | README says 1.85 | Large graph, framework-level fit, preliminary Windows support. |
| `soth-mitm = "0.3.3"` | Not measured as graph | MPL-2.0 | 1.88 | Rejected before graph analysis. |
| `slinger-mitm = "0.0.5"` | Not measured as graph | GPL-3.0-only | Unknown | Rejected before graph analysis. |

## Open proof points

These should become acceptance criteria for the follow-up spike PR:

- Prove `hudsucker` can bind to loopback, accept explicit proxy traffic, and shut down cleanly under a fragcap-owned cancellation signal on Windows.
- Prove HTTP/1.1 request and response events can be emitted without body loss or silent truncation.
- Prove HTTP/2 behavior with a target that speaks HTTP/2 through CONNECT.
- Prove WebSocket message visibility and backpressure behavior.
- Prove whether response and request body decoding should happen in proxy code, output code, or a separate application-event layer.
- Prove local CA generation and import can be separated from certificate trust installation.
- Prove certificate cache state can be bounded, logged, and cleaned.
- Prove whether proxy-owned TLS key-log export is available through public APIs, requires a patch, or should be deferred.
- Prove the native-tls feature set avoids bundled root stores in the lockfile and under the repository license gate.
- Prove the graph compiles under Rust 1.82 if the feature is intended to participate in MSRV, or record a feature-gate exception if Deep Capture is intentionally excluded from the MSRV gate.
- Compare the same traffic through `mitmdump` and the native spike so HAR/event differences are concrete.

## Source notes

- `hudsucker` README and docs: <https://github.com/omjadas/hudsucker>, <https://docs.rs/hudsucker/latest/hudsucker/>
- Pingora README: <https://github.com/cloudflare/pingora>
- `mitmproxy_rs` README: <https://github.com/mitmproxy/mitmproxy_rs>
- `mitmproxy` README and docs: <https://github.com/mitmproxy/mitmproxy>, <https://docs.mitmproxy.org/stable/concepts/modes/>, <https://docs.mitmproxy.org/stable/howto/wireshark-tls/>, <https://www.mitmproxy.org/posts/har-support/>
- `http-mitm-proxy` docs: <https://docs.rs/http-mitm-proxy/latest/http_mitm_proxy/>
- `soth-mitm` docs and crate metadata: <https://docs.rs/soth-mitm/latest/soth_mitm/>
- `rcgen` README and crate metadata: <https://github.com/rustls/rcgen>
