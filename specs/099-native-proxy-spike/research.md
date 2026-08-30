# Research: Native Proxy Backend Spike

## R-1: Candidate and Feature Set

**Decision**: Evaluate `hudsucker = "=0.23.0"` with default features disabled and `decoder`, `http2`, `native-tls-client`, and `rcgen-ca` enabled. Record current `0.25.0` only as maintenance context.

**Rationale**: The accepted backend research and issue #253 name 0.23.0 as the spike target. Its crate metadata declares Rust 1.75, while 0.25.0 declares Rust 1.86 and cannot satisfy the workspace's Rust 1.82 policy. Exact pinning keeps results reproducible and prevents a later compatible-range resolution from silently changing the candidate.

**Alternatives considered**: Latest `hudsucker` is excluded from the executable matrix because it answers a different MSRV question. `http-mitm-proxy` remains the named smaller fallback and is measured only if the candidate decision selects that path.

## R-2: Isolated Workspace Boundary

**Decision**: Put the executable spike in `spikes/native-proxy` with its own `[workspace]`, lock file, target directory, and deny policy. Add a second minimal nested audit manifest containing only the issue-mandated candidate feature set so harness-only dependencies cannot inflate the candidate delta. Do not add either to the root workspace or any product manifest.

**Rationale**: Issue #253 forbids a product dependency or release-artifact change. A nested workspace makes the boundary visible to Cargo metadata and lets dependency, license, MSRV, and build-time evidence be reproduced without contaminating `Cargo.lock`. Separating the audit manifest is a post-plan correction: the executable harness necessarily adds clients, servers, serialization, and test packages that are not part of a product adoption delta.

**Alternatives considered**: A product-crate example was rejected because its dependencies enter that package's graph and publishing surface. A wholly temporary uncommitted crate was rejected because reviewers could not reproduce the measurements.

## R-3: Controlled Traffic Matrix

**Decision**: Use local loopback servers and clients owned by the harness for HTTP/1.1, HTTPS through CONNECT, HTTP/2 through CONNECT, and WebSocket messages. Both backends receive the same fixed payloads and normalized expected observations.

**Rationale**: Local deterministic traffic needs no game account, private service, or uncontrolled network. Fixed bodies make silent loss and decoding visible. A common matrix prevents backend-specific demonstrations from being mistaken for parity.

**Alternatives considered**: Public echo services were rejected because they are uncontrolled and add network variability. Existing game traffic was rejected because it can contain private data and cannot prove expected bodies.

## R-4: Certificate and Trust Boundary

**Decision**: Generate session-private CA material inside a temporary directory, pass it directly to the controlled client and backend, and never add it to an operating-system trust store. Permit disabled upstream verification only for the harness-owned TLS origin, record it explicitly, and never expose that choice as product behavior.

**Rationale**: The proof concerns CA generation/import and proxy interception, not trust installation. Direct client configuration proves separation and avoids residue. The controlled origin is not a protected target and carries no pinning behavior.

**Alternatives considered**: Current-user trust installation was rejected because S099 needs no trust mutation. Reusing a committed private key was rejected because repository history is not an acceptable secret store, even for a test CA.

## R-5: Body and HAR-Source Fidelity

**Decision**: Buffer the small fixed test bodies at the handler boundary, record original length, observed length, content encoding, completeness, and any decode result, then reconstruct an equivalent body for forwarding. The spike does not impose a product payload cap.

**Rationale**: `hudsucker` handlers receive streaming bodies. Merely counting handler calls would miss truncation. Full buffering is acceptable for small controlled fixtures and proves whether the public hooks contain the headers, versions, status, and bodies required for fragcap-owned HAR generation.

**Alternatives considered**: Streaming tee logic was rejected for this non-shipping spike because it adds backpressure and partial-write design that belongs in an adoption issue. Delegating HAR generation to the backend was rejected because HAR is a utility-wide fragcap output contract.

## R-6: Proxy-Owned Key Logging

**Decision**: Test a wrapper implementing `hudsucker::certificate_authority::CertificateAuthority`. It delegates certificate generation to `RcgenAuthority`, clones the public rustls `ServerConfig`, attaches a `KeyLog` implementation, and returns the modified config for the client-facing proxy TLS session.

**Rationale**: The public trait is the only supported point that owns client-facing TLS configuration. A custom upstream connector can log only the proxy-to-origin side and would not answer the analyzer question. The wrapper establishes whether public APIs suffice without modifying `hudsucker`.

**Alternatives considered**: Patching private proxy internals is deferred unless the wrapper fails. Environment-only `SSLKEYLOGFILE` is insufficient evidence unless the client-facing server configuration actually receives a key logger.

## R-7: Certificate Cache Measurement

**Decision**: Configure a finite per-session `RcgenAuthority` capacity, record the configured bound and observed hit or generation signals, inspect source-defined lifetime behavior, and treat loss of the session-owned authority as cleanup. Record that the public API does not expose entry enumeration or explicit invalidation if that remains true.

**Rationale**: The cache is in-memory and authority-owned. Capacity and ownership are controllable, while current public observability may be limited to tracing. The decision must distinguish an acceptable session drop from a requirement for stronger product diagnostics.

**Alternatives considered**: Reimplementing the cache would turn a measurement spike into an unreviewed fork. An unbounded cache is prohibited.

## R-8: Baseline Adapter

**Decision**: Run the installed `mitmdump` binary as a bounded child on loopback with a private configuration directory, an addon that emits normalized local observations, optional HAR output, and `MITMPROXY_SSLKEYLOGFILE` scoped to that child only.

**Rationale**: This matches the shipped external-process posture while keeping configuration, CA material, logs, and keys session-owned. The installed version is recorded in evidence rather than generalized to all versions.

**Alternatives considered**: Importing mitmproxy as a Python library would test a different integration surface. Global `SSLKEYLOGFILE` was rejected because it can affect unrelated processes.

## R-9: Audit and Decision Rule

**Decision**: Audit the isolated lock with Cargo metadata and trees for active and target-conditional paths, apply a copied repository-equivalent `cargo deny` policy, measure Rust 1.82 and pinned builds, and compare root workspace package identities before and after. Select one decision only after every issue criterion has an explicit result.

**Rationale**: Package presence in metadata does not prove an active normal dependency path, and an active tree does not enumerate inactive target-conditional packages. Both views are needed. The four issue outcomes are mutually exclusive and give uncertainty an honest destination.

**Alternatives considered**: Relying on crate-level license fields or a successful pinned build was rejected because neither establishes transitive license or MSRV compatibility.

## Primary Sources

- [`hudsucker` 0.23.0 crate metadata](https://crates.io/crates/hudsucker/0.23.0)
- [`hudsucker` repository](https://github.com/omjadas/hudsucker)
- [`hudsucker` public API](https://docs.rs/hudsucker/0.23.0/hudsucker/)
- [mitmproxy regular proxy documentation](https://docs.mitmproxy.org/stable/concepts/modes/)
- [mitmproxy options reference](https://docs.mitmproxy.org/stable/concepts/options/)
- [mitmproxy TLS key-log documentation](https://docs.mitmproxy.org/stable/howto/wireshark-tls/)
