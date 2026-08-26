# Tasks: Targets Warning Stream Contract

**Input**: Design documents from `specs/082-targets-warning-stderr/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Required by the feature specification and autopilot protocol.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.

## Phase 1: Setup

**Purpose**: Confirm the current failure surface and prepare slice artifacts.

- [X] T001 Create S082 Spec Kit artifacts under `specs/082-targets-warning-stderr/`
- [X] T002 Review warning call sites in `crates/fragcap-cli/src/commands/targets.rs`

---

## Phase 2: Foundational

**Purpose**: Thread the shared diagnostic emitter into the targets command surface.

- [X] T003 Update `crates/fragcap-cli/src/lib.rs` to dispatch `Command::Targets` and bare invocation listing with `Emitter`
- [X] T004 Update targets entry points in `crates/fragcap-cli/src/commands/targets.rs` to accept `Emitter` where warnings can be emitted

---

## Phase 3: User Story 1 - Pipe Targets Listings Without Diagnostics (Priority: P1) MVP

**Goal**: Targets listings keep warning diagnostics off standard output.

**Independent Test**: A warning-producing `targets list` run produces byte-identical stdout to the same listing without warnings, and stderr carries the warning.

### Tests for User Story 1

- [X] T005 [US1] Add failing integration coverage in `crates/fragcap-cli/tests/cli_targets.rs` for warning-free stdout and stderr warning routing
- [X] T006 [US1] Add quiet and silent warning routing assertions in `crates/fragcap-cli/tests/cli_targets.rs`

### Implementation for User Story 1

- [X] T007 [US1] Route hero listing discovery and registration warnings through `Emitter::warn` in `crates/fragcap-cli/src/commands/targets.rs`
- [X] T008 [US1] Run focused targets warning tests and confirm User Story 1 passes

---

## Phase 4: User Story 2 - Keep Structured Diagnostics Structured (Priority: P2)

**Goal**: JSON-mode targets warnings use the shared structured diagnostic shape.

**Independent Test**: A warning-producing targets command with `--json` emits a structured warning record on stderr and no human warning on stdout.

### Tests for User Story 2

- [X] T009 [US2] Add JSON-mode warning diagnostic coverage in `crates/fragcap-cli/tests/cli_targets.rs`

### Implementation for User Story 2

- [X] T010 [US2] Route `targets discover`, `targets scan`, `targets add --steam`, executable detection, and doctor discovery helper warnings through `Emitter::warn` in `crates/fragcap-cli/src/commands/targets.rs`
- [X] T011 [US2] Run focused JSON warning tests and confirm User Story 2 passes

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Finish verification, changelog, and hygiene.

- [X] T012 Update comments in `crates/fragcap-cli/src/commands/targets.rs` that still describe warnings as stdout output
- [X] T013 Add `changelog.d/205-targets-warning-stderr.fixed.md`
- [X] T014 Run `cargo fmt --all -- --check`
- [X] T015 Run `cargo test -p fragcap-cli --test cli_targets`
- [X] T016 Run `cargo xtask ci`
- [X] T017 Review `git diff` for scope, text hygiene, and `.specify/feature.json` exclusion

---

## Dependencies & Execution Order

- Phase 1 precedes all code changes.
- Phase 2 precedes story implementation because both stories need emitter wiring.
- User Story 1 is the MVP and should pass before User Story 2.
- User Story 2 builds on the same routing and validates JSON diagnostics.
- Polish follows both stories.

## Parallel Opportunities

- T003 and T004 touch different entry points but should be reviewed together.
- T005 and T006 can be drafted together in the same test file.
- T013 can be done while focused tests are running only if no source edit is in progress.

## Implementation Strategy

1. Write failing integration tests for stdout isolation and verbosity.
2. Thread `Emitter` into targets.
3. Replace direct warning writes with `Emitter::warn`, keeping result lines on `out`.
4. Add JSON diagnostic coverage.
5. Run focused checks, then the full repository gate.
