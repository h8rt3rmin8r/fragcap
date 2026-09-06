# Tasks: Stable Public Rust API

**Input**: Design documents from `/specs/132-public-rust-api/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/public-api-v1.md

**Tests**: Required by FR-010 through FR-012 and the autopilot TDD protocol.

**Organization**: Tasks are grouped by independently testable user story and executed in dependency order.

## Phase 1: Setup and Contract Baseline

**Purpose**: Freeze the intended version 1 boundary before implementation.

- [x] T001 Record the exact existing CLI capability imports and facade public declarations in `specs/132-public-rust-api/research.md` and `specs/132-public-rust-api/contracts/public-api-v1.md`
- [x] T002 Add failing stable inventory and external-consumer construction tests in `crates/fragcap/tests/public_api.rs`
- [x] T003 Add failing cancellation boundary tests in `crates/fragcap/tests/public_api.rs` and `crates/fragcap/tests/deep_capture_session.rs`
- [x] T004 Add the initially failing no-CLI native example target in `crates/fragcap/examples/native-deep-capture.rs`

## Phase 2: User Story 1 - Reach Every Capability Through the Facade (Priority: P1)

**Goal**: Publish one curated facade surface and prove the shipped command consumes it.

**Independent Test**: Compile the external consumer and the CLI using only `fragcap::deep_capture::api`, then run the production native controlled example.

- [x] T005 [US1] Add the versioned curated export inventory and module documentation in `crates/fragcap/src/deep_capture/api.rs`
- [x] T006 [US1] Expose the curated module while preserving legacy compatibility re-exports in `crates/fragcap/src/deep_capture/mod.rs`
- [x] T007 [US1] Migrate Deep Capture product-policy imports to the canonical facade path in `crates/fragcap-cli/src/commands/deep_capture.rs`
- [x] T008 [US1] Complete the external-consumer and zero-CLI-bypass coverage assertions in `crates/fragcap/tests/public_api.rs`
- [x] T009 [US1] Complete and execute the bounded production native controlled example in `crates/fragcap/examples/native-deep-capture.rs`

## Phase 3: User Story 2 - Evolvable Construction, Ownership, and Cancellation (Priority: P1)

**Goal**: Make compatible input evolution and lifecycle cancellation explicit and testable.

**Independent Test**: Construct session and adapter inputs without field literals, request cancellation before effects and between stages, and verify no later non-cleanup effect plus complete terminal cleanup.

- [x] T010 [US2] Add `SessionConfigBuilder`, defaults, validation, and typed construction errors in `crates/fragcap/src/deep_capture/model.rs`
- [x] T011 [US2] Add `AdapterSetBuilder`, required-capability validation, and hidden production boundary-controller installation in `crates/fragcap/src/deep_capture/adapters.rs`
- [x] T012 [US2] Add the cloneable `CancellationToken` and explicit construction path in `crates/fragcap/src/deep_capture/model.rs` and `crates/fragcap/src/deep_capture/session.rs`
- [x] T013 [US2] Add coordinator cancellation checkpoints, interruption truth, and cleanup preservation in `crates/fragcap/src/deep_capture/session.rs`
- [x] T014 [US2] Finish builder, ownership, type-property, non-exhaustive, and cancellation contracts in `crates/fragcap/src/deep_capture/api.rs`, `crates/fragcap/tests/public_api.rs`, and `crates/fragcap/tests/deep_capture_session.rs`

## Phase 4: User Story 3 - Compatibility Policy and Backend Encapsulation (Priority: P2)

**Goal**: Record the stable promise and keep incidental implementation types outside it.

**Independent Test**: Compare the exact stable inventory, compile contract, crate documentation, and CLI coverage while implementation-only types remain inaccessible through the canonical path.

- [x] T015 [US3] Add canonical API-version, semver, feature, non-exhaustive, concurrency, cancellation, ownership, and legacy-export guidance in `crates/fragcap/README.md` and `crates/fragcap/src/deep_capture/api.rs`
- [x] T016 [US3] Add exact inventory drift assertions in `crates/fragcap/tests/public_api.rs` and record the enum evolution contract in `specs/132-public-rust-api/contracts/public-api-v1.md`
- [x] T017 [US3] Synchronize the API boundary and S132 completion record in `docs/fragcap-specification.md`, `docs/fragcap-spec-outline.md`, and `docs/plans/README.md`
- [x] T018 [US3] Add user-visible and dated architecture fragments in `changelog.d/s132-public-rust-api.feature.md` and `changelog.d/s132-public-rust-api.decisions.md`
- [x] T019 [US3] Correct merged S131 status from Draft to Complete in `specs/131-native-packaging/spec.md`

## Phase 5: Convergence and Verification

**Purpose**: Prove the full slice, release safety, and repository hygiene.

- [x] T020 Run the non-destructive spec, plan, and task analyze gate and resolve every finding in `specs/132-public-rust-api/`
- [x] T021 Run focused public API, Deep Capture session, CLI, documentation example, and native controlled example checks from `specs/132-public-rust-api/quickstart.md`
- [x] T022 Run formatting, all-target Clippy, and every locally supported `cargo xtask ci` component; require the Ubuntu-only wrapper gate in pull-request CI
- [x] T023 Audit `git diff --check`, UTF-8 without BOM, LF endings, punctuation, Markdown wrapping, mojibake, dependency lock stability, and staged file scope
- [x] T024 Mark every completed task and set `specs/132-public-rust-api/spec.md` status to Complete after all gates pass

## Dependencies and Execution Order

- Phase 1 establishes failing contracts before implementation.
- Phase 2 depends on Phase 1 and creates the canonical reachability boundary.
- Phase 3 depends on the canonical module and adds construction plus cancellation.
- Phase 4 depends on the final API shape so documentation and inventories cannot drift.
- Phase 5 depends on all user stories and blocks commit or push on any red result.

## Parallel Opportunities

- Documentation research and failing example scaffolding touch separate files after T001.
- Builder implementation in `model.rs` and API-module documentation in `api.rs` can proceed independently after T005.
- Master specification, crate README, and changelog fragments touch separate files after the API shape settles.

## Implementation Strategy

The minimum viable increment is User Story 1: one curated facade path consumed by the CLI and a no-CLI native example. User Story 2 supplies the compatibility-safe construction and cancellation guarantees required to call that path stable. User Story 3 freezes the promise without freezing backend internals. All three are required to close issue #330.
