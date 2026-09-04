# Tasks: Native Deep Capture Parser Fuzzing

**Input**: Design documents from `specs/126-native-parser-fuzzing/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Required by issue #324 and the autopilot TDD protocol.

## Phase 1: Setup and Contracts

- [x] T001 Validate the active selector, branch, issue #324, shipped parser inventory, and product dependency baseline
- [x] T002 [P] Complete the fuzz gate contract and surface data model in `specs/126-native-parser-fuzzing/contracts/` and `data-model.md`
- [x] T003 [P] Validate adversarial-input requirements in `specs/126-native-parser-fuzzing/checklists/security.md`

## Phase 2: Failing Gate Evidence

- [x] T004 Add failing validator tests for surface, target, corpus, CI, tracking, content, version, and bound drift in `xtask/src/fuzz.rs`
- [x] T005 Add failing stable seed replay coverage in `crates/fragcap/tests/fuzz_seeds.rs`

## Phase 3: User Story 1 - Exercise Every Owned Boundary (Priority: P1)

**Goal**: Map and execute every fragcap-owned parser and state machine.

**Independent Test**: Registry validation proves every surface has one executable target and corpus.

- [x] T006 [US1] Publish the exhaustive versioned surface registry in `fuzz/fuzz-targets.json`
- [x] T007 [US1] Add bounded protocol exercise seams in `crates/fragcap-proxy/src/fuzz_support.rs` and owning modules
- [x] T008 [US1] Add bounded artifact exercise seams in `crates/fragcap/src/deep_capture/fuzz_support.rs` and owning modules
- [x] T009 [US1] Implement six isolated coverage-guided binaries in `fuzz/fuzz_targets/`

## Phase 4: User Story 2 - Deterministic Reproduction (Priority: P2)

**Goal**: Make every permanent fuzz input fast, synthetic, and reproducible on stable Rust.

**Independent Test**: Two stable replays execute all seeds in identical sorted order.

- [x] T010 [US2] Add minimized synthetic corpora and dictionaries under `fuzz/corpus/` and `fuzz/dictionaries/`
- [x] T011 [US2] Implement stable corpus replay in `crates/fragcap/tests/fuzz_seeds.rs`
- [x] T012 [US2] Implement the registry and corpus validator plus `cargo xtask fuzz` in `xtask/src/fuzz.rs` and `xtask/src/main.rs`
- [x] T013 [US2] Add controlled validator rejection tests and seed regression tests

## Phase 5: User Story 3 - Reproducible Campaigns (Priority: P3)

**Goal**: Continuously build and run every target with exact finite limits.

**Independent Test**: The pinned Linux matrix completes every bounded target and uploads any finding.

- [x] T014 [US3] Create the isolated exact-pinned fuzz manifest and lockfile in `fuzz/`
- [x] T015 [US3] Add the complete bounded target matrix in `.github/workflows/fuzz.yml`
- [x] T016 [US3] Document smoke, long campaign, reproduction, minimization, coverage, and corpus handling in `docs/security/deep-capture-fuzzing.md`

## Phase 6: Documentation and Verification

- [x] T017 [P] Record S126 in the master specification, outline, roadmap, glossary/testing guidance, and `AGENTS.md`
- [x] T018 [P] Add S126 feature and dated decision fragments in `changelog.d/`
- [x] T019 Run spec-kit analysis across `spec.md`, `plan.md`, and `tasks.md` and remediate every finding
- [x] T020 Run stable registry and corpus gates plus all focused unit and integration tests
- [x] T021 Build every coverage-guided target locally and bind every bounded run to the pinned Linux matrix
- [x] T022 Run `cargo xtask ci`, text hygiene, lock isolation, and prohibited-capability checks
- [x] T023 Mark all tasks complete and perform final scope and corpus audits

## Dependencies and Execution Order

- Phase 1 fixes the reviewed contract.
- Phase 2 establishes red tests before behavior.
- User Story 1 supplies the shared exercise boundary.
- User Story 2 binds the permanent corpus to that boundary.
- User Story 3 packages the same boundary for coverage-guided CI.
- Documentation and full verification follow implementation.

## Parallel Opportunities

- T002 and T003 touch independent specification artifacts.
- T007 and T008 have separate crate ownership after the registry stabilizes.
- T014 and T016 touch independent harness and documentation files.
- T017 and T018 touch separate documentation groups after behavior stabilizes.

## Implementation Strategy

1. Make registry and stable replay tests fail first.
2. Add bounded owner-local seams and synthetic corpus cases.
3. Bind the same entry points to libFuzzer and exact CI pins.
4. Run stable replay, bounded campaigns, and the complete repository gate.
