# Feature Specification: Direct Route Qualification

**Feature Branch**: `codex/s201-direct-route-qualification` | **Created**: 2026-11-10 | **Status**: Draft

**Input**: Owner-authorized S201 combining #587, #583, and #584; parent #572 closes only after all integrated acceptance criteria pass. Push, official PR, CI, and at most two external review rounds are authorized; final merge is reserved for the owner.

## Clarifications

- **2026-11-10**: #587's 100 ms sweep is the first qualification value, not a production-timeout approval. Choose the smallest value that passes exact selected-route and unrelated-continuity gates; retain a separate stall watchdog.
- **2026-11-10**: Keep #580 independent. Its target-supported TLS key-export research cannot establish this slice's direct-route acceptance.
- **2026-11-10**: Close implementation issues on repository-controlled gates; do not require a future private-title or installed-release observation.
- **2026-11-10**: An arbitrary final-client destination is learned only when that socket opens. The pre-effect plan fixes the destination policy; each selected flow then records its exact original destination under that policy.

## User Scenarios & Testing

### User Story 1 - Reliable exact direct routing (Priority: P1)

An operator selects Deep Capture for a managed target whose final client opens direct TCP or supported UDP sockets. The first packet reaches the native proxy only when its process generation and original destination are proven. Known unrelated packets continue promptly. Uncertain packets remain accounted for and do not acquire game ownership.

**Independent Test**: A controlled cold platform, publisher, and final-client chain sends direct TCP and UDP beside active unrelated traffic; exact final-client flows are selected, while unrelated traffic completes without route admission.

**Acceptance Scenarios**:

1. **Given** proof arrives within the measured candidate window, **when** a pending first packet is reconsidered, **then** available proof is applied before expiry and the exact final-client flow is selected.
2. **Given** a proven helper or unrelated owner, **when** its first packet is seen, **then** it is reinjected promptly without waiting for the candidate deadline or entering application evidence.
3. **Given** missing or ambiguous owner proof, **when** the finite wait ends, **then** the packet is reinjected or explicitly lost with a named outcome and no invented game ownership.

---

### User Story 2 - One guided operator route (Priority: P2)

An operator uses ordinary calibration and Deep Capture commands for a stored target. The reviewed session plan names both complementary child-environment and exact direct-socket effects. A positive route fact requires an exact final-client original-flow receipt, proxy evidence, and independent packet evidence. Old environment-only or launcher-only facts never qualify the new route.

**Independent Test**: A controlled stored target completes guided calibration and an ordinary Deep Capture attempt through the public CLI, with exact route/version facts and consistent terminal, JSON, Doctor, and bundle results.

**Acceptance Scenarios**:

1. **Given** a direct-socket final client, **when** guided calibration measures reachability, **then** it proposes or selects the direct route and records a positive fact only for a correlated final-client flow.
2. **Given** a child proxy flow and a direct final-client flow in one managed chain, **when** both occur, **then** each has one route outcome and no duplicate interception.
3. **Given** only launcher traffic or an older environment-only positive fact, **when** ordinary Deep Capture evaluates direct-route admission, **then** it refuses or requests the exact missing calibration without treating the old fact as proof.

---

### User Story 3 - Accurate first-use instructions (Priority: P3)

An MSI or portable user can identify the actual installation steps, privileges, readiness checks, and meanings of routed, inspected, opaque, refused, bypassed, and packet-only results.

**Independent Test**: Package contents, CLI parser examples, embedded help, README, and site instructions agree with executable installation, CA, Npcap, WinDivert, and extcap behavior.

**Acceptance Scenarios**:

1. **Given** an MSI install, **when** the user follows first-use instructions, **then** installation CA setup is automatic and WinDivert needs no separate installer; Npcap remains a separate prerequisite for live Capture.
2. **Given** a portable ZIP, **when** the user follows first-use instructions, **then** the explicit elevated CA step and Npcap requirement are stated separately from the MSI journey.
3. **Given** a command/mode privilege table, **when** each row is checked against the executable path, **then** its Administrator, ordinary-user, or optional status is accurate, including extcap registration versus live capture.

### Edge Cases

- A socket event arrives after the packet or after its candidate deadline.
- Process proof arrives after socket proof, or a PID or socket tuple is reused.
- An unrelated process connects to the same destination as the selected client.
- The candidate queue saturates, the worker stalls, a packet send fails, or cancellation overlaps pending candidates.
- A selected UDP endpoint closes and another reuses its tuple.
- The proxy receives a route but cannot inspect its encrypted or proprietary payload.
- The exact packaged WinDivert component is missing, damaged, disabled, or cannot open without Administrator rights.

## Requirements

### Functional Requirements

- **FR-001**: The controlled test MUST separately measure packet receipt, socket-event publication, process-proof availability, candidate decision, and actual hold time, and identify the source of the previously observed 10 ms misses where evidence permits.
- **FR-002**: A pending candidate MUST evaluate currently available exact proof before deciding that its deadline expired. Missing or ambiguous proof MUST never imply selected ownership.
- **FR-003**: A first packet with a proven nonselected owner MUST pass through promptly without consuming the full candidate wait. Helper and proxy self-traffic MUST remain excluded.
- **FR-004**: The production candidate wait MUST be selected from controlled exact-route success, unrelated TCP/UDP latency and loss, finite queue occupancy, and dependent event retention. Qualification starts at 100 ms; 250, 500, and 1000 ms are comparison points, not defaults. The chosen bound and any change to S200 FR-003 MUST be recorded explicitly.
- **FR-005**: The route MUST retain finite queue, byte, worker-progress, cancellation, cleanup, and delivered-packet accounting. Pre-receive WinDivert queue loss MUST remain reported as unmeasurable, while active sentinels require zero observed loss.
- **FR-006**: Public Deep Capture and guided calibration MUST authorize one immutable plan combining child-environment routing with an exact final-client direct route, including target and process-generation proof rules, destination policy, transport, address family, component identity, budgets, and cleanup obligations before effects. Each selected flow MUST retain its proven original destination.
- **FR-007**: Direct route admission MUST exclude traffic already reaching the proxy and require exact original final-client process and destination evidence. Launcher, updater, helper, ambiguous, and unavailable observations MUST NOT create positive final-client facts.
- **FR-008**: Compatibility applicability MUST distinguish route kind and version. A positive direct-route fact MUST join exact final-client original-flow receipt, native proxy outcome, and independent passive packet evidence; trust and protocol facts require their own observed boundaries.
- **FR-009**: CLI human and JSON results, Doctor, session artifacts, cancellation, owner release, and recovery MUST agree on selected, bypassed, refused, opaque, inspected, and incomplete route outcomes.
- **FR-010**: Controlled Windows integration and exact MSI/ZIP certification MUST cover cold direct TCP and selected UDP, mixed environment/direct routing, IPv4/IPv6, same-destination unrelated services, saturation, cancellation, selected UDP reply ownership, failure, cleanup, and signed component identity.
- **FR-011**: README, getting started, capture modes, Deep Capture compatibility, embedded and site CLI help, Doctor, troubleshooting, packaging, and migration guidance MUST state the actual MSI and portable journeys, Npcap and WinDivert roles, optional Wireshark/extcap integration, and exact command privileges. Extcap live behavior MUST be checked before publication.
- **FR-012**: Every included issue MUST retain independent acceptance traceability. #572 closes only after #587, #583, #584, and the parent integrated outcome pass repository-controlled gates.

### Key Entities

- **Candidate**: One pending first packet with exact tuple, receipt time, deadline, and bounded disposition.
- **Owner proof**: Socket event and process-generation evidence that may be available at different times.
- **Routing plan**: One pre-effect authority containing complementary environment and direct components.
- **Route fact**: Versioned compatibility evidence scoped to exact target, launch, process, route, transport, and proxy outcome.
- **Route outcome**: Per-flow terminal truth, separate from packet capture and application inspectability.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Every required controlled cold-chain run routes its exact final-client TCP and selected UDP first packets with correct original process and destination, and records no wrong-process application result.
- **SC-002**: Active unrelated same- and different-destination TCP/UDP sentinels report zero missing responses or unexplained loss, and remain within the declared paired latency bound under parallel load and fault cases.
- **SC-003**: Every delivered packet has one reconciled disposition; every saturation, cancellation, stall, refusal, and cleanup case has a named finite outcome. Pre-receive loss is labeled unmeasurable.
- **SC-004**: A controlled public CLI calibration and ordinary Deep Capture journey produce only exact current route facts and consistent human, JSON, Doctor, manifest, and package results.
- **SC-005**: All documented first-use and privilege rows match executable behavior and package contents; parser, links, documentation, source, Windows integration, and package gates pass with zero skipped required cases.

## Assumptions

- S200's signed component, library adapter, bridge, and synthetic matrix are the merged foundation; this slice does not replace WinDivert or introduce target instrumentation.
- The controlled Windows runner and package certification remain the appropriate repository-owned qualification environment. Private-title behavior is not a completion gate.
- #580 application-supported target TLS key-export research is independent and outside S201.
