# Feature Specification: Native Blind Relay

**Slice**: S193
**Branch**: `codex/s193-native-blind-relay`
**Created**: 2026-11-04
**Status**: In progress
**Issue**: #559, under #557
**Architecture of record**: `docs/fragcap-specification.md` sections 20, 27, and 29

## User Scenarios and Testing

### User Story 1: Preserve a launcher connection (P1)

An authorized session routes a launcher and a selected game through the same authenticated proxy. The launcher's connection receives an uninspected decision. Its original TLS connection to the permitted origin succeeds without a Fragcap certificate, TLS key log, or decrypted application export. A game connection to the same hostname remains eligible for native inspection on exact owner proof.

**Independent check**: A controlled pinned launcher and trusting game use the same local origin. Verify original launcher TLS and inspected game TLS, with independent connection decisions and terminal records.

### User Story 2: Relay every supported transport (P2)

Excluded middleware continues through authenticated HTTP/1 CONNECT, plaintext HTTP, cleartext HTTP/2 streams, SOCKS5 TCP and UDP, and QUIC datagrams over IPv4 and IPv6 where supported. The proxy keeps existing destination policy, association ownership, bounded resources, and forwarding semantics.

**Independent check**: Controlled local origin and client fixtures exercise every ingress and address family with excluded and inspected decisions.

### User Story 3: Diagnose relay and cleanup (P3)

An operator can tell which connections were relayed, inspected, unresolved, refused, or failed, with bounded byte and datagram totals and named evidence omissions. The session proxy remains available while an authorized route owner still carries inherited settings, then tears down after exact release.

**Independent check**: Saturation, writer failure, idle versus progressing transfer, game TLS failure, and ancestor-after-game-exit fixtures reconcile terminal and cleanup facts.

## Edge Cases

- An unknown or late owner is relayed with its original uncertainty reason; it cannot become inspected mid-connection.
- A failed game TLS inspection cannot switch to relay on the same connection.
- A relayed CONNECT stops HTTP header timing after tunnel establishment; a genuinely idle tunnel still ends finitely.
- A plaintext HTTP request may require proxy framing and hop-header removal. Its content and metadata are not exported as inspected application evidence.
- Every HTTP/2 stream on a connection inherits the frozen decision; stream-specific paths cannot re-enable inspection.
- An excluded UDP association never enters native QUIC termination, even when its first datagram is a QUIC Initial.
- Retention saturation and writer failure do not stop forwarding or misclassify scope exclusion as packet loss.

## Functional Requirements

- **FR-001**: The S191 pre-inspection decision MUST select a genuine blind relay before any certificate creation, TLS termination, key logging, or application-content export for uninspected traffic. A verified game or explicit launch-chain decision retains native inspection.
- **FR-002**: HTTP/1 CONNECT MUST authenticate, validate the destination, establish the origin connection, send success, and relay both directions with bounded buffers, half-close, cancellation, inactivity, and terminal byte accounting.
- **FR-003**: Plaintext HTTP MUST forward with correct explicit-proxy semantics while suppressing excluded request, response, body, and derived exports. It MUST describe any metadata retained for relay diagnostics accurately.
- **FR-004**: Cleartext HTTP/2 requests and CONNECT streams MUST obey the frozen connection decision, including concurrent streams and stream-level refusal.
- **FR-005**: SOCKS5 TCP MUST preserve authentication and opaque forwarding while suppressing excluded generic payload chunks. UDP MUST preserve exact association/client pinning, destinations, datagram boundaries, and reply ownership while suppressing excluded payload export.
- **FR-006**: An excluded QUIC datagram MUST retain the client's original QUIC/TLS exchange and MUST NOT create a session-CA QUIC endpoint or decrypted HTTP/3 evidence.
- **FR-007**: IPv4 and IPv6 MUST use the same decision and policy rules; unsupported combinations MUST return a named refusal rather than inspection.
- **FR-008**: Forwarding MUST remain independent from optional evidence retention and writer availability. Every admitted connection or association MUST end with one terminal outcome and reconcilable totals, including omission, failure, and cleanup.
- **FR-009**: An established relay MUST use an activity-based idle deadline and an explicit finite session lifetime. Explicit configured duration bounds MUST be honored without an undocumented clamp.
- **FR-010**: Exact route-owner release and proxy drain MUST remain separate from game observation end. Raw packet capture and historical broad-scope evidence MUST retain their original meaning.
- **FR-011**: Public API, artifact, protocol, glossary, chronology, and changelog documentation MUST describe only delivered behavior. #542 causality, #560 calibration, #561 operator integration, field testing, and release remain separate.

## Key Entities

- **Frozen inspection decision**: The S191 action, reason, timing, and owner proof for one connection or UDP association.
- **Relay transfer**: Directional byte or datagram totals, duration, termination cause, and named omission without payload retention.
- **Route owner**: An exact managed process carrying inherited proxy settings until verified release.

## Success Criteria

- **SC-001**: The controlled same-hostname fixture shows original launcher TLS and inspected game TLS in one session.
- **SC-002**: Every supported native ingress and address family enforces one frozen decision before content processing, with no excluded decrypted export or proxy-owned key material.
- **SC-003**: Controlled saturation, writer failure, half-close, idle/progress, TLS failure, and owner-release cases preserve forwarding and reconcile terminal facts.
- **SC-004**: All applicable repository checks and exact-head CI succeed. Installed-product testing follows release; a later observed defect becomes a GitHub issue.

## Clarifications (2026-11-04)

- S193 supplies the library/runtime relay used by future exact-game operator selection. The existing public CLI remains in launch-chain mode until #561.
- Unknown owners are relayed with their exact uncertainty reason. Only exact verified game ownership authorizes inspection.
- Connection scope is frozen before protocol handling. TLS failure never changes it. Destination policy and session authentication are independent of scope.
- Scope exclusion omits application evidence; it does not imply packet loss or permission to bypass routing policy.
