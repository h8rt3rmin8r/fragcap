# Feature Specification: Scoped SOCKS5 UDP Association

**Feature Branch**: `codex/115-socks5-udp-associate`

**Created**: 2026-09-02

**Status**: Draft

**Input**: User description: "S115: implement SOCKS5 UDP ASSOCIATE with scoped ownership under issue #311."

## Overview

S115 closes issue #311 by extending the authenticated native SOCKS5 listener with UDP ASSOCIATE. One authenticated TCP control connection owns one finite UDP relay and one immutable client endpoint identity. The relay supports IPv4, IPv6, and proxy-resolved domain destinations, applies the existing destination policy to every resolved address, and accepts upstream replies only from exact peers the association previously contacted.

The association ends when its control connection closes, the session stops, or its idle budget expires. Peer count, datagram size, socket count, and retained mapping memory are finite. Fragmentation is deliberately unsupported and counted. S115 records transport metadata and loss without retaining generic UDP payloads or inventing a remote endpoint. Generic UDP packet truth remains issue #313, QUIC and HTTP/3 remain #314, and Deep Capture remains incomplete until #334.

## Clarifications

### Session 2026-09-02

- Q: What owns an association? -> A: Exactly one authenticated TCP control connection; its EOF or terminal failure immediately revokes the UDP relay.
- Q: How is the UDP client endpoint established? -> A: The TCP peer IP is immutable. A concrete request port pins that port; port zero learns the first valid datagram source port once. It never repins.
- Q: Is SOCKS5 fragmentation implemented? -> A: No. Every nonzero FRAG datagram is dropped and counted as unsupported fragmentation.
- Q: What remote traffic may return through the relay? -> A: Only datagrams from exact remote socket addresses successfully contacted by this association and retained inside its bounded peer set.
- Q: What evidence is in scope? -> A: Association, address form, destination, datagram length, drop reason, mapping count, and terminal accounting. Generic UDP payload retention and semantics remain #313.

## User Scenarios & Testing

### User Story 1 - Relay Authorized UDP Datagrams (Priority: P1)

An authorized target creates a UDP association, sends framed datagrams to allowed IPv4, IPv6, and domain destinations, and receives correctly framed replies while its TCP control connection remains open.

**Why this priority**: UDP-capable targets cannot use the native scoped route without this transport.

**Independent Test**: A controlled loopback client authenticates, creates an association, and exchanges exact datagrams with real IPv4 and available IPv6 echo peers plus proxy-owned domain resolution.

**Acceptance Scenarios**:

1. **Given** valid session credentials, **when** UDP ASSOCIATE succeeds, **then** the reply names the actual client-facing relay endpoint and no datagram is admitted after control EOF.
2. **Given** a valid unfragmented datagram to an allowed literal or domain destination, **when** the relay forwards it, **then** the origin receives the exact payload and its exact reply is framed with the observed origin endpoint.
3. **Given** several allowed destinations within the peer bound, **when** replies arrive, **then** only exact previously contacted peers can reach the pinned client endpoint.

---

### User Story 2 - Refuse Hijack And Amplification Paths (Priority: P2)

Unrelated local clients, spoofed source ports, unsolicited origins, local destinations, malformed frames, fragments, and saturated mappings cannot use the association.

**Why this priority**: A UDP relay without exact tenancy and reply validation becomes a local hijack or reflection primitive.

**Independent Test**: Controlled attackers send from a different IP or port, request the proxy or ungranted local services, inject unsolicited upstream replies, exceed bounds, and send malformed or fragmented frames; every attempt is dropped and counted with no unauthorized delivery.

**Acceptance Scenarios**:

1. **Given** a pinned client endpoint, **when** another source sends a valid frame, **then** it is dropped without DNS, policy, mapping, or upstream effects.
2. **Given** a destination refused by policy, **when** a valid client requests it, **then** no mapping is created and no datagram is sent.
3. **Given** an upstream source absent from the contacted peer set, **when** it sends to an upstream socket, **then** no response is emitted to the client.
4. **Given** nonzero FRAG, malformed framing, oversized input, peer saturation, or timeout, **when** it occurs, **then** the exact named loss counter advances.

---

### User Story 3 - Reconcile Association Ownership And Cleanup (Priority: P3)

An operator can reconcile each association, accepted datagram, loss, and terminal reason with the existing connection and packet/process evidence, and cleanup leaves no UDP socket or mapping live.

**Why this priority**: Working forwarding without observable ownership and complete cleanup would violate Deep Capture's safety boundary.

**Independent Test**: Event and lifecycle collectors prove one connection identity, truthful destination facts, conservation of received datagrams, and zero association residue after EOF, timeout, cancellation, and forced cleanup.

**Acceptance Scenarios**:

1. **Given** accepted and dropped datagrams, **when** artifacts finalize, **then** typed events and aggregate counters reconcile every client and upstream datagram outcome.
2. **Given** domain resolution with several candidates, **when** a datagram is sent, **then** evidence names only the actual selected remote endpoint and never a guessed endpoint.
3. **Given** control EOF, idle timeout, cancellation, or cleanup, **when** termination finishes, **then** every UDP socket and mapping is released and the association has one terminal outcome.

### Edge Cases

- The request endpoint is all zeros, has the TCP peer IP with port zero, or names a different IP.
- The first valid UDP frame races a spoofed source datagram.
- IPv4-mapped IPv6 input aliases a refused local endpoint.
- A domain resolves to mixed allowed and refused candidates.
- A response arrives as the control connection closes or a mapping expires.
- The peer bound is reached by unique resolved addresses for one domain.
- Empty payloads, maximal legal datagrams, truncated address fields, invalid domains, reserved bytes, and unknown address types arrive.
- Queue pressure drops typed evidence while transport accounting remains complete.

## Requirements

### Functional Requirements

- **FR-001**: The existing authenticated SOCKS5 listener MUST accept UDP ASSOCIATE (`CMD 0x03`) without adding an unauthenticated or system-wide listener.
- **FR-002**: Exactly one UDP association MUST be owned by its authenticated TCP control connection and MUST terminate on control EOF, session cancellation, idle timeout, protocol failure, or runtime cleanup.
- **FR-003**: The request endpoint MUST either be unspecified or match the normalized TCP peer IP; a conflicting IP MUST be refused before binding a relay.
- **FR-004**: A nonzero request port MUST pin the client UDP port. A zero port MUST learn the first valid datagram source port once. The client endpoint MUST never repin.
- **FR-005**: The relay reply MUST report the actual client-facing loopback UDP address and port only after all required UDP sockets exist.
- **FR-006**: UDP framing MUST validate RSV, FRAG, ATYP, address, port, and total length incrementally within a fixed maximum datagram buffer.
- **FR-007**: IPv4, IPv6, and domain destination forms MUST be supported. Domain resolution MUST be proxy-owned and every candidate MUST independently pass the existing normalized destination policy.
- **FR-008**: Fragmentation reassembly MUST NOT be implemented in S115. Every datagram with nonzero FRAG MUST be dropped and counted.
- **FR-009**: The association MUST use a fixed number of sockets, a finite idle deadline, a finite datagram size, and a finite exact peer map. At the peer bound, new peers MUST be dropped and counted without evicting active ownership silently.
- **FR-010**: Client datagrams MUST be accepted only from the immutable TCP peer IP and pinned or learned UDP port. Spoofed sources MUST cause no resolution, policy, mapping, or upstream effect.
- **FR-011**: Upstream replies MUST be accepted only from exact socket addresses successfully contacted by this association and currently retained in its peer map.
- **FR-012**: Listener, loopback, private, link-local, multicast, broadcast, unspecified, and ungranted local destinations MUST retain the existing policy refusal behavior for every resolved address.
- **FR-013**: Forwarding MUST preserve UDP payload bytes and datagram boundaries. Response headers MUST name the actual observed upstream source endpoint.
- **FR-014**: Client ingress, upstream sends, upstream ingress, client sends, malformed drops, fragment drops, spoofed-client drops, unsolicited-peer drops, policy drops, resolution failures, saturation drops, oversized drops, timeouts, cancellations, and transport failures MUST be separately countable.
- **FR-015**: Typed application evidence MUST identify association establishment, endpoint mode, accepted destination form and actual endpoint, lengths, drop reasons, peak peer count, and terminal outcome without retaining payload content.
- **FR-016**: Existing connection identity and open/close windows MUST remain the correlation authority. If a remote endpoint is not observed, evidence MUST report unavailable rather than infer one.
- **FR-017**: Runtime cleanup MUST join the association task, release all UDP sockets, clear every peer mapping, and report no residue before declaring clean shutdown.
- **FR-018**: The controlled protocol lab MUST cover IPv4, available IPv6, domain, malformed frames, fragmentation refusal, spoofed source, unsolicited reply, policy refusal, peer saturation, control EOF, idle timeout, cancellation, and cleanup without Internet, elevation, game, or target data.
- **FR-019**: S115 MUST add no generic UDP payload evidence, fragmentation reassembly, arbitrary local relay access, system proxy mutation, target process access, new runtime dependency, or Deep Capture completion claim.
- **FR-020**: Architecture, glossary, plan status, proxy README, and changelog MUST record S115 as closing #311 while leaving #312 through #318 and #334 open.

### Key Entities

- **UDP Association**: One authenticated control connection, pinned client endpoint, fixed UDP sockets, bounded peer map, idle deadline, accounting, and terminal outcome.
- **Client Endpoint Claim**: Request IP and port interpreted against the immutable TCP peer and optionally learned once from the first valid datagram.
- **SOCKS UDP Datagram**: RSV, FRAG, destination address form, destination port, and payload.
- **Remote Peer Mapping**: Exact allowed resolved socket address successfully sent to and eligible to return traffic.
- **UDP Association Event**: Typed establishment, datagram, drop, or terminal fact keyed to the existing connection id.

## Success Criteria

- **SC-001**: Every controlled IPv4, available IPv6, and domain datagram preserves exact payload bytes and boundaries in both directions within its deadline.
- **SC-002**: One hundred percent of spoofed-client, unsolicited-origin, refused-local, malformed, fragmented, oversized, and saturated inputs produce zero unauthorized deliveries and one named counted outcome.
- **SC-003**: Association memory never exceeds the configured datagram buffer and peer count, and socket count remains fixed for its lifetime.
- **SC-004**: Every received client or upstream datagram reconciles to forwarded, refused, malformed, unsupported, saturated, unsolicited, transport-failed, or cancellation loss accounting.
- **SC-005**: Every terminal path releases all association sockets and mappings before clean shutdown reports success.
- **SC-006**: No event, artifact, debug output, or user message contains the capability or UDP payload bytes.
- **SC-007**: The complete repository verification suite passes with no dependency or lockfile drift and no HTTP, TLS, SOCKS CONNECT, routing, lifecycle, cleanup, or correlation regression.

## Assumptions

- S114 supplies shared-listener SOCKS5 authentication, request parsing, reply encoding, runtime identity, and terminal accounting.
- UDP association support is a transport path under the existing child-scoped `socks5h` route; no new environment value is required.
- The TCP peer IP is loopback under the native listener contract. Exact UDP port ownership is the additional tenancy boundary.
- Issue #313 owns generic UDP packet and payload truth. S115 emits metadata and loss only.
