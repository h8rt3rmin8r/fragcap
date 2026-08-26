# Tasks: Doctor ETW Session Probe

**Input**: Design documents from `specs/081-doctor-etw-session-probe/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/`

**Tests**: Required by FR-008 and FR-009. Write or update tests before the implementation task that satisfies the same behavior.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel because it touches different files or pure docs.
- **[Story]**: Maps to the user story in `spec.md`.
- Paths are repository-relative.

## Phase 1: Setup And Current Behavior Baseline

**Purpose**: Confirm the current doctor and ETW surfaces before changing the probe path.

- [x] T001 Inspect current doctor tracing probe and ETW watcher/session code in `crates/fragcap-cli/src/doctor/probe.rs`, `crates/fragcap-attr/src/etw/watcher.rs`, `crates/fragcap-attr/src/etw/session.rs`, and `crates/fragcap/src/lib.rs`
- [x] T002 Run focused baseline doctor tests with `cargo test -p fragcap-cli doctor` and `cargo test -p fragcap-cli --test cli_doctor`
- [x] T003 Attempt baseline local timing and ETW session leak checks where platform and elevation allow, recording any limitation in `changelog.d/204-doctor-etw-session-probe.decisions.md`

---

## Phase 2: Foundational Probe-Only ETW Entry Point

**Purpose**: Add the watcher-level API needed by doctor without exposing raw session internals.

- [x] T004 [P] Add ETW module tests or code-structure checks for the probe-only entry point in `crates/fragcap-attr/src/etw/watcher.rs`
- [x] T005 Add an `EtwWatcher` probe-only entry point in `crates/fragcap-attr/src/etw/watcher.rs` that starts and drops only `Session`
- [x] T006 Re-export any needed probe surface through `crates/fragcap/src/lib.rs` without exposing `Session`

**Checkpoint**: ETW runtime readiness can be queried without full watcher startup.

---

## Phase 3: User Story 1 - Probe ETW Readiness Without Full Watcher Startup (Priority: P1)

**Goal**: Doctor uses the probe-only ETW entry point instead of `EtwWatcher::start`.

**Independent Test**: Injected doctor tracing tests prove success and failure are mapped through the probe-only path.

### Tests For User Story 1

- [x] T007 [US1] Add tests around `crates/fragcap-cli/src/doctor/probe.rs` proving the tracing probe maps probe-only success to `Some(true)` and failure to `Some(false)`
- [x] T008 [US1] Add a test or static check proving `tracing_availability` no longer calls `EtwWatcher::start`

### Implementation For User Story 1

- [x] T009 [US1] Change `crates/fragcap-cli/src/doctor/probe.rs` so `tracing_availability` calls the probe-only ETW entry point

**Checkpoint**: User Story 1 is functional and independently testable.

---

## Phase 4: User Story 2 - Keep Runtime Truthfulness And Cleanup (Priority: P1)

**Goal**: Doctor preserves the existing `None`, `Some(true)`, and `Some(false)` runtime distinction and the probe leaves no ETW session behind.

**Independent Test**: Existing classifier tests plus focused probe tests cover all three states; local `logman` check records cleanup where possible.

### Tests For User Story 2

- [x] T010 [US2] Preserve or add classifier coverage for not built in, openable, and unavailable tracing states in `crates/fragcap-cli/src/doctor/checks.rs` or `crates/fragcap-cli/src/doctor/probe.rs`
- [x] T011 [US2] Run `logman query -ets` after a local probe-capable run when platform and elevation allow

### Implementation For User Story 2

- [x] T012 [US2] Preserve the feature-off and non-Windows `None` path in `crates/fragcap-cli/src/doctor/probe.rs`
- [x] T013 [US2] Record the local cleanup result or limitation in `changelog.d/204-doctor-etw-session-probe.decisions.md`

**Checkpoint**: User Story 2 is functional and independently testable.

---

## Phase 5: User Story 3 - Preserve Doctor Report Contracts And Record Evidence (Priority: P2)

**Goal**: Existing human and JSON doctor outputs stay stable while timing evidence is recorded.

**Independent Test**: Focused doctor tests and goldens pass unchanged.

### Tests For User Story 3

- [x] T014 [US3] Run focused doctor report tests and confirm no human or JSON golden report body changes are needed

### Implementation For User Story 3

- [x] T015 [US3] Avoid changes to final report rendering, JSON rendering, S079 progress output, and `--timings` output except for the removal of full watcher startup from the tracing probe
- [x] T016 [US3] Record before/after timing evidence or exact limitation in `changelog.d/204-doctor-etw-session-probe.decisions.md`

**Checkpoint**: User Story 3 is functional and independently testable.

---

## Phase 6: Changelog And Gate

**Purpose**: Record the bug fix and prove the slice.

- [x] T017 [P] Add `changelog.d/204-doctor-etw-session-probe.fixed.md` for issue #204 with `spec-impact: none`
- [x] T018 Run `cargo fmt --check`
- [x] T019 Run `cargo test -p fragcap-cli tracing_availability`
- [x] T020 Run `cargo test -p fragcap-cli doctor`
- [x] T021 Run `cargo test -p fragcap-cli --test cli_doctor`
- [x] T022 Run `cargo xtask ci`
- [x] T023 Check new and edited files for mojibake and unintended non-ASCII punctuation

---

## Dependencies And Execution Order

- Phase 1 must complete before implementation.
- Phase 2 blocks all user stories.
- User Story 1 and User Story 2 are both P1; implement US1 first because US2 validates the new path's truthfulness and cleanup.
- User Story 3 depends on the probe behavior from US1 and US2.
- Phase 6 depends on all selected user stories.

## Parallel Opportunities

- T004 and T007 can be drafted in parallel once file context is known.
- T017 can be done independently once implementation behavior is settled.
- Verification commands must run foreground and be read to completion.
