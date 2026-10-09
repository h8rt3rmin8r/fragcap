# Tasks: S171 session retention and garbage collection

## Phase 1: Setup

- [x] T001 Read governing references and current issue #458; bind S171 branch and `.specify/feature.json`.
- [x] T002 Execute specify, clarify, retention checklist and plan research in `specs/171-session-retention-garbage-collection/`.
- [x] T003 Pass blocking spec-kit analyze and record coverage in `analysis.md`.

## Phase 2: Foundational authority

- [x] T004 Agree facade and CLI collection APIs in `contracts/collection.md` before parallel owning implementation.
- [x] T005 Add failing whole-session closure, ownership and exact deletion regressions in `crates/fragcap/tests/session_collection.rs` (FR-004, FR-008, FR-012).
- [x] T006 Implement strict inspection, recognized population and exact deletion pins in `crates/fragcap/src/deep_capture/collection.rs` (FR-004, FR-008).

## Phase 3: US1 finite ordinary history

- [x] T007 [P] [US1] Add policy/active/custom/legacy regressions in `crates/fragcap-cli/src/session_gc.rs` (FR-001, FR-002, FR-012).
- [x] T008 [US1] Implement bounded age/count/byte selection and owner authority in `crates/fragcap-cli/src/session_gc.rs` (FR-001, FR-004, FR-005).
- [x] T009 [P] [US1] Integrate retention choice, digest-bound consent, metadata and post-session maintenance in `cli.rs`, `commands/deep_capture.rs` and `session_ux.rs` (FR-001, FR-002, FR-003).

## Phase 4: US2 backlog reclamation

- [x] T010 [US2] Add interruption, stale-preview, partial effects and byte-accounting regressions in `crates/fragcap/tests/session_collection.rs` (FR-006, FR-009, FR-012).
- [x] T011 [US2] Implement external retirement state and retryable child removal in `crates/fragcap/src/deep_capture/collection.rs` (FR-009).
- [x] T012 [US2] Implement exact registry retirement, aggregate preview/apply and explicit retained/custom selection in `crates/fragcap-cli/src/session_gc.rs` (FR-002, FR-005, FR-006, FR-009).
- [x] T013 [P] [US2] Add CLI collect command/dispatch and measured human plus JSON rendering in `cli.rs` and `commands/bundle.rs` (FR-005, FR-006).

## Phase 5: US3 empty containers and diagnosis

- [x] T014 [US3] Add routine-preservation/explicit-purge and 1,000-session backlog tests in facade collection and CLI Doctor suites (FR-007, FR-010, FR-011, FR-012).
- [x] T015 [US3] Implement separately scoped exact empty purge in `collection.rs` and `session_gc.rs` (FR-007, FR-010).
- [x] T016 [US3] Correct read-only Doctor, registry parsing and recovery scanner capacity/limits in `doctor/residue.rs` and `doctor/fix.rs` (FR-010, FR-011).

## Phase 6: Documentation and gates

- [x] T017 Update master specification, plans README, glossary, public CLI/storage pages and `changelog.d/S171.*.md` (FR-013).
- [x] T018 Audit every #458 acceptance criterion in `issue-acceptance.md` and controlled evidence in `verification.md` (SC-001 through SC-005).
- [x] T019 Run focused regressions, API gates, text hygiene and full `cargo xtask ci`; fix failures (FR-012).
- [ ] T020 Commit/push official PR, attach it, resolve every external finding with at most two review rounds, verify final-head green CI and hand off for owner merge (owner instruction).

## Dependencies and parallel examples

T001-T004 block implementation. T005-T006 establish facade authority; T007-T009 implement US1, T010-T013 add US2, and T014-T016 complete US3. Collection agent and registry agent own distinct modules/tests; root may implement T009/T013 concurrently after agreed interfaces. T017 runs alongside code, then T018-T020 complete in order. Each story has independent synthetic validation, but full #458 completion requires all three stories.

## Implementation strategy

Write exact failing security/lifecycle regressions first, implement facade and policy authorities, integrate ordinary workflows, then verify the complete controlled matrix. Never weaken security assertions or use real owner data to prove collection. The first useful increment is US1, and it is not the complete slice.
