# Research: Smaller Native Proxy Fallback Spike

## Candidate feature set

**Decision**: Use exact `http-mitm-proxy` 0.18.0 with defaults disabled and `native-tls-client` enabled.

**Rationale**: Issue #274 fixes the version. The native TLS path avoids the optional `webpki-roots` dependency and uses the operating-system upstream trust mechanism.

**Alternatives considered**: The rustls client path brings a bundled root package outside the allowlist. Omitting `DefaultClient` would measure a custom proxy rather than this candidate.

## Shared matrix

**Decision**: Adapt the S099 scenario identities, fixed payloads, deadlines, normalized states, and parity rules in a new isolated workspace.

**Rationale**: This preserves S099 evidence while making the three-way comparison exact.

**Alternatives considered**: Editing S099 would blur historical ownership. A new matrix would not answer parity.

## Lifecycle ownership

**Decision**: Own and abort the future returned by `MitmProxy::bind`, then separately measure active-connection cleanup.

**Rationale**: The bind future owns the listener, but accepted connections and CONNECT sessions are spawned internally. Listener cancellation therefore does not prove connection cleanup.

**Alternatives considered**: A custom accept loop around `wrap_service` gives stronger ownership but measures fragcap orchestration rather than the advertised bind path.

## CA and certificate cache

**Decision**: Generate a session-private CA, trust only its public certificate in controlled clients, and pass a caller-owned capacity-bounded `moka` cache.

**Rationale**: The public API accepts both the issuer and cache, making separation, capacity, entry count, and cleanup measurable without system mutation.

**Alternatives considered**: Installing system trust is prohibited. Omitting the cache would skip the advertised state boundary.

## HTTP and HAR source

**Decision**: Buffer fixed controlled request and response bodies at the service seam, record protocol, length, and digest, reconstruct them, and forward through `DefaultClient`.

**Rationale**: Standard Hyper messages expose the method, URI, status, headers, protocol, and body data needed for fragcap-owned HAR generation.

**Alternatives considered**: Streaming is production work; bounded fixtures first prove fidelity.

## WebSocket

**Decision**: Use `DefaultClient::with_upgrades` and measure the public upgraded streams separately from the 101 handshake.

**Rationale**: The candidate exposes raw upgraded endpoints but provides no frame API. Message visibility must account for adapter-owned parsing and forwarding.

**Alternatives considered**: Calling handshake success message parity would be false.

## Client-facing TLS key logging

**Decision**: Record unsupported unless a public candidate interface permits a key logger on the client-facing TLS server configuration.

**Rationale**: The crate constructs that `rustls::ServerConfig` privately. Upstream native TLS logs do not prove the proxy-owned inspected session.

**Alternatives considered**: A dependency fork or patch is outside this fallback evaluation; target key extraction is prohibited.

## Final comparison

**Decision**: Join fallback rows to frozen S099 `hudsucker` and `mitmdump` rows, then select exactly one permitted backend outcome.

**Rationale**: S100 is the one bounded follow-up allowed by S099 and closes the backend search.

**Alternatives considered**: Another candidate issue would violate the defined scope.

S099 committed its sanitized comparison as Markdown rather than a machine-specific JSON run. S100 therefore aligns the shared proof-point keys in its evidence table instead of adding a parser for prose or pretending that a fresh baseline run is the historical S099 run.
