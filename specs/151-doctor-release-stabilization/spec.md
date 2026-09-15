# Feature Specification: Doctor and Published Release Stabilization

**Feature Branch**: `codex/s151-doctor-release-stabilization`

**Created**: 2026-09-15

**Status**: Specified

**Input**: User authorized S151 under autopilot, automatic push and PR, all review feedback, at most two review rounds, and human-only final merge. Approved scope is Doctor slow-probe diagnostics (#372) and published v0.10.0 documentation reconciliation (#331).

## User Scenarios & Testing

### User Story 1 - Explain Slow Doctor Work (Priority: P1)

An operator running read-only Doctor can identify the active late sub-operation while it remains blocked and request all completion timings through normal help, without changing the readiness report.

**Why this priority**: #372 records an apparent elevated first-run stall that coarse completion-only progress cannot attribute.

**Independent Test**: Controlled delayed readiness and tracing operations emit progress before release and return their original facts without host probing.

**Acceptance Scenarios**:

1. **Given** interactive normal human output and a delayed probe, **When** work remains unfinished for one second, **Then** stderr identifies the active phase and elapsed time, repeats no faster than once per second, and reports actual completion duration automatically for slow work.
2. **Given** nested readiness work, **When** residue inventory, manifest inspection, certificate identity/store inspection, or loopback readiness runs, **Then** diagnostics identify that boundary without exposing paths, thumbprints, payloads, or inventing the historical stall's cause.
3. **Given** `doctor --help`, **When** an operator seeks diagnostic timing, **Then** `--timings` is visible with its interactive-only scope.
4. **Given** JSON, redirected human output, quiet, silent, or the fix action path, **When** work is delayed, **Then** no new progress contaminates output and final report content, readiness and exits remain unchanged.

### User Story 2 - Know What Was Published (Priority: P2)

A reader of current repository or site guidance sees the actual published v0.10.0 baseline and knows S151 changes are unreleased, independent review is not complete, and installed testing remains operator-owned.

**Why this priority**: Publication on 2026-09-15 made S150's prepared-only current-state statements obsolete under P-11.

**Independent Test**: A reviewed published-release identity and explicit current-applicability statements agree across current documentation, while historical S150 records remain historical.

**Acceptance Scenarios**:

1. **Given** v0.10.0 is live, **When** current release, migration, CLI or artifact guidance is read, **Then** it identifies v0.10.0 as published and does not attribute newer command syntax to v0.9.0.
2. **Given** open #333, #413, #334 and #278, **When** publication or S151 engineering acceptance is recorded, **Then** it does not claim independent audit acceptance, representative installed-host reproduction, signing, universal inspectability, or Deep Capture completion.

### Edge Cases

- A permanently blocked external probe remains pending with elapsed diagnostics; this slice adds no deadline, abandoned worker, fabricated unavailable result, or cancellation claim.
- Nested completion resumes its parent phase; completed phases do not receive later waiting messages.
- Diagnostic output failure remains best effort and does not change facts or abandon work; worker failure joins owned work and propagates failure rather than producing readiness.
- Fast phases retain untimed completion lines unless `--timings` was requested; slow completion timings do not enter final human or JSON reports.
- Workspace version alone does not prove publication; historical release notes, changelog and earlier slice evidence are not rewritten.

## Requirements

### Functional Requirements

- **FR-001**: Interactive normal read-only Doctor MUST emit the active phase and monotonic elapsed milliseconds after one second of pending work and at one-second intervals thereafter, including nested late readiness boundaries and report rendering.
- **FR-002**: `--timings` MUST be visible in ordinary Doctor help and add actual elapsed completion durations for every observed phase; slow completions MUST include durations automatically without this option.
- **FR-003**: JSON, redirected human output, quiet, silent, fix behavior, final report bytes, classifier facts and exits MUST retain their existing contracts.
- **FR-004**: Progress MUST remain fixed-vocabulary stderr diagnostics, preserve actual pending and indeterminate facts, introduce no new host effect or probe timeout, and leave no independently surviving diagnostic worker on success or failure.
- **FR-005**: Controlled delayed readiness and ETW tests MUST establish waiting-before-completion, nested phase attribution, exact result preservation and owned lifetime without elevation, host trust mutation, actual ETW or installed-product execution.
- **FR-006**: Current repository and site applicability MUST identify the verified published v0.10.0 release, reconcile CLI migration and artifact guidance, distinguish unreleased S151 behavior and preserve historical records.
- **FR-007**: A mechanical offline documentation check MUST reject stale current release/applicability statements against a reviewed published identity, independently of candidate workspace version.
- **FR-008**: Tracking and public claims MUST distinguish S151 engineering delivery from outstanding elevated first-run/repeat measurements (#372), independent installed-build review/retest (#333/#413), and final completion (#334/#278). No new release, immutable release change or registry-environment mutation is authorized by this slice.

### Key Entities

- **Probe phase**: Fixed operator-facing name, monotonic start and completion duration; a readiness sub-operation belongs to one active parent phase.
- **Pending progress**: Active phase and elapsed duration, without verdict, sensitive identity or persistent report schema.
- **Published release identity**: Reviewed version, immutable source commit, publication date and official release URL; separate from candidate package version.
- **Engineering delivery**: Controlled-test acceptance under S151, linked to parent issues without asserting their unperformed external criteria.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Both delayed readiness and tracing scenarios show at least one elapsed diagnostic before controlled work is released; completion returns the unchanged supplied value.
- **SC-002**: Every declared late sub-operation has a named timed boundary; normal help exposes timing and default slow completion includes elapsed time.
- **SC-003**: Suppressed-mode tests and existing final report goldens show zero added report fields or changed facts/exits; diagnostic failure and worker failure leave no detached work.
- **SC-004**: All current applicability surfaces pass the published-identity check, and a stale current-baseline negative specimen fails it while historical records remain accepted.
- **SC-005**: Current-head CI passes, every actionable PR review is addressed, at most one manual second review is requested, and the human receives final review/merge handoff. Outstanding external criteria remain visibly open.

## Assumptions

- One second is a short useful interactive threshold; elapsed updates diagnose pending work, not a total runtime bound. No historical elevated delay is assumed to be ETW rather than adjacent readiness or rendering.
- The published identity is v0.10.0, commit `787739edfa8d748e25cb4b5c4965f5c936850d16`, published 2026-09-15. Publication is verified independently rather than inferred from workspace version.
- Existing fixed probe order, read-only queries, bounded residue scans, temporary session-only tracing and cleanup authority remain unchanged.
- Operator-owned installed sensitive-product and real-game testing is outside agent implementation acceptance. The parent issues remain open where those criteria are outstanding.

## Clarifications

### Session 2026-09-15

- Autopilot decision: retain pending work with elapsed updates, not timeout-to-unavailable or detached worker cancellation. This preserves P-9 and exact cleanup ownership.
- Autopilot decision: one-second threshold/cadence and automatic slow completion timings; all-phase timings remain explicit and discoverable. Faster output would add noise without identifying more work.
- Autopilot decision: keep publication identity separate from package version and immutable historical records. S151 diagnostics remain unreleased pending a later authorized publication.
- Autopilot decision: close only bounded engineering delivery, not parent field measurements or independent audit. User-owned installed validation does not block controlled implementation.
