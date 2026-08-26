# Implementation Plan: Doctor ETW Session Probe

**Branch**: `codex/081-doctor-etw-session-probe` | **Date**: 2026-08-26 | **Spec**: `specs/081-doctor-etw-session-probe/spec.md`

**Input**: Feature specification from `specs/081-doctor-etw-session-probe/spec.md`

## Summary

Fix issue #204 by adding a probe-only ETW availability entry point below `EtwWatcher` that starts and drops only the trace session. Change doctor to call that entry point for the process event tracing readiness check, preserving the existing `Option<bool>` contract while avoiding consumer startup, `ProcessTrace` thread teardown, and the startup process snapshot.

## Technical Context

**Language/Version**: Rust 1.82 minimum, current pinned workspace toolchain for feature-on checks

**Primary Dependencies**: Existing workspace crates only; no new dependency planned

**Storage**: N/A for this slice

**Testing**: `cargo test` focused ETW and doctor tests, doctor golden coverage, `cargo xtask ci`, local `logman query -ets` where available

**Target Platform**: Windows ETW readiness path, with non-Windows and feature-off builds preserved

**Project Type**: Rust CLI and library workspace

**Performance Goals**: Avoid the consumer thread and snapshot costs in elevated `fragcap doctor` process event tracing readiness checks

**Constraints**: Preserve final doctor human and JSON report contracts, preserve S079 progress and `--timings` behavior, preserve ETW watcher capture semantics, do not leak ETW sessions, and do not turn runtime readiness into a compile-time feature check

**Scale/Scope**: One ETW facade/probe entry point, one doctor probe call site, focused tests and local evidence

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- P-1 No covert target instrumentation: PASS. ETW kernel provider usage is already permitted for process lifecycle observation; this slice reduces readiness work and adds no target access.
- P-2 Core stays platform-neutral: PASS. No `fragcap-core` platform dependency change is planned.
- P-3 Capture and attribution stay separate: PASS. Work stays in the process watcher and CLI readiness path, not packet acquisition or flow attribution.
- P-4 No silent loss: PASS. The full watcher capture path and its loss reporting are unchanged.
- P-5 Compatibility outranks richness: PASS. No capture file or analyzer format changes.
- P-6 Glossary first: PASS. No new glossary term is introduced.
- P-7 Wrappers stay thin: PASS. No shell wrapper behavior changes.
- P-8 House standards apply: PASS. Slice artifacts and source changes must pass repository lint, formatting, and text hygiene checks.
- P-9 The instrument does not lie: PASS. The runtime ETW session-open check remains a real observation and failure is not converted to backend absence.
- P-10 One path to a target: PASS. No target storage or resolution behavior changes.
- P-11 Specification describes what shipped: PASS. The master spec's doctor report contract is unchanged; no master-spec edit is planned unless implementation changes that contract.

## Project Structure

### Documentation (this feature)

```text
specs/081-doctor-etw-session-probe/
|-- checklists/
|   `-- requirements.md
|-- contracts/
|   `-- etw-session-probe.md
|-- data-model.md
|-- plan.md
|-- quickstart.md
|-- research.md
|-- spec.md
`-- tasks.md
```

### Source Code (repository root)

```text
crates/fragcap-attr/src/etw/watcher.rs
crates/fragcap-attr/src/etw/session.rs
crates/fragcap-attr/src/etw/mod.rs
crates/fragcap/src/lib.rs
crates/fragcap-cli/src/doctor/probe.rs
crates/fragcap-cli/tests/cli_doctor.rs
changelog.d/204-doctor-etw-session-probe.fixed.md
changelog.d/204-doctor-etw-session-probe.decisions.md
```

**Structure Decision**: Add the probe-only entry point as an `EtwWatcher` associated function so `fragcap-cli` continues using the facade watcher surface and does not import raw `Session` internals. Keep `Session` private to the ETW module.

## Phase 0 Research

### Decision: Start and drop only the ETW session for doctor readiness

`Session::start` starts the real-time system logger session and enables the process provider. That answers whether the runtime can open the process tracing session. The consumer thread and snapshot are needed only by a running capture watcher.

**Rationale**: This is the cheapest call that still observes the runtime fact doctor needs. It preserves P-9 because it is not a compile-time proxy for readiness.

**Rejected Alternative**: Keep using `EtwWatcher::start` and accept the cost. That keeps the known disproportionate work in the first troubleshooting command.

### Decision: Keep `EtwWatcher::start` unchanged for capture

The capture watcher startup order remains session, consumer, snapshot.

**Rationale**: The order prevents a process-created gap during watcher startup and is load-bearing for process ancestry correctness.

**Rejected Alternative**: Reuse the probe-only entry point inside `EtwWatcher::start` by starting the session twice. That would add work and risk a same-name collision.

### Decision: Record local measurement limits honestly

Before/after timing and `logman` checks are local platform evidence. If the local shell cannot exercise elevated ETW or the prior code cannot be measured after implementation, the slice records the limitation.

**Rationale**: The issue asks for evidence, and P-9 makes unsupported timing claims unacceptable.

**Rejected Alternative**: Infer a speedup solely from code structure. The code structure proves avoided work, but not local elapsed time.

## Phase 1 Design

The ETW readiness path will have two public watcher-level operations:

- `EtwWatcher::start(session_name)`: full capture watcher startup with session, consumer, snapshot, and event fanout.
- `EtwWatcher::probe_session(session_name)`: readiness-only startup that owns a `Session`, lets it drop immediately, and returns the same `WatcherError` family if the runtime session cannot open or enable the process provider.

Doctor's `tracing_availability` keeps the existing compile configuration split:

- not `all(windows, feature = "etw")`: `None`;
- linked and `probe_session` succeeds: `Some(true)`;
- linked and `probe_session` fails: `Some(false)`.

Tests will favor injected seams in `doctor::probe` for deterministic coverage of success and failure, plus an ETW module test that constrains the probe entry point to session-only construction where possible. Report golden coverage remains in existing doctor tests.

## Complexity Tracking

No constitution violations.
