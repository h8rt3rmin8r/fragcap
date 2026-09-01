# Feature Specification: Correlated Native Evidence

**Feature Branch**: `codex/108-correlated-native-evidence`

**Created**: 2026-09-01

**Status**: Draft

**Input**: User description: "Spec out S108 with spec-kit, then implement it end-to-end under autopilot. Close the largest coherent set of Deep Capture work by correlating native proxy observations with packet and process evidence, producing truthful HAR 1.2, and versioning the native bundle manifest and artifact authority."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Trace Application Evidence to Its Owner (Priority: P1)

An authorized operator can follow every native proxy observation back to the exact proxy connection, application stream, packet flow, process, and role when those facts were observed. When a join cannot be established, the evidence carries a stable state and reason instead of a guessed identity.

**Why this priority**: Correlation is the provenance boundary for every derived application artifact. HAR and the durable manifest cannot be authoritative until this join is exact and honest.

**Independent Test**: Replay controlled IPv4 and IPv6 connection histories containing multiplexed streams, endpoint reuse, retained attribution, missing packet evidence, and timing races. Verify that permutations produce the same joins, uncertainty reasons, and reconciled counts.

**Acceptance Scenarios**:

1. **Given** one accepted proxy connection and a matching captured flow, **when** HTTP/1.1 observations are recorded, **then** every record carries the same stable flow, connection, process, role, and packet-side join anchors.
2. **Given** concurrent HTTP/2 or gRPC streams on one connection, **when** their observations are recorded, **then** they share the connection and flow anchors while retaining distinct stream and message identities.
3. **Given** a local endpoint is reused after process exit, **when** old and new observations are joined, **then** creation time and retention state prevent ownership from transferring between processes.
4. **Given** a join is unavailable or ambiguous, **when** an observation is written, **then** it carries a stable unavailable state and exact reason without an invented process, role, or flow identifier.
5. **Given** a protocol or transport handler does not yet exist in the product, **when** the correlation contract is evaluated, **then** it represents the transport without claiming observations the product cannot produce.

---

### User Story 2 - Open a Truthful HTTP Archive (Priority: P2)

An operator can open a completed Deep Capture HTTP archive in a standard HAR reader and inspect only the HTTP facts that fragcap actually observed. Partial, failed, binary, large, or interrupted transactions remain identifiable and never acquire placeholder measurements.

**Why this priority**: HAR is the interoperable human-facing projection of the native application evidence, but false status, size, timing, or body values would violate the instrument's trust contract.

**Independent Test**: Project a controlled set of complete and partial HTTP/1.1 and HTTP/2 exchanges into HAR, including duplicate headers, cookies, query parameters, request bodies, redirects, binary bodies, truncation, errors, and interruption. Validate the document with independent HAR readers and trace every populated value to source evidence.

**Acceptance Scenarios**:

1. **Given** a complete observed HTTP transaction, **when** HAR is finalized, **then** method, URL, protocol, headers, cookies, query, post data, response status, content, redirects, sizes, timings, errors, and correlation extensions are populated only where evidence supports them.
2. **Given** a response is missing or required timing evidence is unavailable, **when** HAR is finalized, **then** the transaction remains in a namespaced archive extension with exact missing and loss reasons rather than entering the standard entry list with placeholders.
3. **Given** a standard entry has a truncated or omitted body but all mandatory transaction facts are observed, **when** HAR is finalized, **then** the standard entry remains present and its namespaced provenance declares the body limitation without a placeholder byte count or completion claim.
4. **Given** observed content is binary, **when** it is retained within the evidence bound, **then** it uses the declared binary encoding; when it exceeds the bound, the retained prefix and loss are reported without affecting forwarding.
5. **Given** the session is interrupted before orderly finalization, **when** the bundle is inspected, **then** no incomplete HAR file is labeled complete.

---

### User Story 3 - Audit the Native Bundle Contract (Priority: P3)

An operator or downstream reader can inspect a versioned manifest and determine the authority, sensitivity, content type, completeness, loss, and correlation capability of every native artifact and omission. Existing manifest version 1 bundles remain readable and unchanged.

**Why this priority**: The manifest is the durable index that makes packet truth, native application truth, HAR projections, key logs, and lifecycle sidecars interpretable together over time.

**Independent Test**: Serialize, validate, parse, and round-trip complete, partial, failed, and crash-prefix version 2 manifests; read committed version 1 fixtures; reject contradictory authority or completion claims; and validate examples against the published machine-readable schema.

**Acceptance Scenarios**:

1. **Given** a native session with packet, application, HAR, key-log, and lifecycle artifacts, **when** its manifest is finalized, **then** every produced artifact and omission has exactly one authority owner and an explicit completeness and sensitivity state.
2. **Given** a version 1 bundle, **when** a current reader opens it, **then** it is interpreted through its original contract and its files are never rewritten automatically.
3. **Given** a missing trailer, counted loss, truncation, writer failure, or missing dependency artifact, **when** manifest state is computed, **then** neither the affected artifact nor the bundle is labeled complete.
4. **Given** a schema example or serialized manifest, **when** it is validated independently, **then** schema version, product version, parser, serializer, examples, and documentation agree.

### Edge Cases

- Multiple client connections reuse the same local endpoint within the attribution retention window.
- An application event arrives before the matching packet flow is published, or after process exit.
- One HTTP/2 connection carries many simultaneous streams and interleaved gRPC messages.
- IPv4-mapped IPv6 addresses, IPv6 scope identifiers, and direction reversal could otherwise produce unequal endpoint keys.
- The packet capture is absent, starts late, stops early, or reports loss while application observations continue.
- Headers contain duplicates, empty values, non-text bytes, or values that cannot be represented by a narrower projection.
- Request or response metadata is incomplete, informational responses precede a final response, or trailers arrive after body data.
- Body evidence is binary, compressed, decoded with failure, truncated by evidence bounds, or absent by operator scope.
- HAR finalization fails after application JSON Lines has a valid readable prefix.
- A manifest version is unsupported, malformed, internally contradictory, or references an unsafe path.
- A crash leaves a valid application JSON Lines prefix but no reconciling trailer and no final manifest publication.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Every native proxy observation MUST carry one explicit correlation state and stable reason.
- **FR-002**: Correlated observations MUST preserve session, target, proxy connection, stream, message, flow, process, role, attribution fidelity, protocol, event-time, and loss anchors whenever observed.
- **FR-003**: The join MUST use accepted client endpoint identity and capture-side flow history without opening target process handles.
- **FR-004**: Endpoint normalization and joining MUST support IPv4 and IPv6 and MUST distinguish transport protocol and direction.
- **FR-005**: Multiplexed streams and messages MUST share their owning connection and flow while retaining distinct stream and message identities.
- **FR-006**: Endpoint reuse, creation time, retained attribution, process exit, late publication, and observation races MUST NOT transfer identity incorrectly.
- **FR-007**: Equivalent input histories presented in any order permitted by the contract MUST produce the same correlation result.
- **FR-008**: Packet-side, application-side, correlated, unavailable, ambiguous, and lost observation counts MUST reconcile through named counters.
- **FR-009**: The correlation contract MUST represent current and planned transports without claiming that an unimplemented handler emitted evidence.
- **FR-010**: HAR output MUST conform to HAR 1.2 and contain only values traceable to native application evidence.
- **FR-011**: Standard HAR entries MUST project observed request and response metadata, headers, cookies, query entries, post data, content, redirects, protocol, sizes, timings, and errors only when all mandatory standard values are observed.
- **FR-012**: HAR output MUST preserve transactions that cannot form a standard entry in a namespaced extension carrying exact missing, error, and loss reasons, and MUST NOT use placeholder status, size, timing, body, or completion values.
- **FR-013**: Binary HAR content MUST use an explicit encoding, and large or indefinite content MUST obey a finite evidence bound with declared truncation and loss.
- **FR-014**: HAR projection and finalization MUST operate within fixed memory and disk bounds independent from traffic forwarding capacity.
- **FR-015**: An interrupted or failed HAR writer MUST leave no artifact or manifest claim that can be mistaken for an orderly complete archive.
- **FR-016**: The native bundle manifest MUST use a schema version distinct from the product version.
- **FR-017**: The version 2 manifest MUST index every native artifact and omission with exactly one authority owner, content type, sensitivity, completeness, loss, and correlation capability.
- **FR-018**: Manifest state MUST distinguish complete, partial, failed, and crash-prefix evidence and MUST reject contradictory claims.
- **FR-019**: Current readers MUST continue to read version 1 manifests without silently rewriting them.
- **FR-020**: The repository MUST publish a machine-readable version 2 manifest schema, valid examples, and matching parser, serializer, and round-trip tests.
- **FR-021**: Artifact paths MUST remain normalized, relative, contained within the bundle, and unique.
- **FR-022**: Raw packet and application observations MUST remain authoritative over HAR and other derived projections.
- **FR-023**: Application JSON Lines MUST remain a readable prefix after interruption, and orderly completion MUST require its reconciling trailer.
- **FR-024**: Tests MUST cover deterministic permutations, timing races, endpoint reuse, multiplexing, IPv4 and IPv6, partial HTTP transactions, bounded binary bodies, old manifests, schema conformance, and failure injection without accounts, Internet access, elevation, a game, or a capture driver.
- **FR-025**: Documentation and user-visible status MUST state the exact implemented transport scope and MUST NOT call Deep Capture feature-complete.

### Key Entities

- **Correlation Identity**: A normalized connection identity combining transport, client and proxy endpoints, direction, and accepted-connection time.
- **Correlation Result**: The observed anchors, attribution fidelity, state, reason, and accounting contribution for one proxy observation.
- **HTTP Transaction Projection**: One request and zero or more response phases assembled from authoritative application records for HAR projection.
- **HAR Artifact**: A bounded finalized HTTP archive whose entries retain partiality, provenance, and correlation extensions.
- **Bundle Manifest Version 2**: The durable native bundle index, distinct from product version, containing artifact and omission declarations.
- **Artifact Declaration**: One path or omission with an authority owner, role, content type, sensitivity, completeness, loss, and correlation capability.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: One hundred percent of emitted native proxy observations carry a correlation state and stable reason.
- **SC-002**: All tested permutations and race schedules produce byte-equivalent correlation decisions and accounting.
- **SC-003**: No endpoint-reuse or process-exit case in the controlled matrix transfers a flow or process identity incorrectly.
- **SC-004**: Every populated HAR field in the conformance corpus traces to at least one authoritative application record, with zero placeholder measurements.
- **SC-005**: Standard independent HAR readers accept every complete generated archive in the conformance corpus.
- **SC-006**: Large and indefinite bodies remain forwardable while retained HAR and assembly memory stay within configured finite bounds.
- **SC-007**: Every complete, partial, failed, and crash-prefix example validates or fails exactly as declared by the version 2 schema and reader.
- **SC-008**: All committed version 1 manifest fixtures remain readable and byte-identical after inspection.
- **SC-009**: Every produced native artifact and every expected omission has exactly one declared authority owner.
- **SC-010**: The complete repository verification gate passes with no new dependency package and no forbidden capability.

## Assumptions

- S108 closes issues #303, #302, and #335 as one dependency-ordered slice: correlation first, HAR second, manifest authority last.
- Existing native HTTP/1.1, HTTPS, HTTP/2, WebSocket, SSE, gRPC, application JSON Lines, TLS key-log, and sensitive-artifact capabilities are reused.
- Generic UDP, SOCKS UDP, QUIC, HTTP/3, and full IPv6 product parity remain owned by later transport issues. S108 makes correlation types and reasons capable of representing them but does not fabricate observations from handlers that do not yet exist.
- Application JSON Lines version 2 and packet `.fcapng` remain authoritative evidence. HAR is a derived projection.
- Existing version 1 bundles are migration inputs only. They are read through a compatibility model and are never modified in place.
- General crash recovery and proxy or cleanup sidecar completion remain owned by issues #320 and #336.
- No new third-party package is expected; any proposed dependency requires a fresh license, MSRV, and capability review.
