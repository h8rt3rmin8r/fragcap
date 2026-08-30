# S104 Research: Native HTTP/TLS Production Cutover

## R-1: Cut over only with baseline protocol parity

**Decision**: Implement #290, #292, and #293 in one slice.

**Rationale**: #290 alone would make an HTTP/TLS-empty foundation the default and regress the shipped Deep Capture path. HTTP/1.1 provides ordinary proxying and CONNECT; TLS makes HTTPS inspectable under explicit trust. Together they form the minimum honest cutover.

**Alternatives considered**:

- Cut over under #290 alone: rejected because the native backend currently forwards and observes nothing.
- Complete all milestone-two issues in S104: rejected because HTTP/2, WebSocket, bodies, projections, correlation, client certificates, and broad conformance have separable owners and would turn one slice into an unreviewable milestone.
- Keep mitmdump as fallback: rejected because #290 requires its production removal and fallback would preserve two behavior paths.

## R-2: Use a bounded wire-level HTTP/1.1 boundary

**Decision**: Parse request and response heads with direct `httparse` 1.10.1 and implement strict bounded framing and forwarding in `fragcap-proxy`, while retaining Hyper for later HTTP/2 work.

**Rationale**: Hyper 1.11.1's public server API cannot emit arbitrary 1xx responses returned by an upstream origin. #292 explicitly requires informational responses, and recording then discarding them would violate P-9. A wire boundary can relay every informational head, preserve observed header bytes within declared limits, enforce one framing interpretation, and record the mandatory removal of proxy credentials.

**Alternatives considered**:

- Hyper server plus Hyper client: rejected for S104 HTTP/1.1 because arbitrary 1xx cannot be relayed downstream.
- Narrow #292 to only `100 Continue`: rejected because it changes an accepted tracker requirement to fit a library limitation.
- Fork Hyper or depend on another complete proxy: rejected as a larger dependency and maintenance surface than the bounded protocol owner already required by #292.

## R-3: Standard proxy authentication carries the session capability

**Decision**: Encode the random capability as URL-safe unpadded Base64 and place it as the password in a child-only proxy URL with fixed username `fragcap`. Accept exactly one strict HTTP Basic `Proxy-Authorization` value, decode and compare the 32-byte capability in constant time, zeroize temporary decoded bytes, and strip the header before forwarding.

**Rationale**: The S103 raw 32-byte preface is incompatible with ordinary proxy clients. Standard proxy URL credentials cause compatible clients to produce standard authorization. Authentication occurs before DNS, upstream connection, leaf issuance, body retention, or application observation.

**Alternatives considered**:

- Keep the raw preface: rejected as nonstandard and unusable by real targets.
- Trust every loopback client: rejected because unrelated local applications could enter an authorized inspection session.
- Hand-roll Base64 or use hexadecimal: rejected because the exact `base64` crate is already lock-resolved and URL-safe Base64 is compact and standard for URI userinfo.

## R-4: Borrow post-start access rather than persisting secrets

**Decision**: `ProxyLease` exposes borrowed `ProxySessionAccess` containing a redacted route and public trust material. `TrustManager::acquire` and `LaunchAdapter::launch` consume borrowed access after proxy start. Capability and private keys never enter `SessionPlan`, events, snapshots, artifacts, equality, display, or serialization.

**Rationale**: The capability and authority are created at listener start, after immutable plan authorization. Borrowing makes their lifetime and non-persistence mechanical. The coordinator remains the owner of start, trust, launch, stop, and cleanup ordering.

**Alternatives considered**:

- Put credentials or CA material in `SessionPlan`: rejected because the plan is cloned, debugged, emitted, and persisted.
- Share an untyped CLI runtime cell: rejected because it perpetuates CLI business logic and makes secret flow implicit.
- Generate unrelated trust and proxy authorities: rejected because trusting one identity while the proxy presents another cannot work and produces false cleanup ownership.

## R-5: Route only the selected managed launch

**Decision**: Remove process-global scoped environment mutation. Pass the credential-bearing route only to the adapter that owns the exact retained managed launch. Keep warm and unowned handoffs refused. Steam protocol behavior remains compatibility-gated and later routing issues may expand it.

**Rationale**: Mutating the CLI process environment is visible to unrelated work and unsafe for concurrent sessions. Child-only configuration matches P-1 and the S101 launch contract.

**Alternatives considered**:

- Temporarily change parent environment: rejected as wider than the selected target session.
- Change system proxy settings: constitutionally prohibited by default.
- Launch before capability creation: rejected because the child would receive no authenticated route and cannot be repaired retroactively.

## R-6: Separate client and upstream TLS boundaries

**Decision**: Generate one session CA inside the proxy owner, issue bounded leaves for the validated CONNECT authority, terminate client TLS with TLS 1.2/1.3 and HTTP/1.1 ALPN, and connect upstream with one session-owned native-root client configuration that always verifies the requested name and chain.

**Rationale**: Separate facts and errors prevent a client refusal, leaf failure, origin connection failure, or validation failure from becoming a fabricated decrypted transaction. The ring provider remains explicit and no permissive verifier exists.

**Alternatives considered**:

- Raw CONNECT tunnel only: rejected because #293 requires client-facing HTTPS inspection.
- Install an upstream permissive verifier for controlled tests: rejected; tests use explicit roots through an injected configuration.
- Advertise HTTP/2 now: rejected because #294 owns multiplexing and S104 must not negotiate a protocol it cannot faithfully serve.

## R-7: Buffer only within explicit S104 exchange bounds

**Decision**: Use finite request and response message bounds for HTTP/1.1 forwarding and record truncation/refusal exactly. Do not claim streaming-body completeness or decoding.

**Rationale**: #297 owns streaming body artifact retention and transformations. S104 needs bounded forwarding and current coarse observations, not unbounded storage. Wire bytes pass through incrementally where possible; parser lookahead and retained evidence stay bounded.

**Alternatives considered**:

- Unbounded collection: rejected as a resource exhaustion path.
- Full content decoding and artifact retention: deferred to #297 because it changes authority and loss contracts.
- Metadata-only forwarding that drops bodies: rejected because it changes traffic semantics.

## R-8: Native readiness is compiled capability, not executable discovery

**Decision**: Doctor reports the in-process native backend identity and configuration readiness without binding a listener or spawning a version command. A lint gate scans production paths for external proxy and embedded Python constructs.

**Rationale**: Once the backend is linked, PATH discovery says nothing useful. Readiness still distinguishes platform trust availability and configuration errors, while source policy prevents regression.

**Alternatives considered**:

- Retain `--proxy-backend`: rejected because there is only one supported production backend.
- Keep mitmdump discovery for diagnostics: rejected because it implies a supported fallback and retains process execution.

## Dependency Audit

- `base64` 0.22.1: direct runtime edge, `default-features = false`, `alloc`; already in `Cargo.lock`; MIT OR Apache-2.0; Rust 1.48.
- `httparse` 1.10.1: direct runtime edge; already in `Cargo.lock` through Hyper; MIT OR Apache-2.0; compatible with Rust 1.88.
- No new package, native build input, crypto provider, or inter-crate edge is introduced.
