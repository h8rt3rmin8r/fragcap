---

description: "Task list for S028 Watch / Attach Mode (launch-agnostic capture)"
---

# Tasks: Watch / Attach Mode (Launch-Agnostic Capture)

**Input**: Design documents from `specs/028-watch-attach-mode/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Included (TDD; the attach, wait, and give-up paths are required tests).

## Format: `[ID] [P?] [Story] Description`

## Path Conventions

Rust workspace. CLI in `crates/fragcap-cli/src/`; session in
`crates/fragcap/src/`; docs in `docs/`.

---

## Phase 1: Setup

- [ ] T001 Confirm the build baseline: `cargo build --workspace` green on main at
  eb7a210 (S027 merged), so the S027 resolver types are available to the CLI.

---

## Phase 2: Foundational (plumbing every story needs)

- [ ] T002 Add `CaptureSession::apply_snapshot(&mut self, records:
  &[ProcessRecord], at: Timestamp)` in `crates/fragcap/src/session.rs`: fold the
  snapshot via `ProcessTree::apply_snapshot_at`, run the same matching
  `on_process_event` runs, and transition `Watching -> Capturing` when a
  non-service stage binds. Re-export `ProcessRecord` from `crates/fragcap/src/lib.rs`
  if the CLI needs it by name.
- [ ] T003 [P] Unit test in `session.rs`: applying a snapshot containing a process
  that matches a one-stage identity acquires the target (session reaches
  `Capturing`) with no streamed event; a snapshot with no match leaves it
  `Watching`.
- [ ] T004 Add `startup_snapshot: Vec<ProcessRecord>` and `snapshot_at:
  Option<Timestamp>` to `CaptureComponents` in
  `crates/fragcap-cli/src/assemble.rs`; fill them in the offline builder from
  `ScriptedWatcher::snapshot` and in the live builder from `EtwWatcher::snapshot`
  + `snapshot_taken_at`; default empty elsewhere (extcap).
- [ ] T005 In `crates/fragcap-cli/src/orchestrator.rs`, apply the snapshot at arm
  in both `capture_prerecorded` and `capture_live` before the acquisition loop
  (`session.apply_snapshot(&components.startup_snapshot, snapshot_at | ARMED_AT)`),
  so an already-running match acquires at arm; the loops are otherwise unchanged.
- [ ] T006 Add `WatchArgs` and `Command::Watch(WatchArgs)` in
  `crates/fragcap-cli/src/cli.rs` (`--exe` required, `--path`, `--path-regex`,
  `--wait`, `--duration`, `--out`, `--sink`, `--no-payload`, flattened
  `OfflineArgs`); dispatch `Command::Watch => commands::watch::run` in
  `crates/fragcap-cli/src/lib.rs`.
- [ ] T007 Add `effective_config_for_watch(args, profile)` in `assemble.rs`
  mapping `--wait` to `acquisition_timeout` and the sinks/bounds like
  `effective_config_for_tap`.

**Checkpoint**: the session, components, orchestrator, and command skeleton
compile; no behavior yet beyond a synthesized profile capturing.

---

## Phase 3: User Story 1 -- Capture by identity, launch-agnostic (P1)

- [ ] T008 [US1] Implement `crates/fragcap-cli/src/commands/watch.rs`: synthesize
  a one-stage identity profile (authored) from `--exe` + optional path anchors via
  `Profile::parse` (validated construction), build the config, and hand it to
  `orchestrator::capture` (no roles restriction, sink failure not clean), mirroring
  `tap`.
- [ ] T009 [P] [US1] Offline test (a `commands`/integration test): a modded-Skyrim
  shape (a process under a path outside `steamapps`, arbitrary parent, no
  `steam://`) is acquired and captured by `watch --exe ... --path ...`; the same
  timeline with a non-matching path anchor does not acquire (FR-002, SC-001,
  SC-006).
- [ ] T010 [P] [US1] Test that `watch` output is byte-identical to an equivalent
  single-stage profile capture over the same timeline (FR-007, SC-004).
- [ ] T011 [P] [US1] Test that an empty identity (no predicate) and a
  non-compiling `--path-regex` are refused at construction (exit 2) with the
  profile's diagnostics (FR-008, SC-005).

---

## Phase 4: User Story 2 -- Attach to an already-running game (P2)

- [ ] T012 [US2] Wire the S027 ObservationProvider in `watch.rs`: at arm, build a
  `ProcessTree` from the startup snapshot, resolve the identity via
  `TargetResolver` (`ResolutionRequest::for_observation`), and report the observed
  attach naming the already-running process when it resolves; the session's
  `apply_snapshot` (Phase 2) performs the acquisition.
- [ ] T013 [P] [US2] Offline test using `ProcessScript::with_snapshot`: a process
  matching the identity present at arm with no later start event is acquired at
  arm and captured (FR-003, SC-002).
- [ ] T014 [P] [US2] Test that the ObservationProvider resolves the identity over
  the snapshot to an `observed` target naming the process, distinct from the
  authored identity (FR-006, US2 scenario 2), and that attach + wait compose (an
  already-running target and a later-started one in one run).

---

## Phase 5: User Story 3 -- Give up loudly (P3)

- [ ] T015 [US3] Confirm `--wait` drives `acquisition_timeout` on the `watch`
  surface (from Phase 2 config); no new counter, reuse
  `StopReason::AcquisitionTimeout`.
- [ ] T016 [P] [US3] Offline test: `watch --wait` with a timeline that never
  matches ends `StopReason::AcquisitionTimeout`, surfaces the watch-time discard
  accounting, and exits 1 (FR-005, SC-003).
- [ ] T017 [P] [US3] Offline test: an operator interrupt (`--fire-interrupt`)
  during the watch exits 0, a clean cancellation, not a failure (FR-005, SC-003).

---

## Phase 6: Polish and cross-cutting

- [ ] T018 [P] Extend master spec sections 7.1 and 10.5 in
  `docs/fragcap-specification.md` to name watch mode as the default
  launch-agnostic capture path.
- [ ] T019 [P] Add a `watch mode` glossary entry in
  `docs/glossary/process-and-attribution.md` (near Process watcher and Acquisition
  timeout), referencing `launcher_mediated` (issue #78/#83); keep the index
  reproducible.
- [ ] T020 Add a changelog feature fragment
  `changelog.d/028-watch-attach-mode.added.md` and a decisions fragment
  `changelog.d/028-watch-attach-mode.decisions.md` recording the surface decision,
  the attach-to-running mechanism (session is the single acquisition authority,
  ObservationProvider produces the observed answer), and the authored-vs-observed
  fidelity separation.
- [ ] T021 Run the full gate in the foreground and fix to green: `cargo xtask ci`,
  then `cargo xtask msrv` (1.82) and `cargo xtask neutral`.

---

## Dependencies and order

- Setup (T001) -> Foundational (T002-T007) -> stories.
- US1 (T008-T011) depends on the command skeleton + config (T006-T007). US2
  (T012-T014) depends on the session method + snapshot plumbing (T002-T005) and
  the command (T008). US3 (T015-T017) depends on the config (T007).
- Polish (T018-T021) last; T021 is the verification gate.

## Parallel opportunities

- T003, T009, T010, T011, T013, T014, T016, T017, T018, T019 are `[P]`.

## MVP scope

User Story 1 (capture by identity, launch-agnostic) is the MVP: it delivers the
`watch` surface and the modded-Skyrim case. US2 adds attach-to-running; US3
confirms the loud give-up.
