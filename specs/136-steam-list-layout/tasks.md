# Tasks: Readable Steam Title Listing

**Input**: Design documents from `/specs/136-steam-list-layout/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: S136 requires test-first coverage for every human layout and compatibility boundary.

**Organization**: Tasks are grouped by user story. Shared display policy is foundational because both the Steam and Doctor commands consume it.

## Phase 1: Setup and Contract Baseline

**Purpose**: Bind the implementation to the current S067 contract and dependency baseline before renderer work.

- [x] T001 Record the S136 human layout contract and compatibility boundary in `specs/136-steam-list-layout/contracts/steam-list-human.md`
- [x] T002 Confirm the no-new-package and unchanged-JSON boundaries against `Cargo.lock` and `specs/067-steam-list-identity-json/contracts/steam-list-cli.md`

---

## Phase 2: Foundational Display Policy

**Purpose**: Establish one tested display authority before either human layout is implemented.

- [x] T003 Add failing tests for visible control representation and 40-through-80 stdout width selection in `crates/fragcap-cli/src/display.rs`
- [x] T004 Implement visible control representation and shared stdout width selection in `crates/fragcap-cli/src/display.rs`
- [x] T005 Move Doctor to the shared width selector without changing output in `crates/fragcap-cli/src/commands/doctor.rs`

**Checkpoint**: Display-cell accounting, control representation, and width policy are shared and independently tested.

---

## Phase 3: User Story 1 - Scan installed titles and target state (Priority: P1)

**Goal**: Render fitting results as one aligned four-column listing without tabs while retaining all three target states.

**Independent Test**: Short varied rows at width 80 begin fields at the header's display columns, preserve target-state text, and contain no tab characters.

### Tests for User Story 1

- [x] T006 [US1] Add failing aligned-layout, state-distinction, and no-tab renderer tests in `crates/fragcap-cli/src/commands/steam.rs`

### Implementation for User Story 1

- [x] T007 [US1] Add the single human presentation-row mapping and aligned table renderer in `crates/fragcap-cli/src/commands/steam.rs`

**Checkpoint**: A complete fitting listing is aligned by display cells and all identity states remain distinct.

---

## Phase 4: User Story 2 - Read long and localized titles (Priority: P2)

**Goal**: Preserve every long, localized, or control-bearing value through one labeled vertical layout when the table does not fit.

**Independent Test**: Long names, combining marks, wide characters, embedded controls, and width 40 all select labeled vertical records with no truncation or tabs.

### Tests for User Story 2

- [x] T008 [US2] Add failing vertical-layout, narrow-width, localized-width, control-representation, and no-truncation tests in `crates/fragcap-cli/src/commands/steam.rs`

### Implementation for User Story 2

- [x] T009 [US2] Implement whole-list fit selection and labeled vertical records in `crates/fragcap-cli/src/commands/steam.rs`

**Checkpoint**: Every over-width result uses one unambiguous layout and retains every value.

---

## Phase 5: User Story 3 - Preserve automation and identity behavior (Priority: P3)

**Goal**: Keep JSON, diagnostics, exits, sorting, and store state unchanged while the human command selects its actual stdout width once.

**Independent Test**: Existing JSON and read-only tests pass unchanged, numeric app-id ordering remains stable, and the command-level human and JSON surfaces contain no cross-mode output.

### Tests for User Story 3

- [x] T010 [P] [US3] Strengthen command-level human no-tab and JSON compatibility assertions in `crates/fragcap-cli/tests/cli_steam.rs`
- [x] T011 [US3] Retain and exercise ordering, store fallback, empty state, and snapshot immutability tests in `crates/fragcap-cli/src/commands/steam.rs`

### Implementation for User Story 3

- [x] T012 [US3] Select stdout width once for human mode while leaving the JSON path unchanged in `crates/fragcap-cli/src/commands/steam.rs`

**Checkpoint**: Human presentation is fixed without changing machine or storage semantics.

---

## Phase 6: Documentation and Validation

**Purpose**: Reconcile the architecture of record and prove the slice against all repository gates.

- [x] T013 [P] Reconcile the S067 contract, master specification, outline, roadmap, and CLI reference in `specs/067-steam-list-identity-json/contracts/steam-list-cli.md`, `docs/fragcap-specification.md`, `docs/fragcap-spec-outline.md`, `docs/plans/README.md`, and `site/content/docs/reference/cli.mdx`
- [x] T014 [P] Add issue and decision fragments in `changelog.d/374-steam-list-layout.fixed.md` and `changelog.d/s136-steam-list-layout.decisions.md`
- [x] T015 Run focused commands from `specs/136-steam-list-layout/quickstart.md`, then run `cargo xtask ci` and verify `Cargo.lock` remains unchanged
- [x] T016 Mark every completed task in `specs/136-steam-list-layout/tasks.md`, scan changed files for encoding corruption and prohibited dashes, and review the final diff against issue #374

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: Starts immediately and fixes the compatibility baseline.
- **Foundational (Phase 2)**: Depends on Setup and blocks both human layouts.
- **User Story 1 (Phase 3)**: Depends on the shared display policy.
- **User Story 2 (Phase 4)**: Depends on the presentation row from User Story 1.
- **User Story 3 (Phase 5)**: Depends on both completed human layouts so compatibility is checked against the final boundary.
- **Documentation and Validation (Phase 6)**: Depends on all user stories.

### User Story Dependencies

- **User Story 1 (P1)**: First independently valuable fix after the shared display policy.
- **User Story 2 (P2)**: Extends User Story 1's presentation row with the over-width decision and vertical form.
- **User Story 3 (P3)**: Verifies and wires the full renderer without changing the structured or storage paths.

### Within Each User Story

- Write the named tests first and observe the relevant failure.
- Implement only enough behavior to pass the story's tests.
- Complete the story checkpoint before moving forward.

### Parallel Opportunities

- T010 can be prepared independently from the command-module compatibility tests after both renderer stories exist.
- T013 and T014 affect separate documentation files and can proceed together after behavior stabilizes.

---

## Parallel Example: Documentation and Changelog

```text
Task T013: Reconcile S067, master specification, outline, roadmap, and CLI reference.
Task T014: Add the issue and decision changelog fragments.
```

---

## Implementation Strategy

### MVP First

1. Complete the contract baseline and shared display policy.
2. Add and pass User Story 1's aligned-table tests.
3. Validate a short multi-state listing independently.

### Incremental Delivery

1. Deliver aligned scanability for fitting rows.
2. Add complete-value vertical handling for long and localized rows.
3. Bind actual width selection and prove JSON, ordering, diagnostics, and storage compatibility.
4. Reconcile documentation and run the full gate.

## Notes

- `[P]` tasks touch distinct files or surfaces after their dependencies are complete.
- No task changes Steam discovery, target resolution, persistence, schemas, or dependencies.
- Human control representation is presentation-only; JSON retains original values.
