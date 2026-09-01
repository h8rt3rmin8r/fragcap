# Research: Authenticated SOCKS5 TCP Routing

## Shared Listener Recognition

**Decision**: Recognize SOCKS5 when the first unconsumed client octet is `0x05`; otherwise retain the existing HTTP/1.1 and HTTP/2 detection.

**Rationale**: SOCKS5 has an unambiguous version octet. Sharing the listener preserves one endpoint, lifecycle obligation, route, connection permit pool, and cleanup authority.

**Alternatives considered**: A second listener would require another endpoint, secret route, journal obligation, manifest field, and cleanup path. ALPN cannot classify cleartext SOCKS.

## Session Authentication

**Decision**: Require RFC 1929 username/password with username `fragcap` and the existing base64url session capability password. Reject no-authentication even on loopback.

**Rationale**: This reuses the reviewed entropy and constant-time capability boundary and is supported by standard SOCKS clients and proxy URL syntax.

**Alternatives considered**: No-authentication makes unrelated local clients tenants of the session. GSSAPI introduces identity infrastructure and dependencies outside the product contract. A new token would create competing secret authority.

## DNS Ownership

**Decision**: Publish `socks5h` and resolve domain-form requests inside the proxy. Apply the existing normalized destination policy to each returned address.

**Rationale**: `socks5h` preserves the requested domain until it reaches fragcap, making DNS ownership explicit and ensuring listener, local, and private-destination policy applies after resolution.

**Alternatives considered**: `socks5` permits client-side resolution and would make DNS ownership client-dependent. Accepting a domain while bypassing post-resolution policy would be a local-destination escape.

## Forwarding And Half-Close

**Decision**: Use Tokio's bounded bidirectional copy with the configured per-connection buffer size, wrapped in idle timeout and runtime cancellation. EOF in one direction shuts down the opposite writer while the reverse direction continues.

**Rationale**: The existing runtime already owns Tokio, cancellation, finite tasks, and per-connection buffer configuration. Fixed buffers supply backpressure without retaining tunnel payloads.

**Alternatives considered**: Unbounded channels violate the runtime contract. A hand-written relay duplicates a well-tested half-close state machine. Capturing raw payload here would consume issue #312 and expand sensitive evidence.

## Protocol Classification

**Decision**: Classify a bounded unconsumed prefix as HTTP, TLS, or opaque TCP. Clear HTTP may reuse the HTTP engine only when its bytes and authority contract can be preserved; TLS may reuse the existing client-facing and upstream boundaries only for supported HTTP ALPN. All other traffic remains byte-transparent metadata-only.

**Rationale**: Issue #310 requires feeding classification while issue #312 owns generic TCP and non-HTTP TLS evidence. Unknown input must not be guessed or withheld.

**Alternatives considered**: Treating every tunnel as raw would omit the requested classifier integration. Treating every TLS stream as HTTP would invent semantics. Consuming a prefix into a separate buffer risks loss and reordering.

## Evidence And Correlation

**Decision**: Add typed SOCKS events to the existing application sink and lifecycle projection, keyed by the existing connection id and open/close window. Add protocol counters to the existing runtime observation.

**Rationale**: Existing artifacts and packet/process correlation already own these facts. A SOCKS-specific sidecar would create a competing authority.

**Alternatives considered**: Encoding SOCKS events as HTTP metadata misstates the observation. A new artifact would require another schema, loss account, manifest role, finalizer, and recovery path.

## Dependencies

**Decision**: Add no dependency.

**Rationale**: SOCKS5 framing is small, fixed, and incremental. Tokio, base64, subtle, and zeroize already provide runtime, encoding, constant-time comparison, and secret cleanup.

**Alternatives considered**: A SOCKS server crate would add a protocol and license surface for a bounded grammar that is smaller than its adapter code.
