# Feature Specification: HTTP/2, Metadata, and Streaming Bodies

**Feature Branch**: `codex/105-http2-metadata-bodies`

**Created**: 2026-08-30

**Status**: Complete

**Input**: User description: "Kick off S105 with spec-kit and run through everything end-to-end with autopilot"

## Clarifications

### Session 2026-08-30

- Q: What durable authority retains streaming bodies before the final application stream work? -> A: Include #301 in S105.
- Q: What metadata order is authoritative after HTTP/2 header decompression? -> A: Preserve exposed semantic order and record unavailable wire order.

## User Scenarios & Testing

### User Story 1 - Inspect Multiplexed HTTP/2 Correctly (Priority: P1)

An authorized operator runs Deep Capture against a compatible target that uses HTTP/2 and receives distinct, correctly paired observations for every concurrent request and response without degrading the connection to HTTP/1.1.

**Why this priority**: HTTP/2 is the transport foundation for several remaining milestone protocols. Incorrect stream pairing would produce plausible but false evidence.

**Independent Test**: A controlled client opens one proxy-routed HTTP/2 connection, overlaps multiple streams, resets one stream, and completes the others. The resulting observations retain one connection identity, distinct stream identities, the correct request and response pairing, and exact terminal outcomes.

**Acceptance Scenarios**:

1. **Given** an authorized TLS session whose client and origin negotiate HTTP/2, **When** several requests overlap on one connection, **Then** each request remains paired with its own response and every observation carries both connection and stream identity.
2. **Given** one active stream is reset or cancelled, **When** other streams continue, **Then** the affected stream records its terminal reason and unrelated streams can complete.
3. **Given** the peer sends trailers, GOAWAY, or a connection error, **When** the connection ends, **Then** stream and connection outcomes remain distinct and all accepted work reaches a terminal state.

---

### User Story 2 - Preserve Complete HTTP Metadata (Priority: P1)

An analyst can inspect request and response metadata from HTTP/1.1 and HTTP/2 without losing duplicates, raw value bytes, protocol-specific structure, trailers, cookies, query values, or observed reason details. HTTP/1.1 retains wire field order and casing. HTTP/2 retains pseudo-field structure, binary-safe values, and duplicate value order exposed by the protocol engine while marking original compressed cross-name order unavailable.

**Why this priority**: Method, URL, and status alone are insufficient for debugging authentication, negotiation, caching, and application behavior. Normalizing metadata can silently change its meaning.

**Independent Test**: Controlled HTTP/1.1 and HTTP/2 exchanges use duplicate fields, mixed casing where the protocol exposes it, binary-safe values, repeated query keys, cookies, and trailers. Raw observations round-trip exactly, while convenience projections trace back to those raw values.

**Acceptance Scenarios**:

1. **Given** a request or response with repeated fields, **When** it is observed, **Then** duplication and raw value bytes remain representable, HTTP/1.1 wire order remains exact, and HTTP/2 ordering fidelity is stated explicitly.
2. **Given** HTTP/2 pseudo-headers, **When** metadata is exported, **Then** they remain protocol-specific fields and are never presented as fabricated HTTP/1.1 lines.
3. **Given** query entries, cookies, or trailers, **When** parsed conveniences are produced, **Then** every convenience value can be traced to retained raw observed metadata and any decode failure remains explicit.

---

### User Story 3 - Retain Bounded Streaming Bodies (Priority: P1)

An operator who selected payload capture receives request and response body bytes as they pass through the proxy, including long-lived and partial streams, while memory and storage remain bounded and every omission or transformation is reported.

**Why this priority**: Deep Capture cannot claim full HTTP inspectability while body data disappears silently. Forwarding must remain reliable even when observation storage is slow, full, or unavailable.

**Independent Test**: Controlled exchanges cover fixed-length, chunked, compressed, oversized, indefinite, cancelled, malformed, and metadata-only bodies. Forwarded bytes remain correct, retained raw bytes obey configured limits, and counters reconcile every retained, truncated, omitted, decode-failed, or storage-failed byte.

**Acceptance Scenarios**:

1. **Given** payload capture is authorized, **When** a body is transferred, **Then** original observed bytes are retained up to explicit message, session, and storage limits without delaying forwarding indefinitely.
2. **Given** transfer framing or content encoding is decoded, **When** derived content is recorded, **Then** raw bytes remain authoritative and each transformation names its input, output, encoding, and outcome.
3. **Given** a body exceeds a limit or storage fails, **When** forwarding continues, **Then** truncation or loss is counted, located, and surfaced rather than mislabeled as a complete body.
4. **Given** metadata-only scope, **When** a body passes through the proxy, **Then** no payload artifact is retained and the intentional omission is recorded separately from loss.

---

### User Story 4 - Consume a Crash-Readable Application Stream (Priority: P2)

A machine consumer can read native connection, TLS, HTTP, metadata, body, error, gap, and terminal records while the session runs, identify the schema version, and reconcile a complete file or a crash-readable incomplete prefix.

**Why this priority**: Streaming bodies need one durable authority. A live application stream avoids unbounded in-memory finalization and gives later projections a stable source without claiming that deferred protocol semantics already exist.

**Independent Test**: A controlled session is read during traffic, after orderly completion, and after forced writer interruption. Each readable prefix starts with a versioned header, contains deterministic records, and either ends in a reconciling trailer or remains explicitly incomplete.

**Acceptance Scenarios**:

1. **Given** an active Deep Capture session, **When** native observations occur, **Then** application records are appended during the session rather than accumulated solely for finalization.
2. **Given** an orderly session end, **When** a consumer reads the stream, **Then** its trailer reconciles records, bytes, gaps, truncations, and writer failures.
3. **Given** a process or writer interruption, **When** a consumer reads the surviving prefix, **Then** framing remains valid and the absence of a complete trailer cannot be mistaken for success.
4. **Given** an event from a deferred protocol family, **When** the schema is evaluated, **Then** the family has a reserved documented record shape or an explicit non-export reason without claiming implementation support.

### Edge Cases

- Concurrent streams complete out of order, reuse priorities, or are reset while the connection remains healthy.
- A GOAWAY boundary permits some streams to finish while refusing later stream identifiers.
- Header blocks contain duplicates, empty values, binary-safe HTTP/2 values, or exceed configured bounds.
- Informational responses precede a final response and each carries its own metadata.
- A request is rejected before its body is consumed, including an expectation handshake.
- Transfer framing ends early, contains malformed chunks, or conflicts with declared length.
- Content decoding is truncated, malformed, expands beyond its bound, or uses an unsupported encoding.
- Observation queues or artifact storage saturate while network forwarding remains active.
- A client or origin half-closes during a body, and cleanup interrupts a long-lived stream.

## Requirements

### Functional Requirements

- **FR-001**: Deep Capture MUST negotiate and proxy HTTP/2 over authorized client-facing TLS and separately verified upstream TLS without silently downgrading a connection that selected HTTP/2.
- **FR-002**: The proxy MUST model connection identity and stream identity separately and MUST preserve deterministic request and response pairing across concurrent streams.
- **FR-003**: The proxy MUST enforce finite concurrent-stream, flow-control, header, message, observation, memory, storage, idle, and shutdown bounds.
- **FR-004**: Stream reset, cancellation, refusal, end-of-stream, GOAWAY, protocol error, transport error, and connection shutdown MUST produce distinct terminal evidence.
- **FR-005**: Backpressure or failure on one HTTP/2 stream MUST NOT indefinitely block unrelated streams, and any resulting observation loss MUST be counted.
- **FR-006**: Server push MUST be refused explicitly and recorded; the proxy MUST NOT claim support for pushed resources in this slice.
- **FR-007**: HTTP observations MUST retain duplicates, raw field-name and field-value evidence available at the observed boundary, protocol version, trailers, informational responses, and reason details where the protocol supplies them. HTTP/1.1 MUST retain wire field order and casing. HTTP/2 MUST retain typed pseudo-fields, binary-safe regular values, duplicate value order, and explicit provenance that original compressed cross-name order is unavailable.
- **FR-008**: HTTP/2 pseudo-headers MUST remain distinguishable from ordinary fields, and the system MUST NOT fabricate HTTP/1.1 syntax, casing, reason phrases, or ordering that HTTP/2 did not provide.
- **FR-009**: Cookie, query, and other parsed conveniences MUST be derived from retained raw observations, MUST preserve repeated values, and MUST expose decoding uncertainty or failure.
- **FR-010**: Sensitive metadata MUST retain its existing artifact sensitivity classification and MUST never enter human logs or diagnostics merely because it is now observable.
- **FR-011**: When payload capture is authorized, request and response bodies MUST be observed incrementally and retained without requiring the whole body in memory.
- **FR-012**: Raw observed body bytes MUST remain authoritative. Transfer decoding and content decoding MUST be recorded as separate derived transformations with explicit provenance and outcomes.
- **FR-013**: Fixed-length, chunked, streaming, partial, cancelled, and connection-delimited bodies MUST have exact completion states.
- **FR-014**: gzip, deflate, and Brotli content MUST be decoded when selected and within bounds; unsupported, malformed, truncated, or expansion-limited content MUST preserve raw bytes and record an explicit decode outcome.
- **FR-015**: Per-message, per-session, disk, queue, decompression-expansion, and time limits MUST be explicit, finite, testable, and included in terminal accounting.
- **FR-016**: Metadata-only scope MUST retain no body payload and MUST record an intentional scope omission that is distinct from truncation, queue loss, parse failure, decode failure, and storage failure.
- **FR-017**: Every accepted connection, stream, metadata block, body segment, transformation, refusal, truncation, omission, and loss MUST reconcile through stable counters and terminal evidence.
- **FR-018**: Observation or artifact failure MUST NOT corrupt forwarded bytes, fabricate completeness, or discard already retained evidence.
- **FR-019**: Shutdown MUST cancel, drain, or forcibly terminate all stream and body work within the session budget and MUST leave no detached task or unreported partial record.
- **FR-020**: The controlled protocol lab MUST cover HTTP/2 multiplexing, resets, trailers, GOAWAY, malformed input, metadata fidelity, body framing, supported content decoding, bounds, backpressure, cancellation, and cleanup without Internet access or privileged access.
- **FR-021**: HTTP/1.1 behavior delivered by S104 MUST remain interoperable, including informational responses, expectation handling, upgrades, trailers, persistent connections, half-close, and framing refusal.
- **FR-022**: Application JSON Lines MUST be a versioned, append-only, crash-readable stream written during the session rather than a final projection from an in-memory observation set.
- **FR-023**: The application stream MUST define records for session, connection, TLS, HTTP stream, metadata, body segment, transformation, error, gap, and trailer evidence, plus reserved documented families or explicit non-export reasons for WebSocket, Server-Sent Events, gRPC, generic TCP, UDP, and QUIC observations.
- **FR-024**: Every application record MUST carry available session, target, proxy connection, HTTP stream, packet flow, process, role, attribution, timing, protocol version, scope, truncation, and loss anchors without inventing unavailable values.
- **FR-025**: Application records MUST have deterministic ordering rules, valid prefix framing, a schema version distinguishable from the product version, and a terminal trailer that reconciles record, byte, gap, truncation, and writer-failure counts.
- **FR-026**: A failed application writer MUST be retired and surfaced without blocking proxy forwarding, erasing already written records, or allowing the session to claim a complete application artifact.
- **FR-027**: Golden, round-trip, malformed-record, partial-prefix, and compatibility tests MUST bind the documented application schema to its serializer and reader guidance.
- **FR-028**: This slice MUST NOT implement WebSocket frame capture, Server-Sent Events parsing, gRPC message parsing, HAR completion, client certificate forwarding, generic transport inspection, or feature-completion claims.

### Key Entities

- **Proxy connection**: One admitted client connection with a stable identity, negotiated protocol, endpoints, lifecycle, and zero or more streams.
- **HTTP stream**: One request and response exchange within a proxy connection, with its own identity, metadata, body directions, timing, and terminal outcome.
- **Metadata block**: An ordered, protocol-aware collection of raw observed fields plus optional traceable conveniences.
- **Body segment**: A bounded ordered range of raw observed bytes with direction, offset, timing, storage outcome, and completeness state.
- **Transformation record**: Provenance linking raw bytes to a transfer-decoded or content-decoded result and its exact outcome.
- **Observation account**: Reconciliation of accepted, retained, omitted, truncated, failed, and dropped protocol evidence.
- **Application record**: One versioned append-only machine record carrying raw or derived native protocol evidence and all available correlation and completeness anchors.

## Success Criteria

### Measurable Outcomes

- **SC-001**: A controlled test with at least 32 overlapping streams on one connection preserves 100 percent correct request and response pairing, including reset and out-of-order completion cases.
- **SC-002**: Every accepted stream and connection in the controlled matrix has exactly one terminal outcome, with zero task residue after ten repeated start, traffic, stop, and cleanup cycles.
- **SC-003**: Metadata round-trip tests preserve 100 percent of HTTP/1.1 ordered raw fields and 100 percent of HTTP/2 typed pseudo-fields, duplicate value order, trailers, and binary-safe regular values, with no claim of unavailable compressed cross-name order.
- **SC-004**: Large and indefinite body tests keep memory within declared bounds while forwarded bytes remain byte-identical for every non-refused exchange.
- **SC-005**: Accounting tests reconcile 100 percent of body bytes and protocol observations as retained, intentionally omitted, truncated, decode-failed, storage-failed, or dropped, with no unnamed remainder.
- **SC-006**: Controlled gzip, deflate, and Brotli cases either produce bounded derived content matching the source or an exact failure outcome while retaining the authorized raw observation.
- **SC-007**: The complete repository verification suite passes on supported Windows and portable test paths with no Internet, game account, capture driver, elevation, or trust-store mutation required.
- **SC-008**: Existing HTTP/1.1, HTTPS, lifecycle, security, and cleanup regression suites retain their prior outcomes.
- **SC-009**: Every orderly controlled session produces a schema-valid application stream whose trailer reconciles exactly, and every forced interruption leaves a parseable prefix that cannot be classified as complete.

## Assumptions

- S104 HTTP/1.1, CONNECT, TLS, session authentication, destination policy, and lifecycle contracts are the starting boundary.
- The operator has explicitly selected Deep Capture, authorized the session effects, and selected whether body payloads may be retained.
- Raw observations are the authority; parsed metadata and decoded content are conveniences that never replace them.
- HTTP/2 cleartext upgrade support is limited to traffic explicitly routed through the authorized proxy and is not a system-wide interception mechanism.
- Existing packet capture and process attribution remain separate from proxy protocol handling.
- Issues #295, #298 through #300, #302 through #305, #335, and #336 remain open after this slice unless their individual acceptance criteria are independently satisfied.

## Scope and Traceability

### Included

- Issue #294, HTTP/2 multiplexing and event fidelity.
- Issue #296, complete HTTP metadata without normalization loss.
- Issue #297, bounded streaming bodies and decoding provenance.
- Issue #301, complete versioned streaming application JSON Lines.
- Specification sections 13.7, 19.6, 25, and 28.1 where shipped protocol truth changes.

### Excluded

- WebSocket frames and messages (#295).
- Server-Sent Events and gRPC semantic parsing (#298 and #299).
- TLS key-log artifact completion (#300).
- HAR and cross-artifact correlation contracts (#302 and #303).
- Client certificates and final HTTP/TLS conformance (#304 and #305).
- Versioned native manifest and complete proxy or cleanup sidecars (#335 and #336).

### Done When

- Every acceptance criterion in #294, #296, #297, and #301 is met by implementation and evidence.
- Security, loss-accounting, interoperability, lifecycle, and bounded-resource tests pass.
- The master specification and public support language describe exactly the newly shipped boundary without closing or implying completion of deferred issues.
- All repository gates pass and no Critical analyze finding remains.
