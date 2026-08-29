# Feature Specification: Deep Capture Compatibility Bootstrap

**Feature Branch**: `codex/097-deep-capture-bootstrap`

**Created**: 2026-08-28

**Status**: Draft

**Input**: User description: "Issue #251. Add a deliberate compatibility-bootstrap workflow that lets an authorized operator move one unknown stored target toward evidence-backed Deep Capture eligibility without weakening the existing safety refusal."

## Clarifications

### Session 2026-08-28

- Q: Does calibration bypass the ordinary Deep Capture safety gate? A: No. Calibration is an explicit evidence-producing mode; ordinary Deep Capture continues to require current supporting facts.
- Q: Can proxy reachability and TLS trust be tested in one launch? A: No. Reachability and TLS are separate declared phases so the first phase never requires a trust change.
- Q: Which real launch case is initially supported? A: Cold Steam protocol launch only. Warm Steam and every unowned or unsupported launch case are refused before side effects.
- Q: May a failed phase write a negative aggregate verdict? A: No. It appends only facts directly observed and reports the phase outcome separately.
- Q: Does calibration publish local evidence? A: No. Facts remain in the existing local target store and session artifacts remain local and sensitivity-labelled.

## User Scenarios & Testing

### User Story 1 - Measure Scoped Proxy Reachability (Priority: P1)

An authorized operator selects one stored target, explicitly chooses its supported launch case and the reachability phase, reviews the planned local effects, and confirms the run. fragcap starts a session-owned loopback proxy, performs the managed launch with target-scoped proxy settings, observes whether traffic reaches the proxy and which process ultimately owns the traffic, then cleans up and records only what it observed.

**Why this priority**: An unknown target cannot safely enter ordinary Deep Capture until target-scoped routing and final ownership are measured. Reachability is useful evidence and does not require adding certificate trust.

**Independent Test**: Run the reachability phase against the controlled local target, decline once to prove no mutation, then confirm and verify bounded completion, observed facts, structured progress, a local evidence bundle, and complete cleanup without any trust-store mutation.

**Acceptance Scenarios**:

1. **Given** an unknown stored target with a supported cold Steam launch case, **When** the operator confirms a reachability calibration, **Then** fragcap measures proxy reachability without installing certificate trust and appends only the routing, propagation, launch, and final-owner facts it directly observed.
2. **Given** the operator declines the displayed plan, **When** calibration ends, **Then** no proxy, trust, launch, bundle, or compatibility-fact mutation occurs.
3. **Given** the managed target never reaches the proxy before the phase deadline, **When** calibration completes, **Then** the result distinguishes no proxy traffic from escaped ownership, unsupported protocol, and internal failure.

---

### User Story 2 - Measure TLS Trust And Inspectability (Priority: P2)

After reachability evidence proves that the selected launch case reaches the final client through the scoped proxy, the operator starts a separate TLS phase. fragcap displays the additional trust effects, requires explicit confirmation, observes certificate acceptance or rejection and application-layer inspectability, records only observed facts, and reports every cleanup result.

**Why this priority**: TLS behavior determines whether Deep Capture can inspect encrypted traffic, but trust mutation is unnecessary and inappropriate until scoped routing has already been established.

**Independent Test**: Seed a current reached-client observation for the controlled target, run the TLS phase with injected trust and proxy adapters, and verify distinct accepted, pinned, metadata-only, unsupported-protocol, interrupted, and cleanup-failure outcomes.

**Acceptance Scenarios**:

1. **Given** current reached-client evidence for the same target and launch case, **When** the operator confirms the TLS phase, **Then** fragcap may trust only the session-owned CA and records the observed trust behavior, protocol, and inspectability separately.
2. **Given** no current reached-client evidence exists, **When** the TLS phase is requested, **Then** fragcap refuses before proxy, trust, launch, bundle, or fact mutation and directs the operator to run reachability first.
3. **Given** the target rejects the local CA, **When** the phase finishes, **Then** certificate pinning remains distinct from unknown trust behavior and no bypass is attempted.

---

### User Story 3 - Reuse Honest Evidence (Priority: P2)

An operator reviews the selected target after calibration and sees the new rows through the existing compatibility view. Ordinary Deep Capture consumes the same current facts when deciding whether that target and launch case are eligible, with no second store, aggregate verdict, or manual transfer.

**Why this priority**: Calibration has product value only when its observations become durable, inspectable input to the existing safety gate.

**Independent Test**: Complete controlled reachability and TLS phases, reopen the local target store, verify the compatibility view preserves every row and provenance field, then prove ordinary Deep Capture accepts the supported current facts and still refuses conflicting, stale, partial, or unknown evidence.

**Acceptance Scenarios**:

1. **Given** a completed calibration, **When** the operator views the target, **Then** every new fact appears with launch case, evidence source, backend provenance, final-owner context, and freshness without an aggregate compatible verdict.
2. **Given** current facts support the exact launch case, **When** ordinary Deep Capture preflights the target, **Then** it consumes those existing rows without copying or translating them into another storage path.
3. **Given** calibration fails or is interrupted after a partial observation, **When** the target is viewed later, **Then** only the partial observations appear and uncertainty is never promoted to compatibility.

### Edge Cases

- The selected target is missing, ambiguous, lacks a durable local row, or is not Steam anchored.
- Steam is already running, starts independently during preflight, or the target launch cannot be prepared.
- The bundle path is nonempty or cannot be created.
- The proxy fails before readiness, becomes unreachable mid-phase, or reports malformed observation records.
- The phase sees launcher-only routing, escaped ownership, no proxy traffic, or a final owner that cannot be identified.
- The phase observes HTTP metadata but no complete response, non-HTTP TLS, QUIC, UDP, plaintext traffic, or no traffic.
- Confirmation is unavailable because input is not interactive, or structured output is requested without preconfirmation.
- Interruption occurs before any observation, after one fact is observed, during trust installation, or during cleanup.
- Cleanup succeeds, partially succeeds, fails, or must report a resource it cannot safely remove.
- Existing current, stale, repeated, or conflicting facts already exist for the same target and launch case.

## Requirements

### Functional Requirements

- **FR-001**: The product MUST provide an explicitly selected compatibility calibration flow for exactly one stored target, one declared launch case, and one declared phase.
- **FR-002**: The flow MUST preserve the ordinary Deep Capture refusal for unknown, stale, conflicting, or insufficient facts; calibration MUST NOT silently convert that refusal into a normal session.
- **FR-003**: The first supported real launch case MUST be a cold Steam protocol launch. Warm Steam, direct executable, publisher launcher, and other unowned launch cases MUST be refused before any proxy, trust, launch, bundle, or compatibility-fact mutation.
- **FR-004**: Calibration MUST have separate reachability and TLS phases. The reachability phase MUST NOT install or remove certificate trust.
- **FR-005**: The TLS phase MUST require current reached-client evidence for the same target and launch case before any side effect.
- **FR-006**: Before either phase mutates local state, the product MUST display the selected target, launch case, phase, bounded deadlines, proxy action, launch action, bundle destination, planned fact writes, trust action if any, and cleanup obligations.
- **FR-007**: The operator MUST explicitly confirm the displayed plan. A declined or unavailable confirmation MUST leave proxy, trust, launch, bundle, and fact state unchanged. Preconfirmation MUST remain explicit and MUST NOT suppress the plan.
- **FR-008**: Calibration MUST never change system-wide proxy settings and MUST apply proxy configuration only through the selected managed launch.
- **FR-009**: Each phase MUST use finite launch, observation, proxy shutdown, and cleanup deadlines and surface visible human progress plus structured lifecycle events.
- **FR-010**: Reachability observations MUST distinguish reached final client, launcher-only routing, escaped ownership, no proxy traffic, and inconclusive measurement.
- **FR-011**: TLS observations MUST distinguish local-CA acceptance, certificate pinning, unknown trust behavior, full inspectability, metadata-only visibility, unsupported protocol, and unknown inspectability.
- **FR-012**: Calibration MUST preserve launch case separately from final-owner handoff and MUST record an observed final owner only when it can identify one.
- **FR-013**: The flow MUST append observations through the existing target-owned compatibility fact model with observed-run provenance, timestamp, product version, backend identity, backend version, proxy mode, launch case, final-owner context, and freshness.
- **FR-014**: Failure or interruption MUST write only facts already observed, MUST NOT synthesize an aggregate positive or negative verdict, and MUST preserve repeated or conflicting history.
- **FR-015**: The flow MUST write a local session bundle that records the plan, phase outcome, structured observations, omissions, fact updates, and per-resource cleanup results without storing credentials or publishing local evidence.
- **FR-016**: Every created proxy process, listener, CA material file, trust entry, bundle artifact, and fact write MUST be represented in the auditable result with its performed, skipped, failed, or not-applicable outcome.
- **FR-017**: Ordinary Deep Capture MUST consume successful current calibration facts through the existing compatibility safety gate without a second persistence or resolution path.
- **FR-018**: The existing target detail view MUST render calibration facts individually, preserving provenance and freshness without selecting an aggregate verdict.
- **FR-019**: Automated verification MUST use a controlled local target and injected trust or process boundaries; it MUST NOT require a game account, mutate the real trust store, change system-wide proxy settings, or commit local titles, paths, endpoints, payloads, or account material.
- **FR-020**: The product specification, glossary, command help, compatibility guidance, and structured-event documentation MUST describe the calibration flow and its safety boundaries consistently.
- **FR-021**: The implementation MUST add no process injection, hooks, target memory reads, executable modification, Winsock catalog changes, interception drivers, target TLS key extraction, certificate-pinning bypass, or silent trust mutation.

### Key Entities

- **Compatibility calibration**: One explicitly confirmed, bounded evidence-producing run for one stored target, launch case, and phase.
- **Calibration plan**: The complete side-effect preview shown before confirmation, including deadlines and cleanup obligations.
- **Calibration phase**: Either reachability without trust mutation or TLS behavior after current reached-client evidence.
- **Calibration outcome**: The phase result and cleanup truth, separate from the compatibility facts directly observed.
- **Compatibility fact update**: One append-only observed row written through the existing target-owned fact model.

## Success Criteria

### Measurable Outcomes

- **SC-001**: A controlled unknown target completes reachability calibration within the displayed deadlines and records sufficient current facts for the next TLS phase without any trust-store mutation.
- **SC-002**: Every unsupported, declined, unconfirmed, or insufficient-evidence request exits before all five mutation classes: proxy, trust, launch, bundle, and compatibility facts.
- **SC-003**: Automated scenarios distinguish at least nine outcomes: reached client, launcher-only, escaped ownership, no proxy traffic, inconclusive, local-CA accepted, certificate pinned, metadata-only, and unsupported protocol.
- **SC-004**: Interrupted and failed scenarios retain 100 percent of directly observed facts and create zero inferred facts.
- **SC-005**: Every calibration run reconciles 100 percent of planned resources into performed, skipped, failed, or not-applicable outcomes and names every cleanup failure.
- **SC-006**: The existing target view and ordinary Deep Capture gate consume the same stored rows with no duplicate persistence or resolution path.
- **SC-007**: The controlled end-to-end verification requires no game account, real trust-store mutation, system-wide proxy change, or committed private local evidence.
- **SC-008**: Full repository, specification, documentation, dependency, encoding, and security gates pass with no new prohibited capability.

## Assumptions

- The existing external mitmdump adapter remains the shipped proxy backend for this slice; native backend research is tracked separately by issue #253.
- Real calibration initially supports only the fact-backed cold Steam launch path already owned by the product. Managed direct-executable launch is tracked separately by issue #254.
- A user-visible compatibility calibration term will be added to the glossary before public help or documentation uses it.
- Calibration facts remain append-only local evidence. Staleness and conflict resolution continue to be interpreted by the existing ordinary Deep Capture safety gate.
- This slice may refactor narrow shared helpers inside the current CLI module, but the library-first Deep Capture extraction remains issue #252.
