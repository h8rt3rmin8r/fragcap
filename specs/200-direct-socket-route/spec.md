# Feature Specification: Direct Socket Route

**Feature Branch**: `codex/s200-direct-socket-route` | **Created**: 2026-11-10 | **Status**: Blocked at first-packet route gate

**Input**: Owner-authorized S200 combining #578, #581, and #582 under full Spec-Kit autopilot, including push and official PR publication.

## User Scenarios & Testing

### User Story 1 - Route a final client's direct TCP stream (Priority: P1)

An operator deliberately starts Deep Capture for a managed target whose final client opens an ordinary TCP socket directly to an origin. Fragcap identifies that exact client generation and original connection, makes its stream available to the native proxy, and reports what was actually inspected or remained opaque.

**Independent Test**: A synthetic cold platform, publisher, and final-client chain sends a non-proxy, non-HTTP initial payload to a controlled TCP origin and receives the exact reply while the route and proxy record the same original flow.

**Acceptance Scenarios**:

1. **Given** an authorized cold chain and exact final-client socket, **when** the client sends raw bytes, **then** the origin receives them once and the client receives the reply once, with the original client generation and destination bound to the proxy evidence.
2. **Given** launcher traffic that already honors the child proxy route, **when** the final client opens a direct socket, **then** each connection uses one route path and has one reconciled result.
3. **Given** encrypted bytes without a successful trusted inspection boundary, **when** the route forwards them, **then** the result is opaque or refused with no invented decrypted content.

### User Story 2 - Preserve unrelated workstation traffic (Priority: P1)

An unrelated application and a local service continue communicating while the selected route starts, runs, fails, and stops, including when their destination overlaps the final client's destination.

**Independent Test**: Active sentinels measure byte and reply conservation, packet loss, forwarding errors, and latency before and during every route phase; application artifacts are checked against an exact process oracle.

**Acceptance Scenarios**:

1. **Given** same-destination unrelated traffic, **when** the route is active, **then** the traffic completes without application-artifact admission or route ownership.
2. **Given** missing, delayed, ambiguous, or reused process evidence, **when** a packet arrives, **then** Fragcap does not claim a selected final-client route and reports bounded uncertainty and loss.
3. **Given** a route, proxy, or cleanup fault, **when** shutdown runs, **then** each owned effect is released or named as an exact recovery obligation.

### User Story 3 - Route selected UDP and inspectable protocols (Priority: P2)

A selected final client can exchange datagrams with an original peer through the same authorized session. The result preserves datagram and peer identity and distinguishes generic UDP from a protocol the native proxy can inspect.

**Independent Test**: A controlled UDP peer, QUIC peer, and IPv4/IPv6 cases exercise association and reply ownership with deliberate failures and bounded accounting.

**Acceptance Scenarios**:

1. **Given** a selected direct UDP association, **when** datagrams are sent and replied, **then** boundaries, peer identity, direction, and counts agree across route and proxy records.
2. **Given** an unrecognized UDP payload, **when** it is forwarded, **then** Fragcap reports transport truth without claiming decryption.

### Edge Cases

- A packet can precede its process-bearing socket event; the route must bound the wait and never assign an owner from endpoint coincidence alone.
- A final client may exit while a TCP connection is half closed or a UDP association is active.
- PID and port reuse must create distinct generations and never attach a new flow to an old receipt.
- Helper and proxy loopback legs must not reenter the selected direct route.
- IPv4, IPv6, cancellation, restart, saturation, short-lived sockets, and missed driver events require explicit outcomes.
- A signed component can still fail to load; a failed open must leave no active route and no started managed root.

## Requirements

### Functional Requirements

- **FR-001**: The direct component MUST be an explicit part of the immutable, pre-effect Deep Capture plan, with exact target and final-client scope, TCP/selected UDP transport, the applicable destination admission policy, bounded lifetime, resource ownership, and Administrator prerequisite. Each positively admitted flow MUST bind its exact original destination before packet diversion.
- **FR-002**: The component MUST start process and socket observation before the managed root and MUST bind positive route decisions to exact process generation, causal ancestry, original tuple, transport, and session authorization.
- **FR-003**: Missing, delayed, ambiguous, reused, or out-of-window ownership MUST NOT be reported as a selected final-client route; every affected packet MUST have a bounded disposition and loss record.
- **FR-004**: Selected TCP MUST conserve bidirectional ordered bytes, retransmission behavior, and half-close semantics; selected UDP MUST conserve datagram boundaries and exact contacted-peer replies.
- **FR-005**: Unrelated processes, local services, proxy/helper connections, and loopback relay traffic MUST be excluded from game application evidence and MUST retain measured connectivity during startup, steady state, failure, cancellation, and shutdown.
- **FR-006**: Every packet delivered by `WinDivertRecv` MUST reconcile as reinjected, consumed by a selected route, or explicitly lost. The route MUST report that packet loss in the driver queue before delivery is unmeasurable by WinDivert, avoid claiming a zero total-loss count from adapter counters, and require zero observed loss in the controlled sentinel matrix.
- **FR-007**: One session-authenticated route receipt MUST bind one original flow or association to one helper proxy leg, resist replay, and preserve the original client and destination independently of the helper's PID.
- **FR-008**: The native proxy MUST accept raw direct TCP and selected UDP through its existing protocol engines where valid, applying current destination admission, trust verification, bounded retention, and artifact rules without claiming unsupported inspection.
- **FR-009**: Every original flow MUST have one reconciled result: inspected, opaque, unknown, refused, bypassed, ambiguous, or unavailable; packet facts and proxy facts MUST remain distinct.
- **FR-010**: Stop and recovery MUST close exact driver handles, route tasks, proxy legs, and helper resources, with unresolved effects recorded for Doctor rather than silently abandoned.
- **FR-011**: Ordinary Capture and offline replay MUST neither require nor open WinDivert, and the child-environment route MUST continue to work for launcher connections without duplicate admission.
- **FR-012**: A controlled Windows fixture MUST cover every required TCP, UDP, mixed-route, unrelated sentinel, IPv4/IPv6, uncertainty, failure, cancellation, and restart row with zero skipped required cases and no private target data.

### Key Entities

- **Original flow**: Process generation and causal ancestry, original local/remote tuple, transport, direction, first observation, and terminal state.
- **Route receipt**: Session-bound one-use admission authority connecting an original flow to exactly one helper proxy leg.
- **Route disposition**: Per-packet reinjection, selected consumption, or explicit loss and its reason.
- **Flow result**: Reconciled packet, routing, proxy, and application facts with a precise inspection outcome.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Every required controlled final-client TCP and selected UDP case exchanges its expected bytes or datagrams exactly once and matches the original-flow oracle.
- **SC-002**: Every unrelated sentinel exchange completes with zero route admission, zero missing replies, zero unexplained loss, and latency within the fixture's declared bound during every route phase.
- **SC-003**: All packets delivered to the adapter reconcile to one disposition; every ambiguous, raced, saturated, and injected-failure case produces its expected bounded result. Pre-receive driver queue loss is explicitly reported as unmeasurable, and the controlled sentinel matrix observes zero missing replies or unexplained loss.
- **SC-004**: The complete required Windows matrix has zero skipped rows, and the driver-free ownership corpus passes on every supported development host.
- **SC-005**: Existing proxy, passive Capture, artifact, cleanup, and repository CI gates remain green; no private target identity or traffic appears in evidence.

## Assumptions

- S199's official signed WinDivert 2.2.2-A package and read-only Doctor check are merged and provide the selected component.
- Existing native proxy protocol engines and child-environment routing remain the architectural base.
- The later CLI and guided-calibration selection surfaces are owned by #583, while this slice must provide a complete internally exercised route and bridge.
- Controlled disposable Windows execution, rather than private game testing, is the completion authority for these three issues.

## Clarifications

- **2026-11-10, route scope**: The internally exercised route and proxy bridge belong to S200. Public CLI and calibration selection belong to #583; this does not relax S200's controlled end-to-end gate.
- **2026-11-10, uncertain owner**: A missed first packet or uncertain event is reported as a bypass or ambiguity and never retroactively promoted to successful diversion. The fixture must have positive rows where the direct flow really enters the proxy.
- **2026-11-10, sentinel bound**: The controlled matrix compares each sentinel against a paired same-run baseline, requires zero missing responses and unexplained packet loss, and fails when p99 additional round-trip latency exceeds 50 ms or four times baseline p99, whichever is larger.
- **2026-11-10, driver queue accounting**: The owner approved keeping WinDivert with explicit uncertainty. WinDivert exposes no exact count for packets dropped in its kernel queue before `WinDivertRecv`; exact disposition accounting begins at receipt by the adapter. The route reports that measurement limit, while active sentinels and an exact flow oracle remain the end-to-end loss gate.
- **2026-11-10, destination scope**: A managed client can select a destination at runtime. The pre-effect plan binds the native destination admission policy, and each exact original destination is fixed in the flow receipt before its packets are diverted. A destination observed only after diversion cannot retroactively authorize that flow.
- **2026-11-10, first-packet gate**: Disposable Windows probes observed the correct SOCKET event and opened a narrow NETWORK handle after it, but the initial TCP SYN had already passed. S200 does not satisfy FR-004 or SC-001 under this design; the architecture decision is pending, and no issue closure or production route claim follows from the driver-free bridge tests.
