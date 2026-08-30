# Tasks: Library-First Deep Capture Sessions

**Input**: Design documents from `/specs/098-library-deep-capture/`

**Tests**: Required. S098 changes a security-sensitive public state machine, resource ownership, event delivery, evidence persistence, and CLI architecture. Tests precede implementation in every phase.

## Phase 1: Contract Baseline

- [x] T001 Re-read issue #252, all S098 design documents, `AI_CONTEXT.md`, the constitution, and relevant master specification sections before implementation.
- [x] T002 Preserve the current Deep Capture CLI event, artifact, fact, refusal, cleanup, and exit behavior through the focused parity suite under `crates/fragcap-cli/tests/cli_deep_capture.rs`.
- [x] T003 Add the `deep-capture` facade feature and feature-matrix coverage in `crates/fragcap/Cargo.toml`, `crates/fragcap-cli/Cargo.toml`, and facade tests without implying live capture features.
- [x] T004 Add direct public-API tests in `crates/fragcap/tests/deep_capture_session.rs` proving no CLI dependency and no external effect under controlled adapters.

## Phase 2: Foundational Public Types and Capture Reuse

- [x] T005 [P] [US1] Cover configuration, effective deadlines, prepared plans, plan identifiers, outcomes, observations, facts, cleanup, artifacts, and terminal reports in direct integration tests.
- [x] T006 [P] [US2] Add compile-contract coverage for narrow adapter traits and CLI-free signatures in `crates/fragcap/tests/deep_capture_session.rs`.
- [x] T007 [US1] Expose ordinary Capture preparation and execution through a public Deep Capture adapter seam while retaining the existing acquisition and attribution composition.
- [x] T008 [US1] Bridge the shipped command to its existing prepared Capture path and prove ordinary Capture behavior remains unchanged.
- [x] T009 [US1] Create `crates/fragcap/src/deep_capture/mod.rs` and `model.rs` with documented typed public values and stable reason codes.
- [x] T010 [US2] Create `crates/fragcap/src/deep_capture/adapters.rs` with bounded effect-only traits and owned proxy, trust, launch, and Capture leases.

## Phase 3: User Story 1 - Direct Library Lifecycle

**Goal**: A Rust consumer can preflight and drive a complete Deep Capture session without invoking or depending on the CLI.

**Independent Test**: Controlled adapters complete every public lifecycle stage and return the expected typed report entirely through `fragcap::deep_capture`.

- [x] T011 [US1] Cover target resolution, launch validation, effective deadlines, bundle destination, backend descriptor, and existing compatibility prerequisites.
- [x] T012 [US1] Prove exact plan binding and zero effects for declined, stale, or mismatched approval.
- [x] T013 [US1] Cover invalid lifecycle order and resource reuse through checked transitions and at-most-once tests.
- [x] T014 [US1] Implement side-effect-free preflight and prepared-plan authorization in `crates/fragcap/src/deep_capture/session.rs`.
- [x] T015 [US1] Implement the checked coordinator, granular operations, and end-to-end runner in `crates/fragcap/src/deep_capture/session.rs`.
- [x] T016 [US1] Document ownership, deadlines, sensitive-artifact, adapter, and failure semantics in `crates/fragcap/src/deep_capture/mod.rs` and the API contracts.

## Phase 4: User Story 2 - Injectable Effects and Production Adapters

**Goal**: Integrations can replace every privileged or external effect while the shipped command uses facade-owned production implementations.

**Independent Test**: One fault is injected at every adapter operation, earlier evidence survives, and every safe independent cleanup action is attempted once.

- [x] T017 [US2] Add a controlled-adapter call ledger and stage fault injection to `crates/fragcap/tests/deep_capture_session.rs`.
- [x] T018 [US2] Cover defaults, caps, propagation, late success, total budgets, and cooperative adapter obligations.
- [x] T019 [US2] Add resource ownership tests proving explicit release plus `Drop` cannot duplicate stop or cleanup effects.
- [x] T020 [US2] Put the production external proxy backend and observation ingestion behind the public proxy boundary.
- [x] T021 [US2] Put current-user trust acquisition and rollback behind the public trust boundary, preserving reachability's no-trust path.
- [x] T022 [US2] Put managed launch and ordinary Capture behind public adapters; controlled launch receives proxy variables on its child command only.
- [x] T023 [US2] Put target resolution, compatibility prerequisites, and append-only fact persistence behind public boundaries.
- [x] T024 [US2] Add system clock, identifier, loopback endpoint, filesystem artifact, and event forwarding production bridges.

## Phase 5: User Story 3 - Truthful Partial Results

**Goal**: Every interrupted or failed run returns one authoritative report consistent with observed evidence, fact attempts, cleanup, artifacts, and event delivery.

**Independent Test**: A multi-failure run retains every chronological failure and observation, calls all applicable cleanup once, and cannot report complete.

- [x] T025 [US3] Retain pure policy coverage for routing, propagation, owner, protocol, inspectability, trust, silence, conflicting observations, and missing correlation anchors.
- [x] T026 [US3] Move observation classification and evidence-backed fact selection into `crates/fragcap/src/deep_capture/policy.rs` without changing S097 semantics.
- [x] T027 [US3] Add interruption, fact, bundle, cleanup, event-delivery, startup, launch, trust, and Capture failure tests.
- [x] T028 [US3] Implement chronological failure accumulation, independent fact attempts, and complete per-resource cleanup accounting in the coordinator.
- [x] T029 [US3] Implement immutable post-cleanup terminal snapshot delivery to the artifact boundary with independent artifact results.
- [x] T030 [US3] Implement typed ordered event delivery, delivery-gap accounting, and authoritative terminal reports when terminal delivery itself fails.
- [x] T031 [US3] Verify no missing evidence becomes an affirmative fact and no cleanup or artifact uncertainty can produce `Complete`.

## Phase 6: User Story 4 - Thin Compatible CLI

**Goal**: Existing command behavior remains compatible while CLI code owns no Deep Capture business rules.

**Independent Test**: Equivalent controlled API and CLI scenarios normalize to the same events, artifacts, facts, cleanup, outcome, and exit class.

- [x] T032 [US4] Replace CLI lifecycle orchestration with argument-to-config mapping, plan presentation, exact-plan confirmation, event presentation, exit mapping, and effect bridges.
- [x] T033 [US4] Render library lifecycle events through the established CLI event types while preserving JSON names and fields.
- [x] T034 [US4] Run direct API coverage beside the retained parser, refusal, calibration, bundle, and cleanup integration cases.
- [x] T035 [US4] Audit CLI Deep Capture sources to confirm classification, ordering, fact selection, snapshot authority, and cleanup ordering exist in the facade.

## Phase 7: Documentation and Release Record

- [x] T036 Update `docs/fragcap-specification.md` and `docs/fragcap-spec-outline.md` with the public API, adapter, state, deadline, snapshot, resource, and CLI boundaries.
- [x] T037 Update the relevant architecture and compatibility site references; no existing glossary or output-format term changed.
- [x] T038 Add issue-linked added and decisions fragments under `changelog.d/`, recording facade placement, plan-bound authorization, ordinary Capture reuse, and post-cleanup snapshot authority.

## Phase 8: Verification and Local Commit

- [x] T039 Run focused facade, CLI, feature-matrix, documentation, and controlled offline tests, fixing failures without weakening assertions.
- [x] T040 Run `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all --locked`, and `cargo xtask ci` in the foreground.
- [x] T041 Audit changed files for UTF-8 without BOM, LF endings, trailing whitespace, mojibake, disallowed dash characters, private local evidence, prohibited capabilities, unintended dependencies, and CLI-owned business rules.
- [x] T042 Review the final diff against every S098 requirement and checklist item, create one local feature commit, and halt before push.

## Dependencies and Execution Order

- Phase 1 establishes the compatibility baseline and feature surface.
- Phase 2 blocks all lifecycle work because the coordinator and CLI need public models and ordinary Capture reuse.
- User Stories 1 and 2 establish the executable API and effect boundaries.
- User Story 3 depends on the coordinator and adapters and supplies final truth guarantees.
- User Story 4 depends on all library policy being complete before the command monolith is removed.
- Documentation, full verification, and local commit follow implementation and parity coverage.

## Implementation Strategy

Use test-driven vertical extraction. First establish facade types and ordinary Capture reuse, then make a controlled direct-library success path work. Add one adapter and failure boundary at a time, retaining the old CLI contract until the facade can reproduce it. Switch the CLI only after direct policy, resource, event, fact, and bundle tests are green. No task authorizes a push.
