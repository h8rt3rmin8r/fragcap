# Tasks: Doctor Progress And Timing

**Input**: Design documents from `specs/079-doctor-progress-timing/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/`

**Tests**: Required by FR-015. Write or update tests before the implementation
task that satisfies the same behavior.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel because it touches different files or pure docs.
- **[Story]**: Maps to the user story in `spec.md`.
- Paths are repository-relative.

## Phase 1: Setup And Current Behavior Baseline

**Purpose**: Confirm the existing doctor command surfaces before adding progress.

- [x] T001 Inspect current doctor command, probe gathering, emitter, and tests in `crates/fragcap-cli/src/commands/doctor.rs`, `crates/fragcap-cli/src/doctor/probe.rs`, `crates/fragcap-cli/src/output.rs`, and `crates/fragcap-cli/tests/cli_doctor.rs`
- [x] T002 Run focused baseline doctor tests with `cargo test -p fragcap-cli doctor` and `cargo test -p fragcap-cli --test cli_doctor`
- [x] T003 Record any unavailable baseline environment condition before implementation in the working notes

---

## Phase 2: Foundational Progress And Timing Seam

**Purpose**: Add the shared probe observer vocabulary and pure progress formatting that all stories depend on.

- [x] T004 [P] Add pure progress line formatting tests for begin, complete without timings, and complete with timings in `crates/fragcap-cli/src/doctor/progress.rs`
- [x] T005 Add `doctor::progress` with stable probe labels and elapsed-time formatting in `crates/fragcap-cli/src/doctor/progress.rs` and export it from `crates/fragcap-cli/src/doctor/mod.rs`
- [x] T006 Add a probe observer trait, no-op observer, and measured probe wrapper in `crates/fragcap-cli/src/doctor/probe.rs`

**Checkpoint**: Progress vocabulary and timing data are available without changing command output.

---

## Phase 3: User Story 1 - See Doctor Working Immediately (Priority: P1)

**Goal**: Interactive human doctor runs show named progress promptly while probes run.

**Independent Test**: A test-invoked slow probe emits the first progress line before the slow probe completes.

### Tests For User Story 1

- [x] T007 [US1] Add a test seam that injects a slow probe and asserts the first progress write occurs before completion in `crates/fragcap-cli/src/commands/doctor.rs` or `crates/fragcap-cli/src/doctor/probe.rs`
- [x] T008 [US1] Add a command/progress test proving named probe begin lines are emitted on the enabled interactive path

### Implementation For User Story 1

- [x] T009 [US1] Thread an enabled progress observer from `crates/fragcap-cli/src/commands/doctor.rs` into `probe::gather_with` while preserving the existing `probe::gather()` behavior
- [x] T010 [US1] Instrument identity, platform, capture driver and interfaces, process event tracing, analyzer integration, target stores, and Deep Capture readiness in `crates/fragcap-cli/src/doctor/probe.rs`

**Checkpoint**: User Story 1 is functional and independently testable.

---

## Phase 4: User Story 2 - Preserve Stable Report Surfaces (Priority: P1)

**Goal**: Machine and redirected report outputs remain byte-identical.

**Independent Test**: Existing doctor JSON and human report tests pass unchanged, and suppression paths produce no progress.

### Tests For User Story 2

- [x] T011 [US2] Add or update tests proving `doctor --json`, redirected human output, quiet, silent, and `--fix` suppress progress in `crates/fragcap-cli/tests/cli_doctor.rs` or command unit tests
- [x] T012 [US2] Keep existing doctor golden expectations unchanged and run the focused doctor tests

### Implementation For User Story 2

- [x] T013 [US2] Pass the command emitter into `crates/fragcap-cli/src/commands/doctor.rs` and enable progress only for interactive human stdout
- [x] T014 [US2] Preserve the existing final human and JSON render paths in `crates/fragcap-cli/src/commands/doctor.rs`

**Checkpoint**: User Story 2 is functional and independently testable.

---

## Phase 5: User Story 3 - Attribute Probe Cost With Evidence (Priority: P2)

**Goal**: Maintainers can request per-probe timings and the slice records local evidence.

**Independent Test**: Hidden `--timings` produces elapsed milliseconds for the named probes on the enabled interactive path without changing final reports.

### Tests For User Story 3

- [x] T015 [US3] Add CLI argument coverage for hidden `--timings` in `crates/fragcap-cli/src/cli.rs`
- [x] T016 [US3] Add timing-output tests showing elapsed milliseconds on the interactive progress path and no timing output on suppressed paths

### Implementation For User Story 3

- [x] T017 [US3] Add hidden `--timings` to `DoctorArgs` in `crates/fragcap-cli/src/cli.rs`
- [x] T018 [US3] Include elapsed milliseconds on completed progress lines when timings are enabled
- [x] T019 [US3] Time final report rendering as `report rendering` in `crates/fragcap-cli/src/commands/doctor.rs`
- [x] T020 [US3] Run a local timing-enabled doctor command and record the dominant observed probe cost or concrete limitation in `changelog.d/202-doctor-progress.decisions.md`

**Checkpoint**: User Story 3 is functional and independently testable.

---

## Phase 6: Specification, Changelog, And Gate

**Purpose**: Reconcile the architecture record and prove the slice.

- [x] T021 [P] Update `docs/fragcap-specification.md` section 26.3 for interactive doctor progress and hidden timings
- [x] T022 [P] Add `changelog.d/202-doctor-progress.fixed.md` for the user-visible fix
- [x] T023 Run `cargo fmt --check`
- [x] T024 Run `cargo test -p fragcap-cli doctor`
- [x] T025 Run `cargo test -p fragcap-cli --test cli_doctor`
- [x] T026 Run `cargo xtask ci`
- [x] T027 Check new and edited files for mojibake and unintended non-ASCII punctuation

---

## Dependencies And Execution Order

- Phase 1 must complete before implementation.
- Phase 2 blocks all user stories.
- User Story 1 and User Story 2 are both P1; implement US1 first because the suppression tests in US2 need the progress path to exist.
- User Story 3 depends on the progress observer from US1 and suppression decisions from US2.
- Phase 6 depends on all selected user stories.

## Parallel Opportunities

- T004 and T021/T022 can be done independently once file context is known.
- Tests within a user story can be drafted before the implementation task for that story.
- Documentation and changelog updates can be reviewed independently from code once behavior is settled.
