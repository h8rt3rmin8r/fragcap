# Feature Specification: S165 Route Owner Lifetime

**Feature Branch**: `codex/s165-steam-route-lifetime`
**Created**: 2026-10-05
**Status**: Implemented in PR #453; hosted CI and owner merge pending
**Input**: Active-release defect #452 and the owner's request to resolve the inherited proxy route in one work slice.

## Clarifications

### Session 2026-10-05

- Q: When may a session call route cleanup complete after launching a persistent platform? A: Only after a complete query-only process inventory no longer contains a process that can retain the route. The operator closes the application normally; fragcap never stops it.
- Q: Does a capture deadline also end the inherited route? A: No. Capture may end first, but proxy and trust remain active during a bounded, explicit operator shutdown phase. The existing finite proxy shutdown and cleanup deadlines begin after that phase.
- Q: What if the process remains or the inventory is unavailable when the release budget ends? A: Report an unresolved route owner, give a direct normal-exit and fresh-launch recovery instruction, and mark terminal cleanup partial. Do not assert route release.
- Q: What does the release gate prove? A: Controlled process-inventory and lifecycle tests prove code behavior. They do not claim a successful real-game Deep Capture or protection against abrupt operating-system termination of fragcap.

## User Scenarios & Testing

### User Story 1 - Finish a Steam session without stranding the launcher (Priority: P1)

An operator runs a cold Steam Deep Capture or calibration. When packet capture ends or fails after managed launch, fragcap tells the operator to close Steam normally and keeps the session proxy available during the bounded release interval. Successful cleanup follows observed exit; an unresolved owner produces partial cleanup and recovery guidance.

**Independent Test**: A controlled process inventory presents Steam as running for several observations, then absent. The proxy remains active during the release wait and stops after absence. A separate expired-budget case stops with an explicit unresolved recovery record.

**Acceptance Scenarios**:

1. **Given** a routed Steam root remains open after capture, **when** capture ends, **then** the operator sees a clear normal-exit instruction and the proxy remains available.
2. **Given** the operator closes Steam normally, **when** the next complete process inventory finds no route owner, **then** the proxy stops and cleanup proceeds under the existing finite deadlines.
3. **Given** capture fails or is interrupted after launch was attempted, **when** Steam remains open, **then** the same release barrier applies.
4. **Given** Steam remains open after the release budget, **when** the session retires its proxy, **then** cleanup is partial and the operator is instructed to close and relaunch Steam before using the title again.

### User Story 2 - Apply the same rule to other managed launches (Priority: P1)

An operator launches a direct client or ordered publisher chain with child environment routing. Session shutdown waits for declared route-owning images to exit normally, including an intermediate publisher launcher that outlives the game client.

**Independent Test**: Controlled inventories cover a direct client and a publisher root plus intermediate roles, including a remaining launcher after the client exits.

**Acceptance Scenarios**:

1. **Given** any declared managed-launch image remains in a complete inventory, **when** capture ends, **then** route release is not claimed and proxy shutdown does not begin.
2. **Given** every declared image is absent, **when** the inventory completes, **then** route release is recorded and proxy shutdown begins.

### User Story 3 - Report cleanup truth and recovery (Priority: P2)

The session's structured cleanup stream and terminal report identify the route-owner release decision and never claim the route ended with ordinary Capture when it may persist. Help explains the end-of-session shutdown step and abrupt-exit recovery.

**Independent Test**: A controlled lifecycle ledger proves release precedes proxy stop, while human and structured output preserve the exact result.

**Acceptance Scenarios**:

1. **Given** a release wait, **when** the session ends, **then** the cleanup stream records observed release before proxy cleanup.
2. **Given** fragcap was terminated unexpectedly and an application cannot connect, **when** the operator reads recovery guidance, **then** it instructs normal application shutdown and a fresh launch without implying that an old process environment was changed.

### Edge Cases

- Capture preparation fails before any managed launch attempt.
- The managed launch attempt fails, succeeds without an observable client, or returns a capture error after the root starts.
- Process enumeration fails transiently or reports only a same-named foreign process; the release decision remains conservative.
- The root exits while a declared intermediate or final client remains.
- An operator interrupts capture while the routed application remains open.
- Fragcap itself is forcibly terminated before cleanup can run.

## Requirements

### Functional Requirements

- **FR-001**: After capture, a Deep Capture session MUST keep its proxy and trust active for a finite route-owner release interval. It MUST claim route release only after a complete process inventory shows every declared route owner absent.
- **FR-002**: The operator MUST receive a plain-language instruction to close remaining managed applications normally after capture ends, including failed or interrupted capture and silent output mode.
- **FR-003**: Route-owner release MUST require a complete query-only process inventory and MUST include the Steam platform root, direct client, and every declared publisher stage as applicable. An unavailable inventory MUST NOT be treated as absence. A remaining or unknown owner at the finite deadline MUST become an explicit partial-cleanup recovery state.
- **FR-004**: The session MUST record the route-owner release result before proxy shutdown; launch cleanup MUST NOT report that ordinary Capture ended a process-owned route.
- **FR-005**: A failure before any managed launch attempt MUST proceed to ordinary bounded cleanup without waiting for unrelated applications.
- **FR-006**: The route-owner release wait, proxy shutdown, and cleanup budgets MUST each be finite and disclosed in the authorized plan, without force stopping or modifying any target process.
- **FR-007**: The same release rule MUST apply to ordinary Deep Capture and compatibility calibration, including capture failure and interruption paths.
- **FR-008**: Documentation MUST explain normal exit and recovery after abrupt fragcap termination without claiming that controlled tests prove real-game compatibility.
- **FR-009**: The implementation MUST retain target-scoped routing, plan authorization, evidence integrity, and the P-1 process-observation boundary.

### Key Entities

- **Route owner**: A declared managed-launch process image that may retain or pass on the child-scoped proxy environment.
- **Release observation**: A complete query-only inventory showing all declared route owners absent.
- **Operator shutdown phase**: The interval after capture ends while the proxy and trust remain active and the operator closes routed applications normally.

## Success Criteria

- **SC-001**: Controlled lifecycle tests demonstrate that proxy stop follows a finite route-owner release attempt on success, failure, and interruption paths, and that a surviving owner creates an explicit partial-cleanup outcome.
- **SC-002**: Controlled process-inventory tests demonstrate Steam, direct, and publisher cases, transient enumeration failure, and no-launch behavior.
- **SC-003**: The full repository CI-parity gate passes with no real-game execution or target process control.
