# Feature Specification: S184 Deep Capture session lifecycle

**Feature Branch**: `codex/s184-deep-capture-session-lifecycle`
**Created**: 2026-10-31
**Status**: Draft
**Input**: Owner-authorized S184 autopilot delivery for active-release defects #526 and #527, including automatic PR publication, exact-head CI, at most two external review rounds, and final owner merge handoff.
**Authority**: Constitution 1.6.0, master specification sections 17, 19, 25, 26, and 27, and the installed v0.10.9 evidence summarized in #526 and #527.

## Scope

**In**: Exact managed-owner release in the presence of an unproven same-named event; truthful route, proxy, trust, and cleanup status; mode-specific ordinary observation time and calibration time; plan and terminal deadline consistency; controlled Windows production-adapter and installed-path regressions; operator documentation.

**Out**: Installation-owned persistent CA (#529), final closure of the live-client trust-loss issue #528, control of unrelated or unproven processes, a new routing strategy, real-title testing of unpublished bytes, version publication, and merge.

## User Scenarios & Testing

### User Story 1 - Release proven owners despite an unknown event (Priority: P1)

An operator finishes a cold managed Deep Capture session. The process trace includes the exact launched Steam root and final client, plus a declared-image event whose ancestry cannot be proven. Fragcap releases the independently proven owners while treating the unknown process as unresolved evidence.

**Independent Test**: Exercise the production Windows shutdown adapter with hidden controlled children representing the exact owned root and client and a separate same-named event. Verify which process receives control and which exits.

**Acceptance Scenarios**:

1. **Given** an exact created root and client with valid receipts plus an unproven declared-image event, **when** observation ends, **then** the proven owners receive bounded graceful or force release and verified exit; the unproven process receives no control.
2. **Given** an unresolved plausible route owner after the release deadline, **when** cleanup reports, **then** route-owner, route, and target-route are partial or failed, and the report does not claim the inherited route ended.
3. **Given** all plausible owners and helpers have verified exits, **when** cleanup finishes, **then** route release may be reported as complete.

### User Story 2 - Receive the requested ordinary observation time (Priority: P1)

An operator authorizes ordinary Deep Capture for four minutes. The authorization plan, runtime observation, Capture span, and terminal report all use that effective duration. Guided calibration retains its shorter bound.

**Independent Test**: Run an installed controlled target for more than one minute and compare the emitted plan, arm-to-stop clock, capture span, terminal result, and cleanup.

**Acceptance Scenarios**:

1. **Given** ordinary Deep Capture with `--duration 4m`, **when** the plan is reviewed and the session runs, **then** the plan states four minutes and observation is not stopped by the calibration 60-second limit.
2. **Given** calibration with a requested duration above its limit, **when** it prepares a plan, **then** the calibration-specific effective bound remains visible and enforced.
3. **Given** an ordinary duration outside the supported finite resource bound, **when** preflight validates arguments, **then** it refuses before session effects and names the supported range rather than silently shortening the request.

### Edge Cases

- An event shares a declared image name but has no proven ancestry, an exited PID, a stale receipt, or a mismatched path, account, or session.
- A new owner or helper appears during graceful shutdown or before the event watcher closes.
- Process control fails, exit cannot be verified, or the release deadline expires with a plausible owner alive.
- Ordinary duration is zero, exceeds the supported bound, or is absent; calibration and ordinary defaults remain distinguishable.
- Capture ends early for a documented independent reason such as packet or byte limit; the terminal result must name that reason without misreporting the authorized time bound.

## Requirements

### Functional Requirements

- **FR-001**: Managed shutdown MUST evaluate exact session-created owner receipts independently from unrelated or unproven declared-image events and MUST continue bounded release of proven owners when one such event is unresolved.
- **FR-002**: Process control MUST apply only to the exact authorized process object or proven descendant after creation time, canonical path, account, session, launch receipt, and ancestry checks. Unrelated, warm, escaped, reused-ID, and ambiguous processes MUST receive no control.
- **FR-003**: Release MUST verify actual owner exit, account for late events and helpers, and report unresolved plausible owners without declaring complete route release.
- **FR-004**: Proxy and trust MUST remain available through the authorized owner-release decision. If a plausible routed owner remains live after that decision, cleanup MUST retain an exact recoverable obligation where possible and describe any loss of proxy continuity without claiming the route ended. This interim guard does not complete #528 or replace #529.
- **FR-005**: Route-owner, route, target-route, proxy, trust, Doctor, and terminal cleanup claims MUST distinguish resource cleanup from live inherited routing and agree with the observed owner state. Doctor MUST verify the exit of every observed plausible route owner before replaying route or trust recovery, and incomplete exit authority MUST leave the journal and trust unchanged.
- **FR-006**: Ordinary Deep Capture MUST use its requested observation duration above one minute through plan authorization, facade admission, Capture execution, and terminal reporting. Calibration MUST keep its distinct shorter maximum.
- **FR-007**: Unsupported ordinary duration values MUST be rejected before listener, bundle, trust, launch, route, or capture effects, with a specific supported range. No layer may silently cap a previously authorized ordinary duration.
- **FR-008**: Controlled Windows regressions MUST exercise the production owner-release adapter with exact and unproven processes, and an installed-path ordinary run longer than one minute MUST reconcile plan, actual observation, Capture span, and cleanup.
- **FR-009**: The affected operator documentation MUST state the actual duration bounds, owner-release outcome, and recovery guidance without implying the permanent CA lifecycle is shipped.

### Key Entities

- **Managed owner**: Exact session-created process identity, receipt, path, account, session, ancestry, and verified exit state.
- **Unresolved event**: Declared-image observation without sufficient proof of ownership or exit, recorded separately from the proven owner set.
- **Effective deadlines**: Mode-specific launch, observation, owner-release, shutdown, and cleanup durations bound to the authorization plan and runtime.
- **Cleanup result**: Resource-level status and reason with causal links to remaining route owners and any retained trust or proxy obligation.

## Success Criteria

### Measurable Outcomes

- **SC-001**: In the controlled proven-root and client plus unproven-event case, both proven owners receive exact bounded release and verified exit, and the unknown process receives zero control calls.
- **SC-002**: Every controlled unresolved-owner case reports partial or failed route release with zero false route or target-route success claims; a fully exited case reports complete release.
- **SC-003**: An installed controlled ordinary run requesting four minutes has a four-minute plan and observed arm-to-stop span within the documented timing tolerance, with a Capture span and terminal record consistent with the actual run.
- **SC-004**: Calibration retains its documented shorter maximum; zero and over-limit ordinary durations refuse before effects; no accepted ordinary duration is silently shortened.
- **SC-005**: Spec-Kit analysis, repository-controlled gates, applicable Windows and package checks, exact-head CI, and the authorized external review rounds finish with no unresolved actionable finding at PR handoff.

## Assumptions

- The existing facade's five-minute observation ceiling is the initial ordinary finite bound for this repair. It becomes an explicit pre-effect range rather than a hidden cap. Any deliberate increase requires a separately reviewed product decision.
- Existing explicit session authorization and per-session CA behavior remain until #529. S184 retains an exact recoverable trust obligation on an unresolved plausible live route rather than claiming #528's permanent solution.
- Controlled hidden Windows children and disposable installed packages provide repository-controlled evidence. Private real-title identities, raw traffic, and owner workstation GUI actions are excluded.
