# Tasks: Homepage Positioning And Next-Command Footer

**Input**: Design documents from `specs/078-homepage-positioning-footer/`

**Prerequisites**: `spec.md`, `plan.md`, `research.md`, `data-model.md`, `contracts/`, `quickstart.md`

## Phase 1: Setup And Baseline

- [X] T001 Confirm branch, feature pointer, clean baseline, issue #232/#208 scope, and absence of extension hooks.
- [X] T002 Run focused baseline CLI tests and a production documentation build before edits; record any environmental limitation rather than weakening the gate.

---

## Phase 2: User Story 2 - Recognize The Next Command (Priority: P1)

**Goal**: Label the populated-listing suggestion without changing selection or other output.

**Independent Test**: Populated output has one exact labelled line after table or machine output; empty output is unchanged; bare invocation adds only its help footer.

- [X] T003 [US2] Add failing unit coverage in `crates/fragcap-cli/src/commands/targets.rs` for the exact `Next command:  fragcap capture <row>` text, machine-section separation, and empty-state absence per FR-009/FR-012/FR-019.
- [X] T004 [US2] Strengthen `crates/fragcap-cli/tests/cli_targets.rs` so a real CLI listing requires the exact labelled next-command text rather than an unlabeled substring.
- [X] T005 [US2] Replace the populated-listing bare suggestion in `crates/fragcap-cli/src/commands/targets.rs` while preserving the existing row-selection expression and footer ordering per FR-009/FR-010/FR-011.
- [X] T006 [US2] Run focused `fragcap-cli` unit and `cli_targets` integration tests and prove the pre-implementation failures now pass.

---

## Phase 3: User Story 1 - Understand Fragcap In One Viewport (Priority: P1)

**Goal**: Replace inaccurate homepage positioning with precise Capture and Deep Capture language.

**Independent Test**: The built first viewport leads with process attribution, explains correlation, distinguishes both modes, and contains no retired claim.

- [X] T007 [US1] Rewrite the opening and explanatory prose in `site/app/(home)/page.tsx` per FR-001 through FR-005, keeping exact prose flexible but claims bounded by `contracts/homepage-positioning.md`.
- [X] T008 [US1] Rewrite the dependency/readiness callout in `site/app/(home)/page.tsx` per FR-006 through FR-008, preserving the Npcap link and adding a link to the Deep Capture compatibility reference where useful.
- [X] T009 [US1] Correct passive-only capability and component commentary in `site/app/(home)/page.tsx` so mode-specific safety is explicit per FR-018 without changing layout, navigation, or action count.
- [X] T010 [US1] Run the authored-source retired-claim scan and inspect the first-viewport source order against SC-001/SC-002.

---

## Phase 4: User Story 3 - Trust The Specimen And Specification (Priority: P2)

**Goal**: Show synthetic current CLI output and reconcile the architecture of record.

**Independent Test**: The specimen uses current columns, synthetic rows, and the exact footer; the master specification says the same thing and supersedes S057 without rewriting it.

- [X] T011 [US3] Replace the homepage target specimen with `TARGET`, `CAPTURE`, `ENGINE`, and `SENSITIVITIES`, synthetic rows, and the exact labelled footer in `site/app/(home)/page.tsx` per FR-013/FR-014.
- [X] T012 [US3] Update specification section 17.7's listing example and prose to define the exact labelled next-command contract.
- [X] T013 [US3] Update specification sections 23.1 and 23.3 to define outcome-first, mode-aware, non-absolute homepage positioning and to supersede S057's frozen copy without altering historical artifacts.
- [X] T014 [US3] Add the dated master-specification revision entry and `changelog.d/208-232-homepage-footer.fixed.md` with specification impact `17.7, 23.1, 23.3`.

---

## Phase 5: Verification And Convergence

- [X] T015 Run `cargo fmt --all -- --check`, focused CLI tests, and `cargo xtask docs check`.
- [X] T016 Run `cargo xtask docs build`; scan the generated homepage for required and retired claims, static hosting markers, exact specimen footer, and synthetic-only rows.
- [X] T017 Run `cargo xtask ci` in the foreground with the local Npcap SDK import library where required.
- [X] T018 Run Spec Kit convergence against all requirements, success criteria, plan decisions, tasks, and constitution principles; append and implement any finding before proceeding.
- [X] T019 Audit the complete tracked diff for PII, actual local titles, personal paths, credentials, endpoints, encoding damage, unplanned files, changed historical S057 artifacts, and whitespace errors.
- [X] T020 Mark the specification Implemented and all tasks complete, rerun affected policy checks, create the local conventional commit, and stop before push under the autopilot protocol.

## Dependencies And Execution Order

- Phase 1 establishes the baseline and blocks implementation.
- Phase 2 owns the CLI contract that the Phase 4 homepage specimen must copy.
- Phase 3 can proceed independently after baseline.
- Phase 4 depends on Phase 2 for exact footer vocabulary and on Phase 3 for final page prose.
- Phase 5 depends on all implementation and documentation work.

## Parallel Opportunities

- T003/T004 can be authored together because they exercise different test levels.
- T007/T008/T009 touch one file and therefore execute sequentially despite representing distinct claim groups.
- T012 and T013 touch the same master document and execute as one coherent specification edit.

## MVP Scope

Phases 2 through 4 are the minimum useful delivery. Omitting any one leaves either the live CLI, the homepage specimen, or the architecture of record contradicting the others.

---

## Phase 6: Convergence

- [X] T021 Correct the homepage and master-specification specimens from `needs target` to the live CLI vocabulary `needs a target` per FR-013 and US3/AC1 (partial).
