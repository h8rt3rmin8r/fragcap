# Feature Specification: Streaming Application Protocols

**Feature Branch**: `codex/106-streaming-protocols`

**Created**: 2026-08-30

**Status**: Complete

**Input**: User description: "Kick off S106"

## Clarifications

### Session 2026-08-30

- Q: Which ready work belongs in S106? -> A: Close WebSocket, SSE, and gRPC together.
- Q: Which evidence is authoritative for framed protocols? -> A: Raw frames and bytes remain authoritative; assembled messages and parsed events are derived.
- Q: May observation pressure slow or reject otherwise valid traffic? -> A: No; forwarding and evidence retention remain independently bounded.

## User Scenarios & Testing

### User Story 1 - Inspect WebSocket Conversations (Priority: P1)

An authorized operator runs Deep Capture against a compatible target using WebSocket over HTTP/1.1 or HTTP/2 and receives bidirectional frame evidence plus bounded message assembly without losing the handshake, frame boundaries, or terminal reason.

**Why this priority**: WebSocket is the highest-priority remaining application protocol and changes a finite HTTP exchange into a long-lived bidirectional stream. Incorrect masking, fragmentation, or control-frame handling can corrupt forwarding or produce false evidence.

**Independent Test**: A controlled client completes HTTP/1.1 upgrade and HTTP/2 extended CONNECT handshakes, exchanges every standard opcode in both directions, fragments text and binary messages, negotiates per-message compression, and closes cleanly and incorrectly. Forwarded frames remain valid, raw frame evidence remains authoritative, derived messages reconcile to their source frames, and every connection reaches one terminal outcome.

**Acceptance Scenarios**:

1. **Given** a valid authorized WebSocket handshake, **When** text, binary, continuation, ping, pong, and close frames pass in either direction, **Then** each frame records its direction, opcode, flags, observed size, retained payload outcome, timing, connection identity, and stream identity where applicable.
2. **Given** a message fragmented across frames with interleaved control frames, **When** the final continuation arrives, **Then** the raw frames remain independently visible and one bounded derived message cites exactly those data frames.
3. **Given** per-message compression is negotiated, **When** compressed messages pass, **Then** compressed frame payload remains authoritative and bounded decompression records an exact success or failure outcome.
4. **Given** invalid masking, reserved bits, control fragmentation, length encoding, close payload, or text encoding, **When** the peer sends the invalid frame, **Then** the proxy fails safely, records the exact refusal boundary, and does not mislabel partial evidence as complete.

---

### User Story 2 - Inspect Server-Sent Event Streams (Priority: P2)

An authorized operator can inspect a long-lived event-stream response as both ordered raw body evidence and bounded derived event records without waiting for the response to end.

**Why this priority**: Server-Sent Events are common long-lived HTTP responses. Whole-body parsing would be unbounded and would delay the evidence users need during the session.

**Independent Test**: A controlled origin streams comments, named events, identifiers, retry values, multiline data, split line endings, malformed UTF-8, reconnect metadata, idle periods, and cancellation. Raw chunks continue through the existing body stream while complete events appear incrementally and bounded parser state is accounted at termination.

**Acceptance Scenarios**:

1. **Given** a response identified as an event stream, **When** fields and dispatch boundaries arrive across arbitrary body segments, **Then** comments, data, event, id, retry, unknown fields, and blank-line dispatch boundaries are represented without changing raw bytes.
2. **Given** multiple data fields or a field split across body segments, **When** an event dispatches, **Then** the derived event preserves ordered source ranges and the specified multiline value without buffering the full response.
3. **Given** malformed UTF-8, an invalid retry value, cancellation, idle timeout, or end-of-stream with an incomplete event, **When** parsing terminates, **Then** raw bytes remain available and the decode, ignored-field, partial, or terminal outcome is explicit.
4. **Given** observation queue saturation, **When** the HTTP response continues, **Then** forwarding remains independent and every lost or truncated SSE observation is counted and located.

---

### User Story 3 - Inspect gRPC Streaming Boundaries (Priority: P2)

An authorized operator can distinguish gRPC traffic carried over HTTP/2, inspect request and response metadata and trailers, and observe raw gRPC message envelopes and payload boundaries for unary and streaming calls without requiring a protobuf schema.

**Why this priority**: HTTP/2 body segments alone do not identify remote procedure calls, message boundaries, compression flags, or final gRPC status. Fabricated protobuf decoding would violate evidence fidelity.

**Independent Test**: A controlled protocol lab runs unary, client-streaming, server-streaming, and bidirectional calls with repeated metadata, multiple messages, compression flags, trailers, cancellation, oversized envelopes, partial headers and payloads, and malformed reserved bits. Each call and message retains exact framing evidence and finishes with a reconcilable outcome.

**Acceptance Scenarios**:

1. **Given** a supported gRPC content type and method path, **When** an HTTP/2 stream begins, **Then** the call records method identity, request and response metadata, content subtype, encoding declarations, direction, stream identity, and timing from observed HTTP evidence.
2. **Given** unary or streaming message envelopes split across arbitrary HTTP/2 body segments, **When** complete messages arrive, **Then** each message records direction, ordinal, compression flag, declared size, raw payload bytes according to scope, source ranges, and completion state.
3. **Given** trailers carry gRPC status or details, **When** the call terminates, **Then** observed trailer bytes remain authoritative and the call outcome cites them without inventing application success.
4. **Given** a compressed, partial, oversized, cancelled, or malformed envelope, **When** it is processed, **Then** forwarding policy remains bounded, raw evidence is retained according to scope, and the exact unsupported, partial, limit, cancellation, or protocol outcome is explicit.

### Edge Cases

- A WebSocket control frame arrives between continuation frames, or a close handshake crosses shutdown.
- Client and server masking rules differ by direction, including invalid masked server frames and unmasked client frames.
- Extended CONNECT is attempted without the required HTTP/2 setting or with an invalid pseudo-field set.
- Per-message compression uses context takeover, negotiated window limits, empty messages, fragmented compressed messages, or invalid compressed payloads.
- An SSE byte-order mark, carriage return, line feed, or combined line ending is split across body segments.
- An SSE stream changes `id`, clears it with a null byte, provides an out-of-range retry value, ends before a blank line, or remains idle until cancellation.
- A gRPC envelope header or payload spans many HTTP/2 DATA segments, declares a size above the configured observation limit, or ends early.
- A gRPC content type has a structured suffix, an unsupported encoding, missing trailers, percent-encoded status details, or a nonzero status after message delivery.
- Application-event retention saturates while one or more long-lived protocol streams continue forwarding.
- Session shutdown interrupts partial WebSocket messages, partial SSE events, or partial gRPC envelopes.

## Requirements

### Functional Requirements

- **FR-001**: Deep Capture MUST recognize and validate WebSocket handshakes over HTTP/1.1 upgrade and HTTP/2 extended CONNECT without treating an ordinary HTTP exchange as WebSocket traffic.
- **FR-002**: The proxy MUST forward WebSocket frames bidirectionally while recording direction, opcode, final and reserved flags, masking state, observed length, retained payload outcome, event time, connection identity, and HTTP stream identity where applicable.
- **FR-003**: Raw WebSocket frame evidence MUST remain authoritative. Derived reassembled messages MUST cite their source frame range and MUST NOT hide fragmentation, interleaved control frames, truncation, or loss.
- **FR-004**: Text, binary, continuation, close, ping, and pong frames MUST be represented, while reserved opcodes and extensions not negotiated by the handshake MUST be refused explicitly.
- **FR-005**: Client masking, server masking, length encoding, reserved bits, fragmentation, control-frame, close-payload, and text-encoding rules MUST be validated at the observed boundary and failures MUST produce exact terminal evidence.
- **FR-006**: Negotiated per-message compression MUST preserve compressed raw payload authority and MUST bound decompression input, output, expansion ratio, context state, concurrency, elapsed time, and message retention.
- **FR-007**: WebSocket message assembly MUST be finite per message and per session. Exceeding evidence bounds MUST truncate or omit derived retention with named accounting while permitted forwarding continues.
- **FR-008**: HAR output MUST remain limited to the WebSocket handshake. WebSocket frames and messages MUST belong only to the application observation artifact.
- **FR-009**: Deep Capture MUST recognize Server-Sent Events only from an observed event-stream response media type and MUST parse fields incrementally across arbitrary body-segment boundaries.
- **FR-010**: SSE observations MUST represent comments, data, event, id, retry, unknown fields, blank-line dispatch, source byte ranges, line endings, direction, event time, connection identity, and HTTP stream identity.
- **FR-011**: SSE multiline data, last-event identifier, retry value, byte-order mark, end-of-stream, reconnect metadata, and field-name rules MUST follow the protocol grammar without normalizing or replacing raw body evidence.
- **FR-012**: Malformed SSE text, ignored or invalid fields, incomplete terminal events, cancellation, idle timeout, retention truncation, and queue loss MUST remain distinguishable.
- **FR-013**: SSE parsing MUST use finite line, field, event, stream, and elapsed-time bounds and MUST NOT require a complete or finite response body.
- **FR-014**: Deep Capture MUST identify gRPC calls only from supported observed content types on HTTP/2 and MUST retain the observed method path and request, response, and trailer metadata.
- **FR-015**: gRPC message framing MUST be parsed incrementally from arbitrary HTTP/2 body segmentation and MUST record direction, ordinal, compression flag, declared length, raw payload retention, source byte ranges, timing, and completion state.
- **FR-016**: Unary, client-streaming, server-streaming, and bidirectional calls MUST be representable without requiring or inferring a protobuf schema.
- **FR-017**: gRPC compression declarations and per-message compression flags MUST be recorded. Unsupported or inconsistent compression MUST be explicit and MUST NOT be mislabeled as decoded content.
- **FR-018**: gRPC status, message, status details, cancellation, reset, missing trailers, protocol error, partial envelope, size limit, and transport failure MUST produce distinct call or message outcomes based only on observed evidence.
- **FR-019**: Every WebSocket frame and message, SSE field and event, and gRPC call and message MUST carry all available session, target, proxy connection, HTTP stream, packet flow, process, role, attribution, scope, truncation, loss, and timing anchors without inventing unavailable values.
- **FR-020**: Protocol-specific application records MUST be versioned, append-only, binary-safe, and readable as a valid prefix after interruption. Unknown future record families MUST remain safely skippable or explicitly rejected according to the published reader contract.
- **FR-021**: Forwarding buffers, protocol-parser state, decompression state, message retention, event queues, disk writing, idle time, and shutdown work MUST have explicit finite bounds.
- **FR-022**: Observation saturation, truncation, parser refusal, unsupported encoding, decode failure, writer retirement, cancellation, and forced shutdown MUST advance named accounting and MUST NOT silently alter, reorder, or discard retained evidence.
- **FR-023**: Observation retention or parsing failure MUST NOT indefinitely block permitted network forwarding, corrupt forwarded bytes, or allow a partial application artifact to be labeled complete.
- **FR-024**: Every accepted protocol stream and every started derived message or event MUST reach exactly one terminal state during orderly completion, peer failure, or bounded session cleanup.
- **FR-025**: Metadata-only scope MUST retain no WebSocket, SSE, or gRPC payload bytes while still retaining permitted framing metadata and an explicit scope omission distinct from loss.
- **FR-026**: The controlled protocol lab MUST cover both WebSocket handshake forms, every standard WebSocket opcode, fragmentation, per-message compression, malformed frames, SSE grammar and long-lived cancellation, all four gRPC call patterns, malformed and bounded gRPC envelopes, queue pressure, interruption, and repeated cleanup without Internet access or privileged effects.
- **FR-027**: Existing HTTP/1.1, HTTPS, HTTP/2, metadata, body, application-stream, authentication, destination-policy, lifecycle, and cleanup behavior MUST retain its prior outcomes.
- **FR-028**: This slice MUST NOT implement complete HAR projection, native TLS key-log output, packet and process correlation completion, client-certificate forwarding, generic TCP or UDP inspection, QUIC or HTTP/3, launch-strategy expansion, or a Deep Capture feature-completion claim.

### Key Entities

- **WebSocket handshake**: The protocol switch or extended connection request that authorizes frame processing and records negotiated extensions and subprotocols.
- **WebSocket frame**: One raw directional frame with opcode, flags, masking, length, payload retention, and terminal parsing outcome.
- **WebSocket message**: One bounded derived text or binary message linked to its ordered source data frames and interleaved control-frame context.
- **SSE field**: One parsed field or comment linked to an exact range of raw event-stream body evidence.
- **SSE event**: One blank-line-dispatched derived event carrying ordered data, event type, last-event identifier, retry update, source ranges, and completion state.
- **gRPC call**: One HTTP/2 stream recognized from observed content type and method metadata, with directional message sequences, trailers, and terminal outcome.
- **gRPC message**: One five-byte framed message envelope plus bounded raw payload evidence, compression state, ordinal, source ranges, and completion outcome.
- **Protocol observation account**: Reconciliation of accepted, emitted, retained, intentionally omitted, truncated, malformed, unsupported, queue-dropped, writer-failed, cancelled, and forcibly terminated evidence.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Controlled WebSocket tests preserve 100 percent of valid raw frame boundaries and payload bytes within scope, and derived fragmented messages cite exactly their contributing data frames.
- **SC-002**: Every standard WebSocket opcode and both handshake forms pass controlled interoperability tests, while every required malformed-frame class fails with one exact terminal outcome.
- **SC-003**: Per-message compression tests preserve byte-identical forwarded messages and either produce bounded derived content matching the source or an exact named failure while retaining authorized raw evidence.
- **SC-004**: An SSE soak test processes at least 10,000 dispatched events across arbitrary body-segment boundaries with bounded memory, no forwarding delay attributable to artifact I/O, and complete field, event, and loss reconciliation.
- **SC-005**: SSE grammar tests preserve 100 percent of raw source bytes and produce protocol-correct results for comments, multiline data, event, id, retry, line endings, byte-order mark, malformed text, cancellation, and incomplete termination.
- **SC-006**: Unary, client-streaming, server-streaming, and bidirectional gRPC tests preserve 100 percent of message boundaries, direction, compression flags, declared sizes, raw retained payloads, metadata, and observed trailer status.
- **SC-007**: Oversized, partial, compressed, cancelled, reset, malformed, and missing-trailer gRPC cases each produce a stable exact outcome with no schema inference or fabricated protobuf fields.
- **SC-008**: Ten repeated mixed-protocol start, traffic, stop, and cleanup cycles leave zero owned protocol tasks and exactly one terminal state for every accepted stream and started derived unit.
- **SC-009**: Under forced observation-queue saturation and writer failure, forwarded valid traffic remains byte-identical while 100 percent of missing evidence is assigned to named counters and the artifact cannot be classified complete.
- **SC-010**: The complete repository verification suite passes with no Internet, game account, capture driver, elevation, or persistent trust-store mutation required.

## Assumptions

- S105 HTTP/1.1, HTTPS, HTTP/2, metadata, streaming body, and live application-record contracts are the starting boundary.
- Deep Capture has already been selected explicitly and the existing session capability, destination policy, certificate, and cleanup boundaries remain authoritative.
- Raw observed bytes and frames are the authority. Reassembled WebSocket messages, parsed SSE events, and parsed gRPC envelopes are derived observations linked back to their source evidence.
- Payload authorization and metadata-only scope apply identically across all three protocol families.
- WebSocket per-message compression is the only extension required by this slice; unsupported extensions remain explicit refusals or omissions.
- gRPC framing is observable without protobuf schemas. Application message decoding and schema discovery are out of scope.

## Scope and Traceability

### Included

- Issue #295, WebSocket handshake, frames, messages, compression, validation, and accounting.
- Issue #298, bounded incremental Server-Sent Events observations.
- Issue #299, gRPC metadata, framing, streaming observations, and terminal outcomes.
- Specification sections 13.7, 19.6, 25, and 28.1 where application protocol support changes.

### Excluded

- Native proxy-owned TLS key logs (#300).
- Complete HAR 1.2 projection (#302).
- Packet, flow, process, role, and stream correlation completion (#303).
- Client-certificate forwarding and final TLS refusal classification (#304).
- Final HTTP/TLS interoperability matrix (#305).
- Versioned native manifests and complete proxy or cleanup sidecars (#335 and #336).
- Launch and transport coverage in later native Deep Capture milestones.

### Done When

- Every acceptance criterion in #295, #298, and #299 is met by implementation and controlled evidence.
- Security, loss-accounting, interoperability, lifecycle, bounded-resource, and long-lived-stream tests pass.
- The master specification and public support language describe exactly the newly shipped boundary without implying completion of deferred issues.
- All repository gates pass and no Critical analyze finding remains.
