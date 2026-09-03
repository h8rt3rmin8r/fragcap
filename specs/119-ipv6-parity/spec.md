# Feature Specification: Complete IPv6 Parity

**Feature Branch**: `codex/119-ipv6-parity`

**Created**: 2026-09-03

**Status**: Complete

**Input**: User description: "S119: complete IPv6 listener, routing, transport, and correlation parity under issue #315."

## Overview

S119 closes issue #315 by making address family an explicit part of Deep Capture planning, authorization, routing, evidence, and readiness. A session binds one exact IPv4 or IPv6 loopback address, never a wildcard or externally reachable interface. IPv4 remains the default for compatibility, while an operator may explicitly select IPv6. Every generated proxy route uses unambiguous bracketed IPv6 syntax.

Proxy-owned resolution and upstream connection establishment preserve IPv4 and IPv6 candidates. TCP connection attempts use one finite, staggered dual-stack race with exactly one winner and cancelled losers. IPv4-mapped IPv6 addresses are normalized for policy and identity without losing their observed family in evidence. Numeric scope identifiers are accepted only on scoped IPv6 literals and remain local socket metadata. HTTP, HTTPS, SOCKS, generic TCP, generic UDP, and QUIC controlled rows exercise IPv6 end to end. Doctor reports IPv4 and IPv6 loopback readiness independently.

This slice does not add wildcard listening, transparent interception, global proxy mutation, address-family fallback hidden from the operator, or a Deep Capture completion claim. Exhaustive classification remains #316, and Deep Capture remains incomplete until #334.

## Clarifications

### Session 2026-09-03

- Q: Does one session bind both address families? -> A: No. One immutable plan authorizes one exact IPv4 or IPv6 loopback socket. IPv4 is the compatibility default; IPv6 is an explicit selection.
- Q: How are DNS candidates attempted? -> A: Allowed candidates are deterministically interleaved by family and started with a fixed 250 ms stagger inside one existing connect deadline. The first successful socket wins and every other attempt is cancelled.
- Q: How are IPv6 scope identifiers represented? -> A: A bracketed scoped literal may carry a bounded decimal interface index. It is preserved in the socket address only for link-local or multicast IPv6; a scope on any other address is refused.
- Q: How are IPv4-mapped IPv6 addresses handled? -> A: Policy and correlation use the canonical IPv4 identity, while evidence retains the originally observed socket address and family.
- Q: What does Doctor prove? -> A: It independently attempts exact ephemeral binds to `127.0.0.1` and `::1`, reports ready, unavailable, or failed for each, and never binds a wildcard.

## User Scenarios & Testing

### User Story 1 - Run Deep Capture On IPv6 Loopback (Priority: P1)

An operator can explicitly prepare and authorize Deep Capture on IPv6 loopback and receive correctly bracketed target-scoped proxy routes.

**Why this priority**: Listener identity is the root of routing scope. No IPv6 transport claim is sound until the exact authorized listener is represented throughout the session.

**Independent Test**: Prepare controlled sessions for each family, verify exact plan and route identity, complete an IPv6 loopback request, and prove wildcard and external addresses are refused before effects.

**Acceptance Scenarios**:

1. **Given** IPv6 is selected and available, **when** preflight completes, **then** the immutable plan names `[::1]:port` and every proxy URL and lifecycle record preserves that family.
2. **Given** IPv4 is selected or no family is specified, **when** preflight completes, **then** the plan names `127.0.0.1:port` with existing behavior preserved.
3. **Given** a wildcard, mapped wildcard, or non-loopback address, **when** endpoint validation runs, **then** preparation refuses it before any listener or route effect.

---

### User Story 2 - Route Every Supported Transport Over IPv6 (Priority: P2)

An authorized target can use IPv6 literal and DNS destinations through HTTP, HTTPS, SOCKS, generic TCP, generic UDP, and QUIC routes without losing address-family identity.

**Why this priority**: Partial IPv6 support would make the selected listener appear ready while silently failing individual protocol families.

**Independent Test**: The controlled lab runs the six required protocol families through exact IPv6 loopback peers and reconciles forwarded units, evidence endpoints, flow identities, and terminal accounting.

**Acceptance Scenarios**:

1. **Given** an IPv6 literal or DNS result, **when** any required protocol route succeeds, **then** its client, listener, destination, and upstream socket families remain explicit in events and artifacts.
2. **Given** a scoped IPv6 literal, **when** its numeric interface index is valid for that address scope and policy permits the destination, **then** the socket uses that index while emitted authority text never invents or leaks a different zone.
3. **Given** an IPv4-mapped IPv6 address, **when** policy and correlation evaluate it, **then** it has one canonical IPv4 identity and cannot create a duplicate or mismatched flow.

---

### User Story 3 - Survive Dual-Stack Failure Without Duplicate Evidence (Priority: P3)

An operator receives one exact outcome when a DNS name resolves to both families, including when the preferred path is slow or unavailable.

**Why this priority**: A naive sequential connector delays working paths, while an unowned race can create duplicate upstream effects and observations.

**Independent Test**: Inject ordered IPv4 and IPv6 candidate sets with successful, refused, timed-out, and cancelled attempts, then verify deterministic staggering, one winner, loser cancellation, exact selected-peer evidence, and conservation.

**Acceptance Scenarios**:

1. **Given** allowed candidates from both families, **when** the first attempt does not finish before the stagger, **then** the next family begins within the same bounded deadline.
2. **Given** two attempts become connectable, **when** one succeeds first, **then** exactly one stream is returned and all remaining attempts are cancelled before application forwarding.
3. **Given** every candidate fails or is refused, **when** the deadline expires, **then** one stable aggregate outcome is returned without a fabricated selected peer.

---

### User Story 4 - Diagnose Family Readiness (Priority: P4)

An operator can see whether the local host can bind the exact IPv4 and IPv6 loopback endpoints Deep Capture uses.

**Why this priority**: Separate readiness prevents an IPv4-ready host from being presented as IPv6-ready and gives a concrete explanation before launch.

**Independent Test**: Construct and probe independent ready, unavailable, and failed states for both families, then verify human and JSON reports name each check separately.

**Acceptance Scenarios**:

1. **Given** both loopback families bind, **when** Doctor runs, **then** both readiness checks are `ok`.
2. **Given** IPv6 is unavailable, **when** Doctor runs, **then** IPv4 remains `ok` and IPv6 carries its own non-fabricated reason.
3. **Given** a probe cannot determine a family, **when** Doctor renders, **then** that family is reported as undetermined rather than absent or ready.

### Edge Cases

- IPv6 loopback is disabled or unavailable while IPv4 remains available.
- A requested port is zero during reservation and becomes exact only after bind.
- A bracketed literal omits a port, carries an empty zone, a non-decimal zone, or a zone on a global address.
- DNS returns repeated, mapped, refused, or same-family-only candidates.
- Cancellation occurs before DNS, between staggered attempts, or after one socket connects.
- Two connection attempts become ready during the same scheduler turn.
- A SOCKS5 IPv6 reply or UDP datagram contains an address from the wrong family.
- QUIC uses an IPv6 client and origin endpoint while connection identifiers rotate.
- Correlation receives equivalent mapped and native IPv4 observations in different orders.
- Evidence retention fails after forwarding selected exactly one winning upstream socket.

## Requirements

### Functional Requirements

- **FR-001**: One Deep Capture plan MUST contain one exact `SocketAddr` on either IPv4 loopback or IPv6 loopback and MUST reject wildcard, mapped wildcard, and non-loopback listeners before effects.
- **FR-002**: IPv4 MUST remain the default listener family, and IPv6 MUST be explicitly selectable through the CLI and testable through the facade preparation seam.
- **FR-003**: Endpoint reservation, authorization, runtime bind, resource journal, lifecycle, routing environment, plan presentation, and cleanup MUST use the same exact endpoint rather than reconstructing it from a port.
- **FR-004**: Proxy URLs MUST bracket IPv6 hosts and MUST preserve IPv4 output compatibility.
- **FR-005**: Destination authorities MUST accept ordinary DNS, IPv4, bracketed IPv6, and bounded decimal scoped IPv6 literals, while refusing ambiguous, malformed, credential-bearing, missing-port, and invalid-scope forms.
- **FR-006**: A numeric scope identifier MUST be retained only for IPv6 link-local or multicast destinations, MUST be used for socket operations, and MUST remain local metadata rather than application authority or TLS identity.
- **FR-007**: Destination policy MUST canonicalize IPv4-mapped IPv6 for listener comparison, exact grants, public-scope decisions, peer identity, and correlation while retaining the originally observed address in evidence.
- **FR-008**: Proxy-owned DNS resolution MUST deduplicate candidates canonically and retain at least one allowed candidate from each returned family.
- **FR-009**: TCP destination candidates MUST be deterministically interleaved across families, use a fixed 250 ms connection-attempt delay, share one finite connect deadline, and honor session cancellation.
- **FR-010**: The first successful TCP attempt MUST become the sole upstream stream. Every pending or unstarted loser MUST be cancelled or discarded before application forwarding and MUST NOT emit a second success observation.
- **FR-011**: Failed dual-stack attempts MUST return stable cancellation, timeout, policy, or transport outcomes and MUST NOT claim a selected peer when none connected.
- **FR-012**: Successful TCP and TLS upstream streams MUST expose the exact selected peer and local socket addresses to protocol evidence.
- **FR-013**: HTTP, HTTPS, SOCKS5 CONNECT, generic TCP, SOCKS5 UDP, generic UDP, QUIC, and HTTP/3 paths MUST accept and preserve IPv6 endpoints under their existing authentication, policy, TLS, retention, and loss contracts.
- **FR-014**: IPv6 SOCKS5 commands and replies MUST use the IPv6 address form; mapped addresses MUST not create duplicate peer ownership.
- **FR-015**: IPv6 QUIC and HTTP/3 MUST retain the existing immutable route, trust, 0-RTT refusal, migration, stream, datagram, and cleanup guarantees.
- **FR-016**: Application JSON Lines, HAR, manifest, resource journal, proxy lifecycle, cleanup, and correlation MUST preserve exact socket text and address family without schema-breaking replacement or invented values.
- **FR-017**: Flow identifiers and socket ownership MUST be stable under IPv4, IPv6, mapped-address, DNS-order, and observation-order permutations.
- **FR-018**: Doctor MUST probe exact ephemeral IPv4 and IPv6 loopback binds independently, close each probe immediately, and report each result separately in human and JSON output.
- **FR-019**: The controlled lab MUST contain passing IPv6 rows for HTTP, HTTPS, SOCKS, generic TCP, generic UDP, and QUIC without Internet, elevation, game, wildcard bind, or target data.
- **FR-020**: All race attempts, sockets, tasks, candidate collections, scope text, and evidence remain finite and join or release on success, failure, timeout, and cancellation.
- **FR-021**: S119 MUST add no packet interception, process access, target key extraction, global proxy mutation, transparent fallback, wildcard bind, or Deep Capture completion claim.
- **FR-022**: Architecture, glossary, plan status, proxy README, AGENTS, and changelog MUST record S119 as closing #315 while leaving #316 through #318 and #334 open.

### Key Entities

- **Loopback Family**: The explicit IPv4 or IPv6 family selected for one session listener.
- **Exact Loopback Endpoint**: The immutable socket address shared by plan, bind, route, evidence, and cleanup.
- **Scoped IPv6 Literal**: An IPv6 address plus a bounded local numeric interface index used only where address scope permits it.
- **Canonical Peer Identity**: The normalized socket identity used for policy, deduplication, ownership, and correlation.
- **Dual-Stack Attempt Set**: A finite ordered set of allowed connection candidates sharing one deadline and one winner.
- **Selected Upstream Peer**: The exact remote socket of the sole successful connection.
- **Family Readiness**: Doctor's independent observed ability to bind one exact loopback family.

## Success Criteria

- **SC-001**: Every controlled IPv4 and IPv6 session binds exactly the endpoint authorized by its plan, and zero tests or production paths bind wildcard or external interfaces.
- **SC-002**: One hundred percent of required IPv6 HTTP, HTTPS, SOCKS, TCP, UDP, and QUIC lab rows pass with exact family-bearing endpoints.
- **SC-003**: Every controlled dual-stack race returns exactly one connected peer or one stable failure, with zero duplicate forwarded requests or observations across at least 100 deterministic race iterations.
- **SC-004**: Every mapped-address and candidate-order permutation produces the same policy, peer ownership, flow identity, and correlation result.
- **SC-005**: Every scoped-literal vector is either converted to the exact IPv6 socket scope index or refused with a stable reason; no zone identifier is transmitted as TLS identity or application authority.
- **SC-006**: Doctor's independent IPv4 and IPv6 checks render correctly for all ready, unavailable, failed, and undetermined combinations.
- **SC-007**: Existing IPv4 controlled and golden outputs remain compatible except for intentional additive family facts.
- **SC-008**: The complete repository verification suite passes with dependency, license, analyzer, formatting, lint, encoding, and text-hygiene policy satisfied.

## Assumptions

- S114 through S118 supply authenticated SOCKS, generic TCP and UDP, QUIC, HTTP/3, and bounded application evidence.
- S108 and S109 supply protocol-neutral correlation, manifest, lifecycle, recovery, and cleanup authorities.
- The operating system resolver returns a finite iterator; this slice bounds the retained candidate set before racing.
- Numeric interface indexes are the portable socket-level representation of an IPv6 zone. Human-readable interface-name lookup is outside this slice.
- IPv4 remains the default because existing target routing and compatibility evidence were calibrated against it. Selecting IPv6 is an operator-visible plan change.
