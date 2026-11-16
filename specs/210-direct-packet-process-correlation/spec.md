# Feature Specification: Direct Packet and Process Correlation

**Feature Branch**: `codex/s210-direct-packet-process-correlation`
**Created**: 2026-11-16
**Status**: Ready for Review
**Input**: Owner-approved S210 autopilot for #594 and #618 under #607. Reconcile exact direct-route evidence even when diversion consumes the original packet, and bind only proven post-exit socket tails to their process generation.

## Scope

**In**: Exact final-game direct-route receipts, demonstrated client transfer, original packet presence or absence, finite packet-history omissions, process-generation and tuple conflicts, post-exit RST and continuity-proven ACK/FIN attribution, versioned artifact readback, and controlled source/package gates.

**Out**: Broader launch-chain scope correlation (#596), local buffer-loss reduction (#620), launcher and generic TLS repairs (#617, #619, #621), new routing techniques, installed ESO success, and release publication.

**Done when**: Controlled original-present and original-absent TCP/UDP route cases, post-exit reset and ACK/FIN continuity cases, and their negative reuse/loss cases reconcile across route, packet, process, compatibility, result, and manifest evidence; the repository gates pass and the PR is ready for owner merge.

## Clarifications

### Session 2026-11-16

- **Independent authorities**: An exact route receipt and demonstrated client transfer may establish route reachability while the original passive packet remains unavailable. The result retains both statements separately.
- **Tail boundary**: The process exit timestamp remains exact. A short post-exit tail is an explicit attribution basis only when socket creation, same-flow continuity or reset flags, unique process generation, image, and complete event history support it. An empty socket image is never filled from an unrelated process record.
- **Physical evidence**: Controlled packaged driver readback is required for issue acceptance, but a private installed game run is not a repository-controlled completion gate.

## User Scenarios & Testing

### User Story 1 - Trust an exact direct route (Priority: P1)

An operator starts game-only Deep Capture. A selected direct flow may have its original packet consumed before passive capture observes it. The operator can still see a proven route and client transfer, with packet absence clearly reported.

**Independent Test**: Controlled TCP and UDP routes with the original packet present, absent, or history-truncated yield distinct route and packet facts.

**Acceptance Scenarios**:

1. Given exact one-use route, process generation, destination, and demonstrated client transfer, when the original packet is absent, then route reachability is positive and original packet evidence remains unavailable.
2. Given the same proof with a retained original packet, when the bundle is read, then the packet flow and game owner are correlated without labeling any helper packet as game traffic.
3. Given no client transfer, conflicting owner, reused tuple, ambiguous receipt, refusal, or missing event history, when evidence is reconciled, then no positive route result is inferred from a tunnel or packet count alone.

### User Story 2 - Explain post-exit socket tails (Priority: P1)

An operator reads a process trace after a game exits. Late reset and contiguous closing packets keep their exact socket owner only when the observed facts prove the same process generation.

**Independent Test**: A parser-to-flow-to-process matrix covers reset, ACK/FIN continuity, PID and tuple reuse, missing birth or socket creation, event loss, empty image, and tails beyond the finite bound.

**Acceptance Scenarios**:

1. Given a reset within the finite tail bound, exact socket creation inside a unique process lifetime, and complete events, when the packet arrives after exit, then the trace names that generation and `post-exit-reset-tail` basis while retaining the actual exit time.
2. Given ACK/FIN after exit and a prior same-flow packet owned in lifetime, when creation and continuity are proven, then the trace names `post-exit-contiguous-tail` basis.
3. Given a missing or conflicting proof, when a tail arrives, then the owner remains unresolved with a named limitation.

### Edge Cases

- WinDivert consumes the first packet before Npcap observes it, while the helper and proxy traffic are visible.
- The passive registry retains a later packet but omits the first, or its finite history overflows.
- A tuple or PID is reused after the first process exits.
- A reset has a PID but the socket image is empty.
- Process events or packet history are lost or unparseable.
- A route receipt exists but TCP return data is not acknowledged or a UDP peer reply is absent.

## Requirements

### Functional Requirements

- **FR-001 (#594)**: A selected direct route MUST retain its exact one-use receipt, original tuple, final-client generation, selected packet disposition, demonstrated client transfer, proxy leg, and passive original-packet state as independent facts.
- **FR-002 (#594)**: Route reachability MUST NOT depend on the passive driver retaining a consumed first packet. A helper packet MUST NOT count as the game's original packet.
- **FR-003 (#594)**: Absent, incomplete, conflicting, refused, no-payload, and unproven-transfer cases MUST have distinct outcomes across compatibility, route journal, terminal result, and manifest.
- **FR-004 (#618)**: A post-exit socket-owner interval MUST bind only to a unique same-PID and same-image generation whose lifetime contains the socket creation time, with a finite lag and complete process-event evidence.
- **FR-005 (#618)**: A post-exit reset MUST carry the observed TCP reset flag. A non-reset ACK/FIN tail MUST additionally prove earlier same-flow in-lifetime ownership and no conflicting reuse.
- **FR-006 (#618)**: Missing image, birth, creation time, event completeness, unique generation, or bounded lag MUST leave ownership unresolved and report the limitation without moving the process exit time.
- **FR-007**: Versioned readers MUST validate record counts and required fields, preserve older read-only records, and reconcile process and packet loss without an unreported discard.
- **FR-008**: The selected route remains exact-game and exact-destination scoped. Capture remains passive and no target memory or unrelated payload is acquired.

### Key Entities

- **Direct-route receipt**: One selected original flow with its tuple, process generation, packet disposition, and helper admission window.
- **Route transfer proof**: Transport-specific evidence that the final client received and acknowledged returned traffic.
- **Packet observation**: A passive packet with timestamp, flags, socket creation time and owner, or an explicit omission.
- **Process instance**: One unique PID generation with actual start and exit timestamps, image, and causal ancestry.
- **Tail basis**: The exact reset or continuous closing-flow proof used for a post-exit owner link.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Every controlled original-present and original-absent TCP/UDP route case reports route and packet truth separately, with zero helper packets attributed to the game.
- **SC-002**: Every controlled post-exit reset or ACK/FIN case either binds one proven generation inside the finite bound or reports one explicit unresolved limitation; no negative case invents an owner.
- **SC-003**: All controlled versioned artifact readers agree on counts, outcomes, and incompleteness; none promotes an unobserved packet to observed.
- **SC-004**: The full repository gate and applicable packaged Windows qualification complete without relying on a private game run.

## Assumptions

- The existing signed direct-routing component and passive packet driver remain the only route and packet authorities.
- Existing route and process artifacts can be extended additively; older versions remain readable.
- The 100 ms post-exit bound observed in the diagnostic prototype is the initial finite candidate and must be covered by positive and negative tests.
