# Tasks: Attribution Session-to-Pipeline Integration

**Slice**: 015 (S13 follow-up; issues #18, #19) | **Branch**: `feat/attribution-pipeline-integration`

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md)

Tests are required (the spec's acceptance is tier-1 tests). TDD order: the
failing test lands with or just before the change that makes it pass. Every task
names its file. `[P]` marks tasks that touch different files with no incomplete
dependency.

## Phase 1: Setup

- [ ] T001 Confirm the branch and clean tree: `git status -sb` on `feat/attribution-pipeline-integration`; confirm `cargo xtask ci` is green at baseline before any change.

## Phase 2: Foundational (blocking prerequisites for all stories)

- [ ] T002 Add the `OwnedEndpoint` value type in `crates/fragcap-core/src/flow.rs` (`{ endpoint: Endpoint, owner: Option<u32> }`, derives `Clone, Copy, Debug, PartialEq, Eq`, helper `unowned(endpoint)`), with a unit test. Re-export via `flow` module if the module pattern requires it.
- [ ] T003 Change the `FlowAttributor` trait in `crates/fragcap-core/src/traits.rs`: `refresh(&self)`; add `fn wants_refresh(&self) -> bool { false }`; add `fn active_endpoints_owned(&self) -> Vec<OwnedEndpoint>` with the default that maps `active_endpoints()` to owner `None`. Update the trait doc to reference the section 29 deviation.
- [ ] T004 Update the in-file `StubAttributor` in `crates/fragcap-core/src/traits.rs` to `refresh(&self)`; confirm the dyn-compatibility, `Send`, and `Sync` compile-time assertion tests still pass.

**Checkpoint**: `cargo build -p fragcap-core` compiles; trait assertions green.

## Phase 3: User Story 1 - A connection opened mid-run becomes attributable (P1)

**Goal**: The pipeline control thread drives `FlowAttributor::refresh`, so a
connection absent at start becomes resolvable after a refresh (SC-001).

**Independent test**: a test attributor whose `refresh(&self)` flips its published
answer, run through `Pipeline`; unresolvable before, resolvable after.

- [ ] T005 [US1] Give `SocketTableAttributor` interior mutability in `crates/fragcap-attr/src/socket.rs`: move `source`, `namer`, `retained` into `Mutex<RefreshState>`; implement `refresh(&self)` locking it and publishing (logic unchanged); implement trait `wants_refresh(&self)` (promote the inherent method). Keep the existing socket.rs tests passing (adapt `&mut` call sites to `&self`). NOTE: the `active_endpoints_owned` override on `SocketTableAttributor` is deferred to T011a (US2) because it depends on the index `endpoints_owned` method (T011); until then `SocketTableAttributor` uses the trait default.
- [ ] T006 [P] [US1] Update `PublishedResolver::refresh` to `&self` (no-op) in `crates/fragcap-attr/src/resolver.rs` and soften the module doc to note the read/write split is now optional.
- [ ] T007 [P] [US1] Update `ScriptedAttributor::refresh` to `&self` (no-op) in `crates/fragcap-attr/src/scripted.rs`.
- [ ] T008 [US1] In `crates/fragcap-core/src/pipeline/mod.rs`, add to the control-thread loop (before `active_endpoints()`): `if attributor.wants_refresh() { let _ = attributor.refresh(); }`. Update the `StubAttributor` and `PanicOnEndpoints` test doubles in this file to `refresh(&self)`.
- [ ] T009 [US1] Add the tier-1 pipeline test in `crates/fragcap-core/src/pipeline/mod.rs` tests (or a pipeline integration test): a test attributor resolving `None` until `refresh` flips its answer, with `wants_refresh` returning true; drive it through `Pipeline::run` over a stub source and assert the flow becomes resolvable after the control thread's refresh, and that `wants_refresh() == false` suppresses refresh.
- [ ] T010 [US1] Adapt the several-threads concurrency test in `crates/fragcap-attr/src/socket.rs` (and/or `index.rs`) so `refresh(&self)` is driven on one thread through a shared `Arc<dyn FlowAttributor>` while others resolve, proving the resolve path is not blocked (SC-003).

**Checkpoint**: `cargo test -p fragcap-core -p fragcap-attr` green; SC-001 and SC-003 demonstrated.

## Phase 4: User Story 2 - The kernel filter narrows to the target's sockets only (P1)

**Goal**: Narrowing is restricted to endpoints owned by profiled processes (SC-002).

**Independent test**: a `RoleStampingAttributor` over an inner returning owned
endpoints for a profiled and an unprofiled PID, snapshot naming only the
profiled; `active_endpoints()` returns only profiled.

- [ ] T011 [US2] Add `AttributionIndex::endpoints_owned(&self, at) -> Vec<OwnedEndpoint>` in `crates/fragcap-attr/src/index.rs` (mirror `endpoints`, carry the PID), with a unit test covering table + retained + wildcard UDP bind.
- [ ] T011a [US2] Override `active_endpoints_owned(&self)` on `SocketTableAttributor` in `crates/fragcap-attr/src/socket.rs` to return `self.published.load().endpoints_owned(self.clock.now())` (depends on T011). Deferred here from T005 so the index method exists first.
- [ ] T012 [US2] Override `RoleStampingAttributor::active_endpoints()` in `crates/fragcap/src/session.rs` to filter `inner.active_endpoints_owned()` to endpoints whose owner is a key in the `BindingPublisher` snapshot; forward `refresh(&self)` and `wants_refresh` to inner; update the `Fixed` test double to `refresh(&self)`.
- [ ] T013 [US2] Add the tier-1 facade test in `crates/fragcap/src/session.rs` stamping tests: profiled vs unprofiled endpoint split (IPv4, IPv6, wildcard UDP) asserts `active_endpoints()` admits only profiled; empty snapshot yields empty (bootstrap retained).

**Checkpoint**: `cargo test -p fragcap` green; SC-002 demonstrated.

## Phase 5: User Story 3 - Every implementor moves to the new signature cleanly (P2)

**Goal**: The trait deviation is complete and recorded (SC-004).

- [ ] T014 [US3] Grep the workspace for any remaining `refresh(&mut self)` / `.refresh()` call sites on a `FlowAttributor` and confirm all seven implementors/doubles are `&self`: `SocketTableAttributor`, `PublishedResolver`, `ScriptedAttributor`, `RoleStampingAttributor`, `StubAttributor` (x2), `Fixed`, `PanicOnEndpoints`. `cargo build --workspace` compiles.
- [ ] T015 [US3] Add the dated section-29 deviation decision fragment `changelog.d/S015-attribution-pipeline-integration.decisions.md` recording the `refresh(&self)` signature change and the two added trait methods.

## Phase 6: CLI wiring (cfg-gated; #19 collapse)

- [ ] T016 In `crates/fragcap-cli/src/assemble.rs` (`#[cfg(all(feature = "socket-table", windows))]`): remove `RefreshDriver`, `REFRESH_POLL_INTERVAL`, and the `refresh_driver` field; change `live_components` to wrap the real `SocketTableAttributor` via `RoleStampingAttributor::new(Arc::new(attributor))` instead of `attributor.resolver()`.
- [ ] T017 In `crates/fragcap-cli/src/orchestrator.rs`: remove the `refresh_driver` stop call; change the "filter narrowed to N endpoint(s)" message (offline and live) to report the profiled (stamper-filtered) endpoint count, not the unfiltered inner set (FR-007).
- [ ] T018 Build the gated path: `cargo build -p fragcap-cli --features socket-table` compiles; record that tier-2 stays unexecuted.

## Phase 7: Docs and spec promotion

- [ ] T019 [P] Add glossary entries in `docs/glossary.md` for `OwnedEndpoint` and "profiled endpoint set" (P-6).
- [ ] T020 [P] Update `docs/plans/README.md`: note that `015-attribution-pipeline-integration` is an S13 follow-up (not roadmap S15), and the roadmap S15-S18 map to directory ordinals 018-021.
- [x] T021 [P] Record the two resolutions and the `refresh(&self)` deviation for promotion to the specification (sections 11, 12.2, 29) in the changelog decisions fragment. DECISION: do NOT edit `docs/fragcap-specification.md` in this slice. The established pattern (the S09 `Send` and S10 `Sync` deviations are recorded in their changelog decisions and are still not reflected in the spec's section 8.5 trait listing) promotes deviations to the spec at the next version, not per-slice; editing the shared architecture-of-record doc on a feature branch would conflict with concurrent slices exactly as editing `CHANGELOG.md` would. Promotion happens at release D.
- [ ] T022 [P] Add the `changelog.d/S015-attribution-pipeline-integration.added.md` feature fragment.

## Phase 8: Verification (foreground, watched)

- [ ] T023 Run `cargo xtask ci` in the foreground to completion; all green.
- [ ] T024 Run `cargo xtask neutral` and `cargo xtask msrv`; both exit 0 (or 2 = could-not-run, reported as such).
- [ ] T025 Confirm offline goldens byte-identical: `cargo test -p fragcap --test goldens` and `cargo test -p fragcap-cli --test cli_run` pass with no regeneration.
- [ ] T026 Stage only this slice's files (never `.specify/feature.json`); do NOT push. Prepare the pre-push halt breakdown.

## Dependencies

- Phase 2 (T002-T004) blocks everything (the type and trait).
- US1 (T005-T010) and US2 (T011-T013) both depend on Phase 2; US2's T011 depends on T002. US1 and US2 are otherwise independent and could interleave, but touch shared files (`socket.rs`, `index.rs`) so run US1 then US2.
- US3 (T014-T015) depends on US1 + US2 (all implementors moved).
- Phase 6 (CLI) depends on the trait + facade being complete.
- Phases 7-8 last.

## Parallel opportunities

- T006, T007 are `[P]` (different files, both trivial no-op signature changes).
- T019-T022 docs are `[P]` (different files).

## MVP scope

User Story 1 (the driven refresh) is the MVP: it is the capability that makes live
attribution function at all. US2 (narrowing) is the refinement that makes the
filter do its job; US3 is the mechanical completeness of the deviation.
