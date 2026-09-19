# Feature Specification: S158 Windows gate determinism

**Feature Branch**: `codex/s158-windows-gate-determinism`\
**Created**: 2026-09-19 UTC\
**Status**: Implemented, pending hosted acceptance\
**Input**: Permanently correct the post-S157 Windows performance and platform gate failures recorded in issues #413 and #429. Preserve bounded nonblocking application evidence, exact loss accounting, finite lifecycle deadlines, cancellation and cleanup, use deterministic structured completion instead of scheduler timing as success authority, isolate calibration tests from lock poisoning, and do not run the installed product or a real game locally.

## Clarifications

### Session 2026-09-19

- No critical ambiguity required operator input. The retained failed runs, issue acceptance criteria, constitution and existing S128 through S150 authorities define the required behavior and prohibit retry-based acceptance.
- The S150 writer batching correction remains valid. S158 addresses the remaining writer-start scheduling race without changing the 4,096-event queue, the 64-event storage-failure bound, payload policy, event identity, conservation equation or performance budgets.
- Observation duration, proxy shutdown and proxy observation drain are separate lifecycle stages. A completed structured drain remains success even when later scheduler time crosses the earlier capture deadline. An incomplete or timed-out drain remains a failure under its own finite stage budget.

## User Scenarios & Testing

### User Story 1 - Start application evidence before traffic (Priority: P1)

A maintainer needs the Windows native performance campaign to begin traffic only after the bounded application writer can consume events, so operating-system thread startup timing cannot decide whether observations are lost.

**Why this priority**: The post-S157 recurrence reached the exact 4,096-event queue ceiling and lost evidence on one Windows run while an identical-head retry passed. A green retry cannot make that gate trustworthy.

**Independent Test**: Inject a deterministic writer-start stall, prove that lease construction does not publish the sink before the worker reports readiness, then release the worker and verify exact ordered evidence, unchanged queue capacity and zero loss.

**Acceptance Scenarios**:

1. **Given** a newly created application artifact writer whose worker has not reported readiness, **When** the caller attempts to acquire the lease, **Then** acquisition does not complete and no producer can receive its sink.
2. **Given** a ready writer and the existing bounded queue, **When** a metadata-heavy QUIC workload emits its finite event burst, **Then** records remain ordered and conserved with unchanged loss and queue accounting.
3. **Given** a worker that cannot start or terminates before readiness, **When** lease acquisition completes, **Then** it fails specifically and leaves no live writer or writable artifact authority.

### User Story 2 - Judge observation drain by completed evidence (Priority: P1)

A calibration operator needs a controlled session that completed capture, proxy shutdown, structured observation drain and cleanup to remain successful even when Windows schedules the final return after the earlier capture-duration boundary.

**Why this priority**: Run 35263766600 produced reached-client evidence and complete cleanup but failed solely because proxy observation collection was compared with the already consumed capture deadline.

**Independent Test**: Use a scripted clock and proxy drain result to cross the former shared boundary after clean shutdown, verify success from complete drain evidence, then independently test incomplete drain, finite drain timeout, cancellation and cleanup failure.

**Acceptance Scenarios**:

1. **Given** capture returned within its observation duration and proxy shutdown produced a complete structured drain, **When** collection returns after the former shared wall-clock boundary, **Then** the completed observations remain authoritative and no observation-deadline failure is invented.
2. **Given** a proxy drain that is incomplete or does not complete within its finite budget, **When** the session stops, **Then** it records an exact stage-specific failure and still performs cleanup.
3. **Given** cancellation or a genuine shutdown deadline violation, **When** stop and drain run, **Then** cancellation, timeout and cleanup truth remain distinct and bounded.

### User Story 3 - Keep one test failure local (Priority: P2)

A maintainer needs one failed controlled calibration assertion to leave later tests runnable rather than poisoning the suite-wide environment guard and obscuring the primary failure.

**Why this priority**: The primary #429 failure caused twenty-five unrelated `PoisonError` failures, multiplying noise and making diagnosis needlessly expensive.

**Independent Test**: Panic while holding the controlled-environment guard, reacquire it through the suite helper, and prove environment restoration plus a later controlled test remain available.

**Acceptance Scenarios**:

1. **Given** a prior test unwound while holding the suite guard, **When** a later test acquires the controlled environment, **Then** it recovers ownership instead of panicking on poison.
2. **Given** a test mutates the controlled executable environment variable, **When** that test unwinds, **Then** the prior value is restored by scope-owned cleanup.
3. **Given** a genuine later assertion failure, **When** the suite reports results, **Then** it appears as its own failure rather than a secondary lock-poison failure.

### Edge Cases

- The writer thread reports readiness and then its storage write fails.
- Lease construction fails before or after the writer has taken queue ownership.
- Queue pressure occurs after readiness and must still produce exact gap and byte counters.
- Proxy shutdown is clean but returns no completed drain authority.
- Proxy drain completes with zero observations, which is valid structured completion but not reached-client evidence.
- Capture itself exceeds its observation duration before stop begins.
- Shutdown exceeds its own deadline while cached completed observations remain readable.
- Cancellation arrives before capture stop, during proxy stop or before drain collection.
- A test process inherited a pre-existing controlled executable value rather than an absent value.

## Requirements

### Functional Requirements

- **FR-001**: Application artifact lease construction MUST publish a usable sink only after the dedicated writer thread reports that it has entered its receive-ready boundary.
- **FR-002**: Writer readiness coordination MUST be deterministic, finite in ownership and failure-safe. A startup failure MUST retire queue ownership, join the worker when present and return a specific error.
- **FR-003**: The existing 4,096-event nonblocking queue, 64-event pending-storage bound, 64 KiB write buffer, event content, record order, forwarding independence, timeout flush, final flush and storage-failure reconciliation MUST remain unchanged.
- **FR-004**: Queue and storage loss MUST remain exactly counted by event and applicable byte class. S158 MUST NOT convert loss into success, block forwarding on storage, pool samples or weaken the S128 hard invariants.
- **FR-005**: A deterministic regression MUST hold the writer before readiness and prove the lease cannot expose the sink until the hold is released. The test MUST fail against the pre-S158 publication order.
- **FR-006**: Capture observation duration MUST govern capture work only. Proxy shutdown MUST remain governed by its finite shutdown budget, and observation drain MUST consume a finite stage-specific budget with an explicit completion result.
- **FR-007**: A completed structured observation drain MUST be the positive collection authority. Crossing the earlier capture deadline after that completion MUST NOT create a failure.
- **FR-008**: Missing, incomplete, timed-out or failed observation drain MUST retain an exact stage-specific failure. Cancellation, shutdown timeout, cleanup failure and observation incompleteness MUST remain distinguishable.
- **FR-009**: Session tests MUST deterministically cover completed boundary crossing, incomplete drain, drain error or timeout, cancellation, genuine capture deadline violation, shutdown deadline violation and cleanup continuation.
- **FR-010**: Controlled calibration tests MUST recover a poisoned suite guard without suppressing the original panic, and every environment mutation MUST use unwind-safe restoration that preserves any prior value.
- **FR-011**: The exact post-S157 primary calibration case MUST pass on Windows without producing unrelated lock-poison failures in later cases.
- **FR-012**: The master specification, slice ordering and changelog MUST record the corrected readiness and lifecycle authorities without changing the published v0.10.1 baseline.
- **FR-013**: Local verification MUST use source tests, controlled loopback fixtures and repository gates only. No installed product, real game, real trust mutation or sensitive live capture may run on the owner workstation.
- **FR-014**: Fresh hosted Windows native performance and platform workflows MUST pass from the final pull-request head without accepting a rerun as success.
- **FR-015**: Every received review comment MUST be answered and resolved within no more than two review rounds before owner handoff.

### Key Entities

- **Writer readiness boundary**: The one-way worker signal proving the event consumer reached queue ownership before the lease and sink become available.
- **Observation drain result**: Structured terminal evidence declaring whether proxy observations were completely collected within the drain stage.
- **Stage deadline authority**: The exact lifecycle stage whose finite budget can produce a named timeout failure.
- **Controlled environment guard**: Suite-owned serialization and restoration scope for process environment mutation in calibration tests.

## Success Criteria

### Measurable Outcomes

- **SC-001**: The injected pre-ready stall test blocks lease publication in 100 percent of runs and completes with zero dropped events after release.
- **SC-002**: Existing queue capacity, event order, serialized output, conservation and every queue or storage failure test remain unchanged and green.
- **SC-003**: A complete observation drain remains successful in 100 percent of scripted cases that cross the former shared deadline after capture and shutdown.
- **SC-004**: Every incomplete, failed or timed-out drain case produces its expected bounded failure and completes cleanup.
- **SC-005**: A deliberately poisoned controlled-environment mutex can be reacquired, and prior environment state is restored after both success and unwind.
- **SC-006**: The exact #429 calibration regression and the complete calibration test binary pass on Windows without secondary `PoisonError` failures.
- **SC-007**: Fresh Windows native performance evidence contains fourteen passing case terminals, zero hard invariant failures, zero application event loss, zero queue or storage byte loss and clean shutdown under the unchanged S128 registry.
- **SC-008**: All required pull-request checks and both affected hosted Windows workflows are green on the final head without retry-based acceptance.
- **SC-009**: No installed product, real game, real trust mutation or sensitive live capture runs locally during S158.

## Assumptions and Scope

The exact 4,096 queue ceiling in every retained #413 failure, compared with passing peaks near 125 to 150, identifies unpublished writer startup as the remaining deterministic scheduling boundary. S158 does not claim arbitrary traffic can never exceed a bounded nonblocking queue. It guarantees that producers cannot begin before the consumer exists and preserves truthful loss if later traffic exceeds capacity.

The native proxy already caches its terminal runtime observation after clean stop. S158 exposes completion explicitly at the facade adapter boundary instead of treating a later clock sample as evidence that already completed observations were late.

No dependency, package, storage schema, artifact schema, release version, workflow trigger, trust behavior, proxy protocol or payload policy change is in scope. Independent review #333, documentation closure #331 and final Deep Capture acceptance #334 remain later work.

Specification, clarification, requirements checklist, plan, tasks and blocking analysis precede implementation. Explicit push and pull-request authorization satisfies the autopilot pre-push pause; the operator remains the merger.
