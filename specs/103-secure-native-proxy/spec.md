# Feature Specification: Secure Native Proxy Foundation

**Feature Branch**: `codex/103-native-proxy-foundation-completion`

**Created**: 2026-08-30

**Status**: Complete

**Input**: Complete the native Deep Capture foundation by resolving issues #283 through #289 as one dependency-ordered slice: listener isolation and authentication, bounded upstream connection policy, native certificate authority and leaf lifecycle, exact Windows trust mutation, loss-accounted observations, and a deterministic protocol lab.

## Clarifications

### Session 2026-08-30

- Q: How does one session capability authenticate clients across future proxy protocols? -> A: Use each protocol's standard authentication field where one exists, compare one opaque session capability in constant time, and refuse protocols that cannot carry it until they have an equally scoped adapter.
- Q: May upstream policy connect to private or loopback destinations? -> A: Refuse them by default; permit only explicit, exact test-destination grants, while always refusing the inspection listener itself and any address change outside the granted set.
- Q: Is certificate-authority identity reusable across sessions? -> A: Generate one purpose-specific authority per session, never reuse its private key, and keep trust as a separate explicitly authorized action tied to that exact thumbprint.
- Q: Which observation is discarded when a bounded queue is full? -> A: Drop the oldest queued observation, preserve monotonic loss accounting, and mark downstream completeness false for the affected stream.

## User Scenarios & Testing

### User Story 1 - Admit only the selected session (Priority: P1)

As an authorized operator, I can start a local inspection listener that accepts traffic only from the selected Deep Capture session and never exposes a proxy to the network or unrelated local software.

**Why this priority**: A listener that any local or remote client can use violates the selected-session scope before inspection begins.

**Independent Test**: Start the listener with one session capability, attempt connections from the authorized client, an unrelated local client, both loopback families, and a remote interface, then compare admitted and refused outcomes with the reported counters.

**Acceptance Scenarios**:

1. **Given** a listener bound for one Deep Capture session, **When** the session presents its capability, **Then** the connection is admitted exactly once and the capability is not exposed in logs or observations.
2. **Given** an unrelated local client, **When** it omits or supplies the wrong capability, **Then** it is refused before upstream work or payload collection and the refusal is counted.
3. **Given** any remote-interface connection attempt, **When** it targets the listener port, **Then** it cannot reach the listener.
4. **Given** a listener stop and immediate port reuse, **When** a stale client races the replacement listener, **Then** it cannot authenticate to the new session.

---

### User Story 2 - Connect upstream without weakening policy (Priority: P1)

As an operator, I can trust the native proxy to resolve and connect only to the requested, permitted destination under finite budgets, with certificate and hostname verification enabled and exact failures reported.

**Why this priority**: Native protocol handlers cannot safely forward traffic until destination parsing, name resolution, connection policy, and cancellation behavior are bounded and explicit.

**Independent Test**: Exercise valid and invalid authorities, IPv4 and IPv6 results, rebinding, listener recursion, private destinations, certificate failures, timeouts, and cancellation against controlled local resolvers and origins.

**Acceptance Scenarios**:

1. **Given** a valid permitted authority, **When** resolution and connection succeed within budget, **Then** the exact selected address, authority, verification policy, and timing outcome are observable.
2. **Given** an invalid authority, listener recursion, prohibited address transition, or policy-refused destination, **When** connection is requested, **Then** no upstream socket is returned and a typed refusal identifies the stage.
3. **Given** an upstream certificate or hostname mismatch, **When** secure connection is attempted, **Then** verification fails without downgrade or fabricated application status.
4. **Given** cancellation or any exhausted DNS, connect, read, or write budget, **When** work stops, **Then** sockets and tasks are released within the configured shutdown budget.

---

### User Story 3 - Own certificate and trust state exactly (Priority: P1)

As an authorized Windows operator, I can create session-purpose certificate material, explicitly trust only that certificate, issue bounded host leaves, and remove exactly the state the session owns without external certificate commands.

**Why this priority**: Native TLS inspection requires private material and machine trust to have one protected, auditable, reversible owner.

**Independent Test**: Create synthetic session certificate material in owned storage, inspect its protection and provenance, issue and evict concurrent leaf certificates, add/query/remove the exact certificate in a controlled current-user store, and inject partial, duplicate, mismatch, and access-denied failures.

**Acceptance Scenarios**:

1. **Given** a new session identity, **When** certificate material is created, **Then** the purpose, identity, validity, thumbprint, storage protection, and cleanup inventory are recorded without revealing private keys.
2. **Given** validated DNS names or IP addresses, **When** leaf certificates are requested concurrently, **Then** each certificate is valid only for its requested identities and cache count, memory, and lifetime remain within declared bounds.
3. **Given** explicit authorization to trust one session certificate, **When** trust is added, queried, or removed, **Then** only the exact current-user Root entry with the authorized thumbprint changes.
4. **Given** duplicate, wrong-store, mismatch, denial, interruption, or crash residue, **When** state is inspected or cleaned, **Then** the condition is typed, unrelated certificates remain unchanged, and owned residue remains discoverable.

---

### User Story 4 - Preserve every native observation and gap (Priority: P1)

As an operator or downstream consumer, I receive one versioned stream of lifecycle, connection, DNS, transport, security, application, refusal, error, and loss observations without mistaking missing data for complete inspection.

**Why this priority**: Protocol work cannot land safely if handlers use incompatible records or if overflow and truncation disappear silently.

**Independent Test**: Produce each event family, malformed and unknown records, concurrent ordered events, queue overflow, payload truncation, and refused input, then round-trip the stream and prove conservation between admitted inputs, emitted events, and named loss counters.

**Acceptance Scenarios**:

1. **Given** events from multiple native components, **When** they enter the observation stream, **Then** stable identifiers, timestamps, ordering keys, provenance, and payload ownership allow deterministic correlation.
2. **Given** queue or payload limits, **When** data exceeds them, **Then** dropped, truncated, refused, and unparsed counts identify the loss and prevent a full-inspectability claim.
3. **Given** an unknown event version or malformed observation, **When** a consumer reads it, **Then** the raw condition remains representable without rewriting it into a known success.
4. **Given** raw observations and an output projection, **When** projection cannot represent a record, **Then** the raw record remains available and the projection gap is explicit.

---

### User Story 5 - Prove the foundation in a controlled protocol lab (Priority: P1)

As a maintainer, I can validate the native foundation against deterministic local clients and origins for every required protocol family and failure class without Internet access, accounts, elevation, a game, or real traffic.

**Why this priority**: The production cutover and later protocol slices need one reusable source of packet truth, application truth, timing, failures, and cleanup expectations.

**Independent Test**: Run the offline lab matrix for HTTP, HTTPS, HTTP/2, WebSocket, streaming HTTP, gRPC, raw TCP, non-HTTP TLS, SOCKS, UDP, and QUIC, including positive fixture generation plus refusal, malformed, timeout, cancellation, and cleanup cases.

**Acceptance Scenarios**:

1. **Given** any required protocol family, **When** its controlled client and origin run, **Then** the lab produces deterministic synthetic payload, timing, connection, certificate, and shutdown truth without claiming unsupported proxy inspection.
2. **Given** a malformed, refused, timed-out, cancelled, disconnected, or cleanup-failure scenario, **When** the lab runs it repeatedly, **Then** the same typed outcome and expected accounting result are produced.
3. **Given** packet, event, artifact, key-log, or cleanup output, **When** the lab compares it with source truth, **Then** mismatches and intentionally unavailable outputs are distinguished.
4. **Given** Windows-only trust behavior, **When** portable lab checks run elsewhere, **Then** the platform-specific entry point is isolated and the portable matrix remains runnable.

### Edge Cases

- An authentication capability is empty, malformed, replayed after cleanup, logged accidentally, or presented across a listener-generation boundary.
- IPv4 and IPv6 loopback endpoints coexist, one family is unavailable, or a wildcard-mapped address could widen exposure.
- A destination authority contains user information, ambiguous delimiters, an invalid port, an internationalized name, or an address literal with a zone identifier.
- Resolution returns no address, duplicate addresses, mixed families, the listener itself, or an allowed result followed by a prohibited rebinding result.
- Cancellation races DNS completion, connection establishment, secure negotiation, an active read or write, or runtime shutdown.
- Certificate creation stops between key, certificate, metadata, and permission writes; cleanup must distinguish complete from partial state.
- Leaf requests race for the same identity, exceed count or byte limits, outlive the signing certificate, or arrive after policy or CA rotation.
- Trust stores contain an exact duplicate, a same-subject different-key certificate, the authorized certificate in the wrong store, or an inaccessible store.
- Observation sequence numbers approach exhaustion, clocks move backward, producers race shutdown, or loss reporting itself reaches capacity.
- A protocol lab fixture resembles a real credential, address, account identifier, or certificate and must be rejected from the committed corpus.

## Requirements

### Functional Requirements

- **FR-001**: The inspection listener MUST be reachable only through explicit IPv4 or IPv6 loopback endpoints and MUST never bind a wildcard or remote interface.
- **FR-002**: Every admitted client MUST prove possession of session-specific, unguessable authentication material before any upstream connection or application payload collection begins.
- **FR-002a**: Supported proxy protocols MUST carry the opaque capability in their standard authentication field where available, compare it without timing-dependent early exit, and remain explicitly unsupported when no equally scoped authentication adapter exists.
- **FR-003**: Authentication material MUST be protected in memory and storage where applicable, MUST never appear in logs or observations, MUST be invalid after session cleanup, and MUST not authenticate a replacement listener after port reuse.
- **FR-004**: Unauthorized, malformed, replayed, remote, and race-lost attempts MUST be refused and counted without retaining their application payloads.
- **FR-005**: Destination authority parsing MUST preserve the requested host and port exactly enough for name and certificate verification while rejecting ambiguous or malformed authorities.
- **FR-006**: Name resolution, upstream connection establishment, reads, and writes MUST use independent finite budgets and MUST release owned work within the session shutdown budget after cancellation.
- **FR-006a**: Budget conformance tests MUST allow no more than 250 milliseconds of scheduler tolerance beyond the declared operation or shutdown budget.
- **FR-007**: Upstream policy MUST prevent listener recursion, prohibited rebinding, and disallowed destination classes while preserving exact IPv4 and IPv6 outcomes.
- **FR-007a**: Loopback and private destinations MUST be refused by default; controlled tests MAY grant exact destinations, but the inspection listener and any resolution result outside the grant MUST remain refused.
- **FR-008**: Secure upstream connections MUST verify certificate chains and requested identities by default, MUST expose no silent downgrade, and MUST report DNS, TCP, security, timeout, cancellation, and policy failures as distinct outcomes.
- **FR-009**: Session certificate-authority material MUST be generated without invoking another process and MUST have a purpose-specific identity, bounded validity, non-secret thumbprint, and discoverable provenance.
- **FR-009a**: Each session MUST receive a new certificate-authority key and certificate, and no authority private key may be reused by a later session.
- **FR-010**: Private certificate material MUST remain in application-owned storage with restrictive Windows access controls, minimized plaintext lifetime, and a cleanup inventory that survives partial creation or a crash.
- **FR-011**: Certificate generation and explicit trust mutation MUST remain separate operations; no certificate may become trusted without the operator authorization required by the existing Deep Capture contract.
- **FR-012**: Leaf certificates MUST preserve validated DNS names or IP addresses, bounded validity, correct intended usage, and compatibility with the session's selected security policy.
- **FR-013**: The leaf cache MUST have explicit count, memory, and lifetime bounds; concurrent issuance MUST be race-free; CA or policy changes MUST invalidate affected entries; and eviction MUST remove private material from the owned inventory.
- **FR-014**: Windows trust operations MUST add, query, and remove only the exact authorized certificate in the current-user Root store and MUST not invoke a command-line certificate utility.
- **FR-015**: Trust operations MUST preserve unrelated certificates and report duplicate, wrong-store, mismatch, access-denied, interruption, and partial-cleanup states through typed outcomes shared by sessions and diagnostics.
- **FR-016**: All native components MUST emit through one versioned raw observation contract covering lifecycle, connection, DNS, TCP, security, HTTP, stream, message, refusal, error, and loss event families.
- **FR-017**: Each observation MUST carry stable session and connection correlation, a deterministic ordering key, an observation timestamp, provenance, and explicit payload ownership or omission state.
- **FR-018**: Observation queues and payloads MUST be bounded; dropped, truncated, refused, and unparsed items MUST advance named counters that are surfaced in snapshots and terminal reports.
- **FR-018a**: A full observation queue MUST evict its oldest queued record, count that exact loss, preserve later sequence order, and mark the affected stream incomplete.
- **FR-019**: Any observation gap MUST prevent a false complete-inspection claim, and unknown or malformed data MUST remain representable without being normalized into a different observation.
- **FR-020**: Raw observations MUST remain independent of HAR, key-log, and other projections so projection failure or omission cannot destroy source evidence.
- **FR-021**: The deterministic lab MUST provide controlled local clients and origins for HTTP, HTTPS, HTTP/2, WebSocket, streaming HTTP, gRPC, raw TCP, non-HTTP TLS, SOCKS, UDP, and QUIC.
- **FR-022**: Every lab protocol family MUST provide deterministic synthetic positive, refusal, malformed, timeout, cancellation, disconnect, and cleanup-failure scenarios, including certificate and payload inputs where applicable.
- **FR-023**: Lab fixtures MUST require no Internet service, account, real credential, real captured traffic, game, capture driver, or elevation, and any committed fixture resembling sensitive real data MUST be rejected.
- **FR-024**: The lab MUST distinguish packet truth, raw observations, projections, key logs, and cleanup truth, including an explicit unavailable result for behavior not implemented by the current proxy.
- **FR-025**: Windows-only trust scenarios MUST have an isolated test entry point while the remaining lab stays portable.
- **FR-026**: This slice MUST preserve the external production Deep Capture path until #290 and MUST NOT claim production protocol inspection, native CLI cutover, broad proxy support, or feature completeness.
- **FR-027**: The implementation MUST NOT introduce system-wide proxy mutation, target instrumentation, target memory access, target key extraction, certificate-pinning bypass, traffic interception drivers, unrelated-client inspection, or any other constitutionally prohibited technique.
- **FR-028**: The repository MUST mechanically prevent production certificate-command use in the native path and MUST keep all new dependency edges within the approved native proxy and platform adapter boundaries.

### Key Entities

- **Session capability**: Session-specific authentication material with generation identity, protected lifetime, use state, and cleanup state.
- **Authorized client**: A loopback connection that proves possession of the current session capability before upstream or payload work.
- **Destination authority**: Validated requested host, port, address family, resolution history, policy decision, and verification identity.
- **Upstream attempt**: One bounded resolution, connection, and optional secure-negotiation lifecycle with typed terminal outcome.
- **Session certificate authority**: Purpose-specific certificate, private key ownership, validity, thumbprint, provenance, storage protection, and cleanup inventory.
- **Leaf certificate entry**: Validated identities, signing-authority generation, policy generation, validity, private material, size, cache state, and eviction reason.
- **Trust record**: Exact certificate thumbprint, store scope, authorization, observed state, mutation result, and cleanup obligation.
- **Raw observation**: Versioned event family, session and connection identifiers, order, time, provenance, payload state, and typed content.
- **Observation accounting**: Queue and payload bounds plus emitted, dropped, truncated, refused, unparsed, and projection-gap counters.
- **Protocol lab scenario**: Controlled protocol family, synthetic client and origin inputs, timing, expected packet/application truth, expected outputs, and cleanup result.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Zero remote-interface attempts can reach the listener, and zero unauthenticated local attempts create upstream work or retain application payloads across the IPv4, IPv6, port-reuse, replay, and local-race matrix.
- **SC-002**: One hundred percent of admitted and refused listener attempts reconcile with named counters and have no unowned task after cleanup.
- **SC-003**: Every DNS, connection, read, write, secure-negotiation, and cancellation scenario terminates within its declared budget plus at most 250 milliseconds of scheduler tolerance, with zero sockets or tasks left unreported.
- **SC-004**: Upstream chain and identity verification is enabled in every secure success path, and every tested mismatch or downgrade attempt yields a typed failure rather than an application response.
- **SC-005**: One hundred percent of certificate-authority creation stages and injected partial failures leave either a protected complete inventory or a discoverable cleanup obligation, with no private key bytes in logs or public observations.
- **SC-006**: Concurrent leaf issuance never exceeds configured entry, byte, or lifetime bounds, and every eviction, CA rotation, and policy rotation removes or invalidates 100 percent of affected entries.
- **SC-007**: Trust mutation tests change exactly one authorized current-user Root certificate or none, preserve every unrelated test certificate, and classify every duplicate, wrong-store, mismatch, denial, and partial-cleanup case.
- **SC-008**: For every observation stress case, admitted inputs equal emitted records plus named dropped, truncated, refused, and unparsed outcomes, with every sequence and correlation invariant passing round-trip validation.
- **SC-009**: The offline lab executes every required protocol family and six failure classes deterministically, uses only synthetic local data, and distinguishes unsupported proxy behavior from a failed implementation claim.
- **SC-010**: The full repository format, lint, test, dependency, license, advisory, documentation, packaging, and Windows feature gates pass with the completed foundation.

## Assumptions

- S103 resolves issues #283, #284, #285, #286, #287, #288, and #289 as one dependency-ordered foundation slice because they share the native proxy owner and each later issue consumes contracts established earlier in the slice.
- Issue #290 remains out of scope because its production CLI cutover depends on every implementation and lab obligation in this slice being complete and reviewable first.
- The current external-backed Deep Capture CLI remains the shipped behavior during S103; the native foundation is exercised through library and controlled-test boundaries.
- Lab support for a protocol means deterministic clients, origins, truth, and failure fixtures exist. It does not mean the native proxy already parses, forwards, decrypts, or claims inspectability for that protocol.
- Private-destination policy is explicit and testable rather than inferred from address shape; controlled loopback destinations used by the lab are authorized test exceptions that cannot recurse into the inspection listener.
- Certificate and trust tests use unmistakably synthetic identities and isolated stores or adapters. They never mutate an operator's unrelated trust state during ordinary continuous integration.
