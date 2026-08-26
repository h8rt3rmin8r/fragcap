# Tasks: Known-Roots Discovery Corrections

**Input**: Design documents from `/specs/077-known-roots-discovery-corrections/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Required. The slice corrects a P-4 accounting defect and a durable path identity defect, so regression and conservation tests precede implementation.

## Phase 1: Setup And Baseline

**Purpose**: Confirm the focused surfaces and preserve existing behavior before changing public verdict and account types.

- [X] T001 Run the focused baseline suites for `crates/fragcap-targets/tests/known_roots.rs`, `crates/fragcap-targets/tests/detection_walk.rs`, and target account rendering in `crates/fragcap-cli/src/commands/targets.rs`.

---

## Phase 2: Foundational Contracts

**Purpose**: Establish exhaustive container and accounting types used by both P1 stories.

- [X] T002 Add failing classifier tests for distinct-engine, repeated-same-engine, and non-engine evidence in `crates/fragcap-targets/src/classifier.rs`.
- [X] T003 Add `ClassifierVerdict::Container` and implement distinct engine-product classification in `crates/fragcap-targets/src/classifier.rs` per FR-001/FR-002/FR-003.
- [X] T004 Add `container_descended` and `container_descent_truncated` to conservation, aggregation, and unit tests in `crates/fragcap-targets/src/source.rs` per FR-007/FR-008.

**Checkpoint**: Classification can name a container and every aggregate can carry its two terminal outcomes.

---

## Phase 3: User Story 1 - Discover Titles Inside Containers (Priority: P1)

**Goal**: Suppress multi-engine aggregate directories and find separately classifiable children while preserving title stop-on-hit.

**Independent Test**: A synthetic two-engine container yields its child titles, never itself, while a one-engine title still stops descent.

- [X] T005 [US1] Add failing fixture-walk coverage for a traversable container, child candidates, and unchanged title stop-on-hit in `crates/fragcap-targets/tests/known_roots.rs`.
- [X] T006 [US1] Handle `ClassifierVerdict::Container` in `crates/fragcap-targets/src/sources/known_roots.rs`: suppress the candidate, increment `container_descended`, and recurse only while the existing bound permits.
- [X] T007 [US1] Add production-classifier temporary-tree coverage proving a multi-engine aggregate yields synthetic child titles in `crates/fragcap-targets/tests/detection_walk.rs`.

**Checkpoint**: Issue #210's false aggregate candidate and hidden-child failure are reproduced and corrected.

---

## Phase 4: User Story 2 - See Bounded Discovery Loss (Priority: P1)

**Goal**: Report containers whose children remain outside the shallow traversal bound.

**Independent Test**: A terminal-depth container increments only the truncation outcome, emits one named warning, and leaves the account conserved.

- [X] T008 [US2] Add failing terminal-depth container and mixed-outcome conservation tests in `crates/fragcap-targets/tests/known_roots.rs`.
- [X] T009 [US2] Implement terminal-depth container accounting and its named reduced-coverage warning in `crates/fragcap-targets/src/sources/known_roots.rs` per FR-007/FR-009/FR-010.
- [X] T010 [US2] Render both container outcomes in the discovery account line and update CLI assertions in `crates/fragcap-cli/src/commands/targets.rs`.

**Checkpoint**: Every observed container states whether descent happened, and bounded loss is visible.

---

## Phase 5: User Story 3 - Receive Canonical Path Identities (Priority: P2)

**Goal**: Compose real known roots with native separators before listing so candidate identity and install root stay canonical.

**Independent Test**: A real temporary filesystem walk emits no mixed-separator candidate path and preserves the shared root definitions.

- [X] T011 [US3] Add failing real-filesystem native-separator assertions for candidate identity and install root in `crates/fragcap-targets/tests/detection_walk.rs`.
- [X] T012 [US3] Normalize the separator-neutral root at the real filesystem boundary in `crates/fragcap-targets/src/sources/mod.rs` while leaving `KNOWN_ROOTS` and fixture keys unchanged.

**Checkpoint**: Issue #209's mixed-separator identity cannot be emitted by the real known-roots walk.

---

## Phase 6: Documentation And Verification

**Purpose**: Reconcile architecture, terminology, release notes, privacy, and the complete repository gate.

- [X] T013 [P] Update the known-roots, discovery-account, and stop-on-hit definitions and regenerate the index in `docs/glossary/process-and-attribution.md` and `docs/glossary/index.md` per P-6.
- [X] T014 [P] Reconcile the container-aware descent and account outcomes in `docs/fragcap-specification.md` section 7.1 per P-11.
- [X] T015 [P] Add `changelog.d/209-210-known-roots-discovery.fixed.md` with specification impact 7.1 and user-visible outcomes.
- [X] T016 Run focused tests, `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all --locked`, and `cargo xtask ci`; inspect the tracked diff for PII, real local titles, mixed encoding, and unplanned files.

## Dependencies

- Phase 2 blocks both P1 stories because the verdict and counters must exist first.
- User Story 1 establishes traversed-container behavior before User Story 2 adds the terminal-depth branch.
- User Story 3 is behaviorally independent after Phase 2 but remains sequential because it edits the same walker file.
- Documentation tasks T013-T015 can proceed in parallel after behavior is stable. T016 is the final gate.

## Implementation Strategy

1. Establish classifier and account contracts under failing tests.
2. Deliver issue #210 end to end, including real signature classification and P-4 truncation visibility.
3. Deliver issue #209 at the filesystem boundary with cross-platform temporary-tree coverage.
4. Reconcile the specification and glossary, then run the repository's complete gate before committing.
