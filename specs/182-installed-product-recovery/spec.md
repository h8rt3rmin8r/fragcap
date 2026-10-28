# Feature Specification: S182 Installed-product recovery

**Feature Branch**: codex/s182-installed-product-recovery
**Created**: 2026-10-27
**Status**: Implemented and qualified; owner review and merge pending
**Input**: Owner-approved S182 outline and explicit autopilot authorization to push a pull request, resolve CI and up to two review rounds, then stop for final owner review and merge.
**Authority**: Audit #513 and findings #514, #515, #516, #517, #518, #520, #521, and #522; constitution 1.6.0; master specification sections 13, 15, 16, 17, 19, 25, 26, and 27.

## Scope

**Scope in**: Recover relocated stored Steam targets; reconcile real calibration observation time; retain temporally and causally meaningful packet omission evidence; correlate only exact final-client traffic; release exactly proven managed owners despite normal helpers; reconcile proxy, route, terminal, and artifact truth; correct same-version operator documentation; make the official MSI and portable executable start in a clean Windows environment; repair the physical Windows matrix and current release-reference evidence.
**Scope out**: Publication, tagging, merge, or closure of release-delivery #519; new proxy routing strategies; TLS pinning bypass; control of warm or unrelated processes; automatic mutation of ambiguous or unavailable target locations; real-game testing of unpublished bytes; unrelated feature and UX changes.
**Done when**: Every included issue's repository-controlled acceptance criteria has direct regression evidence, the complete controlled reachability-to-TLS-to-ordinary-Deep-Capture path succeeds through production boundaries, the required source, protocol, physical Windows, clean-package, supply-chain, and two-hour performance gates pass, and one reviewable PR has no unresolved actionable CI or review findings. The owner retains final merge and release authority.

## User Scenarios & Testing

### User Story 1 - Recover a moved stored target (Priority: P1)

An operator can calibrate an existing authored target after a platform library move without losing its stable identity or launch-role declarations.

**Why this priority**: A stale stored root blocks calibration before the session begins, and re-registration currently reports success while preserving the stale root.

**Independent Test**: Move a synthetic installed target between platform libraries, keep the old target row, and exercise resolution and the supported recovery operation against a controlled store.

**Acceptance Scenarios**:

1. **Given** one authoritative same-anchor new installation and an absent old root, **When** the operator selects recovery, **Then** the complete proposed location change is shown, revalidated, and applied only to the selected target while preserving stable ID, handle, authored roles, and unrelated rows.
2. **Given** an absent platform manifest, unavailable volume, changed executable, or more than one plausible new root, **When** target resolution runs, **Then** it reports the exact uncertainty and performs no silent rewrite.
3. **Given** a workflow snapshot made before a location change, **When** calibration resumes, **Then** the location and compatibility applicability are checked afresh before any effect or readiness claim.

### User Story 2 - Complete evidence-backed calibration (Priority: P1)

An operator can complete the authorized reachability and TLS stages when a selected cold target genuinely reaches the local proxy and packet evidence proves the final client.

**Why this priority**: The real Capture adapter can consume the full observation deadline after setup, and untimed withheld packet counts can invalidate later attributable traffic for an entire flow.

**Independent Test**: Run the production calibration boundary with controlled packet, process, and proxy observations, including early withheld packets, later exact final-client traffic, genuine loss, and reused tuples.

**Acceptance Scenarios**:

1. **Given** a finite launch and observation plan, **When** ordinary Capture starts after setup and reaches its requested duration, **Then** stage deadlines include bounded setup and return work without mislabeling a normal completion or accepting a genuine overrun.
2. **Given** withheld observations outside a proven connection and ownership interval, **When** later retained packet and proxy observations agree on the exact final client, **Then** the selected routing case may produce a supported current fact.
3. **Given** overlapping missing history, conflicting owners, port reuse, acquisition loss, or incomplete route propagation, **When** correlation runs, **Then** it refuses a positive fact and reports the actual uncertainty and omission cause.
4. **Given** current exact routing evidence and a supported TLS case, **When** the separately authorized TLS stage runs, **Then** it can produce a supported current protocol fact without treating routing as proof of TLS inspectability.

### User Story 3 - Finish an exact managed session truthfully (Priority: P1)

An operator can complete ordinary Deep Capture and see accurate shutdown, route release, and application evidence even when the platform creates ordinary helper processes.

**Why this priority**: Normal helpers currently block a graceful request to independently proven owners, and a stop timeout can cause terminal zero counts despite retained application records.

**Independent Test**: Drive a controlled cold platform-root, final-client, and helper tree through the production release and proxy-drain boundaries, including helper exit after parent close, timeout, ambiguous identity, and recovery.

**Acceptance Scenarios**:

1. **Given** exact session-created root and client ownership plus a helper that cannot receive direct process control, **When** observation ends, **Then** fragcap may gracefully request the proven owners to close and must observe the helper's and owners' actual exits before claiming route release.
2. **Given** a warm, escaped, reused-ID, mismatched-account/session/path, or ambiguous process, **When** shutdown evaluates it, **Then** no process control is applied to that process and incomplete release remains visible.
3. **Given** proxy stop times out but later cleanup obtains real connection totals and finalizes application records, **When** the terminal assessment is produced, **Then** observed counts and classifications reconcile across terminal, manifest, application, and cleanup authorities while timeouts and missing records remain explicit.
4. **Given** a complete supported calibration, **When** ordinary Deep Capture runs in the controlled environment, **Then** its packet, application, process, manifest, and cleanup artifacts identify the same selected ownership and terminal state.

### User Story 4 - Install and understand the corrected product (Priority: P2)

An operator can start the official package on a clean supported Windows installation, read accurate lifecycle instructions, and trust the current physical Windows qualification result.

**Why this priority**: The v0.10.8 package fails before main without an undeclared runtime, the physical matrix has a stale privilege assertion and obsolete reference, and the live documentation describes an outdated manual shutdown sequence.

**Independent Test**: Install candidate MSI and portable bytes in a fresh Windows guest without developer runtimes, execute all seven physical Windows rows against exact official v0.10.8 bytes and the corrected harness, retain candidate evidence separately, and compare version-matched documentation with observed behavior.

**Acceptance Scenarios**:

1. **Given** a clean supported Windows guest without Npcap or developer runtimes, **When** the official MSI and portable executable run version, help, and Doctor, **Then** they start successfully and Doctor reports the absent capture driver as a separate actionable prerequisite.
2. **Given** an ordinary Windows user, **When** the physical matrix checks Capture and Deep Capture readiness, **Then** the assertion matches the intended blocking privilege refusal without weakening product behavior.
3. **Given** exact official v0.10.8 bytes and fresh physical evidence, **When** the current-release reference validator runs, **Then** the accepted reference names that product, revision, date, and authority; candidate and older historical evidence remain separately identified.
4. **Given** a reader of the same-version pages, **When** they prepare and finish calibration or Deep Capture, **Then** they can identify the automatic shutdown scope, deadlines, unsaved-work consequence, refusal and recovery paths, and required versus optional human actions in plain English.
5. **Given** synthetic valid user state before installation, **When** the candidate package is installed, repaired, and uninstalled, **Then** package-owned executable state changes as declared while preserved user data and unrelated state remain intact.

### Edge Cases

- A moved target's old root may still exist, the platform may expose duplicate anchors, or a new executable may differ from the authored client; no ambiguous condition grants new launch authority.
- Packet omission may originate from explicit scope, startup timing, a bounded queue, or history saturation; a count without interval and cause cannot serve as proof.
- A finite deadline may expire during setup, observation, drain, shutdown, or cleanup; each phase retains its own observed result and the session never fabricates successful completion.
- Platform helpers may exit after their parent receives a graceful request, remain live past the release ceiling, or have ambiguous ancestry; only exact proven process objects may receive authorized control.
- A completed application writer can coexist with a failed listener stop or a missing connection terminal; reports distinguish those independent facts.
- A clean guest can lack Npcap and still run non-capture commands; package certification cannot rely on a developer runner's preinstalled C runtime.
- Unpublished source tests do not establish compatibility with a real title; such observations use published bytes and private evidence.

## Requirements

### Functional Requirements

- **FR-001**: The target system MUST detect a unique authoritative relocation of an existing anchored target and provide a bounded, reviewable recovery that preserves identity, authored roles, and unrelated records.
- **FR-002**: Target resolution and workflow resume MUST distinguish moved, uninstalled, unavailable, ambiguous, and changed-executable conditions before authorization-sensitive session effects.
- **FR-003**: Real Capture and guided calibration MUST share finite stage timing semantics that account for setup, observation, return, drain, shutdown, and cleanup without accepting genuine deadline overruns.
- **FR-004**: Packet omission evidence MUST retain bounded cause and temporal/ownership information sufficient to identify whether missing observations overlap the exact proxy connection interval.
- **FR-005**: Final-client correlation MUST grant a positive routing fact only from exact supported packet and proxy evidence; it MUST refuse unresolved overlap, conflicting ownership, tuple reuse, and unavailable evidence with a specific reason.
- **FR-006**: Routing and TLS calibration MUST remain separate authorizations and facts, and ordinary Deep Capture MUST require the latest exact applicable evidence for its selected case.
- **FR-007**: Managed shutdown MUST allow a bounded graceful request to independently proven session-created owners while ordinary helpers are observed, without expanding process control to helpers, warm, escaped, unrelated, or ambiguous processes.
- **FR-008**: Route, target-route, managed-launch, route-owner, proxy, application, and terminal statuses MUST reconcile with verified exit and artifact evidence; unavailable metrics MUST not be presented as observed zero.
- **FR-009**: The official MSI and portable package MUST start non-capture commands on a clean supported Windows host without relying on unrelated developer runtime installation, while separately reporting absent Npcap.
- **FR-010**: The physical Windows matrix MUST exercise exact official v0.10.8 bytes with the corrected harness and separately qualify exact candidate bytes; current-release reference validation MUST preserve release identity, intentional privilege refusal, and historical provenance.
- **FR-011**: Same-version operator documentation MUST state the actual cold-session automatic shutdown, deadlines, consequences, exceptional manual recovery, moved-library recovery, package prerequisites, and current command examples in plain English.
- **FR-012**: Every included issue MUST receive a focused controlled regression and a truthful repository-controlled acceptance result without publishing real-title identity or raw capture data.
- **FR-013**: Package certification MUST verify install, repair, and uninstall against synthetic user state and report actual ownership and preservation separately from executable startup.

### Key Entities

- **Stored target**: Stable identity, platform anchor, install root, authored launch roles, discovery provenance, and versioned compatibility facts.
- **Relocation proposal**: Exact old and proposed roots, platform evidence, preserved fields, ambiguity state, and confirmation authority.
- **Observation interval**: Bounded stage clock, acquisition and retained-packet times, connection interval, and terminal reason.
- **Omission evidence**: Flow identity, cause, bounded time/ownership scope, and loss count.
- **Managed owner**: Session-created process object and exact ancestry, path, account, session, creation time, launch receipt, and verified exit.
- **Session evidence**: Packet, application, process, proxy, manifest, cleanup, and terminal authorities with separate completeness states.
- **Package certification**: Exact MSI and portable bytes, ordinary imports, clean-guest startup, lifecycle results, and current reference provenance.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Every unique supported moved-library case recovers the selected stored target with zero unrelated row or authored-role changes; every ambiguous or unavailable case refuses mutation.
- **SC-002**: The controlled production-path reachability, TLS calibration, and ordinary Deep Capture sequence completes with one supported exact final-client fact per measured case, zero fabricated facts, and complete artifact and cleanup reconciliation.
- **SC-003**: All controlled genuine deadline, overlapping-loss, reused-owner, ambiguous-process, and stop-timeout cases report their causal failure without a false success or false zero.
- **SC-004**: Both candidate package forms start version, help, and Doctor in a clean supported Windows guest without a separately preinstalled developer runtime; absent Npcap is diagnosed after startup, and install, repair, and uninstall preserve all synthetic user-state sentinels.
- **SC-005**: All seven physical Windows rows pass against exact official v0.10.8 bytes with the corrected harness, every external effect reconciles, and the current reference validator accepts only fresh exact-release provenance; candidate qualification remains separately identified.
- **SC-006**: The full repository gate, protocol conformance, all applicable advisory checks, and a continuous two-hour native proxy soak complete with zero failed required cases and a passing validator.
- **SC-007**: Every changed operator page matches the tested command and lifecycle behavior, and the documentation checks report zero broken required links or stale lifecycle claims.
- **SC-008**: Each of the eight included issues has direct regression evidence and a reviewable PR disposition; no open actionable CI or review finding remains at the owner merge handoff.

## Assumptions

- The v0.10.8 audit's private raw evidence remains private; public tests and documentation use synthetic targets and no identifying game details.
- Current main includes the S181 Tokio graph correction; #519 tracks release publication and is outside S182 source implementation.
- Existing per-target and per-session authorization, retention, certificate, and route-scope contracts remain in force.
- Implementation qualification uses controlled targets and production boundaries. Real-game field validation uses an already published release and cannot be claimed from this unreleased branch.
- Existing GitHub milestone, labels, project item, and native issue relations remain authoritative; S182 adds the active Slice and Stage fields for its eight included findings.
