# Tasks: Filter Manager Install Acknowledgement

**Slice**: 016 (S13 follow-up; issue #20) | **Branch**: `feat/filter-install-ack`

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md)

Tests required (tier-1). Every task names its file.

## Phase 1: Setup

- [ ] T001 Confirm branch `feat/filter-install-ack` and a clean baseline (`cargo xtask ci` green on the merged main before changes).

## Phase 2: FilterManager (US1 + US2)

- [ ] T002 [US2] In `crates/fragcap-core/src/filter.rs`, add `pending: Option<BTreeSet<Endpoint>>` to `HandleState` (init `None` in `new`).
- [ ] T003 [US2] Change `FilterManager::poll`: remove the optimistic `installed = Narrowed(wanted)` and `gapped.clear()`; keep `last_install = Some(now)` at issue; add "skip if `pending.is_some()`" after the idempotence check; on issue set `pending = Some(wanted.clone())`. The gap-accounting loop is unchanged.
- [ ] T004 [US1] Add `FilterManager::acknowledge(&mut self, handle: usize, installed_ok: bool)`: take `pending` (return if none); on success set `installed = Narrowed(pending)` and `gapped.clear()`; on failure leave `installed`/`last_install`/`gapped` unchanged.
- [ ] T005 [US1] Make `FilterManager::retire` also clear `pending`.
- [ ] T006 [US2] Update the existing `FilterManager` unit tests in `filter.rs` to call `acknowledge(handle, true)` after each successful install (debounce, rate-limit, idempotence, empty-set, gap, multi-handle, retired tests). Assertions otherwise unchanged.
- [ ] T007 [US1] Add `filter.rs` tests: (a) a rejection acknowledgement leaves the handle not installed and a later poll (after the interval) re-issues the same program; (b) a persistently rejecting handle retries at one attempt per `min_reinstall_interval`, not once per poll; (c) a success acknowledgement commits and preserves idempotence and the cleared gap set.

**Checkpoint**: `cargo test -p fragcap-core filter` green.

## Phase 3: Pipeline wiring (US3)

- [ ] T008 [US3] In `crates/fragcap-core/src/pipeline/mod.rs` `Pipeline::run`, create `let (ack_tx, ack_rx) = mpsc::channel::<(usize, bool)>();`. In the capture-thread spawn loop, `.enumerate()` for the handle index, clone `ack_tx`, and pass the index and the sender clone into `acquire`.
- [ ] T009 [US3] Change `acquire`'s signature to take the handle index and `&Sender<(usize, bool)>`; after `source.set_filter(&program)`, send `(handle, result.is_ok())`. Update the doc comment (the reinstall is still non-fatal; the result is now reported so the manager can retry).
- [ ] T010 [US3] In the control-thread loop, before `poll`, drain the ack channel: `while let Ok((handle, ok)) = ack_rx.try_recv() { manager.acknowledge(handle, ok); }`. Keep the existing `retire`-on-send-failure behavior.
- [ ] T011 [US3] Update the in-file pipeline test doubles/callers of `acquire` for the new signature (`StubSource`-based tests, `RecordingSource`, the refresh test, the panic test).
- [ ] T012 [US3] Add a pipeline test: a source double that rejects maintenance `set_filter` (returns `Err` for a non-bootstrap program) records more than one attempt of the same program, proving the control thread retried, and the run ends on its own.

**Checkpoint**: `cargo test -p fragcap-core` green.

## Phase 4: Docs and verification

- [ ] T013 [P] Add `changelog.d/S016-filter-install-ack.added.md` (feature fragment). Add a glossary entry in `docs/glossary.md` for "Install acknowledgement" / "Pending install" if these are treated as new terms (P-6).
- [ ] T014 Run `cargo xtask ci` in the foreground to completion; all green.
- [ ] T015 Run `cargo xtask neutral` and `cargo xtask msrv`; both exit 0 (or 2 = could-not-run, reported).
- [ ] T016 Confirm corpus goldens byte-identical: `cargo test -p fragcap --test goldens`, `cargo test -p fragcap-cli --test cli_run`, `cargo test -p fragcap corpus_pipeline`.
- [ ] T017 Stage only this slice's files (never `.specify/feature.json`); do NOT push. Prepare the pre-push halt breakdown.

## Dependencies

- Phase 2 (T002-T007) is the core; T003 depends on T002; T004/T005 independent of T003 but same file (sequential). T006/T007 after T002-T005.
- Phase 3 depends on Phase 2 (the `acknowledge` method must exist).
- Phase 4 last.

## MVP scope

User Story 1 (rejection not treated as installed, retried) is the fix; US2
(success committed, existing behavior preserved) and US3 (pipeline wiring) make it
real end to end.
