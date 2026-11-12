# Feature Specification: Packet Correlation Integrity

**Feature Branch**: `codex/s206-correlation-integrity`

**Created**: 2026-11-12

**Status**: Draft

**Input**: Owner-authorized S206 autopilot for #594 and #596 after merged S205. The two issues share exact flow, process-generation, packet-window, and omission authority, but retain independent closure gates.

## Scope

**In**: Direct-route calibration when a selected original packet is or is not retained by Npcap (#594); launch-chain packet scope and proxy-flow omission localization (#596); controlled cold launcher/publisher/client and undeclared-helper cases; reconciliation among packet, route, process, proxy, compatibility, fact-store, terminal, and manifest evidence; package and hosted qualification.

**Out**: Private-title compatibility, a release, TLS refusal issue #597 (closed as not planned), key-export research #580, target process instrumentation, changing passive Capture into active routing, and claiming an unobserved physical driver order.

**Done when**: Every applicable controlled and hosted gate passes, review findings are resolved, and the PR is ready for owner merge. #594 and #596 close separately only when each issue's full acceptance evidence exists. An unavailable physical original-present ordering remains an explicit open #594 row and does not block independent #596 completion.

## Clarifications

### Session 2026-11-12

- **Physical packet order**: A driver-free fixture validates decisions but does not satisfy #594's packaged Npcap/WinDivert observation. Use the existing disposable hosted Windows runner before requesting another local administrator prompt.
- **Unresolved scope**: Do not call every unresolved packet unrelated traffic, and do not turn an unrelated flow's omission into a failure of another exact connection. Preserve observed counts and unknown pre-receive loss separately.
- **Issue closeout**: A passing #596 result remains independently closable if the original-present order for #594 cannot be observed. A private game run is not a standing gate.

## User Scenarios & Testing

### User Story 1 - Calibrate a direct route from exact client evidence (#594, Priority: P1)

An operator selects a cold final client. The route may consume the first packet before passive capture observes it. Calibration reports the client's proven exchange without inventing passive packet truth, and a matching later Deep Capture decision uses the same current fact.

**Independent Test**: Controlled TCP and supported UDP cases exercise original-present and original-absent packet evidence, successful and negative delivery, compatible fact write, and later admission or refusal. A packaged Npcap/WinDivert run separately establishes physical ordering.

**Acceptance Scenarios**:

1. Given an exact one-use route receipt, generation, destination, client response, and two-way exchange, when the original packet is absent from passive capture, then calibration may record a positive direct-route fact while packet evidence remains unavailable.
2. Given the same route proof with an original passive packet present, when calibration runs, then the routing decision agrees with the absent case and separately reports the retained packet.
3. Given incomplete delivery, competing or reused identity, helper-only traffic, truncated history, refusal, or no-payload exchange, when calibration runs, then no positive route fact or later authorization is inferred.

### User Story 2 - Resolve launch-chain packet scope per flow (#596, Priority: P1)

An operator selects a cold platform-to-publisher-to-client chain. Eligible packet observations before and after final-client acquisition remain linked to the exact causal descendant, while ambiguous or unrelated traffic retains a local, named omission.

**Independent Test**: A controlled chain includes a declared launcher, publisher, final client, undeclared owned helper, unrelated process, and proxy flows before and after acquisition. Each packet and proxy interval has an expected scope decision and loss account.

**Acceptance Scenarios**:

1. Given a proven causal descendant and eligible packet window, when its packet is observed, then packet and proxy evidence join to the exact owner or state the precise missing proof.
2. Given an unresolved interval on one connection, when another connection has complete proof, then the latter remains complete; the former reports its own waiting, history, ancestry, or window cause.
3. Given unrelated or out-of-window traffic, when capture runs, then it is not silently promoted into the selected chain and its observed disposition is conserved.

### Edge Cases

- A socket event arrives after the first packet, has uncertain clock ordering, or belongs to a reused PID or endpoint.
- Packet history is bounded, and later proof cannot reconstruct a discarded observation.
- A route receipt is replayed, ambiguous, or names a different generation or destination.
- Proxy intervals overlap a pending-head interval, a cooldown, or route-owner release.
- A selected packet is consumed by the route while Npcap sees only later packets or none.

## Requirements

### Functional Requirements

- **FR-001 (#594)**: Positive direct-route calibration MUST require exact final-client generation, one-use receipt, original tuple/destination, observed client response, and nonzero two-way transfer. Packet presence MUST be independent.
- **FR-002 (#594)**: Compatibility observation, terminal verdict, target-store fact, and later admission MUST agree on one current case. Incomplete and changed cases MUST refuse before effects.
- **FR-003 (#594)**: The packaged Npcap/WinDivert qualification MUST record both original-present and original-absent selected flows with separately named route and packet authority; no synthetic test may stand in for a physical observation.
- **FR-004 (#596)**: Packet scope MUST be resolved per exact flow and causal generation through route-owner release. An omission on another flow or outside the interval MUST NOT taint a proven flow.
- **FR-005 (#596)**: Waiting, scope-unresolved, out-of-window, capture-buffer, and sink outcomes MUST each retain a distinct cause and conserve observed packet counts. Unobserved pre-receive traffic MUST remain unknown.
- **FR-006 (#596)**: Process trace, compatibility, terminal human/JSON, and manifest MUST agree on completeness and localized omission counts. A partial trace MUST NOT become complete.
- **FR-007**: Controlled tests MUST cover cold chain stages, undeclared owned helper, unrelated process, reused/competing ownership, incomplete socket timing, and exact proxy windows without target memory access.
- **FR-008**: Ordinary Capture MUST remain passive; unrelated traffic in Deep Capture retains the existing finite header-only pass-through and measured continuity requirements.

### Key Entities

- **Flow ownership decision**: Exact flow, observed time and clock uncertainty, process generation, causal ancestry, scope window, decision, and reason.
- **Route receipt**: One-use selected original packet identity, disposition, client response, proxy leg, and transfer proof.
- **Packet omission**: Flow-local cause, observed versus unobserved status, bounded count, and interval.
- **Compatibility fact**: Current case identity and evidence lineage used by later admission.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Every required controlled TCP/UDP direct-route positive and negative case yields the expected verdict and later admission decision, with zero invented packet observations.
- **SC-002**: Both physical original-packet orderings are observed on package bytes for #594 closure; a missing ordering remains explicitly unverified.
- **SC-003**: Every controlled launch-chain flow has a localized scope and proxy-window result, with no cross-flow omission contamination and conserved observed packets.
- **SC-004**: Normal, JSON, compatibility, process-trace, and manifest outputs agree on complete versus partial and all counted loss causes.
- **SC-005**: Required checks and reviews pass on the pushed candidate; unrelated sentinel loss is zero and latency remains inside the existing paired bound where the route runs.

## Assumptions

- Existing native route, process-trace, and compatibility schemas are preferred; any schema change must preserve readback of existing native bundles and be recorded as an explicit versioned decision.
- Hosted disposable Windows runners can run the signed-driver matrix. An installed-product default-path claim requires actual default-path readback and no unapproved storage override.
- The original v0.11.0 private field bundle is diagnostic context, not a required test dependency or public artifact.
