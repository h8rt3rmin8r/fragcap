# Feature Specification: S187 Slow launcher session ownership

**Feature Branch**: `codex/s187-slow-launcher-session-ownership`
**Created**: 2026-11-01
**Status**: Draft
**Input**: Owner-approved S187 pathway for issues #540 and #541.

## User Scenarios & Testing

### User Story 1 - Explicit finite session timing (Priority: P1)

An operator authorizes one slow managed launch and can choose a two-minute calibration observation, a ten-minute ordinary Deep Capture observation, and a correspondingly longer finite launch wait. The authorization plan and result show the requested and effective values without silently shortening them.

**Independent Test**: A controlled installed-path slow launcher fixture exercises accepted and refused bounds, plan output, actual observation span, cancellation, cleanup, and finalized artifacts.

**Acceptance Scenarios**:

1. **Given** a two-minute calibration request and matching launch wait, **when** the plan is emitted, **then** both values remain exact and the observation runs through its bound unless interrupted.
2. **Given** a ten-minute ordinary observation request, **when** the plan is emitted, **then** the effective value is ten minutes and terminal evidence retains that value.
3. **Given** an unsupported or zero duration, **when** the command is checked, **then** it refuses before effects and names the supported range.

---

### User Story 2 - Cold Steam publisher chain (Priority: P1)

An operator declares a cold Steam title that starts a dispatch shim, publisher launcher, helpers, and final client. The plan distinguishes declared roles from observed descendants and names exact shutdown authority before effects. A late or absent client remains an unsuccessful acquisition rather than a claimed match.

**Independent Test**: A controlled Steam-to-publisher fixture creates multiple same-image helpers and a late client, then tests exact ancestry and timeout behavior.

**Acceptance Scenarios**:

1. **Given** declared ordered launcher and client roles, **when** a cold Steam launch is prepared, **then** the immutable plan retains those roles and their allowed shutdown paths.
2. **Given** a shim or helper with proven ancestry but no exact control authority, **when** release runs, **then** its birth, exit, or unresolved obligation is visible and it is not controlled.
3. **Given** an unrelated same-name process, **when** the session times out, **then** it is neither attributed to the target nor controlled.

---

### User Story 3 - Reconciled launch and connection evidence (Priority: P1)

The operator can inspect the full bounded managed-descendant timeline and every proxy connection's exact process-instance ownership when available. Missing or ambiguous evidence remains explicit from the first connection. On acquisition timeout, proxy continuity lasts through bounded owner release, and unresolved descendants have a concrete recovery path.

**Independent Test**: Controlled first-second connections, later helper births, missing process events, and release-time helper creation reconcile process, proxy, cleanup, and recovery records.

**Acceptance Scenarios**:

1. **Given** a helper born under the exact launch root, **when** process evidence is serialized, **then** the operator sees its birth and exit or a counted omission reason.
2. **Given** a proxy connection before final-client acquisition, **when** exact process evidence is available, **then** the connection names that process instance; otherwise it says unavailable or ambiguous and never claims final-client ownership.
3. **Given** a helper born during shutdown, **when** bounded release ends, **then** it either has a verified exit or remains an exact unresolved obligation with a recovery action.

### Edge Cases

- PID reuse, same-image unrelated processes, incomplete event streams, missing creation timestamps, ambiguous ancestry, and launch root exit before a late client.
- A helper may start after the first graceful close and before process watcher closure.
- An explicit duration can exceed a supported ceiling or overflow a combined budget.
- A proxy connection can arrive before any final-client proof and without a packet-flow match.
- An absent client, a late client, and a client that starts then exits quickly remain distinct observations.

## Requirements

### Functional Requirements

- **FR-001 (#540)**: Calibration MUST accept an explicit observation of at least two minutes and ordinary Deep Capture MUST accept an explicit observation of at least ten minutes, each within finite published maxima.
- **FR-002 (#540)**: Managed launch MUST support an explicit finite wait sufficient for the accepted slow case or refuse the excess before authorization.
- **FR-003 (#540)**: Authorization plans, JSON and human presentation, and terminal records MUST distinguish requested from effective launch and observation durations, including defaulted values.
- **FR-004 (#540)**: Unsupported durations MUST refuse before effects; no path may silently clamp operator-requested bounds.
- **FR-005 (#540)**: Capture, proxy, owner release, shutdown, and cleanup deadlines MUST remain independently bounded; cancellation MUST finalize evidence and run cleanup.
- **FR-006 (#541)**: A cold Steam target MUST be able to declare the ordered intermediary publisher stages and final client in immutable preparation and pre-effect authorization.
- **FR-007 (#541)**: Process control MUST require exact created-root or proven-descendant identity, path, account, session, creation time, ancestry, and preauthorized scope; image-name similarity alone MUST grant no authority.
- **FR-008 (#541)**: The operator-visible process trace MUST include relevant managed descendants and their births and exits under a finite bound, or report explicit omission counts and reasons.
- **FR-009 (#541)**: Every target-scoped proxy connection MUST have an exact process-instance correlation from inception when evidence supports it, or an explicit unavailable or ambiguous state; pre-client evidence MUST NOT be called final-client traffic.
- **FR-010 (#541)**: Acquisition timeout MUST preserve proxy continuity through finite owner release, reconcile late descendants, and expose exact unresolved owner and Doctor recovery obligations.
- **FR-011**: Production-path controlled tests MUST cover slow launch, helper born during shutdown, absent and late client, same-name unrelated process, and incomplete ownership evidence without private title data.

### Key Entities

- **Requested and effective deadlines**: Operator-selected or defaulted launch and observation durations, with independent release and cleanup budgets.
- **Managed stage declaration**: Ordered exact executable roles bound to the cold Steam root and final client.
- **Process instance**: PID plus observed creation and exact identity, ancestry, account, and session evidence.
- **Connection ownership**: Exact, ambiguous, or unavailable instance relation for each observed proxy connection.
- **Release obligation**: A live possible route owner with verified exit, authorized control, or explicit recovery state.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Controlled two-minute calibration and ten-minute ordinary plans retain 120000 and 600000 milliseconds respectively; unsupported requests yield zero session effects.
- **SC-002**: The slow-launch fixture waits for the requested finite window and records exact terminal timing, including cancellation and completed artifact finalization.
- **SC-003**: All observed descendants in the controlled chain are present in the bounded trace or covered by counted omissions; no unrelated same-name process gains ownership or control.
- **SC-004**: Every controlled proxy connection has an exact, ambiguous, or unavailable instance result from its first observation; no pre-client connection claims final-client identity.
- **SC-005**: Timeout and release tests cover a late helper and absent or late client with zero unaccounted route-owner obligations and a recoverable partial-cleanup outcome where exit cannot be proved.
- **SC-006**: Repository CI and applicable controlled Windows/package gates pass for the candidate; issue #540 and #541 acceptance are evaluated independently.

## Assumptions

- The installed 0.10.11 observations in #540 and #541 are accurate but do not establish the cause of first-second proxy failures tracked in #542.
- The controlled fixture can model slow launcher timing and ownership without a real game; an installed product claim requires documented defaults and actual canonical path evidence under constitution 1.8.0.
- Existing proxy and packet evidence remain distinct. A missing packet-flow join is not repaired by guessing from proxy authentication.
- This slice does not publish a release or close #542.

## Clarifications

### Session 2026-11-01

- **Finite timing**: Keep the current 30-second launch and 60-second calibration defaults. An explicit request may select a launch wait through five minutes and an observation through ten minutes. Preserve independent 120-second route-owner release and existing shutdown/cleanup defaults; reject any explicit value above a facade ceiling before authorization.
- **Steam chain declaration**: A stored Steam target may declare ordered publisher roles and one terminal client. A transient shim or helper without an exact declared path remains observed and an exit obligation, without process-control authority. Exact descendant proof never follows from a matching image name alone.
- **Connection truth**: Correlate by observed process and socket evidence where possible. If the production evidence cannot establish an exact instance at first connection, report unavailable or ambiguous; do not infer an instance from session authentication or a later final-client match.
- **Acceptance boundary**: Controlled production-path tests may use isolated fixtures. An installed-product default-path claim follows constitution 1.8.0 and requires actual canonical path evidence. The live first-second proxy errors remain #542.
