# Tasks: Crash-Safe Lifecycle Authority

**Input**: Design documents from `specs/109-lifecycle-authority/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Required by FR-029 and the autopilot test-first protocol.

## Phase 1: Setup

- [x] T001 Confirm the no-new-package baseline and affected dependency graph in `Cargo.toml`, `Cargo.lock`, and `crates/fragcap/Cargo.toml`
- [x] T002 [P] Add failing manifest role and compatibility fixtures for proxy and cleanup lifecycle artifacts in `crates/fragcap/tests/manifest.rs` and `crates/fragcap-cli/tests/cli_deep_capture.rs`

## Phase 2: Foundational Models

- [x] T003 Add failing route plan, authorization mismatch, unavailable strategy, and verification-state tests in `crates/fragcap/tests/deep_capture_routing.rs`
- [x] T004 [P] Add failing journal parser, transition, durability-prefix, compaction, and recovery-decision tests in `crates/fragcap/tests/deep_capture_journal.rs`
- [x] T005 [P] Add failing proxy and cleanup stream header, prefix, trailer, loss, and reconciliation tests in `crates/fragcap/tests/deep_capture_lifecycle.rs`
- [x] T006 Implement route strategy, effect declaration, verification, and immutable plan models in `crates/fragcap/src/deep_capture/routing.rs` and `model.rs`
- [x] T007 Implement resource obligation, ownership, transition, recovery, and accounting models in `crates/fragcap/src/deep_capture/journal.rs`
- [x] T008 Implement common bounded lifecycle stream records, readers, writers, and accounting in `crates/fragcap/src/deep_capture/lifecycle.rs`

## Phase 3: User Story 1 - Authorize One Exact Route (Priority: P1)

**Goal**: Make every target-scoped route explicit, immutable, verifiable, and reversible before external effects.

**Independent Test**: Prepare and authorize every modeled strategy, apply only child environment, and prove exact refusal and verification states without starting a real proxy.

- [x] T009 [US1] Add `RoutingAdapter` and `RoutingLease` public seams and wire them through `AdapterSet` in `crates/fragcap/src/deep_capture/adapters.rs`
- [x] T010 [US1] Bind `RoutingPlan` to `SessionPlan`, plan events, policy, and authorization equality in `crates/fragcap/src/deep_capture/model.rs`, `policy.rs`, and `session.rs`
- [x] T011 [US1] Implement the child-environment strategy with secret-bearing value resolution and zeroized launch material in `crates/fragcap/src/deep_capture/routing.rs`
- [x] T012 [US1] Move child route application out of launch preparation and require an applied route lease in `crates/fragcap/src/deep_capture/session.rs`, `native.rs`, and CLI adapters
- [x] T013 [US1] Implement evidence-only route verification states and cleanup results in `crates/fragcap/src/deep_capture/routing.rs` and compatibility policy tests
- [x] T014 [US1] Run the focused routing and coordinator tests and mark US1 complete in `specs/109-lifecycle-authority/tasks.md`

## Phase 4: User Story 2 - Recover Every Owned Effect (Priority: P2)

**Goal**: Persist an exact obligation before every current effect and recover owned residue safely after interruption or restart.

**Independent Test**: Interrupt every controlled transition, parse and replay the journal repeatedly, and prove exact cleanup or refusal with no unrelated mutation.

- [x] T015 [US2] Implement protected journal creation, synchronized append, crash-prefix parsing, validation, and bounded accounting in `crates/fragcap/src/deep_capture/journal.rs`
- [x] T016 [US2] Implement ownership-checked recovery planning, execution adapter seams, retry recording, and terminal compaction in `crates/fragcap/src/deep_capture/journal.rs` and `adapters.rs`
- [x] T017 [US2] Journal proxy, trust, route, launch, Capture, artifact, and cleanup transitions in coordinator order in `crates/fragcap/src/deep_capture/session.rs`
- [x] T018 [US2] Integrate current native listener, trust thumbprint, child route, temporary files, retained evidence, and cleanup ownership into journal targets in `crates/fragcap/src/deep_capture/native.rs` and CLI adapters
- [x] T019 [US2] Expose shared startup and doctor journal inspection and exact recovery entry points in `crates/fragcap/src/deep_capture/journal.rs` and `crates/fragcap-cli/src/doctor/`
- [x] T020 [US2] Add kill-boundary, repeated recovery, resource reuse, corrupt record, path containment, and interrupted recovery coverage in `crates/fragcap/tests/deep_capture_journal.rs`
- [x] T021 [US2] Add Windows protected-file, synchronization, trust ownership, and restart-replay tests under `cfg(windows)` in `crates/fragcap/tests/deep_capture_journal.rs`
- [x] T022 [US2] Run focused journal, session, doctor, and Windows-compiling tests and mark US2 complete in `specs/109-lifecycle-authority/tasks.md`

## Phase 5: User Story 3 - Audit Lifecycle Evidence as It Happens (Priority: P3)

**Goal**: Stream complete bounded proxy and cleanup chronologies with crash-readable prefixes and reconciling trailers.

**Independent Test**: Parse and reconcile successful, pressured, interrupted, and writer-failed sidecars against application, journal, manifest, summary, and terminal truth.

- [x] T023 [US3] Implement `ProxyLifecycleLease` and `CleanupLifecycleLease` append-only writers, readers, gaps, and trailers in `crates/fragcap/src/deep_capture/lifecycle.rs`
- [x] T024 [US3] Add a bounded application-event fan-out sink and lifecycle accounting without blocking forwarding in `crates/fragcap-proxy/src/application.rs` and `crates/fragcap/src/deep_capture/native.rs`
- [x] T025 [US3] Stream listener, connection, TLS, protocol, error, loss, stop, drain, and unavailable DNS/upstream evidence into `proxy.jsonl` from `crates/fragcap/src/deep_capture/native.rs`
- [x] T026 [US3] Stream obligation, attempt, retry, result, retained, and recovery links into `cleanup.jsonl` from `crates/fragcap/src/deep_capture/session.rs` and `journal.rs`
- [x] T027 [US3] Derive `cleanup.json` only from completed cleanup chronology and include its source role in `crates/fragcap-cli/src/commands/deep_capture.rs`
- [x] T028 [US3] Declare proxy chronology, cleanup chronology, cleanup summary, journal, completion, and loss truth in manifest v2 and share/doctor readers in `crates/fragcap/src/deep_capture/manifest.rs`, `artifacts.rs`, and CLI assembly
- [x] T029 [US3] Add connection/resource conservation, sidecar writer failure, missing trailer, gap, and manifest reconciliation coverage in facade and CLI integration tests
- [x] T030 [US3] Run focused lifecycle, native proxy, artifact, manifest, and CLI tests and mark US3 complete in `specs/109-lifecycle-authority/tasks.md`

## Phase 6: Boundedness, Documentation, and Verification

- [x] T031 Bound localized body-loss identities and add exact unlocalized overflow records and tests in `crates/fragcap/src/deep_capture/application.rs`
- [x] T032 Update the master specification, outline, plans, glossary, schema documentation, site output contract, and AGENTS current-state prose for S109
- [x] T033 Add S109 feature and dated architecture decision fragments under `changelog.d/`
- [x] T034 Verify issue closure mapping for #306, #320, and #336 while leaving #305, #319, and #321 open
- [x] T035 Run formatting, clippy, all locked tests, xtask CI, MSRV, dependency, platform, encoding, mojibake, and diff checks

## Dependencies and Execution Order

1. T001-T008 establish shared models and failing tests.
2. US1 completes the route contract required by the journal.
3. US2 completes durable obligations and recovery required by cleanup chronology.
4. US3 consumes route and journal truth to complete sidecars and projections.
5. T031-T035 close boundedness, documentation, issue mapping, and verification.

## Parallel Opportunities

- Routing, journal, and lifecycle failing test files can be authored independently after T001.
- Journal format implementation and lifecycle record modeling touch separate files after their models stabilize.
- Documentation can begin after contracts are stable while focused test suites run.

## Implementation Strategy

Implement in strict dependency order under test-first discipline. Each user story reaches its independent checkpoint before the next begins. The deliverable is the full slice and all three issue closures, not the route seam alone.
