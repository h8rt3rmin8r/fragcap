# Tasks: Native Deep Capture Threat Model

**Input**: Design documents from `specs/125-native-threat-model/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Required by FR-016 and the autopilot TDD protocol.

## Phase 1: Setup and Contracts

- [x] T001 Validate the active feature selector, branch, issue scope, and dependency baseline in `.specify/feature.json`, issue #323, and `Cargo.lock`
- [x] T002 [P] Finalize the gate contract and threat data model in `specs/125-native-threat-model/contracts/` and `data-model.md`
- [x] T003 [P] Validate security requirement quality in `specs/125-native-threat-model/checklists/security.md`

## Phase 2: Failing Validator Evidence

- [x] T004 Add failing unit tests for malformed rows, duplicate and unknown references, absent and ignored tests, protocol drift, and dependency drift in `xtask/src/threat_model.rs`
- [x] T005 Add the initial canonical registry fixture in `docs/security/deep-capture-threats.v1.json` and prove the new command fails before validation is implemented

## Phase 3: User Story 1 - Audit Every Native Trust Boundary (Priority: P1)

**Goal**: Establish one complete, versioned, executable threat inventory.

**Independent Test**: Every high-risk row resolves all control, evidence, and negative-test ownership.

- [x] T006 [US1] Implement structural, vocabulary, cross-reference, and high-risk completeness validation in `xtask/src/threat_model.rs`
- [x] T007 [US1] Implement path-specific, attributed, non-ignored Rust test reference validation in `xtask/src/threat_model.rs`
- [x] T008 [US1] Complete the canonical trust boundaries, sensitive assets, and threat rows in `docs/security/deep-capture-threats.v1.json`
- [x] T009 [US1] Publish the reviewer-facing model and control map in `docs/security/deep-capture-threat-model.md`

## Phase 4: User Story 2 - Prove Abuse Fails Closed (Priority: P2)

**Goal**: Bind each high-risk native abuse case to executable negative evidence.

**Independent Test**: Focused proxy, facade, CLI, artifact, and recovery suites prove no open proxy or hidden normalization path.

- [x] T010 [US2] Audit every registry test reference against the shipped native path and add focused negative coverage for any material gap in `crates/fragcap-proxy/tests/`, `crates/fragcap/tests/`, or `crates/fragcap-cli/tests/`
- [x] T011 [US2] Reconfirm the P-1 prohibition and open-proxy refusal across every routing and protocol row in the registry and model

## Phase 5: User Story 3 - Force Attack-Surface Review (Priority: P3)

**Goal**: Fail CI when protocol or direct proxy dependency scope changes without threat review.

**Independent Test**: Controlled inventory mutations fail until reviewed inventories match.

- [x] T012 [US3] Implement exhaustive protocol-family inventory comparison in `xtask/src/threat_model.rs`
- [x] T013 [US3] Implement direct normal and Windows-target proxy dependency comparison in `xtask/src/threat_model.rs`
- [x] T014 [US3] Add the `threat-model` command and wire it into `cargo xtask ci` in `xtask/src/main.rs`

## Phase 6: Documentation and Verification

- [x] T015 [P] Add threat-model vocabulary and review guidance in `docs/glossary/capture-and-networking.md` and `docs/glossary/index.md`
- [x] T016 [P] Record S125 and the #323/#324-#334 boundary in `docs/fragcap-specification.md`, `docs/fragcap-spec-outline.md`, `docs/plans/README.md`, and `AGENTS.md`
- [x] T017 [P] Add S125 feature and dated decision fragments in `changelog.d/`
- [x] T018 Run spec-kit analysis across `spec.md`, `plan.md`, and `tasks.md` and remediate every finding
- [x] T019 Run focused validator and abuse-case tests from `specs/125-native-threat-model/quickstart.md`
- [x] T020 Run `cargo xtask ci`, text hygiene, dependency lock, and forbidden-capability checks
- [x] T021 Mark every completed task in `specs/125-native-threat-model/tasks.md` and perform the final scope audit

## Dependencies and Execution Order

- Phase 1 fixes the reviewed contract.
- Phase 2 establishes red tests before validator behavior.
- User Story 1 blocks the executable evidence and drift stories.
- User Stories 2 and 3 may proceed after the registry schema stabilizes.
- Documentation and full verification follow implementation.

## Parallel Opportunities

- T002 and T003 touch independent review artifacts.
- T012 and T013 validate independent attack-surface inventories.
- T015, T016, and T017 touch separate documentation groups after behavior stabilizes.

## Implementation Strategy

1. Make malformed and drift cases fail before implementing the validator.
2. Populate the registry from existing executable evidence, adding only focused gaps.
3. Wire review currency into ordinary CI.
4. Run focused suites, then the complete repository gate.
