# S106 Research

## WebSocket wire ownership

**Decision**: Use a bounded incremental parser that observes original octets while the existing relay forwards them unchanged. Validate RFC 6455 masking, lengths, control frames, fragmentation, and text UTF-8. Use the existing ring SHA-1 implementation only for the protocol-required `Sec-WebSocket-Accept` calculation.

**Rationale**: Message-oriented WebSocket libraries hide frame and masking evidence that Deep Capture must retain. The parser has no authority over forwarding.

**References**: RFC 6455, <https://www.rfc-editor.org/rfc/rfc6455>; RFC 8441, <https://www.rfc-editor.org/rfc/rfc8441>.

## WebSocket compression

**Decision**: Promote exact-resolved `flate2` 1.1.10 as a direct dependency and implement negotiated `permessage-deflate` with direction-specific context takeover.

**Rationale**: RFC 7692 uses raw DEFLATE and can preserve state across messages. `flate2::Decompress` exposes that finite state without adding a lockfile package. Unsupported parameters remain explicit.

**Reference**: RFC 7692, <https://www.rfc-editor.org/rfc/rfc7692>.

## HTTP/2 WebSocket activation

**Decision**: Enable the RFC 8441 CONNECT protocol setting and classify a stream as WebSocket only when the extended CONNECT protocol is `websocket` and the upstream accepts it.

**Rationale**: Ordinary CONNECT DATA is not necessarily WebSocket traffic. Negotiated activation prevents false protocol claims.

## Server-Sent Events

**Decision**: Parse identity `text/event-stream` response bodies incrementally using the WHATWG line and field algorithm. Preserve comments, `id`, `event`, `data`, retry values, malformed UTF-8 outcomes, and EOF behavior. Raw body segments remain authoritative.

**Rationale**: SSE is an indefinite stream. Whole-body decoding violates finite-memory and latency requirements.

**Reference**: WHATWG HTML Server-sent events, <https://html.spec.whatwg.org/multipage/server-sent-events.html>.

## gRPC observation boundary

**Decision**: Detect gRPC from HTTP/2 content type and metadata, retain method and metadata, and parse only the five-byte compressed flag plus big-endian message length envelope. Payloads remain opaque and compression is reported from `grpc-encoding`.

**Rationale**: Schema-free protobuf decoding would fabricate meaning. One bounded envelope parser covers every gRPC streaming pattern.

**References**: gRPC HTTP/2 protocol, <https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-HTTP2.md>; gRPC compression guide, <https://grpc.io/docs/guides/compression/>.

## Artifact compatibility

**Decision**: Keep application schema version 2 and add the protocol families reserved by S105.

**Rationale**: Existing record meanings do not change, and readers reconcile unknown additive types generically.
