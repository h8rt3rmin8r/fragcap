# Tasks: S155 independent review execution and findings intake

## Phase 1: Setup

- [x] T001 Record the immutable v0.10.1 candidate and six public asset identities in `docs/security/native-product-review-candidate.v1.json`
- [x] T002 [P] Record the pinned workflow and script decision in `changelog.d/S155-independent-review-intake.decisions.md`
- [x] T003 [P] Record the operator-visible security change in `changelog.d/S155-independent-review-intake.security.md`

## Phase 2: Foundational

- [x] T004 Add strict candidate-registry loading and exact identity comparison helpers in `xtask/src/review_record.rs`
- [x] T005 Wire the bounded `review-record <PATH>` command into `xtask/src/main.rs`

## Phase 3: User Story 1 - Validate an independent review record (P1)

**Goal**: Reject incomplete, unsafe or mechanically blocked reviewer records without claiming independent acceptance.

**Independent test**: Run positive and mutation tests for the completed-record validator while the committed readiness record remains `not-started`.

- [x] T006 [US1] Add synthetic positive and at least twelve independent mutation tests in `xtask/src/review_record.rs`
- [x] T007 [US1] Implement closed-shape, size, bounds, public-safety, twelve-area and installed-case validation in `xtask/src/review_record.rs`
- [x] T008 [US1] Implement critical, high and medium finding disposition plus retest validation in `xtask/src/review_record.rs`
- [x] T009 [US1] Preserve readiness semantics and test both review commands in `xtask/src/review_handoff.rs` and `xtask/src/main.rs`

## Phase 4: User Story 2 - Re-execute published installed-build evidence (P1)

**Goal**: Verify exact public v0.10.1 assets and run separate portable and installed smoke on disposable hosted Windows.

**Independent test**: Validate report schema 3 with two unique passing smoke surfaces and reject missing, duplicate, incomplete or non-contained observations.

- [x] T010 [US2] Add report-schema-3 positive and mutation tests for portable and installed surfaces in `xtask/src/package_certification.rs`
- [x] T011 [US2] Extend package report validation to require two unique contained smoke surfaces in `xtask/src/package_certification.rs`
- [x] T012 [US2] Refactor controlled smoke into a reusable hidden bounded helper in `scripts/Test-PackageCertification.ps1`
- [x] T013 [US2] Execute and report portable plus post-install smoke with exhaustive cleanup in `scripts/Test-PackageCertification.ps1`
- [x] T014 [US2] Add exact six-asset download, registry verification, certification and sanitized upload in `.github/workflows/published-review-candidate.yml`

## Phase 5: User Story 3 - Preserve the independent acceptance boundary (P2)

**Goal**: Make hosted evidence useful without allowing implementation automation to close the independent review gate.

**Independent test**: Verify handoff, specification and plan text keep #333 and #413 open and identify the installed QUIC limitation.

- [x] T015 [P] [US3] Update reviewer instructions, command semantics, hosted replay and QUIC limitation in `docs/security/native-product-review-handoff.md`
- [x] T016 [P] [US3] Add S155 chronological state and next-gate language in `docs/plans/README.md`
- [x] T017 [US3] Reconcile S154 lineage and record the S155 evidence boundary in `docs/fragcap-specification.md`

## Phase 6: Polish and Cross-Cutting Concerns

- [x] T018 Mark every completed task and final verification command in `specs/155-independent-review-intake/tasks.md`
- [x] T019 Run focused Rust tests, static PowerShell parsing, workflow lint and all quickstart checks from `specs/155-independent-review-intake/quickstart.md`
- [x] T020 Run `cargo xtask ci`, inspect the complete output and perform UTF-8, LF, forbidden-dash and mojibake sanity checks across every changed file
- [x] T021 Reconcile issue #423 and Project 3 stage with the final pull-request state

## Dependencies

- Phase 1 precedes all implementation.
- Phase 2 precedes User Story 1 and provides candidate identity to User Story 2.
- User Story 1 and User Story 2 are independently testable after Phase 2.
- User Story 3 depends on the final behavior and limitations from User Stories 1 and 2.
- Phase 6 follows all user stories.

## Parallel Opportunities

- T002 and T003 are independent after T001 scope is fixed.
- User Story 1 Rust validation and User Story 2 package automation touch separate authorities after T004.
- T015 and T016 are independent documentation updates after implementation behavior stabilizes.

## Implementation Strategy

Implement candidate identity and review-record mutation tests first. Complete the validator as the minimum independently useful increment, then extend the existing package report and script, add hosted replay, and finish with honest governance documentation and the full repository gate.
