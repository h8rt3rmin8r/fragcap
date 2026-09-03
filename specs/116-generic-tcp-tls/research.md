# Research: Generic TCP And Non-HTTP TLS Evidence

## Transport Entry Points

**Decision**: Use authenticated SOCKS5 CONNECT for byte-transparent generic TCP and opaque TLS, and the existing authenticated HTTP CONNECT path for trusted TLS interception.

**Rationale**: These are the two already shipped child-scoped routing strategies. Reusing them retains one listener, capability, destination policy, connection identity, and cleanup authority.

**Alternatives considered**: A new raw listener creates another route and tenancy boundary. Guessing TLS from a destination port is false. Retrofitting transparent interception into SOCKS after consuming a ClientHello cannot provide a truthful fallback.

## Generic TLS Discriminator

**Decision**: HTTP/2 ALPN remains HTTP/2. Explicit HTTP/1.1 ALPN remains HTTP/1.1. No-ALPN client TLS is inspected through a bounded non-consuming buffered prefix: a recognizable HTTP method stays HTTP/1.1 and every other prefix becomes protocol-unknown TLS.

**Rationale**: Many binary TLS protocols use no ALPN. A buffered prefix preserves every decrypted byte for either consumer. Unknown and incomplete input is never upgraded into an HTTP claim.

**Alternatives considered**: Treating all no-ALPN TLS as HTTP causes the current `client-tls-no-http` failure. Treating all of it as generic would regress HTTP/1.1 clients that omit ALPN. Custom ALPN dispatch belongs to a later compatibility slice unless directly observed.

## Evidence Model

**Decision**: Add one typed generic stream chunk event to the existing application sink. Each record names direction, offset, observed length, retained bytes, outcome, and provenance. Terminal and TLS events continue through existing types.

**Rationale**: The application stream already owns payload evidence, queue accounting, correlation, and crash-readable finalization. A raw-stream sidecar would duplicate those authorities.

**Alternatives considered**: Reusing HTTP body segments would falsely claim request/response semantics. Aggregate byte counts alone do not satisfy direction, timing, truncation, or correlation acceptance.

## Retention And Forwarding

**Decision**: Reuse the body per-connection and session retention limits and split observations at the existing maximum event chunk size. Forwarding uses its existing independent fixed buffers and proceeds after evidence is omitted.

**Rationale**: One storage budget prevents protocol-specific paths from multiplying retained sensitive data. Independent forwarding and retention preserve large or indefinite valid streams.

**Alternatives considered**: A second generic-stream budget complicates configuration and can exceed the operator's intended session bound. Buffering a whole stream is unbounded.

## TLS Refusals

**Decision**: Preserve the S107 structured rustls refusal taxonomy and terminate after any failed interception boundary. Do not fall back to opaque relay.

**Rationale**: After client TLS bytes are consumed, fallback cannot replay the original stream and would misstate both security and evidence provenance.

## Dependencies

**Decision**: Add no dependency.

**Rationale**: Tokio, rustls, bytes, base64, and the existing body budget primitives provide every required mechanism.
