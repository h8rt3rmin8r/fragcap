# Feature Specification: Generic UDP Observations

**Feature Branch**: `codex/117-generic-udp-observations`

**Created**: 2026-09-02

**Status**: Complete

**Input**: User description: "S117: implement generic UDP relay and datagram observations under issue #313."

## Overview

S117 closes issue #313 by adding bounded generic UDP payload evidence to the authenticated, control-owned SOCKS5 UDP relay shipped in S115. Every accepted datagram remains one exact observation with direction, source and destination endpoints, sequence, timestamp, observed and retained lengths, retention outcome, and loss provenance. Forwarding stays byte-transparent and independent from retention.

S117 does not create another listener, accept unrouted UDP, infer application semantics, reassemble SOCKS fragments, or replace packet capture as packet truth. The existing association ownership, immutable client endpoint, proxy-owned DNS, destination policy, exact contacted-peer map, finite sockets, and cleanup authority remain unchanged. QUIC and HTTP/3 remain #314, IPv6 parity remains #315, and Deep Capture remains incomplete until #334.

## Clarifications

### Session 2026-09-02

- Q: What is one evidence unit? -> A: One complete UDP payload as received or sent. A record never merges, splits, or silently drops a datagram boundary.
- Q: How does retention interact with a datagram larger than the remaining evidence budget? -> A: The same record retains the available prefix, reports the full observed length, reports the retained length, and names `retention-limit`; forwarding still uses the complete payload.
- Q: What endpoint facts are authoritative? -> A: The client-to-upstream record names the pinned client and actual selected remote endpoint. The upstream-to-client record names the exact observed contacted peer and pinned client. Missing facts remain absent.
- Q: How are duplicate and reordered datagrams represented? -> A: Every ingress receives a monotonic sequence in its own direction and its own timestamp. No deduplication or reordering is performed.
- Q: How is ICMP represented? -> A: Only as a typed socket error when the operating system exposes one to the relay. Absence of such an error is explicitly not evidence that no ICMP occurred.
- Q: Is UDP that did not traverse an authenticated association supported? -> A: No. It remains packet-only and is reported as unrouted and unsupported for application evidence.

## User Scenarios & Testing

### User Story 1 - Retain Exact Routed Datagrams (Priority: P1)

An authorized target routes UDP through its authenticated association and receives bounded application evidence without altering transport behavior.

**Why this priority**: Generic UDP is useful only if each application datagram remains recognizable and correlated without becoming a second packet authority.

**Independent Test**: A controlled loopback client exchanges IPv4, available IPv6, and domain-routed binary datagrams, including empty, duplicate, and reordered values, then reconciles every forwarded payload with exactly one typed record.

**Acceptance Scenarios**:

1. **Given** an accepted client datagram, **when** it is forwarded, **then** one client-to-upstream record retains its exact boundary, selected endpoint, sequence, timestamp, and payload according to policy.
2. **Given** an accepted reply from an exact contacted peer, **when** it is relayed, **then** one upstream-to-client record names the observed source and pinned client without inferred endpoints.
3. **Given** duplicate or reordered datagrams, **when** they traverse the relay, **then** every ingress remains a distinct record in observed sequence with no deduplication or sorting.

---

### User Story 2 - Preserve Forwarding Under Evidence Bounds (Priority: P2)

An operator can disable payload capture, exhaust per-association or session retention, or saturate the application writer without changing valid UDP delivery.

**Why this priority**: Observation must remain a bounded sidecar and cannot become a transport dependency.

**Independent Test**: Controlled exchanges exceed each bound and fill the writer queue while the echo peer still receives complete datagrams and every omitted, truncated, or queue-dropped byte is counted.

**Acceptance Scenarios**:

1. **Given** payload capture disabled, **when** a datagram forwards, **then** metadata records the full observed length with no payload and outcome `intentionally-omitted`.
2. **Given** insufficient retention budget, **when** a datagram forwards, **then** one record retains only the available prefix, reports `retention-limit`, and forwarding preserves all bytes.
3. **Given** application queue or storage pressure, **when** an evidence record cannot persist, **then** the exact lost datagram and byte totals advance independently from relay transport counters.

---

### User Story 3 - Reconcile Loss, Errors, And Unsupported Traffic (Priority: P3)

An operator can distinguish forwarded datagrams, relay drops, observation loss, observable socket errors, and traffic that never traversed the proxy.

**Why this priority**: UDP has asymmetric OS error reporting and no connection completion signal, so false completeness claims are especially easy.

**Independent Test**: The controlled lab injects malformed, fragmented, oversized, refused, saturated, unsolicited, cancelled, storage-failed, and platform-observable socket-error outcomes and verifies conservation plus clean association teardown.

**Acceptance Scenarios**:

1. **Given** any S115 relay refusal, **when** it occurs, **then** no payload is retained and its existing named transport loss remains authoritative.
2. **Given** a socket error visible to the runtime, **when** it terminates or drops work, **then** evidence names its direction, stage, affected endpoint when known, and platform visibility without claiming a remote protocol fact.
3. **Given** UDP that bypasses the association, **when** artifacts finalize, **then** application evidence explicitly identifies unrouted UDP as unsupported and packet capture remains the only payload authority.

### Edge Cases

- Empty datagrams and maximum legal datagrams.
- A datagram larger than the remaining retention budget by one byte.
- Shared session retention exhausted by another protocol or association.
- Duplicate payloads from the same peer and identical payloads from different peers.
- Replies arriving out of request order or immediately before control EOF.
- IPv4-mapped IPv6 endpoint normalization.
- OS error reporting that is synchronous, asynchronous, or unavailable.
- Queue saturation after bytes are forwarded but before the event persists.
- Storage failure after earlier datagrams were written.

## Requirements

### Functional Requirements

- **FR-001**: Generic UDP evidence MUST exist only for datagrams traversing an authenticated S115 UDP association.
- **FR-002**: Each accepted ingress datagram MUST produce at most one typed generic datagram event and MUST never be merged with or split across another event.
- **FR-003**: Each event MUST identify direction, monotonic per-direction sequence, timestamp, observed length, retained length, retention outcome, client endpoint, and exact selected or observed remote endpoint when available.
- **FR-004**: Duplicate and reordered datagrams MUST remain distinct in observed order without inference, deduplication, or sorting.
- **FR-005**: Payload retention MUST honor `capture_payloads`, the existing per-connection body limit, and the existing session body limit.
- **FR-006**: When a budget retains only a prefix, the event MUST retain that prefix in the same datagram record, report the full observed length, and use `retention-limit`.
- **FR-007**: Forwarding MUST use the complete accepted payload and MUST remain independent from event creation, retention, queue admission, serialization, and storage.
- **FR-008**: Generic datagram accounting MUST separately count observed datagrams, observed bytes, retained bytes, omitted bytes, truncated datagrams, and event-queue-dropped datagrams and bytes.
- **FR-009**: Queue-loss accounting MUST retain bounded localized identity by connection, direction, and endpoint where possible, then report exact aggregate overflow totals without unbounded maps.
- **FR-010**: Existing S115 malformed, fragmented, source, policy, resolution, peer-limit, oversized, unsolicited, transport, owner, timeout, and cancellation outcomes MUST remain distinct and MUST retain no refused payload.
- **FR-011**: Observable UDP socket errors MUST be typed by direction and stage, include the OS error kind and known endpoint, and state that visibility is platform-dependent.
- **FR-012**: The implementation MUST NOT infer ICMP type, code, delivery, or absence from a generic socket error or lack of one.
- **FR-013**: Packet capture MUST remain packet truth. Generic UDP records MUST claim only proxy-observed application datagram truth.
- **FR-014**: Unrouted UDP MUST remain unsupported for application evidence and MUST be represented as an explicit omission rather than inferred capture coverage.
- **FR-015**: Application JSON Lines MUST serialize retained payload in base64, omit payload fields when none is retained, and preserve the existing version 2 crash-readable stream contract.
- **FR-016**: Correlation MUST reuse the existing session, connection, timestamp window, and endpoint authorities without creating a second correlation index.
- **FR-017**: Runtime cleanup MUST release all sockets and mappings and finalize generic UDP accounting before reporting clean shutdown.
- **FR-018**: The controlled lab MUST cover IPv4, available IPv6, domain routing, exact boundaries, duplicate and reordered datagrams, maximum size, retention exhaustion, capture disabled, queue pressure, observable socket errors, and cleanup without Internet, elevation, game, or target data.
- **FR-019**: S117 MUST add no new listener, unauthenticated relay, wildcard tenant, fragmentation reassembly, application semantic decoder, system proxy mutation, target-process access, runtime dependency, or Deep Capture completion claim.
- **FR-020**: Architecture, glossary, plan status, proxy README, AGENTS, and changelog MUST record S117 as closing #313 while leaving #314 through #318 and #334 open.

### Key Entities

- **Generic UDP Datagram**: One accepted application payload with exact ingress boundary, direction, endpoints, sequence, timestamp, bytes, and retention outcome.
- **Datagram Retention State**: Shared per-association and per-session byte claims independent from forwarding.
- **UDP Observation Loss**: Bounded localized and exact aggregate accounting for an event that could not be queued or stored.
- **UDP Socket Error**: A platform-observable local socket result with direction, stage, endpoint, and no inferred remote protocol meaning.

## Success Criteria

- **SC-001**: One hundred percent of controlled forwarded datagrams reconcile one-to-one with their exact ingress boundary or a named evidence-loss outcome.
- **SC-002**: Duplicate, reordered, empty, IPv4, available IPv6, domain, and maximum-size datagrams preserve complete forwarding bytes and distinct observed sequences.
- **SC-003**: Retained payload never exceeds the existing per-association or session bounds, and zero observation backpressure changes valid delivery.
- **SC-004**: For every generic UDP event, `observed_bytes = retained_bytes + omitted_bytes`; aggregate observed datagrams equal persisted plus queue-dropped and storage-failed datagrams.
- **SC-005**: Every refused S115 path retains zero payload, and every platform-observable socket error is explicit without an ICMP inference.
- **SC-006**: Unrouted UDP has no application payload claim and remains available only through packet capture.
- **SC-007**: The complete repository verification suite passes with no dependency or lockfile drift and no SOCKS5, HTTP, TLS, artifact, routing, lifecycle, cleanup, or correlation regression.

## Assumptions

- S115 supplies the secure relay, exact datagram boundaries, endpoint ownership, and transport loss accounting.
- S116 supplies the shared bounded retention and queue-loss pattern.
- Application JSON Lines version 2 can add a new typed record without a schema version increment because it is an additive event kind under the published version 2 envelope.
- OS-specific UDP errors are best-effort observations. Portable tests inject the classification seam; real loopback tests cover only errors exposed deterministically by the host.
