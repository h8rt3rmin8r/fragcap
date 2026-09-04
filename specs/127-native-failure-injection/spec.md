# Feature Specification: Native Deep Capture Failure Injection

**Feature Branch**: `codex/127-native-failure-injection`

**Created**: 2026-09-04

**Status**: Draft

**Input**: User description: "Spec out and implement S127 under autopilot, closing the exhaustive native Deep Capture lifecycle, I/O, and cleanup failure-injection boundary described by issue #325."

## User Scenarios & Testing

### User Story 1 - Exercise Every Effect Boundary (Priority: P1)

A maintainer can run one deterministic matrix that injects a failure immediately before and immediately after every journaled external effect and every checked coordinator state transition.

**Why this priority**: A partial native effect can leave trust, routes, processes, listeners, evidence, or cleanup authority in an uncertain state. Point tests do not prove the complete ordering contract.

**Independent Test**: Generate the matrix from the closed boundary inventory, execute every row against the production coordinator and journal transition rules, and confirm that no boundary or injection side is absent.

**Acceptance Scenarios**:

1. **Given** the current journaled effect inventory, **when** the matrix is generated, **then** each effect has distinct before-effect and after-effect scenarios.
2. **Given** the checked lifecycle transitions, **when** the matrix is generated, **then** each transition has distinct before-transition and after-transition scenarios.
3. **Given** a new effect or lifecycle state without matrix ownership, **when** the ordinary repository gate runs, **then** it fails with the missing boundary.

---

### User Story 2 - Preserve Independent Terminal Truth (Priority: P2)

An operator can trust a failed Deep Capture report because terminal outcome, artifact status, fact persistence, event delivery, and cleanup are evaluated independently rather than collapsed into one success bit.

**Why this priority**: A cleanup success cannot make a partial artifact complete, and an event-delivery failure cannot erase retained evidence or prevent later safe cleanup.

**Independent Test**: Execute every generated failure row and compare the report, journal prefix, artifacts, facts, events, and cleanup attempts against the row's explicit expected disposition.

**Acceptance Scenarios**:

1. **Given** a failure before an effect, **when** the session finalizes, **then** the effect is not called, later unsafe effects do not start, and every previously acquired resource receives safe cleanup.
2. **Given** a failure after an effect may have acquired a resource, **when** the session finalizes, **then** the durable obligation remains recoverable and cleanup is attempted exactly once.
3. **Given** a partial or corrupted writer result, **when** artifacts reconcile, **then** no affected artifact is labeled complete.
4. **Given** absent, failed, or ambiguous observation evidence, **when** facts are considered, **then** no success fact is written from that unobserved evidence.
5. **Given** event delivery itself fails, **when** the session continues, **then** the failure is retained separately and later cleanup and artifact attempts still occur.

---

### User Story 3 - Cover Native Failure Families and Recovery (Priority: P3)

A reviewer can trace disk exhaustion, permission denial, broken pipes, task panic, timeout, cancellation, trust denial, listener port theft, network reset, and writer corruption to deterministic executable evidence and an exact recovery outcome.

**Why this priority**: These are the concrete failures introduced by native sockets, TLS, storage, trust, routing, and concurrent tasks. Each must remain observable and bounded.

**Independent Test**: Validate that every mandatory failure family has an executable scenario, stable failure code, expected cleanup result, and Doctor-compatible recovery disposition where exact ownership permits recovery.

**Acceptance Scenarios**:

1. **Given** any mandatory failure family is removed, duplicated, or lacks executable evidence, **when** the matrix gate runs, **then** it fails deterministically.
2. **Given** cleanup fails or times out for one resource, **when** later resources are processed, **then** their safe cleanup attempts still occur and the omission remains named.
3. **Given** exact ownership supports recovery, **when** the journal prefix is inspected, **then** the shared Doctor recovery planner offers only that exact action.
4. **Given** ownership is uncertain or corrupted, **when** recovery is planned, **then** mutation is refused and the residue remains visible.

### Edge Cases

- Failure occurs while synchronizing the pending obligation, so the external effect must not begin.
- Failure occurs while synchronizing the applied or terminal transition after the external effect returned.
- An adapter returns success after its deadline and still owns a resource requiring cleanup.
- Capture or proxy shutdown fails while other cleanup resources remain.
- A task panics after accepting work but before publishing terminal accounting.
- A writer accepts some records and then reports disk exhaustion, permission denial, broken pipe, or corrupt output.
- Event delivery fails for the failure event or terminal event itself.
- A cleanup result is failed or timed out and the lifecycle sidecar also cannot record it.
- Recovery sees a crash prefix, a completed journal, an unsupported version, an identity change, or a torn final record.
- Two scenario rows claim the same boundary and injection side, or a boundary has only one side.

## Requirements

### Functional Requirements

- **FR-001**: The repository MUST carry one versioned canonical native failure-boundary registry and a deterministic generated transition matrix derived from it.
- **FR-002**: The registry MUST enumerate every journaled external effect: proxy listener, proxy runtime tasks, trust entry, target-scoped route, managed child, Capture runner, and retained bundle evidence.
- **FR-003**: The registry MUST enumerate every checked lifecycle transition from prepared through terminal, including failure-shortened paths that stop before running.
- **FR-004**: The generated matrix MUST contain exactly one before-boundary and one after-boundary scenario for every effect and lifecycle transition.
- **FR-005**: Every scenario MUST declare the failure family, stable injection point, expected adapter calls, terminal outcome, artifact disposition, fact disposition, event disposition, cleanup attempts, journal state, and recovery disposition.
- **FR-006**: Matrix execution MUST use the production coordinator, resource transition validation, artifact status types, fact status types, event path, and recovery planner rather than a parallel lifecycle implementation.
- **FR-007**: Failures before a pending obligation is durably synchronized MUST prevent the corresponding effect from starting.
- **FR-008**: Failures after a possibly applied effect MUST retain its obligation and attempt bounded cleanup exactly once without skipping later safe cleanup.
- **FR-009**: Cleanup failure or timeout for one resource MUST NOT prevent cleanup attempts for remaining acquired resources.
- **FR-010**: Disk full, permission denial, broken pipe, task panic, timeout, cancellation, trust denial, port theft, network reset, and writer corruption MUST each have deterministic executable coverage.
- **FR-011**: Terminal outcome, artifact status, fact writes, event delivery, and cleanup results MUST be asserted independently for every applicable matrix scenario.
- **FR-012**: No artifact affected by a failed, torn, corrupt, or incomplete writer may be reported as complete or written successfully.
- **FR-013**: No compatibility fact may be written from absent, failed, ambiguous, or otherwise unobserved evidence.
- **FR-014**: Event-delivery failure MUST remain separately reported and MUST NOT prevent bounded cleanup, journal settlement, or artifact attempts.
- **FR-015**: Every incomplete cleanup MUST retain a stable resource identity, status, reason, and exact shared recovery disposition. Doctor recovery is allowed only where existing ownership evidence is exact.
- **FR-016**: The matrix gate MUST reject schema drift, duplicate boundaries, duplicate sides, missing failure families, missing expected-result fields, stale test references, ignored tests, and inventory disagreement with production resource and lifecycle enums.
- **FR-017**: The matrix command MUST follow the repository 0/1/2 check contract and run inside `cargo xtask ci` without a game, account, Internet service, elevation, capture driver, real trust mutation, or externally routable endpoint.
- **FR-018**: Tests MUST remain deterministic, bounded, synthetic, and portable, with native Windows-only behavior represented through existing narrow adapters rather than real host mutation.
- **FR-019**: Documentation MUST distinguish S127 failure evidence from S124 Doctor behavior, S126 parser fuzzing, S128 performance work, S129 Windows integration, and the final #334 completion gate.
- **FR-020**: S127 MUST add no prohibited capability, third-party dependency, or Deep Capture completion claim.

### Key Entities

- **Failure Boundary Registry**: Versioned reviewed inventory of effect and lifecycle boundaries.
- **Generated Transition Matrix**: Deterministic Cartesian expansion of every boundary into before and after injection scenarios.
- **Failure Scenario**: One executable injection row with independent expected dispositions.
- **Injection Point**: Stable identity for the exact side of an owned boundary where failure is introduced.
- **Outcome Vector**: Independent terminal, artifact, fact, event, cleanup, journal, and recovery expectations.
- **Recovery Disposition**: Exact-action, no-action, or refusal result from the shared resource-journal recovery authority.

## Success Criteria

### Measurable Outcomes

- **SC-001**: One hundred percent of enumerated effect and lifecycle boundaries have both before and after executable scenarios.
- **SC-002**: All ten mandatory native failure families have deterministic executable evidence.
- **SC-003**: Every executed scenario asserts at least five independent authorities: terminal outcome, artifacts, facts, events, cleanup, journal, or recovery.
- **SC-004**: Controlled failures produce zero falsely complete affected artifacts and zero facts from unobserved evidence.
- **SC-005**: Every acquired resource receives exactly one safe cleanup attempt unless the matrix explicitly proves no acquisition occurred.
- **SC-006**: Every seeded registry omission, duplicate, stale test reference, enum drift, or incomplete expectation produces a deterministic gate failure.
- **SC-007**: The complete repository gate passes with no new dependency package and no prohibited host effect.

## Assumptions

- S109 owns durable resource obligations and the shared recovery planner; S127 tests and strengthens those authorities rather than creating another recovery path.
- S124 owns Doctor inventory and confirmed repair; S127 proves whether a failure leaves an exact action or refusal for that existing path.
- Existing facade adapters are the approved portable boundary for trust, routing, launch, Capture, artifacts, facts, events, and native proxy behavior.
- Dependency-owned protocol internals remain covered by their own libraries and S126 fuzzing; S127 targets fragcap-owned lifecycle and I/O outcomes.

## Clarifications

### Session 2026-09-04

- The closed matrix covers both journaled effects and checked coordinator transitions; adapter-only and writer failure families map onto those owned boundaries rather than becoming an unbounded list of call sites.
- After-effect means the effect may own a resource even if its success transition cannot be persisted, so cleanup and recovery remain mandatory.
- Portable controlled adapters are authoritative for destructive host failures. The slice does not fill a real disk, alter real trust, steal an unrelated port, or route external traffic.
