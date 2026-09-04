# Tasks: Native Deep Capture Doctor Readiness

**Input**: Design documents from `specs/124-native-doctor-readiness/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Required by FR-020 and the autopilot TDD protocol.

## Phase 1: Setup and Contracts

- [x] T001 Validate the active feature selector, branch, issue scope, and dependency baseline in `.specify/feature.json`, issue #321, and `Cargo.lock`
- [x] T002 [P] Finalize mode-verdict and owner-lease contracts in `specs/124-native-doctor-readiness/contracts/`
- [x] T003 [P] Validate runtime-recovery security requirements in `specs/124-native-doctor-readiness/checklists/security.md`

## Phase 2: Foundational Ownership and Inventory

- [x] T004 Add failing owner-record, lease-liveness, PID-reuse, and exact custom-root tests in `crates/fragcap-cli/src/doctor/residue.rs`
- [x] T005 Replace PID-only session registration with a held generation-specific lease in `crates/fragcap-cli/src/doctor/residue.rs` and `fix.rs`
- [x] T006 Add failing bounded journal, manifest, artifact, listener, malformed, and overflow inventory tests in `crates/fragcap-cli/src/doctor/residue.rs`
- [x] T007 Implement the bounded read-only native residue inventory and stable health mapping in `crates/fragcap-cli/src/doctor/residue.rs`
- [x] T008 Reuse the shared inventory and owner authority from `crates/fragcap-cli/src/doctor/fix.rs` and `probe.rs`

## Phase 3: User Story 1 - Separate Mode Verdicts (Priority: P1)

**Goal**: Report Capture and Deep Capture readiness independently from one ordered check set.

**Independent Test**: Every Capture-only, Deep-only, shared, and ready matrix case agrees between human and JSON output.

- [x] T009 [US1] Add failing mode-scope, verdict-matrix, JSON Lines, and human summary tests in `crates/fragcap-cli/src/doctor/mod.rs`
- [x] T010 [US1] Add stable check mode scopes and derive both verdicts in `crates/fragcap-cli/src/doctor/mod.rs`
- [x] T011 [US1] Render separate human and JSON verdicts while preserving the command exit contract in `crates/fragcap-cli/src/doctor/mod.rs`

## Phase 4: User Story 2 - Audit Native Readiness and Residue (Priority: P2)

**Goal**: Expose every provable native runtime and residue state without legacy external-backend placeholders or false clean results.

**Independent Test**: Controlled inventories produce exactly one stable state per authority, preserve every unknown, and never infer ownership from PID or port alone.

- [x] T012 [P] [US2] Add readiness checks for native backend, loopback families, session storage, lease state, resource findings, artifacts, trust, and limitations in `crates/fragcap-cli/src/doctor/checks.rs`
- [x] T013 [US2] Remove external proxy and orphan-process placeholder inputs and checks from `crates/fragcap-cli/src/doctor/probe.rs` and `checks.rs`
- [x] T014 [US2] Thread one inventory through production probing and both report formats in `crates/fragcap-cli/src/doctor/probe.rs` and `mod.rs`
- [x] T015 [US2] Add CLI contract cases for healthy history, active work, stale work, cleanup failure, unknown evidence, unsupported platform, and unrelated listeners in `crates/fragcap-cli/tests/`

## Phase 5: User Story 3 - Repair Only Proven Owned Residue (Priority: P3)

**Goal**: Offer and execute only exact journal-authorized recovery while preserving active and ambiguous state.

**Independent Test**: Active, unknown, unrelated, already-terminal, partial-failure, and recoverable matrices mutate only the exact confirmed resources.

- [x] T016 [US3] Add failing recovery-offer, active-preservation, refusal, partial-failure, and re-inventory tests in `crates/fragcap-cli/src/doctor/fix.rs`
- [x] T017 [US3] Derive offers solely from inventory-carried journal recovery plans and preserve the existing confirmation contract in `crates/fragcap-cli/src/doctor/fix.rs`
- [x] T018 [US3] Re-inventory after repair and report performed, refused, skipped, and failed outcomes in `crates/fragcap-cli/src/doctor/fix.rs`

## Phase 6: Documentation and Verification

- [x] T019 [P] Add native Doctor, mode verdict, session owner lease, and residue vocabulary in `docs/glossary/capture-and-networking.md` and `docs/glossary/index.md`
- [x] T020 [P] Record S124 runtime scope and the #321/#329 boundary in `docs/fragcap-specification.md`, `docs/fragcap-spec-outline.md`, `docs/plans/README.md`, and `AGENTS.md`
- [x] T021 [P] Add S124 feature and dated decision fragments in `changelog.d/`
- [x] T022 Run spec-kit analysis across `spec.md`, `plan.md`, and `tasks.md` and remediate every finding
- [x] T023 Run focused Doctor inventory, readiness, repair, and CLI contract tests from `specs/124-native-doctor-readiness/quickstart.md`
- [x] T024 Run `cargo xtask ci`, text hygiene, dependency lock, and forbidden-capability checks
- [x] T025 Mark every completed task in `specs/124-native-doctor-readiness/tasks.md` and perform the final scope audit

## Dependencies and Execution Order

- Phase 1 establishes the reviewed contract.
- Phase 2 blocks every user story and replaces the unsafe PID-only authority.
- User Story 1 defines report semantics, User Story 2 supplies native findings, and User Story 3 wires exact repair.
- Documentation can proceed after contracts stabilize; full verification follows implementation.

## Parallel Opportunities

- T002 and T003 are independent requirements checks.
- T012 can begin against the data model while output contracts are implemented.
- T019, T020, and T021 touch separate documentation groups after behavior stabilizes.

## Implementation Strategy

1. Make ownership and inventory tests fail before production behavior.
2. Establish two mode verdicts before replacing legacy Deep Capture checks.
3. Reuse journal recovery rather than adding cleanup policy.
4. Run focused tests after each story, then the complete repository gate.
