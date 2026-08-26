# Tasks: Doctor Single Enumeration

**Input**: Design documents from `specs/080-doctor-single-enumeration/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/`

**Tests**: Required by FR-007 and FR-008. Write or update tests before the implementation task that satisfies the same behavior.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel because it touches different files or pure docs.
- **[Story]**: Maps to the user story in `spec.md`.
- Paths are repository-relative.

## Phase 1: Setup And Current Behavior Baseline

**Purpose**: Confirm the current doctor live probe and report tests before changing the probe path.

- [x] T001 Inspect the current live probe, live driver detection, interface enumeration, and doctor tests in `crates/fragcap-cli/src/doctor/probe.rs`, `crates/fragcap-capture/src/live/driver.rs`, `crates/fragcap-capture/src/live/enumerate.rs`, `crates/fragcap/src/lib.rs`, and `crates/fragcap-cli/tests/cli_doctor.rs`
- [x] T002 Run focused baseline doctor tests with `cargo test -p fragcap-cli doctor` and `cargo test -p fragcap-cli --test cli_doctor`
- [x] T003 Record any unavailable baseline environment condition before implementation in the working notes

---

## Phase 2: Foundational Single-Enumeration Seam

**Purpose**: Add the pure or injected seam needed to prove doctor no longer enumerates twice.

- [x] T004 [P] Add tests covering loopback `Some(true)` from explicit loopback evidence and description-marker evidence in `crates/fragcap-cli/src/doctor/probe.rs` or the shared live-capture helper
- [x] T005 [P] Add tests covering loopback `Some(false)` after successful enumeration with no loopback evidence and `None` after enumeration failure
- [x] T006 Add a test seam or helper that lets doctor live-probe tests count successful enumeration calls without requiring npcap hardware

**Checkpoint**: The desired loopback contract is testable before changing the live doctor path.

---

## Phase 3: User Story 1 - Avoid Duplicate Capture Driver Enumeration (Priority: P1)

**Goal**: A successful doctor live probe obtains interfaces and loopback support from one enumeration.

**Independent Test**: The injected live probe seam reports exactly one enumeration call for a successful inventory.

### Tests For User Story 1

- [x] T007 [US1] Add or update a doctor live-probe test proving a successful inventory produces interfaces and loopback support with exactly one enumeration call

### Implementation For User Story 1

- [x] T008 [US1] Change `crates/fragcap-cli/src/doctor/probe.rs` so the successful live probe derives loopback support from the returned interface inventory instead of calling `fragcap::detect_driver()`
- [x] T009 [US1] Share the loopback predicate with `detect_driver()` if code inspection shows duplication would otherwise be introduced

**Checkpoint**: User Story 1 is functional and independently testable.

---

## Phase 4: User Story 2 - Preserve Honest Unknown States (Priority: P1)

**Goal**: Failed or unavailable enumeration leaves loopback support unknown.

**Independent Test**: Enumeration failure, backend absence, and `wpcap.dll` failure paths keep `loopback_supported = None`.

### Tests For User Story 2

- [x] T010 [US2] Add or update tests proving enumeration failure yields `None` loopback support and preserves the interface error
- [x] T011 [US2] Add or preserve tests proving unavailable live backend and unloadable `wpcap.dll` paths do not fabricate `Some(false)`

### Implementation For User Story 2

- [x] T012 [US2] Preserve the existing non-enumerated paths in `crates/fragcap-cli/src/doctor/probe.rs` and keep their loopback support value as `None`

**Checkpoint**: User Story 2 is functional and independently testable.

---

## Phase 5: User Story 3 - Keep Report Contracts Stable (Priority: P2)

**Goal**: Existing human and JSON doctor outputs stay stable for equivalent inputs.

**Independent Test**: Focused doctor tests and goldens pass unchanged.

### Tests For User Story 3

- [x] T013 [US3] Run focused doctor report tests and confirm no human or JSON golden report body changes are needed

### Implementation For User Story 3

- [x] T014 [US3] Avoid changes to final report rendering, JSON rendering, S079 progress output, and `--timings` output except for the removal of duplicate probe work

**Checkpoint**: User Story 3 is functional and independently testable.

---

## Phase 6: Changelog And Gate

**Purpose**: Record the bug fix and prove the slice.

- [x] T015 [P] Add `changelog.d/203-doctor-single-enumeration.fixed.md` for issue #203 with `spec-impact: none`
- [x] T016 Run `cargo fmt --check`
- [x] T017 Run `cargo test -p fragcap-cli doctor`
- [x] T018 Run `cargo test -p fragcap-cli --test cli_doctor`
- [x] T019 Run `cargo xtask ci`
- [x] T020 Check new and edited files for mojibake and unintended non-ASCII punctuation

---

## Dependencies And Execution Order

- Phase 1 must complete before implementation.
- Phase 2 blocks all user stories.
- User Story 1 and User Story 2 are both P1; implement US1 first because the failed-enumeration assertions should be checked against the new path.
- User Story 3 depends on the probe behavior from US1 and US2.
- Phase 6 depends on all selected user stories.

## Parallel Opportunities

- T004 and T005 can be drafted in parallel once file context is known.
- T015 can be done independently once implementation behavior is settled.
- Verification commands must run foreground and be read to completion.
