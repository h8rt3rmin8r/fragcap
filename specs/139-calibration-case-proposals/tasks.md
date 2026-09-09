# Tasks: Guided Calibration Case Proposals

**Input**: Design documents from `specs/139-calibration-case-proposals/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/proposal-api.md, quickstart.md

**Tests**: Required by the feature specification and autopilot protocol. Each behavior phase begins with a failing test before implementation.

**Organization**: Tasks are grouped by user story so topology discovery, evidence proposal, and exact override behavior remain independently reviewable.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel because it touches a different file and has no incomplete dependency.
- **[Story]**: Maps the task to the user story in spec.md.
- Every task names its concrete file path.

## Phase 1: Setup and Specification

**Purpose**: Establish the bounded tracked slice and complete its design authority.

- [X] T001 Confirm clean `main`, create `codex/s139-calibration-case-proposals`, and point `.specify/feature.json` to `specs/139-calibration-case-proposals`
- [X] T002 Create issue #392 as an S139 child of #380, add it to the repository delivery project, and set its Slice and Stage fields
- [X] T003 [P] Complete and validate `specs/139-calibration-case-proposals/spec.md` and `specs/139-calibration-case-proposals/checklists/requirements.md`
- [X] T004 [P] Complete the safety requirements gate in `specs/139-calibration-case-proposals/checklists/safety.md`
- [X] T005 Record architecture, research, model, contract, validation, deviation, and no-effect decisions in `specs/139-calibration-case-proposals/plan.md`, `research.md`, `data-model.md`, `contracts/proposal-api.md`, and `quickstart.md`

---

## Phase 2: Foundational Contract

**Purpose**: Define the stable typed surface required by all proposal stories.

- [X] T006 Add failing type, token, constructor, and accessor tests for the proposal request and output contract in `crates/fragcap/src/deep_capture/proposal.rs`
- [X] T007 Implement request, process snapshot, topology kind, launch readiness, limitation, reason, step, deferred protocol, and proposal types in `crates/fragcap/src/deep_capture/proposal.rs`
- [X] T008 Export the additive contract through `crates/fragcap/src/deep_capture/mod.rs` and `crates/fragcap/src/deep_capture/api.rs`
- [X] T009 Extend the reviewed stable symbol inventory and compile-time consumer coverage in `crates/fragcap/src/deep_capture/api.rs`
- [X] T010 Run the blocking spec-kit analysis over `specs/139-calibration-case-proposals/spec.md`, `plan.md`, and `tasks.md`, then resolve every actionable finding in those artifacts

**Checkpoint**: The later CLI can consume one reviewed typed proposal boundary without importing backend modules.

---

## Phase 3: User Story 1 - Discover One Safe Launch Case (Priority: P1) MVP

**Goal**: Derive strict Steam, direct, or publisher topology and report cold, warm, ambiguous, or unavailable readiness without effects.

**Independent Test**: Synthetic targets and snapshots cover every topology and readiness branch without machine I/O.

### Tests for User Story 1

- [X] T011 [US1] Add failing Steam and direct cold, warm, missing, and ambiguous topology tests in `crates/fragcap/src/deep_capture/proposal.rs`
- [X] T012 [US1] Add failing publisher order, role, conflicting-image, cold, launcher-warm, and game-start-clean-warm tests in `crates/fragcap/src/deep_capture/proposal.rs`
- [X] T013 [US1] Add failing unavailable-snapshot, case-insensitive image, duplicate-image, and zero-step safety tests in `crates/fragcap/src/deep_capture/proposal.rs`

### Implementation for User Story 1

- [X] T014 [US1] Implement strict stored-target topology derivation and stable topology limitations in `crates/fragcap/src/deep_capture/proposal.rs`
- [X] T015 [US1] Implement complete-snapshot readiness and operator-owned warm-to-cold guidance in `crates/fragcap/src/deep_capture/proposal.rs`

**Checkpoint**: Every supported or uncertain target shape has one deterministic pre-effect readiness result.

---

## Phase 4: User Story 2 - Propose Only Useful Exact Measurements (Priority: P1)

**Goal**: Propose reachability first, then only missing or uncertain exact protocol work, with stable evidence reasons.

**Independent Test**: Exact fact-history permutations produce missing, stale, legacy, mismatch, negative, conflict, positive suppression, and deferred outcomes deterministically.

### Tests for User Story 2

- [X] T016 [US2] Add failing routing fact-state and reachability-first tests in `crates/fragcap/src/deep_capture/proposal.rs`
- [X] T017 [US2] Add failing protocol inspectability, conflict, positive suppression, and deferred-protocol tests in `crates/fragcap/src/deep_capture/proposal.rs`
- [X] T018 [US2] Add failing fact and protocol permutation determinism tests in `crates/fragcap/src/deep_capture/proposal.rs`

### Implementation for User Story 2

- [X] T019 [US2] Implement exact fact assessment through S121 applicability plus conservative conflict detection in `crates/fragcap/src/deep_capture/proposal.rs`
- [X] T020 [US2] Implement reachability-first step construction, protocol deferral, positive suppression, and stable ordering in `crates/fragcap/src/deep_capture/proposal.rs`

**Checkpoint**: Every runnable or deferred item is exact, minimal, ordered, and evidence-explained.

---

## Phase 5: User Story 3 - Keep Defaults and Overrides Explainable (Priority: P2)

**Goal**: Apply conservative defaults and exact supported overrides without hidden fallback.

**Independent Test**: Routing, family, version-context, and invalid-protocol permutations retain one complete case or a zero-step limitation.

### Tests for User Story 3

- [X] T021 [US3] Add failing default and IPv6 override case-identity tests in `crates/fragcap/src/deep_capture/proposal.rs`
- [X] T022 [US3] Add failing unsupported-routing, invalid-context, and invalid-protocol limitation tests in `crates/fragcap/src/deep_capture/proposal.rs`

### Implementation for User Story 3

- [X] T023 [US3] Implement child-environment and IPv4 defaults plus exact family override propagation in `crates/fragcap/src/deep_capture/proposal.rs`
- [X] T024 [US3] Implement zero-step context, routing, and protocol limitation handling in `crates/fragcap/src/deep_capture/proposal.rs`

**Checkpoint**: Defaults and overrides use one policy path and every unsupported request remains explicit.

---

## Phase 6: Documentation, Analysis, and Verification

**Purpose**: Reconcile the shipped boundary, close artifact gaps, and prove repository parity.

- [X] T025 [P] Extend the compatibility-calibration glossary entry in `docs/glossary/capture-and-networking.md`
- [X] T026 [P] Record S139 in `docs/fragcap-specification.md`, `docs/fragcap-spec-outline.md`, `docs/plans/README.md`, and `AGENTS.md`
- [X] T027 [P] Add the user-visible change fragment at `changelog.d/S139-calibration-case-proposals.changed.md`
- [X] T028 Run implementation convergence against `specs/139-calibration-case-proposals/` and append or complete any remaining tasks
- [X] T029 Mark `specs/139-calibration-case-proposals/spec.md` complete and verify every task and checklist is complete
- [X] T030 Run `cargo fmt --all -- --check`, focused facade tests, `cargo xtask ci`, and encoding/mojibake checks in the foreground

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1** is complete and establishes scope.
- **Phase 2** depends on Phase 1 and blocks every user story.
- **Phase 3** depends on Phase 2 and supplies launch readiness to later stories.
- **Phase 4** depends on Phase 3 because runnable steps require one cold launch case.
- **Phase 5** depends on Phase 4 because overrides alter exact applicability.
- **Phase 6** depends on all user stories.

### User Story Dependencies

- **US1**: Independent topology and readiness MVP after the foundational types.
- **US2**: Requires US1's cold launch identity but is independently testable with synthetic facts.
- **US3**: Uses US2's exact case construction and is independently testable through overrides and limitations.

### Parallel Opportunities

- T003 and T004 target distinct checklist files.
- T024, T025, and T026 target distinct documentation and changelog files after behavior is stable.
- Source implementation remains sequential because all three stories share `proposal.rs` and TDD ordering is load-bearing.

## Implementation Strategy

1. Complete the stable model and token contract.
2. Deliver topology and readiness as the independently useful MVP.
3. Add exact evidence assessment and reachability-first sequencing.
4. Add defaults and override refusal without another policy path.
5. Reconcile documentation, analyze, converge, and run the complete repository gate.

## Notes

- Tests must fail before the corresponding implementation task begins.
- `tasks.md` remains chronological, and completed tasks are marked `[X]` immediately.
- No task may add a CLI command or invoke process, store, trust, capture, network, launch, artifact, or fact-write effects.
- Issue #392 closes on merge. Parent issue #380 remains open.
