# Tasks: Native Deep Capture Failure Injection

**Input**: Design documents from `specs/127-native-failure-injection/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Required by FR-010, FR-016, and the autopilot TDD protocol.

## Phase 1: Setup and Contracts

- [x] T001 Validate the active feature selector, branch, issue scope, merged S126 baseline, and dependency lock in `.specify/feature.json`, issue #325, and `Cargo.lock`
- [x] T002 [P] Finalize the failure-matrix contract and data model in `specs/127-native-failure-injection/contracts/failure-matrix.md` and `specs/127-native-failure-injection/data-model.md`
- [x] T003 [P] Validate requirements and resilience quality in `specs/127-native-failure-injection/checklists/requirements.md` and `specs/127-native-failure-injection/checklists/resilience.md`

## Phase 2: Failing Registry and Validator Evidence

- [x] T004 Add failing validator tests for schemas, boundaries, sides, families, outcomes, inventory drift, and attributed test references in `xtask/src/failure_matrix.rs`
- [x] T005 Add the initial registry in `docs/security/deep-capture-failures.v1.json` and prove the new command reports incomplete matrix evidence before validator completion

## Phase 3: User Story 1 - Exercise Every Effect Boundary (Priority: P1)

**Goal**: Generate and execute both sides of every journaled effect and checked lifecycle transition.

**Independent Test**: The closed inventories generate exactly two uniquely identified scenarios per boundary and reject any source drift.

- [x] T006 [US1] Implement registry schema, identifier, vocabulary, cross-reference, and complete outcome-vector validation in `xtask/src/failure_matrix.rs`
- [x] T007 [US1] Derive and compare production resource, coordinator effect, and lifecycle state inventories in `xtask/src/failure_matrix.rs`
- [x] T008 [US1] Generate stable before and after scenario identities and enforce complete mandatory failure-family ownership in `xtask/src/failure_matrix.rs`
- [x] T009 [US1] Populate all journaled effect and checked lifecycle boundaries in `docs/security/deep-capture-failures.v1.json`
- [x] T010 [US1] Add attributed non-ignored executable test-reference validation in `xtask/src/failure_matrix.rs`

## Phase 4: User Story 2 - Preserve Independent Terminal Truth (Priority: P2)

**Goal**: Prove every failure keeps terminal, artifact, fact, event, cleanup, journal, and recovery authorities independent.

**Independent Test**: Generated scenarios execute through the production coordinator and compare each applicable outcome dimension separately.

- [x] T011 [US2] Add the controlled generated-matrix adapter harness beside the existing controlled adapters in `crates/fragcap/tests/deep_capture_session.rs`
- [x] T012 [US2] Add before-effect and after-effect production coordinator cases for proxy, trust, route, launch, Capture, and bundle boundaries in `crates/fragcap/tests/deep_capture_session.rs`
- [x] T013 [US2] Add lifecycle transition, invalid-order, deadline, cancellation, and event-delivery cases in `crates/fragcap/tests/deep_capture_session.rs`
- [x] T014 [US2] Assert terminal, artifact, fact, event, cleanup, journal, and recovery dispositions independently for every generated case in `crates/fragcap/tests/deep_capture_session.rs`
- [x] T015 [US2] Add focused fact refusal, incomplete artifact, and later-cleanup continuation regressions in `crates/fragcap/tests/deep_capture_session.rs`

## Phase 5: User Story 3 - Cover Failure Families and Recovery (Priority: P3)

**Goal**: Bind every required native failure family to deterministic evidence and exact recovery behavior.

**Independent Test**: All ten families resolve to executable rows, and journal recovery mutates only exactly owned residue.

- [x] T016 [US3] Cover disk full, permission denial, broken pipe, and writer corruption through controlled artifact and lifecycle writers in `crates/fragcap/tests/deep_capture_session.rs`
- [x] T017 [US3] Cover task panic, timeout, cancellation, trust denial, port theft, and network reset through controlled native adapter outcomes in `crates/fragcap/tests/deep_capture_session.rs`
- [x] T018 [US3] Add exact-action, no-action, and refusal recovery assertions for failed and uncertain journal prefixes in `crates/fragcap/tests/deep_capture_journal.rs`
- [x] T019 [US3] Add the `failure-matrix` command and wire it into `cargo xtask ci` in `xtask/src/main.rs`

## Phase 6: Documentation and Verification

- [x] T020 [P] Publish matrix semantics, failure-family mapping, and reproduction guidance in `docs/security/deep-capture-failure-injection.md`
- [x] T021 [P] Add failure-injection vocabulary in `docs/glossary/capture-and-networking.md` and regenerate `docs/glossary/index.md`
- [x] T022 [P] Record S127 and the #325/#326-#334 boundary in `docs/fragcap-specification.md`, `docs/fragcap-spec-outline.md`, `docs/plans/README.md`, and `AGENTS.md`
- [x] T023 [P] Add S127 feature and dated decision fragments in `changelog.d/`
- [x] T024 Run spec-kit analysis across `spec.md`, `plan.md`, and `tasks.md` and remediate every finding
- [x] T025 Run the focused validator, coordinator, journal, and documentation checks from `specs/127-native-failure-injection/quickstart.md`
- [x] T026 Run `cargo xtask ci`, text hygiene, dependency lock, forbidden-capability, and mojibake checks
- [x] T027 Run post-implementation convergence, complete any appended tasks, mark every task in `specs/127-native-failure-injection/tasks.md`, and perform the final scope audit

## Dependencies and Execution Order

- Phase 1 fixes the reviewed contract and scope.
- Phase 2 establishes red validator evidence before validation behavior.
- User Story 1 blocks generated execution and failure-family evidence.
- User Story 2 blocks the recovery completeness claim in User Story 3.
- Documentation and full verification follow executable convergence.

## Parallel Opportunities

- T002 and T003 touch independent specification artifacts.
- T006 and T011 may begin after the registry shape is fixed, but shared inventory changes remain sequential.
- T020, T021, T022, and T023 touch independent documentation groups after behavior stabilizes.

## Implementation Strategy

1. Make incomplete registries and production inventory drift fail first.
2. Generate both matrix sides from one boundary inventory.
3. Execute controlled failures through production coordinator and journal authorities.
4. Bind every required family to attributed evidence and ordinary CI.
5. Run focused checks, convergence, then the complete repository gate.
