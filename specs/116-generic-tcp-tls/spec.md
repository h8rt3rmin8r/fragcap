# Feature Specification: Generic TCP And Non-HTTP TLS Evidence

**Feature Branch**: `codex/116-generic-tcp-tls`

**Created**: 2026-09-02

**Status**: Draft

**Input**: User description: "S116: record generic TCP and non-HTTP TLS streams under issue #312."

## Overview

S116 closes issue #312 by turning approved native TCP tunnels into bounded evidence streams without inventing an application protocol. Plain SOCKS5 TCP carries retained plaintext chunks. TLS that remains byte-transparent through SOCKS5 is explicitly opaque and retains only encrypted provenance. A trusted HTTP CONNECT tunnel with no negotiated HTTP protocol may terminate at the session authority, establish a separately verified upstream TLS connection, and retain decrypted chunks as protocol-unknown evidence.

Every chunk records direction, time, offset, observed size, retained size, truncation outcome, connection correlation, and plaintext, encrypted, or decrypted provenance. Forwarding does not depend on evidence capacity. Client trust rejection, certificate pinning, upstream client-auth failure, protocol mismatch, and transport refusal remain distinct and never fall back after bytes have been consumed. S116 adds no custom dissector, UDP or QUIC semantics, or Deep Capture completion claim.

## Clarifications

### Session 2026-09-02

- Q: Which route owns plain and opaque TCP? -> A: The authenticated child-scoped SOCKS5 CONNECT route supplied by S114.
- Q: When may non-HTTP TLS be decrypted? -> A: Only an explicit HTTP CONNECT tunnel that successfully completes client-facing session-CA TLS and separately verified upstream TLS, then presents no HTTP ALPN or recognizable HTTP request prefix.
- Q: Can failed interception fall back to opaque forwarding? -> A: No. A handshake or policy failure is an explicit refusal because consumed TLS bytes cannot be replayed truthfully.
- Q: What does protocol-unknown mean? -> A: The proxy records byte chunks and transport/TLS provenance only. It assigns no message, field, request, response, or schema meaning.
- Q: How are storage limits applied? -> A: Per-connection and session retention budgets cap evidence while the fixed-buffer forwarding path continues independently.

## User Scenarios & Testing

### User Story 1 - Record A Generic TCP Stream (Priority: P1)

An authorized target opens an allowed SOCKS5 TCP tunnel and exchanges non-HTTP bytes. The proxy forwards every byte and retains bounded directional chunks correlated to the connection.

**Why this priority**: Generic TCP is the base transport gap named by issue #312.

**Independent Test**: A controlled authenticated SOCKS client exchanges binary chunks and half-closes against a loopback origin while evidence proves exact bytes, direction, offsets, timing, bounds, and terminal conservation.

**Acceptance Scenarios**:

1. **Given** an allowed plain TCP destination, **when** bytes flow in either direction, **then** forwarding preserves them and evidence records plaintext protocol-unknown chunks.
2. **Given** payload capture is disabled or retention is exhausted, **when** more bytes flow, **then** forwarding continues and evidence reports intentional omission or exact truncation.
3. **Given** half-close, timeout, cancellation, or transport failure, **when** the tunnel terminates, **then** retained chunks and aggregate byte counts reconcile to one terminal outcome.

---

### User Story 2 - Distinguish Opaque And Intercepted TLS (Priority: P2)

An operator can tell encrypted byte-transparent TLS from TLS that the session authority successfully terminated and observed as protocol-unknown plaintext.

**Why this priority**: Conflating encrypted and decrypted bytes would make the artifact materially false.

**Independent Test**: Controlled loopback TLS clients exercise a SOCKS opaque tunnel and a trusted CONNECT interception path, proving separate encrypted and decrypted provenance and exact round trips.

**Acceptance Scenarios**:

1. **Given** TLS through SOCKS5, **when** the tunnel stays byte-transparent, **then** its classification is opaque TLS and retained chunks are marked encrypted.
2. **Given** trusted CONNECT TLS with no HTTP ALPN or HTTP prefix, **when** both TLS boundaries succeed, **then** decrypted chunks are retained as protocol-unknown and upstream TLS remains separately verified.
3. **Given** an HTTP ALPN or recognizable HTTP request, **when** TLS succeeds, **then** the existing HTTP engine remains authoritative rather than the generic stream path.

---

### User Story 3 - Refuse Honestly And Stay Bounded (Priority: P3)

An operator receives an explicit non-success outcome when trust, pinning, client authentication, policy, protocol, or transport prevents observation, with no unsafe or ambiguous fallback.

**Why this priority**: Interception boundaries must remain deliberate, auditable, and truthful.

**Independent Test**: Controlled rejection, oversized evidence, queue pressure, cancellation, and cleanup cases prove no bypass, no unbounded storage, no guessed semantics, and complete loss accounting.

**Acceptance Scenarios**:

1. **Given** client trust rejection, pinning, or upstream client-auth failure, **when** interception fails, **then** the exact refusal class is retained and no opaque fallback is claimed.
2. **Given** a chunk larger than the event limit or budgets, **when** it is observed, **then** records remain finite and name every omitted byte.
3. **Given** queue pressure or session cleanup, **when** events cannot be retained, **then** the existing loss and lifecycle authorities count the outcome and leave no stream task live.

### Edge Cases

- The first generic prefix arrives one byte at a time or after the classification budget.
- A no-ALPN TLS stream begins with bytes that resemble an incomplete HTTP method.
- A binary stream contains HTTP-looking bytes after its first chunk.
- Retention ends partway through one forwarded buffer.
- Evidence queue pressure occurs after retention was claimed.
- Either side half-closes while the reverse direction remains active.
- Client and upstream negotiate different ALPN values.
- Client trust rejection races shutdown.

## Requirements

### Functional Requirements

- **FR-001**: Every accepted generic TCP stream MUST retain the existing authenticated, target-scoped routing, destination-policy, connection-limit, cancellation, and cleanup authorities.
- **FR-002**: Plain TCP, opaque TLS, intercepted protocol-unknown TLS, and refusal MUST be distinct outcomes.
- **FR-003**: Plain and opaque SOCKS5 TCP MUST remain byte-transparent and preserve byte order, content, full duplex, and half-close under fixed buffers and finite deadlines.
- **FR-004**: A trusted CONNECT tunnel with no HTTP ALPN or recognizable HTTP prefix MUST use the existing session certificate authority for the client boundary and the existing verified TLS configuration for the upstream boundary before recording decrypted bytes.
- **FR-005**: HTTP/2 and recognizable HTTP/1.1 MUST retain the existing native HTTP engines and artifact contracts.
- **FR-006**: A TLS failure after interception begins MUST terminate with its observable refusal class and MUST NOT silently downgrade, bypass validation, or claim opaque forwarding.
- **FR-007**: Generic chunk evidence MUST record connection id, direction, event time, byte offset, observed length, retained length, retention outcome, and plaintext, TLS-encrypted, or TLS-decrypted provenance.
- **FR-008**: Generic evidence MUST NOT assign custom messages, fields, request/response semantics, or protocol names beyond transport and TLS provenance.
- **FR-009**: Payload retention MUST honor `capture_payloads`, the existing per-connection body limit, session body limit, maximum event chunk size, and bounded application queue.
- **FR-010**: Forwarding capacity MUST be independent from evidence retention capacity; truncation or omission MUST NOT refuse an otherwise valid stream.
- **FR-011**: Observed, retained, intentionally omitted, retention-limited, and queue-dropped bytes MUST remain separately countable with directional forwarded byte totals.
- **FR-012**: Every generic stream MUST produce one terminal event covering complete, timeout, cancelled, refused, protocol error, transport error, shutdown, or forced cleanup.
- **FR-013**: Connection id and exact open/close windows MUST remain the correlation authority for generic chunks and packet/process evidence.
- **FR-014**: Application JSON Lines and proxy lifecycle JSON Lines MUST serialize generic stream facts with schema-stable names and base64 payloads only when bytes were retained.
- **FR-015**: The controlled protocol lab MUST cover plain TCP, opaque TLS, intercepted no-ALPN TLS, HTTP preservation, omission, partial retention, refusal, half-close, cancellation, and cleanup without Internet, elevation, game, or target data.
- **FR-016**: S116 MUST add no target key extraction, pinning bypass, system-wide route, unauthenticated listener, custom dissector, UDP or QUIC payload semantics, new runtime dependency, or Deep Capture completion claim.
- **FR-017**: Architecture, glossary, plan status, proxy README, and changelog MUST record S116 as closing #312 while leaving #313 through #318 and #334 open.

### Key Entities

- **Generic Stream**: One approved TCP tunnel with transport/TLS outcome, two forwarding directions, retention budgets, connection identity, and one terminal outcome.
- **Generic Chunk**: One bounded directional observation with offset, observed and retained lengths, bytes, outcome, and provenance.
- **TLS Observation Mode**: Opaque encrypted forwarding, intercepted protocol-unknown plaintext, or explicit refusal.
- **Generic Stream Accounting**: Directional observed, retained, omitted, and forwarded totals plus stream outcome counts.

## Success Criteria

- **SC-001**: Controlled plain, opaque TLS, and intercepted TLS cases preserve 100 percent of forwarded bytes and report the exact expected outcome.
- **SC-002**: Every generic chunk has monotonic per-direction offsets and reconciles observed bytes to retained plus omitted bytes.
- **SC-003**: Evidence memory never exceeds fixed forwarding buffers, event chunk size, per-connection retention, session retention, and queue bounds.
- **SC-004**: Payload-disabled and retention-exhausted cases forward unchanged while reporting 100 percent of unretained bytes.
- **SC-005**: Every trust, pinning, client-auth, ALPN, policy, timeout, cancellation, and transport failure has one explicit non-success result and no silent fallback.
- **SC-006**: No generic record claims application semantics beyond observed transport and TLS provenance.
- **SC-007**: Full repository verification passes without dependency drift or regression in HTTP, HTTPS, HTTP/2, WebSocket, SSE, gRPC, SOCKS5, UDP association, cleanup, or correlation.

## Assumptions

- S114 supplies authenticated SOCKS5 TCP and classification; S104 and S107 supply the client and upstream TLS boundaries and refusal taxonomy; S105 through S109 supply bounded application artifacts, correlation, and lifecycle authority.
- No-ALPN TLS is the safe generic interception discriminator. Recognized HTTP prefixes retain HTTP/1.1 behavior.
- Existing body retention resources can govern generic chunks without creating a second session storage budget.
