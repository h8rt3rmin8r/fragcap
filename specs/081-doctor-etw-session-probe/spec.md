# Feature Specification: Doctor ETW Session Probe

**Feature Branch**: `codex/081-doctor-etw-session-probe`

**Created**: 2026-08-26

**Status**: Draft

**Input**: User description: "Implement issue #204: `fragcap doctor` starts a full ETW watcher, consumer thread, and process snapshot just to answer whether process-event tracing is available. Add a cheaper runtime session-open probe, preserve doctor truthfulness, measure the result, and run the slice end to end."

## User Scenarios & Testing

### User Story 1 - Probe ETW Readiness Without Full Watcher Startup (Priority: P1)

An elevated operator runs `fragcap doctor` and the process event tracing check answers whether a process trace session can open without starting the full watcher machinery needed only by capture.

**Why this priority**: `doctor` is the first troubleshooting command. Starting a real consumer thread and taking a full process snapshot for one readiness boolean is disproportionate and can make doctor look slow even after S079 progress and S080 single enumeration.

**Independent Test**: Exercise an ETW probe entry point that starts and drops only the ETW session and prove the doctor tracing probe calls that entry point instead of `EtwWatcher::start`.

**Acceptance Scenarios**:

1. **Given** the `etw` feature is linked on Windows, **When** doctor probes process event tracing, **Then** it asks a probe-only ETW session-open entry point for the runtime answer.
2. **Given** the probe-only ETW session starts successfully, **When** doctor classifies tracing readiness, **Then** `etw_available` is `Some(true)`.
3. **Given** the probe-only ETW session cannot start, **When** doctor classifies tracing readiness, **Then** `etw_available` is `Some(false)` and the existing classifier renders the blocking or warning verdict.

---

### User Story 2 - Keep Runtime Truthfulness And Cleanup (Priority: P1)

An operator receives a runtime tracing verdict that still distinguishes not built in, built in and openable, and built in but unavailable, while the probe leaves no ETW session behind.

**Why this priority**: P-9 forbids replacing the runtime probe with a compile-time feature check, and ETW sessions outlive the creating process if not stopped.

**Independent Test**: Run classifier coverage for `None`, `Some(true)`, and `Some(false)`, and run a local `logman query -ets` check after the probe where the platform allows it.

**Acceptance Scenarios**:

1. **Given** the binary is built without the `etw` feature, **When** doctor gathers inputs, **Then** tracing availability remains `None`.
2. **Given** the binary is built with `etw` and the session opens, **When** the probe returns, **Then** the session is stopped by owned cleanup and no `fragcap-doctor-probe` session remains.
3. **Given** the binary is built with `etw` and the session cannot open, **When** doctor gathers inputs, **Then** the failure is reported as `Some(false)`, not silently converted to `None`.

---

### User Story 3 - Preserve Doctor Report Contracts And Record Evidence (Priority: P2)

Existing doctor human and JSON report consumers see unchanged report bodies for equivalent inputs, while maintainers get recorded timing evidence for the cheaper probe.

**Why this priority**: Issue #204 is a probe implementation defect, not a new report contract.

**Independent Test**: Run focused doctor tests and goldens unchanged, then run a timing-enabled local doctor command before and after implementation where possible and record the evidence or the exact limitation.

**Acceptance Scenarios**:

1. **Given** `fragcap doctor --json`, **When** doctor runs after this slice, **Then** the JSON record shape and values for equivalent inputs are unchanged.
2. **Given** the existing human doctor golden tests, **When** they run after this slice, **Then** no golden report body changes are required.
3. **Given** local elevated ETW measurement is available, **When** before and after commands run with `--timings`, **Then** the slice records the process event tracing timing change and the `logman query -ets` result.
4. **Given** local elevated ETW measurement is unavailable, **When** the slice completes, **Then** the slice records the exact limitation instead of claiming a measured speedup.

### Edge Cases

- `EtwWatcher::start` remains the full capture watcher path and still opens the consumer and snapshot in its load-bearing order.
- The probe-only entry point must not leak a session if provider enabling succeeds or fails.
- The doctor tracing check must not become a compile-time `cfg!(feature = "etw")` answer.
- The probe must not introduce a timeout that converts an undetermined result into `Some(false)`.
- Non-Windows or non-`etw` builds must continue compiling and returning `None` for tracing availability.

## Requirements

### Functional Requirements

- **FR-001**: Doctor's process event tracing probe MUST stop using `EtwWatcher::start` for readiness checking.
- **FR-002**: A probe-only ETW entry point MUST answer the runtime question "can this binary open and enable a process trace session" by starting and dropping only the ETW session.
- **FR-003**: The probe-only entry point MUST NOT open an ETW consumer, spawn a `ProcessTrace` thread, or take a process snapshot.
- **FR-004**: `EtwWatcher::start` MUST remain the full watcher startup path for capture and MUST preserve its session, consumer, snapshot order.
- **FR-005**: Doctor MUST keep returning `None` for tracing availability when the ETW backend is not linked.
- **FR-006**: Doctor MUST return `Some(true)` when the probe-only ETW session opens and `Some(false)` when the linked probe cannot open.
- **FR-007**: The implementation MUST preserve existing final doctor human and JSON report contracts for equivalent inputs.
- **FR-008**: Tests MUST cover the three tracing availability states: not built in, openable, and unavailable.
- **FR-009**: Tests or code structure MUST mechanically constrain the doctor probe to use the probe-only ETW entry point rather than full watcher startup.
- **FR-010**: The completed slice MUST record before and after local timing evidence for the process event tracing probe, or record the concrete reason representative measurement could not be performed.
- **FR-011**: The completed slice MUST verify that no `fragcap-doctor-probe` ETW session remains after a local probe run where the platform allows the check, or record the concrete reason that check could not be performed.
- **FR-012**: The completed slice MUST include a changelog fragment for issue #204.

### Key Entities

- **Probe-Only ETW Session Check**: A runtime availability check that starts the ETW session, enables the process provider, and lets session ownership stop it immediately.
- **Full ETW Watcher**: The capture watcher that starts a session, opens a consumer, takes a startup snapshot, publishes process events, and owns teardown.
- **Tracing Availability Verdict**: Doctor's existing `Option<bool>` fact: `None` not built in, `Some(true)` built in and session opened, `Some(false)` built in and session would not open.

## Success Criteria

### Measurable Outcomes

- **SC-001**: A focused test or code-level assertion proves the doctor tracing probe uses the probe-only ETW entry point and not `EtwWatcher::start`.
- **SC-002**: Classifier or probe tests cover `None`, `Some(true)`, and `Some(false)` tracing availability behavior.
- **SC-003**: Existing focused doctor tests pass without doctor golden output changes.
- **SC-004**: Local timing evidence for `process event tracing` before and after the change is recorded, or the exact measurement limitation is recorded.
- **SC-005**: A local `logman query -ets` check after the probe records no surviving `fragcap-doctor-probe` session, or the exact check limitation is recorded.
- **SC-006**: `cargo xtask ci` passes, or any unavailable platform/toolchain component is reported with the exact command and observed result.

## Assumptions

- Starting and enabling the ETW session is sufficient to answer the readiness boolean; consumer opening and snapshotting are capture-start obligations, not readiness obligations.
- The existing doctor classifier and renderers are correct and remain the authority for final report output.
- No new glossary term is required; existing Event Tracing for Windows, diagnostics, readiness, and process watcher vocabulary covers this slice.
- The master specification already defines the doctor report contract and does not need a revision unless implementation changes that contract.
