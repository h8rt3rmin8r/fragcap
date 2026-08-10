# Tasks: The Session Gates Sink Writes (Watch From Arm, Hard Bounds)

**Slice**: 017 (S14 follow-up; issue #22) | **Branch**: `feat/session-gate-writes`

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md)

Tests required (tier-1). Every task names its file.

## Phase 1: Setup

- [ ] T001 Confirm branch `feat/session-gate-writes` off a clean main and a green baseline (`cargo xtask ci`).

## Phase 2: Core seam and counter (US3, FR-001..FR-004)

- [ ] T002 [US3] In `crates/fragcap-core/src/traits.rs`, add the `WriteGate` trait (`Send + Sync`, `fn admit(&self, packet: &CapturedPacket) -> bool`) with the doc comment from data-model.md. Import `CapturedPacket`.
- [ ] T003 [US3] In `crates/fragcap-core/src/stats.rs`, add `pub gate_dropped: u64` to `CaptureStats` with the by-cause doc; leave `absorb` NOT summing it (capture-wide, like `buffer_dropped`/`sink_dropped`); extend the conservation-identity discussion. Re-export nothing new here.
- [ ] T004 [US3] In `crates/fragcap-core/src/lib.rs`, re-export `WriteGate` alongside the other trait exports.
- [ ] T005 [US3] In `crates/fragcap-core/src/pipeline/mod.rs`, add `gate: Option<Arc<dyn WriteGate>>` to `Pipeline` (default `None` in `new`), add `set_write_gate(&mut self, gate: Arc<dyn WriteGate>)` (mirroring `set_filter_config`), and move the gate into `output_loop` in `run`.
- [ ] T006 [US3] Change `output_loop`'s signature to take `gate: Option<Arc<dyn WriteGate>>`; before the per-sink loop, if a gate is present and `!gate.admit(&packet)`, increment a local `gate_dropped` and `continue`; set `stats.gate_dropped = gate_dropped` alongside `stats.sink_dropped`. Update the module's `# Accounting` doc for the four-term identity.
- [ ] T007 [US3] In `pipeline/mod.rs` tests, extend the conservation-identity helper/assertions to the four-term form `received + buffer_dropped + gate_dropped + refusals == packets_captured`, so every existing pipeline test now asserts it (`gate_dropped == 0` where no gate is attached).

**Checkpoint**: `cargo test -p fragcap-core` green (no-gate runs unchanged, FR-004).

## Phase 3: The scripted-gate pipeline test (US3, SC-004)

- [ ] T008 [US3] In `pipeline/mod.rs` tests, add a `ScriptedGate` stub implementing `WriteGate` (admits a chosen set of markers/indices, counts what it rejects). Add a test that attaches it via `set_write_gate`, runs frames, and asserts the four-term identity per sink and that `gate_dropped` equals the rejected count.

**Checkpoint**: `cargo test -p fragcap-core pipeline` green.

## Phase 4: The facade SessionGate (US1, US2, FR-005..FR-007, FR-009)

- [ ] T009 [US2] In `crates/fragcap/src/session.rs`, add the `WindowState` encoding (`AtomicU8`: Watching/Capturing/Other) and the `SessionGate` struct (window `Arc<AtomicU8>`, `packet_bound`, `byte_bound`, tally atomics, `bound_hit`, and the `Sender<(u32, Timestamp)>`), plus a constructor from `SessionConfig` and the tee sender.
- [ ] T010 [US1] Implement `WriteGate for SessionGate::admit` per data-model.md: Watching -> count watch discard, reject; Other -> count out-of-window, reject; Capturing -> reject-and-count-out-of-window if at/over the bound, else admit (count, forward the receipt, set `bound_hit` if the bound is now reached). Add the driver handles (set Capturing/Other on the window; tally accessors).
- [ ] T011 [US1] In `crates/fragcap/src/lib.rs`, re-export `SessionGate` and `WriteGate` through the facade.
- [ ] T012 [US2] Add `session.rs` unit tests: (a) a Watching window admits nothing and advances the watch count (`the_gate_counts_a_watch_time_discard`, SC-003); (b) a Capturing window admits exactly `packet_bound` and rejects beyond it into out-of-window (FR-006 unit level); (c) a byte bound admits the crossing packet then rejects (FR-006, D-4); (d) the reconciliation invariant `gate_dropped-equivalent == watch + out_of_window` holds; (e) an unbounded Capturing window admits everything (pass-through, FR-011 unit level).

**Checkpoint**: `cargo test -p fragcap session` green.

## Phase 5: Orchestrator offline path (US1, US4, FR-006..FR-008, FR-011)

- [ ] T013 [US4] In `crates/fragcap-cli/src/orchestrator.rs`, remove `TeeCountingSink`. Build the `SessionGate` (from `config.session_config()` and the tee sender), keep a clone for the driver, and in `spawn_pipeline` attach it with `set_write_gate` and stop prepending a tee sink (sink list is the user sinks only).
- [ ] T014 [US4] In `capture_prerecorded`, set the gate window to `Capturing` before `spawn_pipeline` (the session is already capturing) and to `Other` when the session drains. Keep `drive` feeding `session.on_packet(len)` and `on_tick(ts)` from admitted receipts so `VolumeReached` and the duration bound still fire in the session (FR-009).
- [ ] T015 [US1] Change `build_summary` to source `retained`, `retained_bytes`, `watching_discarded`, and `discarded_out_of_window` from the gate tallies, and `packets_captured`/`packets_attributed`/`buffer_dropped`/`sink_dropped`/`gate_dropped` from the pipeline report (FR-007).

**Checkpoint**: `cargo test -p fragcap-cli --test cli_run` green with the unbounded goldens unchanged (FR-011).

## Phase 6: Orchestrator live path (US2, FR-008)

- [ ] T016 [US2] In `capture_live` (cfg `etw`, windows), spawn the pipeline at arm (before the acquisition loop) with the gate window `Watching`; set the window `Capturing` when a stage acquires the target and `Other` on drain; the acquisition loop now runs while the pipeline reads. Verify with `cargo check -p fragcap-cli --features live,socket-table,etw`.

## Phase 7: Completion summary (US1, US3)

- [ ] T017 [US1] In `crates/fragcap-cli/src/output.rs`, keep `CompletionSummary`'s fragcap-drops line at `buffer_dropped + sink_dropped`; the watch-time and out-of-window lines are the gate's discards (no double count). Update the summary unit tests only if a provenance assertion needs it.

## Phase 8: CLI bound tests (US1, SC-001, SC-002, FR-006)

- [ ] T018 [US1] In `crates/fragcap-cli/tests/cli_run.rs`, replace `each_bound_stops_for_its_named_reason`'s packet/byte cases with `a_packet_bound_produces_an_exactly_bounded_file`: run `--max-packets N` to real output files, count packet records in the produced pcapng and JSON Lines (each exactly N), assert the summary reports N retained and zero out of window and stop reason `volume-reached`. Keep a `--duration` stop-reason case.
- [ ] T019 [US1] Add `a_byte_bound_produces_an_exactly_bounded_file`: `--max-bytes B` yields the first prefix of packets whose cumulative captured length reaches or exceeds B, with the summary's retained bytes equal to the bytes on disk (FR-006).

**Checkpoint**: `cargo test -p fragcap-cli` green.

## Phase 9: Docs and verification

- [ ] T020 [P] Add `changelog.d/S017-session-gate-writes.added.md` (feature fragment) and `changelog.d/S017-session-gate-writes.decisions.md` recording that this reverses D-c/D-e for the watch-time and bound cases and keeps them for the offline unbounded case. Add a glossary entry for "Write gate" / "Capture window" in `docs/glossary.md` if treated as new terms (P-6).
- [ ] T021 Run `cargo xtask ci` in the foreground to completion; all green. Scan new `.md`/`.rs` for em/en dashes first (`perl -ne 'print if /[\x{2014}\x{2013}]/'`).
- [ ] T022 Run `cargo xtask neutral` and `cargo xtask msrv`; both exit 0 (or 2 = could-not-run, reported). Run `cargo check -p fragcap-cli --features live,socket-table,etw`.
- [ ] T023 Confirm goldens byte-identical: `cargo test -p fragcap-cli --test cli_run` (unbounded), `cargo test -p fragcap --test corpus_pipeline`, `cargo test -p fragcap --test goldens`.
- [ ] T024 Stage only this slice's files (never `.specify/feature.json`); do NOT push. Prepare the pre-push halt breakdown.

## Dependencies

- Phase 2 is the core seam; T005/T006 depend on T002; T007 after T006.
- Phase 3 (T008) depends on Phase 2.
- Phase 4 (SessionGate) depends on the `WriteGate` trait (T002) and re-export (T004).
- Phase 5 depends on Phase 4 (the gate must exist to attach).
- Phase 6 (live) depends on Phase 5 (the offline wiring is the template).
- Phase 8 depends on Phase 5 (the offline gate must bound the file).
- Phase 9 last.

## Parallel example

- T020 (changelog/glossary) is `[P]` and can be written alongside the Phase 8 tests;
  everything else is sequential by file or by dependency.
