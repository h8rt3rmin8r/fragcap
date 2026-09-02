# Research: Scoped SOCKS5 UDP Association

## Wire And Lifetime

**Decision**: Implement RFC 1928 UDP ASSOCIATE on the authenticated SOCKS5 control connection. Return the actual UDP relay endpoint and end the association when the control connection ends.

**Rationale**: The standard makes the TCP connection the lifetime authority and defines the UDP framing and relay endpoint. Keeping this ownership exact avoids a free-standing UDP service.

**Alternatives considered**: A session-wide relay weakens tenancy. One association surviving control EOF contradicts the protocol and cleanup contract.

## Client Endpoint Pinning

**Decision**: Require the request IP to be unspecified or equal to the TCP peer IP. Pin a declared nonzero port; otherwise learn the first valid datagram port once.

**Rationale**: RFC 1928 requires filtering by expected source IP. Exact port pinning additionally prevents unrelated local processes from becoming tenants after association creation.

**Alternatives considered**: Accepting every loopback port permits local hijack. Repinning on activity creates a race the attacker can win repeatedly.

## Remote Peer Validation

**Decision**: Retain a bounded set of exact allowed remote socket addresses actually sent to. Relay replies only from members of that set.

**Rationale**: This prevents unsolicited traffic and reflection while ensuring evidence names observed endpoints rather than requested or guessed ones.

**Alternatives considered**: IP-only mappings conflate services on different ports. Domain-only mappings invent which resolved endpoint replied. Unbounded mappings violate finite memory.

## Address Families

**Decision**: Use one loopback client-facing UDP socket plus fixed IPv4 and IPv6 upstream sockets. Normalize IPv4-mapped IPv6 addresses before policy and identity comparisons.

**Rationale**: The client relay endpoint can match the control listener family while fixed upstream sockets cover both destination families without dynamically growing socket state.

**Alternatives considered**: One family-specific upstream socket fails the other address form. One socket per peer grows resource state with peer count.

## Fragmentation And Datagram Bounds

**Decision**: Refuse SOCKS-layer fragmentation and count every nonzero FRAG datagram. Receive into one fixed maximum datagram buffer and count truncation-risk input as oversized.

**Rationale**: RFC 1928 permits implementations without fragmentation support. Reassembly adds timers, queues, ambiguity, and amplification surface beyond #311.

**Alternatives considered**: Silent drop violates loss accounting. Unbounded reassembly violates memory and time bounds.

## Evidence Boundary

**Decision**: Emit metadata-only association, datagram, drop, and terminal events through the existing application sink and aggregate counters through `ProtocolAccounting`.

**Rationale**: Existing artifacts own connection identity, loss, finalization, and correlation. Payload retention belongs to #313.

**Alternatives considered**: A new UDP sidecar creates a competing authority. Recording payloads consumes later scope and expands sensitive evidence.

## Dependencies

**Decision**: Add no package.

**Rationale**: Tokio already supplies bounded asynchronous UDP sockets, deadlines, and cancellation. The framing grammar is small and fixed.
