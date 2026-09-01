# Feature Specification: Warm-To-Cold Restart

**Feature Branch**: `codex/113-warm-to-cold-restart`

**Created**: 2026-09-01

**Status**: Draft

**Input**: User description: "S113: add an explicit, safe warm-to-cold close-and-retry workflow for direct, Steam platform, and publisher Deep Capture launch cases under issue #309."

## Overview

S113 closes issue #309 by giving an authorized operator a bounded path from an observed warm launch case to a newly prepared cold Deep Capture session. fragcap reports only the image-name evidence it can observe, explains that this does not prove selected-target identity, and asks the operator to close the relevant application through its normal controls. fragcap never terminates a process. After every observed image is absent, it resolves the target and launch state again, prepares a new cold plan, and requires authorization again before any bundle, proxy, trust, routing, launch, or compatibility effect.

The workflow covers direct executables, Steam platform launches, and declared publisher chains. Without explicit selection, all existing warm refusals remain unchanged.

## Clarifications

### Session 2026-09-01

- Q: May fragcap terminate a warm process after confirmation? -> A: No. The permitted snapshot cannot prove that a same-named process belongs to the selected target, so fragcap only waits while the operator uses normal application shutdown.
- Q: What constitutes a successful transition? -> A: Every observed declared image is absent before the finite deadline, the target and launch state are resolved again, and the resulting case is an exact supported cold case.
- Q: Is the original restart confirmation sufficient to launch? -> A: No. The newly prepared cold plan requires a separate authorization before effects.
- Q: What happens without interactive input? -> A: `--yes` pre-confirms both prompts; machine-readable or noninteractive runs without it refuse before effects.
- Q: Does S113 expand compatibility calibration? -> A: No. Warm calibration expansion remains issue #317; the restart option applies to ordinary Deep Capture only.

## User Scenarios & Testing

### User Story 1 - Reach A Cold Managed State Safely (Priority: P1)

An operator requests warm restart for a selected target. fragcap names the observed warm launch class and image names, obtains consent, waits while the operator closes the application normally, then proves the declared images absent and prepares the cold session again.

**Why this priority**: Child-scoped routing cannot affect an already-running process, while automatic termination would risk an unrelated same-named process.

**Independent Test**: A scripted inventory moves from one warm direct, Steam, or publisher state to cold and proves that the workflow emits a plan, requests consent, performs no stop action, and returns one exact cold case within the deadline.

**Acceptance Scenarios**:

1. **Given** a warm direct, Steam, or publisher observation and explicit restart selection, **when** the operator confirms and closes the application normally, **then** fragcap waits until every declared image is absent and resolves the target again.
2. **Given** an observed same-named process whose selected-target identity cannot be proven, **when** the workflow is presented, **then** fragcap labels the identity uncertain and never claims ownership or terminates it.
3. **Given** the observed images become absent before the deadline, **when** the target is re-resolved, **then** exactly one supported cold launch case is prepared from current facts.

---

### User Story 2 - Keep Every Non-Success Explicit (Priority: P2)

An operator receives a precise no-effect result when consent is declined, the application remains warm, identity or inventory is uncertain, the target changes, or cold preparation fails.

**Why this priority**: A timeout or ambiguous transition must never fall through into a launch whose route or ownership is unproven.

**Independent Test**: Scripted consent and inventory sequences cover decline, timeout, inventory failure, changed launch declaration, still-warm recheck, and preparation failure, each with a distinct result and zero Deep Capture effects.

**Acceptance Scenarios**:

1. **Given** the operator declines either confirmation, **when** the workflow ends, **then** it reports which authorization was declined and applies no Deep Capture effects.
2. **Given** any declared image remains present at the deadline, **when** waiting ends, **then** fragcap reports a warm-state timeout and does not launch or force kill anything.
3. **Given** the target, launch declaration, or observable state changes unexpectedly, **when** fragcap re-resolves it, **then** it refuses with the exact uncertainty or preparation reason.

---

### User Story 3 - Authorize The Newly Prepared Cold Session (Priority: P3)

After the warm state clears, an operator reviews the resulting cold launch summary and separately authorizes the session that will create effects.

**Why this priority**: The plan after shutdown may differ from the facts observed before shutdown, so the earlier consent cannot authorize it truthfully.

**Independent Test**: A warm-to-cold sequence proves no session effect occurs between cold detection and the second authorization, and that authorization binds to the newly prepared cold plan.

**Acceptance Scenarios**:

1. **Given** cold state has been reached, **when** fragcap prepares the session again, **then** it displays or emits the target, original warm case, resulting cold case, deadline, and no-force-kill policy before authorization.
2. **Given** the operator authorizes the new cold plan, **when** Deep Capture starts, **then** the ordinary exact direct, owned platform, or publisher launch path executes and retains its existing cleanup authority.
3. **Given** the later launch or session fails, **when** the run finalizes, **then** existing failure and cleanup evidence remains authoritative and the restart result does not hide it.

### Edge Cases

- A same-named unrelated process starts or remains present while the operator closes the intended application.
- A launcher closes but a helper, intermediate, or client image remains present.
- A process exits and a same-named process starts before the next snapshot.
- The process inventory cannot be read during initial detection or waiting.
- The stored target, install root, application identifier, or launch roles change before re-preparation.
- The operator interrupts during either prompt or during the wait.
- `--wait` is omitted, zero, or longer than the restart maximum.
- Restart is requested for a target already in a supported cold state.
- Restart is combined with compatibility calibration or the controlled test target.

## Requirements

### Functional Requirements

- **FR-001**: The warm-to-cold workflow MUST be selected explicitly and MUST NOT weaken the existing default warm refusals.
- **FR-002**: Initial detection MUST distinguish direct, platform, publisher-launcher-only, and publisher-chain warm observations without claiming that a same-named process belongs to the selected target.
- **FR-003**: Before waiting for shutdown, fragcap MUST present the selected target, observed warm case, observed declared image names, finite deadline, identity limitation, no-force-kill policy, and next steps.
- **FR-004**: Waiting MUST require interactive affirmative consent or `--yes`; structured and noninteractive execution without `--yes` MUST refuse before effects.
- **FR-005**: fragcap MUST NOT terminate, signal, message, inject into, open a handle to, or relaunch an observed warm process. The operator MUST use the application's own normal shutdown mechanism.
- **FR-006**: The wait MUST be finite, MUST use the shorter of the operator's acquisition bound and a two-minute maximum, and MUST report expiry distinctly.
- **FR-007**: Cold detection MUST require absence of every declared image associated with the observed warm case. Partial closure MUST remain warm.
- **FR-008**: Every inventory read failure, interruption, timeout, declined consent, changed declaration, unresolved target, and re-preparation failure MUST be a distinct visible non-success outcome before session effects.
- **FR-009**: After cold detection, fragcap MUST resolve the stored target, launch declaration, local platform facts, and launch case again rather than reuse the warm preparation.
- **FR-010**: The resulting case MUST be the corresponding supported cold direct, platform, or publisher case; any other result MUST refuse.
- **FR-011**: The newly prepared cold plan MUST require a second interactive authorization or the same explicit `--yes` preconfirmation before effects.
- **FR-012**: The second authorization MUST bind to the re-prepared plan and MUST occur before bundle, proxy, trust, routing, process launch, or compatibility mutation.
- **FR-013**: Structured and human output MUST expose the restart plan and terminal restart outcome, including warm case, cold case when available, target, deadline, authorization state, and exact reason.
- **FR-014**: After authorization, the existing direct, owned Steam platform, or publisher managed launch path and its bounded cleanup MUST remain authoritative.
- **FR-015**: A later launch, session, or cleanup failure MUST remain visible and MUST NOT be replaced by a successful restart outcome.
- **FR-016**: Compatibility calibration, ordinary Capture, targets not using restart, and the controlled target MUST retain their existing behavior.
- **FR-017**: S113 MUST add no force-kill default or fallback, target process handle, system-wide proxy mutation, new runtime dependency, or real target data in fixtures.
- **FR-018**: The architecture of record and changelog MUST record S113 as closing issue #309 without claiming generic transport support or Deep Capture completion.

### Key Entities

- **Warm Restart Plan**: Immutable pre-effect description of the selected target, observed warm class, declared image set, bounded deadline, identity limitation, and no-force-kill policy.
- **Restart Consent**: Explicit permission to wait while the operator performs normal application shutdown. It grants no process-control authority.
- **Cold Observation**: Evidence that every declared image from the current target launch declaration is absent in one complete snapshot.
- **Reprepared Cold Plan**: A newly resolved supported managed launch plan created after cold observation.
- **Restart Outcome**: Named result covering cold-ready, initial decline, authorization decline, timeout, inventory failure, changed state, re-preparation failure, interruption, or later session result.

## Success Criteria

### Measurable Outcomes

- **SC-001**: One hundred percent of controlled warm direct, Steam, and publisher sequences perform zero process-control actions and reach cold-ready only after every declared image is absent.
- **SC-002**: One hundred percent of decline, timeout, uncertainty, inventory failure, changed-state, and re-preparation scenarios apply zero Deep Capture effects.
- **SC-003**: No controlled session effect occurs before the second authorization of the newly prepared cold plan.
- **SC-004**: Every wait terminates within the displayed effective deadline plus one inventory interval.
- **SC-005**: Existing warm behavior is unchanged when restart is not selected, and compatibility calibration continues to reject restart selection.
- **SC-006**: The full repository verification suite passes with no new runtime dependency and no prohibited process-control primitive.

## Assumptions

- S109 supplies immutable routing and recovery authority, S111 supplies publisher chain ownership, and S112 supplies owned cold Steam platform launch.
- The permitted startup snapshot reports image names only, so observed presence remains identity-uncertain and absence is the only safe generic cold criterion.
- The operator can use each application's normal user-facing exit path. No generic automated graceful shutdown is safe across the supported target types.
- The existing `--wait` option is the appropriate shorter operator-supplied bound; an omitted or longer value is capped at two minutes.
- Compatibility calibration expansion remains issue #317, generic transports remain issues #310 through #315, and final Deep Capture completion remains issue #334.
