# Feature Specification: Scoped QUIC And HTTP/3 Inspection

**Feature Branch**: `codex/118-quic-http3-inspection`

**Created**: 2026-09-03

**Status**: Complete

**Input**: User description: "S118: implement scoped QUIC and HTTP/3 inspection under issue #314."

## Overview

S118 closes issue #314 by adding native QUIC and HTTP/3 inspection to the authenticated, target-scoped UDP route completed by S115 and S117. A routed QUIC connection is admitted only when one exact approved destination, certificate identity, and session authority can be established. fragcap terminates client-facing QUIC under the session certificate authority, establishes a separately verified upstream QUIC connection, and records bounded connection, stream, datagram, TLS, and HTTP/3 evidence without claiming visibility into traffic that did not traverse the route.

Unsafe behavior is refused rather than downgraded. Zero round-trip application data is not accepted or sent. Active connection migration is disabled, a changed routed endpoint terminates the affected connection, and pinning or trust failure never becomes opaque forwarding. HTTP/3 is recognized only through negotiated `h3` application protocol identifiers. Unknown or absent QUIC application protocols are explicitly refused because this slice has no sound generic stream pairing contract for them. IPv6 parity remains #315, exhaustive classification remains #316, and Deep Capture remains incomplete until #334.

## Clarifications

### Session 2026-09-03

- Q: Which route may carry inspected QUIC? -> A: Only an authenticated S115 UDP association owned by the selected target and current session.
- Q: How is destination scope preserved? -> A: Each intercepted QUIC connection is pinned to one policy-approved origin endpoint and certificate identity for its entire lifetime.
- Q: What is the 0-RTT policy? -> A: Client and upstream 0-RTT application data are refused; no replay-sensitive data is forwarded or retained.
- Q: What is the migration policy? -> A: Active migration is disabled on both QUIC halves, and any routed endpoint change is a terminal scoped refusal.
- Q: Which application semantics are claimed? -> A: Negotiated HTTP/3 is parsed as HTTP; absent and unknown QUIC application protocols are explicitly refused without fallback.

## User Scenarios & Testing

### User Story 1 - Inspect A Scoped QUIC Connection (Priority: P1)

An authorized target routes QUIC through its authenticated session and receives a bounded, correlated view of both client-facing and upstream transport halves.

**Why this priority**: No higher-level HTTP/3 observation is trustworthy until routing scope, independent TLS verification, connection identity, and forwarding are proven together.

**Independent Test**: A controlled loopback HTTP/3 client connects through the authenticated UDP route to a separately trusted origin, exchanges request streams and QUIC datagrams, and reconciles both connection halves, every accepted evidence unit, and terminal accounting.

**Acceptance Scenarios**:

1. **Given** one authenticated route and approved origin, **when** a client completes QUIC TLS, **then** the client half uses a session-authority leaf for the requested identity and the upstream half independently validates that same identity.
2. **Given** negotiated HTTP/3, **when** streams or datagrams traverse the paired connections, **then** they forward under finite limits and retain bounded transport and semantic evidence.
3. **Given** a trust, identity, pinning, protocol, or transport failure, **when** connection establishment cannot preserve policy, **then** the connection is refused with a stable reason and never becomes transparent forwarding.

---

### User Story 2 - Observe HTTP/3 Transactions (Priority: P2)

An operator can inspect HTTP/3 requests and responses with the same boundary-faithful metadata and bounded body principles as existing HTTP/1.1 and HTTP/2 evidence.

**Why this priority**: HTTP/3 is the common semantic use of QUIC and must feed existing application artifacts without creating a competing format.

**Independent Test**: Two controlled client and origin lineages negotiate HTTP/3, exchange concurrent requests with headers and bodies, and reconcile transaction, stream, connection, TLS, body, loss, and terminal records through the production artifact reader.

**Acceptance Scenarios**:

1. **Given** negotiated HTTP/3, **when** a request and response complete, **then** method, authority, path, status, ordered fields, body segments, stream identity, and both connection identities are retained within existing bounds.
2. **Given** concurrent or reset HTTP/3 streams, **when** they progress independently, **then** each stream has distinct identity, terminal state, and loss accounting without blocking unrelated streams.
3. **Given** an HTTP/3 body exceeds retention or queue capacity, **when** forwarding continues, **then** the complete body remains transport-authoritative while every omitted evidence byte is counted.

---

### User Story 3 - Refuse Unsafe QUIC Behavior (Priority: P3)

An operator can distinguish supported scoped inspection from 0-RTT, migration, unrouted traffic, and other cases that cannot retain session policy.

**Why this priority**: QUIC deliberately supports replayable early data and path changes, both of which can escape the one-target, one-session authority if accepted casually.

**Independent Test**: The controlled lab attempts early data, endpoint changes, migration, destination changes, unknown application protocols, connection and stream saturation, malformed traffic, and unrouted traffic, then verifies stable refusals, conservation, and finite cleanup.

**Acceptance Scenarios**:

1. **Given** attempted 0-RTT application data, **when** either QUIC half negotiates, **then** the data is refused and a stable replay-safety outcome is recorded.
2. **Given** attempted migration or a changed routed endpoint, **when** the path no longer matches the immutable connection scope, **then** the affected connection terminates without forwarding on the new path.
3. **Given** traffic outside an authenticated route, **when** artifacts finalize, **then** QUIC and HTTP/3 application inspection is explicitly unavailable and packet capture remains the only authority.

### Edge Cases

- A valid QUIC Initial arrives after generic UDP evidence has already observed the datagram.
- Multiple connection identifiers belong to one logical connection during ordinary rotation without a path change.
- A client requests an IP destination while TLS presents a DNS identity.
- The negotiated application protocol is absent, unknown, or falsely resembles HTTP/3 payload bytes.
- The client closes while an upstream connection, stream, or datagram is pending.
- A stream is reset while the opposite direction still has buffered evidence.
- QUIC datagrams are unsupported by one peer or exceed its negotiated limit.
- Connection, stream, datagram, body, event queue, or storage capacity is exhausted.
- Key updates occur repeatedly during a long-lived connection.
- The operating system cannot provide the requested address family.

## Requirements

### Functional Requirements

- **FR-001**: QUIC inspection MUST exist only inside an authenticated, target-scoped UDP association owned by the current session.
- **FR-002**: Each inspected connection MUST bind one immutable approved origin endpoint, one certificate identity, one client-facing connection identity, and one upstream connection identity.
- **FR-003**: The client-facing connection MUST use a leaf issued by the current session authority for the admitted identity, negotiate TLS 1.3, and expose the negotiated application protocol and certificate facts.
- **FR-004**: The upstream connection MUST resolve under proxy ownership, pass the existing destination policy, validate the admitted server identity against configured roots, and retain its TLS outcome separately from the client half.
- **FR-005**: Certificate pinning, client trust rejection, upstream validation failure, client-certificate requirements, identity mismatch, and unsupported protocol MUST remain distinct stable refusal classes and MUST NOT downgrade to transparent forwarding.
- **FR-006**: Zero round-trip application data MUST be disabled on the client-facing and upstream halves; attempted early data MUST be refused and counted without replay.
- **FR-007**: Active QUIC migration MUST be disabled. A changed outer client endpoint, selected destination, or upstream path that violates the immutable route MUST terminate the affected connection with a stable scoped refusal.
- **FR-008**: Ordinary connection identifier rotation that preserves the admitted path MUST retain one logical connection identity and MUST NOT be reported as migration.
- **FR-009**: HTTP/3 request streams and negotiated QUIC datagrams MUST forward one-to-one between paired connections under finite connection, stream, byte, datagram, idle, task, and queue limits.
- **FR-010**: QUIC and HTTP/3 evidence MUST identify session, both connection halves, direction, stream or datagram identity, per-direction sequence, timestamp, observed and retained lengths, retained bytes, provenance, and terminal outcome.
- **FR-011**: Negotiated application protocol `h3` MUST select HTTP/3 handling. Payload resemblance without negotiation MUST NOT select HTTP/3 or infer HTTP semantics.
- **FR-012**: HTTP/3 evidence MUST retain request method, scheme, authority, path, status, ordered fields, body segments, stream identity, both connection identities, timing, and terminal state using the existing application evidence authority.
- **FR-013**: HTTP/3 request and response forwarding MUST remain independent from observation retention, queue admission, serialization, storage, and unrelated stream failures.
- **FR-014**: QUIC connection, stream, datagram, reset, stop, timeout, protocol, transport, queue, storage, and retention loss MUST be counted by direction and authority with exact bounded localized identity and aggregate overflow.
- **FR-015**: Key updates MUST remain supported by the transport stack without changing logical connection or stream identity; fragcap MUST NOT claim packet-key material it did not export or observe.
- **FR-016**: Application JSON Lines version 2 and HAR MUST add QUIC and HTTP/3 facts, while manifest, lifecycle, correlation, and cleanup MUST continue through their existing protocol-neutral authorities without creating a second artifact authority or inventing unavailable fields.
- **FR-017**: Packet capture MUST remain packet truth. Proxy observations MUST claim only accepted routed application data after QUIC termination.
- **FR-018**: Unrouted QUIC, unsupported address families, unsafe migration, 0-RTT, and connections lacking an exact approved identity MUST receive explicit unsupported or refused outcomes.
- **FR-019**: Runtime shutdown MUST cancel and join every endpoint, connection, stream, and writer task, release sockets and retained state, and finalize accounting before clean completion.
- **FR-020**: The controlled lab MUST cover two independent QUIC and HTTP/3 client/origin lineages, authenticated production routing, request and response streams, capacity, endpoint immutability, 0-RTT disablement, and cleanup without Internet, elevation, game, or target data.
- **FR-021**: S118 MUST add no interception driver, process access, target key extraction, global proxy mutation, pinning bypass, silent certificate trust, unbounded task or memory owner, or Deep Capture completion claim.
- **FR-022**: Architecture, glossary, plan status, proxy README, AGENTS, and changelog MUST record S118 as closing #314 while leaving #315 through #318 and #334 open.

### Key Entities

- **Scoped QUIC Pair**: One client-facing and one upstream QUIC connection joined under one immutable route, origin identity, and session.
- **QUIC Connection Evidence**: Negotiated TLS, application protocol, endpoint, connection identity, transport parameters, key-update capability, and terminal outcome for one half.
- **QUIC Stream Evidence**: One directional portion of a bidirectional or unidirectional stream with logical identity, byte retention, loss, and terminal state.
- **QUIC Datagram Evidence**: One complete unreliable application datagram with direction, sequence, lengths, retention, and loss provenance.
- **HTTP/3 Transaction**: One request and response exchange bound to an HTTP/3 stream and the scoped connection pair.
- **QUIC Refusal**: A stable security or capability outcome that prevents unsafe inspection or downgrade.

## Success Criteria

- **SC-001**: One hundred percent of controlled admitted QUIC connections reconcile exactly one client-facing and one independently verified upstream connection, or one stable refusal.
- **SC-002**: One hundred percent of controlled forwarded HTTP/3 body bytes reconcile with retained evidence or named loss, while observation pressure changes zero valid forwarding bytes.
- **SC-003**: Two independent client and origin lineages complete concurrent HTTP/3 exchanges whose metadata, bodies, TLS facts, connection identities, and terminal states reconcile through production artifact readers.
- **SC-004**: Every attempted 0-RTT, unsafe migration, endpoint change, trust failure, pinning failure, and unscopable route is refused with no transparent fallback.
- **SC-005**: Connection identifier rotation and repeated key updates preserve logical identity and complete forwarding without a false migration or key-material claim.
- **SC-006**: All endpoint, connection, stream, datagram, task, queue, retention, and storage bounds are finite and every omitted unit advances a named counter.
- **SC-007**: Unrouted QUIC and HTTP/3 produce no application visibility claim, and packet capture remains available as the independent packet authority.
- **SC-008**: The complete repository verification suite passes, including native conformance and unmodified analyzer consumption, with dependency and license policy satisfied.

## Assumptions

- S115 supplies authenticated UDP association ownership and immutable outer client identity.
- S117 supplies exact routed datagram observation and conservation before QUIC classification.
- The existing session certificate authority, root store, destination policy, application writer, correlation, HAR, manifest, lifecycle, and cleanup authorities are reused.
- TLS 1.3, QUIC transport, and HTTP/3 protocol handling use reviewed native Rust dependencies with exact workspace pins and the existing ring cryptographic provider.
- IPv4 loopback is the required production and controlled-lab baseline. Complete IPv6 parity remains issue #315.
