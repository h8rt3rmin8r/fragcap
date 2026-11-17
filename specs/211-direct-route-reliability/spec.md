# Feature Specification: Direct Route Reliability

**Feature Branch**: `codex/s211-direct-route-reliability`
**Created**: 2026-11-16
**Status**: In progress
**Input**: Owner-approved S211 autopilot, issue #619 helper timing repair and the remaining issue #594 packaged WinDivert/Npcap acceptance.

## Scope

**In**: Exact-game direct TCP helper admission, bounded SOCKS handshake work, truthful stage and error evidence, controlled delayed success/refusal/timeout, unrelated packet continuity, and packaged physical original-present/absent packet ordering with history truncation and artifact reconciliation.

**Out**: New routing techniques, launcher proxy support, installed ESO gameplay qualification, release publication, and general packet-loss optimization (#620).

**Done when**: The delayed-success and failure matrix passes without false refusal or worker stall; selected and unrelated traffic remain conserved; both physical packet orderings and incomplete history are demonstrated from a packaged build; all required artifacts agree; repository and hosted checks pass.

## Clarifications

### Session 2026-11-16

- The original-packet physical gate from S210 remains required. Source-only tests cannot close #594.
- A real refusal, a helper I/O error, and an elapsed deadline are separate outcomes. A late proxy CONNECT cannot turn a selected flow into a false refusal.
- The packet worker must keep servicing unrelated pass-through during upstream latency. The handshake has a finite lifetime and exact flow ownership.
- Controlled portable package testing uses a disposable client and finite external peer. It is separate from any claim that an installed product's default paths work.

## User Scenarios & Testing

### User Story 1 - Delayed valid connection (Priority: P1)

A game flow reaches an upstream endpoint after a short delay. The operator receives the exact game response and a positive route result while unrelated workstation packets continue promptly.

**Independent Test**: A controlled origin delays completion by 60-100 ms while selected and unrelated clients run concurrently.

**Acceptance Scenarios**:

1. Given a valid selected flow and a 60-100 ms upstream delay, when SOCKS CONNECT completes, then the flow transfers in both directions and is not recorded as refused.
2. Given unrelated TCP and UDP traffic during the delay, when the route runs, then it is passed through once within the established sentinel bound, without worker watchdog release.

### User Story 2 - Explain a real failure (Priority: P1)

An operator can distinguish a proxy refusal, helper I/O failure, and timeout for the selected flow without treating a later proxy success as a failed route.

**Independent Test**: Controlled refusal, malformed reply, and no-reply endpoints produce different stage, error-kind, elapsed, and terminal records.

### User Story 3 - Trust original-packet evidence (Priority: P1)

A packaged run reports route transfer separately from whether passive capture retained the first selected packet.

**Independent Test**: Controlled WinDivert/Npcap orderings include original-present, original-absent, and truncated history for TCP and supported UDP, with negative ownership and no-payload cases.

### Edge Cases

- The selected socket closes or its endpoint generation changes while the helper waits.
- A retransmitted SYN arrives while the first packet is held, or pending packet capacity is exceeded.
- Session shutdown begins before the helper returns, or the helper task panics.
- The proxy accepts a helper connection but refuses the original destination.
- The passive packet driver retains only a later packet with the same tuple, or its history is truncated.
- A selected UDP tunnel has no peer reply, a selected TCP tunnel has zero bytes, or process events are ambiguous or lost.

## Requirements

### Functional Requirements

- **FR-001 (#619)**: Each selected TCP helper attempt MUST retain exact flow identity, operation stage, elapsed time, and actual I/O error kind or proxy reply category without credential material.
- **FR-002 (#619)**: An upstream stage taking 60-100 ms MUST be able to complete without a premature local read timeout or packet-worker stall. Success, genuine refusal, and true timeout MUST remain distinct.
- **FR-003 (#619)**: Slow helper admission MUST preserve unrelated packet pass-through, selected packet conservation, bounded pending work, cancellation, and one route per exact flow generation.
- **FR-004 (#594)**: Packaged WinDivert/Npcap tests MUST exercise original-present and original-absent orderings, including truncated history, and retain packet truth separately from route transfer.
- **FR-005 (#594)**: Exact process generation, ancestry, tuple, one-use receipt, selected disposition, native proxy leg, and actual client transfer MUST reconcile across route, compatibility, result, and manifest. No packet or helper traffic may be invented as a game original.
- **FR-006**: TCP and supported UDP reuse, socket/packet event reordering, ambiguous or lost socket evidence, refusal, no payload, and incomplete histories MUST remain separate non-positive or limited outcomes where proof is insufficient. A packet admitted to an old selected route MUST NOT enter a different observed generation's bridge. An outbound UDP datagram with insufficient owner proof MUST receive one attempt to return its unchanged original bytes and address as explicitly uninspected pass-through, without claiming old-owner attribution or proxy/client delivery; a failed return remains Lost. Synthetic replies MUST be withheld when the old owner is no longer the unique observed live owner at the send decision; an event still in flight on the separate socket handle remains an explicit limit.
- **FR-007**: Controlled test files, child processes, peer listeners, and native outputs MUST have exact ownership and fallible cleanup after success, failure, and timeout. Any unresolved native evidence remains available for read-only reconciliation.

### Key Entities

- **Pending helper**: One bounded selected flow waiting for SOCKS admission while the packet worker remains available.
- **Helper outcome**: Exact stage, elapsed duration, and success or typed failure for one flow.
- **Original packet state**: Passive present, absent, or history-incomplete evidence independent of selected transfer.

## Success Criteria

- **SC-001**: Every 60-100 ms delayed-success case completes with zero false refusal, worker stall, unrelated sentinel loss, or duplicate selected route.
- **SC-002**: Every controlled refusal and timeout has a distinct bounded terminal outcome and exact helper diagnostic fields.
- **SC-003**: A portable packaged build demonstrates both physical original-packet orderings and truncated history, with TCP/UDP and negative-case artifact reconciliation.
- **SC-004**: Full repository gates and applicable hosted Windows package checks pass at the PR head.

## Assumptions

- Existing signed WinDivert, installed Npcap, native proxy, receipt, route journal, and session bundle remain the authorities.
- No Steam or game is needed for the controlled acceptance matrix.
