# Feature Specification: Causal Launch-Chain Coverage

**Feature Branch**: `codex/s197-causal-chain-coverage`

**Created**: 2026-11-06

**Status**: Implemented, CI green and review feedback resolved; ready for owner final review and merge

**Input**: Owner-approved S197 bundle of GitHub issues #570, #571, and #573, with full causal launch-chain coverage as the default Deep Capture behavior and an end-to-end autopilot delivery through an official reviewed pull request.

## Scope

**Scope in**: Default and explicit Deep Capture inspection scope; packet evidence for exactly proven managed descendants from launch inception through route-owner release; actual packet rejection and application-correlation reasons; exact post-client shutdown of session-owned helpers; scope-specific compatibility, authorization, Doctor, terminal, artifact, documentation, and controlled package evidence.

**Scope out**: A new route for direct sockets (#572), diagnosis or correction of client-facing launcher TLS refusals (#574), general process management, passive Capture policy changes, target instrumentation, unreviewed machine-wide proxy effects, a new release, and private-title compatibility claims.

**Done when**: Every acceptance criterion in #570, #571, and #573 is demonstrated by controlled tests; every packet and resource outcome is reconciled; the requirements and domain checklists are complete; Spec-Kit analysis has no unresolved critical finding; local checks and exact-head CI pass; review comments from the initial and at most one requested second external round are addressed; and the official PR is ready for owner final review and merge.

## User Scenarios & Testing

### User Story 1 - Full chain is the ordinary Deep Capture scope (Priority: P1)

An operator selects a stored game and starts ordinary Deep Capture without an inspection-scope override. Before effects, the plan identifies the full causal launch chain as the effective inspection scope, the target-scoped route, the packet scope, and the bounded shutdown authority. The resulting evidence includes eligible traffic from Steam, publisher launchers, updaters, the game, and unexpected descendants whenever their exact process instances are proven to belong to this launch. Application inspection is attempted only where the selected route actually reaches a connection, and every inspection boundary remains visible.

**Why this priority**: The owner changed the default in #571 after installed v0.10.14 selected game-only handling for a launcher-mediated title.

**Independent Test**: Start a controlled cold chain with declared and undeclared descendants under the ordinary default and compare the authorized plan, retained packets, application records, and unrelated-process exclusions without depending on the separate packet-gate regression case.

**Acceptance Scenarios**:

1. **Given** a cold launcher-mediated target with no scope override, **when** Deep Capture resolves its plan, **then** the effective inspection scope is full launch-chain and the plan states independent packet and application coverage before any effect.
2. **Given** a controlled game that starts an undeclared utility, **when** both instances send packets during the session, **then** eligible packets from the proven game and utility are retained while an unrelated same-name process is excluded.
3. **Given** explicit game-only application inspection, **when** a proven publisher sends packets, **then** those packets remain eligible even though publisher application content follows the separately declared game-only policy.
4. **Given** a middleware inspection failure, **when** the attempt ends, **then** the result reports a full-chain inspection failure or an authorized, separately identified application-scope fallback; the packet scope remains full chain and no connection silently changes inspection mode.
5. **Given** historical game-only facts or a resumed workflow with different scope intent, **when** a full-chain attempt is requested, **then** those facts cannot qualify it and mismatched resume is refused before effects.

### User Story 2 - Packet evidence spans the managed route lifetime (Priority: P1)

An operator who explicitly requests launch-chain inspection receives packet evidence for proven causal descendants before final-client acquisition, during the client observation window, and while inherited routing remains active through route-owner release. If a packet cannot be retained, the reported reason matches the actual admission decision, and application correlation uses that reason rather than a generic scope label.

**Why this priority**: #570 observed 31 exactly owned proxy connections, but no intermediate packets in the installed packet file across three distinct time windows.

**Independent Test**: Replay and live-control a cold platform, publisher, client, and updater chain across the three windows; compare process identity, packet comments, proxy connection ownership, omission reasons, and conservation totals.

**Acceptance Scenarios**:

1. **Given** a proven publisher connection before final-client acquisition, **when** it sends packets, **then** the packet file retains its eligible traffic and correlation can point to the same exact process generation.
2. **Given** an undeclared but proven updater during client observation, **when** it sends packets, **then** the packet file retains its eligible traffic without a profile-stage declaration.
3. **Given** a proven publisher still using the inherited route after the client observation interval, **when** it sends packets before verified route release or the finite release deadline, **then** the packet file retains its eligible traffic or reports a specific counted capture failure.
4. **Given** a rejected packet, **when** statistics and correlation are finalized, **then** waiting, out-of-window, out-of-scope, and unresolved decisions remain distinct and identify the actual reason.
5. **Given** process proof that arrives after the packet, **when** ownership can still be established within the bounded evidence window, **then** eligible packet evidence is retained; exhausted capacity or missing proof is counted and reported rather than silently dropped or attributed by name alone.

### User Story 3 - Exactly owned helpers exit after the client finishes (Priority: P1)

After successful final-client acquisition and packet observation, the previously authorized shutdown covers every exactly proven session-created publisher helper, including same-image siblings. Fragcap first attempts graceful exit where supported, then uses only the plan's bounded force authority for surviving owned instances, and verifies exit before marking inherited routing released. Unproven or unrelated processes remain untouched and leave a named recovery obligation where relevant.

**Why this priority**: #573 observed six session-owned publisher helpers surviving the release ceiling after the client had acquired and observation had completed.

**Independent Test**: Use a controlled managed chain with mixed exiting and surviving same-image siblings after successful client acquisition, plus unrelated and ambiguous candidates, and inspect process actions, listener lifetime, route release, journal, and Doctor.

**Acceptance Scenarios**:

1. **Given** multiple exact session-owned helpers after successful client acquisition, **when** observation ends, **then** the authorized graceful and bounded force sequence reaches every surviving owned instance and verifies each exit before route release succeeds.
2. **Given** a helper with missing creation-time, path, account, session, receipt, or ancestry proof, **when** shutdown evaluates it, **then** no process-control action is sent and the missing proof and finite recovery obligation are reported.
3. **Given** a warm, unrelated, same-name, or ambiguous process, **when** shutdown runs, **then** it receives no control action.
4. **Given** a helper that remains alive at the release deadline, **when** the proxy and journal finalize, **then** the session reports incomplete release and current Doctor state names the unresolved resource without claiming cleanup success.

### Edge Cases

- The managed root can exit before an undeclared child starts, and the final client can hand off to a proven successor; ownership follows exact causal generations rather than one declared role or image string.
- A packet can precede a delayed process-start or socket-owner observation; a bounded evidence path must preserve eligible traffic when proof later becomes available and count capacity loss or unresolved identity separately.
- A port or process identifier can be reused during the session; neither reuse nor an old compatibility row can transfer ownership or eligibility to a new instance.
- Capture may fail or hit a packet or byte limit while the proxy remains active; the post-observation gap and its cause must be explicit, and no complete packet-coverage verdict may be emitted.
- A route owner can exit between validation and shutdown, ignore graceful close, fail termination, or spawn a late child; exit verification and the finite release deadline determine the outcome.
- A connection can fail TLS inspection or use a direct socket that never reaches the proxy; the packet record, route outcome, inspection outcome, and cleanup outcome remain independent.
- Ordinary Capture, warm processes, and ambiguous ownership acquire no new managed-shutdown authority.

## Requirements

### Functional Requirements

- **FR-001 (#571)**: An ordinary Deep Capture request without an inspection-scope override MUST resolve to full causal launch-chain application inspection before effects for every supported managed launch topology. The existing explicit game-only choice MUST remain available.
- **FR-002 (#571)**: The pre-effect plan MUST separately state requested and effective application inspection scope, full-chain packet scope, exact target-scoped route, possible authorized application-scope fallback, bounded process-control authority, and resource lifetime. Human and JSON output MUST agree.
- **FR-003 (#570/#571)**: Deep Capture packet admission MUST start with the managed launch and cover every exactly proven causal descendant, including undeclared intermediates and descendants of the final client, independently of executable name, profile role, and application inspection scope. Unrelated or unproven instances MUST NOT be admitted as owned.
- **FR-004 (#570)**: Packet collection MUST continue through verified route-owner release or the finite release deadline while the proxy can still accept inherited-route traffic. A capture failure or intentional shorter window MUST appear as an exact post-observation coverage gap.
- **FR-005 (#570)**: Delayed ownership proof MUST have a bounded evidence path that can retain eligible earlier packets once proven. Saturation, attribution failure, missing proof, and observation loss MUST be counted with exact outcome identity; no name-only or PID-only inference may fill a gap.
- **FR-006 (#570)**: Each gate rejection MUST carry its actual waiting, out-of-window, out-of-scope, or unresolved reason into loss statistics and flow omission correlation. Existing packet and sink conservation identities MUST remain true.
- **FR-007 (#570/#571)**: Packet records, process trace, application connections, manifest, terminal result, and Doctor MUST distinguish packet coverage, route reachability, application inspectability, proxy lifetime, and cleanup without inventing a positive fact from another dimension.
- **FR-008 (#571)**: Guided calibration, exact compatibility facts, no-progress checks, target guidance, resume equality, and ordinary Deep Capture eligibility MUST use the new effective full-chain policy identity. Historical game-only facts MUST remain readable but MUST NOT qualify a full-chain attempt.
- **FR-009 (#571)**: An authorized application-scope fallback MUST identify the original failure and new scope as a new attempt, retain full-chain packet evidence, and never silently switch an established connection from inspection to relay.
- **FR-010 (#573)**: The route-owner inventory MUST track every session-created descendant by exact process generation and causal ancestry from managed launch through final-client acquisition and shutdown, including multiple same-image helpers.
- **FR-011 (#573)**: Only previously authorized exact owned instances MAY receive bounded graceful or force shutdown after the actual observation end or another terminal condition. The implementation MUST verify exit before route release succeeds and MUST not control passive Capture, warm, unrelated, escaped, reused-ID, or ambiguous processes.
- **FR-012 (#573)**: Missing ownership proof, process-control failure, late child, or unverified exit MUST retain a finite incomplete release and an actionable recovery obligation. Listener, route, journal, Doctor, and terminal output MUST agree on the present and historical states.
- **FR-013 (#570/#571/#573)**: Controlled cases MUST independently cover default versus explicit application scope, cold launch inception, pre-client and post-client traffic, undeclared descendants, same-name unrelated processes, delayed identity, process-ID reuse, mixed helper exits, failed shutdown, route deadline, and loss conservation.
- **FR-014**: The master specification, slice chronology, public CLI/help, target and Doctor guidance, native artifact documentation, glossary, and changelog MUST describe the delivered behavior and its remaining direct-routing and TLS compatibility boundaries.
- **FR-015**: S197 MUST add no target memory access, injection, function hooking, packet interception or filtering driver, executable modification, silent system-wide proxy effect, target TLS key extraction, or new authority over ordinary Capture processes.

### Key Entities

- **Causal launch instance**: One process generation with exact creation time, path, account, session, launch receipt, and proven ancestry to the session-created root.
- **Packet coverage window**: The managed launch inception through verified route release or a finite deadline, with independently recorded observation and failure boundaries.
- **Packet admission outcome**: Retained evidence or one actual rejection reason with counted loss and optional flow correlation.
- **Inspection policy identity**: Requested and effective application scope, middleware treatment, packet scope, and versioned policy identity used by authorization and exact compatibility facts.
- **Route-owner release outcome**: Verified exit of every applicable owner, or a named incomplete state with remaining process and resource obligations.

## Success Criteria

### Measurable Outcomes

- **SC-001**: In controlled default, explicit launch-chain, and explicit game-only sessions, 100% of packets from exactly proven managed descendants across launch, client observation, and route release are retained unless a named capacity or source failure is induced; zero unrelated-process packets are admitted as owned.
- **SC-002**: Every controlled rejected packet increments exactly one actual reason and all sink and capture conservation equations hold with zero unexplained loss.
- **SC-003**: Every controlled post-client owned-helper scenario either verifies all applicable exits before successful route release or reports the exact remaining proof, process, and resource obligations by the finite deadline; zero unowned processes receive a control action.
- **SC-004**: Every default and resume scenario uses the full-chain policy identity; zero historical game-only facts authorize a full-chain attempt, and every scope fallback retains the prior attempt and full-chain packet evidence.
- **SC-005**: Controlled source, package, and exact-head CI checks pass and the PR completes the permitted external review rounds with no unresolved actionable comment. Private-title compatibility and a new release are not claimed.

## Assumptions

- The owner-approved scope is the three-issue S197 bundle; #572 and #574 remain separate open issues and their observed field outcomes are not retroactively declared fixed.
- A managed launch has one pre-effect authorization and an exact session-created root. Existing process and socket evidence can support exact causal identity; a bounded capacity failure remains an explicit incomplete evidence outcome.
- Existing route-owner release and shutdown deadlines remain finite. S197 may adjust packet recording lifetime to the already reviewed route lifetime without creating an unbounded session.
- Controlled tests and hosted package evidence establish implementation acceptance. Optional private-title testing follows a published release and may create a new release-specific defect.
