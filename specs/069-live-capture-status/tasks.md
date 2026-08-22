# Tasks: Live capture status display

**Input**: Design documents from `/specs/069-live-capture-status/`
**Prerequisites**: plan.md, research.md, data-model.md, `contracts/status-block.md`,
`contracts/heartbeat-line.md`, `contracts/capture-progress-event.md`, quickstart.md

Tests are written first per this project's TDD convention; each
implementation task follows its test task. Every new module and struct
introduced here is designed to be a pure function over plain data (research
R-5), so its tests run on every CI platform, not only Windows.

## Phase 1: Setup

- [X] T001 Re-read `crates/fragcap-cli/src/orchestrator.rs` (lines 106-450,
  500-1000, the two drivers, `spawn_pipeline`, `FilterNarration`),
  `crates/fragcap-core/src/pipeline/mod.rs` (lines 340-540, 925-1075, the
  `Pipeline` struct, `run`, `output_loop`), `crates/fragcap-core/src/pipeline/buffer.rs`
  (the whole file), `crates/fragcap/src/session.rs` (lines 840-1000, `GateShared`/`GateHandle`),
  `crates/fragcap-cli/src/color.rs`, `crates/fragcap-cli/src/emit.rs`,
  `crates/fragcap-cli/src/events.rs`, and `crates/fragcap-cli/src/output.rs`
  to confirm no signature has drifted since `research.md`/`data-model.md`
  were written.

## Phase 2: Foundational (blocking prerequisites)

**Purpose**: The live-readable pipeline handle and the stream-correct color
predicate both User Story 1 (the status block) and User Story 3 (the
`capture.progress` event, and non-interference generally) depend on.

- [X] T002 [P] Add a failing test in `crates/fragcap-core/src/pipeline/buffer.rs`'s
  test module asserting `Consumer::next_and_evicted()` returns the same item
  `next()` would have, and that the evicted count it returns matches
  `Consumer::evicted()` read separately, across a push-evict-pop sequence.
- [X] T003 [P] Implement `Consumer::next_and_evicted(&self) -> (Option<Item>, u64)`
  in `crates/fragcap-core/src/pipeline/buffer.rs`, beside the existing,
  unmodified `next()`: the same lock acquisition and pop, additionally
  reading `shared.evicted` before releasing the lock, per research R-2.
  Making T002 pass; run `cargo test -p fragcap-core buffer` to confirm the 7
  existing `next()`-based tests are unaffected.
- [X] T004 [P] Add a failing test in a new `crates/fragcap-core/src/pipeline/live_stats.rs`
  asserting: a fresh `LiveStats::new()` reports zero for every counter and an
  empty holder-tally snapshot; after `sink_dropped.fetch_add(1, Relaxed)` and
  a `holder_tally.lock().unwrap().insert(...)` (simulating what `output_loop`
  will do), the same values are observable through a cloned `LiveStats`
  handle from a different "thread" (a second clone read after the first
  clone's mutation, no real thread needed for this unit test).
- [X] T005 [US-shared] Implement `LiveStats` in
  `crates/fragcap-core/src/pipeline/live_stats.rs` per `data-model.md`:
  `sink_dropped: Arc<AtomicU64>`, `holder_tally: Arc<Mutex<BTreeMap<Arc<str>, u64>>>`,
  `buffer_dropped: Arc<AtomicU64>`, a `pub(crate) fn new() -> Self`
  constructor, `Clone`, and a `pub fn holder_tally_snapshot(&self) ->
  Vec<(Arc<str>, u64)>` returning entries sorted by count descending then
  name ascending (matching `CaptureStats::dominant_holder`'s tiebreak
  discipline, per `data-model.md`'s `LiveStatusSnapshot.holder_tally` field).
  Export the type from `crates/fragcap-core/src/pipeline/mod.rs`'s public
  surface. Making T004 pass.
- [X] T006 Add a `live: LiveStats` field to `Pipeline` in
  `crates/fragcap-core/src/pipeline/mod.rs`, constructed in `Pipeline::new`;
  add `pub fn live_stats(&self) -> LiveStats` (a cheap clone); destructure
  `live` alongside the existing fields in `run(self)` and pass it into
  `output_loop`.
- [X] T007 [P] Add a failing test in `crates/fragcap-core/src/pipeline/mod.rs`'s
  test module (extend an existing pipeline test, or add one) that spawns a
  pipeline, reads `pipeline.live_stats()` before calling `run`, runs a
  synthetic capture producing at least one sink drop and at least two
  distinct holder images, and asserts the `LiveStats` handle's values, read
  through a background thread mid-run (or immediately after `run()` returns
  via the same handle, whichever this project's existing pipeline test
  helpers make easiest), match `report.stats.sink_dropped` and
  `report.stats.holder_tally` exactly once the run ends.
- [X] T008 Change `output_loop`'s signature in
  `crates/fragcap-core/src/pipeline/mod.rs` to accept `live: LiveStats`; at
  the existing `sink_dropped = sink_dropped.saturating_add(1)` sites (three:
  a retired-sink write, a countable sink error, and a failing sink) also call
  `live.sink_dropped.fetch_add(1, Ordering::Relaxed)`; at the existing
  `holder_tally.entry(...).or_insert(0)` site also update
  `live.holder_tally`'s locked map the same way; replace the loop's
  `rx.next()` call with `rx.next_and_evicted()` and store the returned count
  into `live.buffer_dropped` (`Ordering::Relaxed`) on every iteration.
  Making T007 pass; run `cargo test -p fragcap-core pipeline` to confirm
  every existing pipeline/conservation test is unaffected (the values
  `output_loop` returns in `PipelineReport` are unchanged; only a live mirror
  is added).
- [X] T009 [P] In `crates/fragcap-cli/src/orchestrator.rs`'s `spawn_pipeline`,
  capture `let live = pipeline.live_stats();` right after `Pipeline::new`
  and before `std::thread::spawn(move || pipeline.run())`; add `live` to the
  `SpawnedPipeline` tuple type and its one construction site.
- [X] T010 [P] Change `crate::color::use_color()` in
  `crates/fragcap-cli/src/color.rs` to `use_color(stream: Stream) -> bool`
  where `Stream` is a new small enum (`Stdout`, `Stderr`), testing
  `std::io::stdout().is_terminal()` or `std::io::stderr().is_terminal()`
  accordingly (both still gated on `NO_COLOR`); update the module's own test
  to cover both variants' predicate shape (the terminal check itself is not
  independently testable in an automated run, matching the existing test's
  posture, but the `NO_COLOR` short-circuit is). Update
  `crates/fragcap-cli/src/commands/doctor.rs`'s two existing `use_color()`
  call sites to `use_color(Stream::Stdout)`, and
  `crates/fragcap-cli/src/commands/targets.rs` /
  `crates/fragcap-cli/src/commands/target_resolve.rs`'s call sites the same
  way. Run `cargo test -p fragcap-cli color doctor targets` to confirm zero
  behavior change.

**Checkpoint**: `cargo test -p fragcap-core buffer pipeline live_stats` and
`cargo test -p fragcap-cli color doctor targets` all green. Every counter
FR-001 needs is now live-readable from the CLI orchestrator: the four
scope/window discards and packets/bytes written from the existing
`GateHandle`, `sink_dropped`/`holder_tally`/`buffer_dropped` from the new
`LiveStats`, active endpoints from the existing `stamper.active_endpoints()`.

---

## Phase 3: User Story 1 - See capture working while it runs (Priority: P1)

**Goal**: On a terminal, the live status block renders and redraws in place
during `drive_live`, showing everything FR-001 lists.

**Independent Test**: Feed a hand-built `LiveStatusSnapshot` sequence through
the pure renderer and a `RedrawState`, asserting each frame's content and
that the erase-then-write byte sequence matches `contracts/status-block.md`.

### Tests for User Story 1

- [X] T011 [P] [US1] Add a failing test in a new
  `crates/fragcap-cli/src/live_status/mod.rs` asserting `LiveStatusSnapshot`
  is constructible from plain values (no live handle) and that
  `render_status` on a snapshot with `process: None` renders the waiting
  header rather than panicking.
- [X] T012 [P] [US1] Add a failing test in the same file asserting a snapshot
  with a configured byte bound renders "written X / bound Y", and a snapshot
  with no bound renders written volume with no bound comparison (spec
  Acceptance Scenario 3).
- [X] T013 [P] [US1] Add a failing test in the same file asserting
  `narrowed: false` renders an explicit "not yet narrowed" line and
  `narrowed: true, active_endpoints: N` renders "narrowed, N endpoint(s)"
  (spec Acceptance Scenario 4).
- [X] T014 [P] [US1] Add a failing test in the same file asserting every one
  of the six discard counters (`watch_discarded`, `out_of_window_discarded`,
  `buffer_dropped`, `sink_dropped`, `scope_discarded`,
  `scope_unresolved_discarded`) appears, labeled, in the rendered output, and
  that a non-zero counter renders with the `WARN` color when `use_color` is
  true and with no color when false.
- [X] T015 [P] [US1] Add a failing test in the same file asserting a
  `holder_tally` with 3 entries renders all 3 with no overflow line, and a
  `holder_tally` with 8 entries renders exactly the top 5 by count plus a
  trailing "... and 3 more" line (Clarifications session, 2026-08-22).
- [X] T016 [P] [US1] Add a failing test in a new
  `crates/fragcap-cli/src/live_status/redraw.rs` asserting: the first call to
  `RedrawState::frame(text)` writes only the frame's bytes (no erase
  sequence, since there is no previous frame); a second call writes
  `\x1b[<n>A\x1b[0J` (where `n` is the previous frame's line count) followed
  by the new frame's bytes; and that `RedrawState` tracks the new frame's
  line count for the next call.
- [X] T017 [P] [US1] Add a failing test in the same file asserting a snapshot
  rendered with a `width` narrower than a line's natural length truncates
  that line rather than wrapping or emitting a byte past the truncation
  point (spec Edge Cases: narrow terminal).

### Implementation for User Story 1

- [X] T018 [US1] Implement `LiveStatusSnapshot` (per `data-model.md`) and
  `render_status(&LiveStatusSnapshot, use_color: bool, width: Option<usize>)
  -> String` in `crates/fragcap-cli/src/live_status/mod.rs`, following
  `contracts/status-block.md`'s five content sections and layout rules,
  reusing `crate::color::{WARN, RESET}`. Making T011-T015 pass.
- [X] T019 [US1] Implement `RedrawState` in
  `crates/fragcap-cli/src/live_status/redraw.rs`: `previous_lines: usize`,
  and a `fn frame(&mut self, out: &mut dyn Write, text: &str)` writing the
  erase-then-write sequence per `contracts/status-block.md`'s redraw
  sequence. Making T016 pass. Add the width-aware truncation to
  `render_status` (T018) or a small helper `redraw.rs` calls, making T017
  pass.
- [X] T020 [US1] Wire the terminal branch into `drive_live` in
  `crates/fragcap-cli/src/orchestrator.rs`: on each `tick` wakeup (the
  existing `loop { ... match rx.recv_timeout(tick) { ... } }` in
  `drive_live`), when `use_color::is_terminal(Stream::Stderr)` (or an
  equivalent predicate added alongside T010), build a `LiveStatusSnapshot`
  from `gate_handle`, the new `live` handle (threaded in as a new parameter
  to `drive_live`, sourced from `spawn_pipeline`'s T009 addition),
  `stamper_reader.active_endpoints()`, `started.elapsed()`, and `bound`; call
  `render_status` and `RedrawState::frame` to write it to the emitter's
  underlying writer. Skip entirely (no construction, no write) when stderr
  is not a terminal or verbosity is not `Normal` (FR-001, FR-006).
- [X] T021 [US1] Ensure the redraw is resolved before the completion summary:
  at every point `drive_live` calls `emitter.summary(...)` (the acquisition-
  failure early return and the normal end-of-run path), clear any
  outstanding redrawn frame first (an empty `RedrawState::frame(out, "")`
  call, or an explicit "clear" method, whichever `contracts/status-block.md`'s
  FR-012 language implies is cleaner) so the two never interleave.
- [X] T022 [US1] Run `cargo test -p fragcap-cli live_status`; confirm
  T011-T017 pass and `cargo build -p fragcap-cli --features etw,live` still
  compiles the new `drive_live` call sites (Windows-only build check; if this
  session cannot run it, record that explicitly rather than claiming it, per
  `AGENTS.md`'s verification discipline for the Tier 2 path).

**Checkpoint**: User Story 1 independently functional and unit-tested on any
platform; the live-wired call site inside `drive_live` compiles under
`etw,live` and is the only part deferred to Tier 2 manual verification.

---

## Phase 4: User Story 2 - Non-terminal runs stay exactly as they are today, and are not silent either (Priority: P2)

**Goal**: No escape sequence when stderr is not a terminal; a 30-second
heartbeat line otherwise.

**Independent Test**: Drive the heartbeat timer with synthetic elapsed times
and progress-line events, asserting it fires only at/after 30 seconds of no
progress and resets correctly.

### Tests for User Story 2

- [X] T023 [P] [US2] Add a failing test in a new
  `crates/fragcap-cli/src/live_status/heartbeat.rs` asserting: a fresh
  `Heartbeat` does not fire before 30 seconds have elapsed since
  construction; it fires at/after 30 seconds; calling
  `Heartbeat::note_progress()` resets the clock so a subsequent check before
  a further 30 seconds have passed does not fire.
- [X] T024 [P] [US2] Add a failing test in the same file asserting the
  heartbeat line's rendered text contains elapsed time and a packet count and
  contains no `\x1b` byte.

### Implementation for User Story 2

- [X] T025 [US2] Implement `Heartbeat` (`last_progress_at: Instant`, a fixed
  30-second constant, `fn due(&self, now: Instant) -> bool`,
  `fn note_progress(&mut self, now: Instant)`) and a `render_heartbeat(elapsed:
  Duration, packets: u64) -> String` function in
  `crates/fragcap-cli/src/live_status/heartbeat.rs`. Making T023, T024 pass.
- [X] T026 [US2] Wire the non-terminal branch into `drive_live`: when stderr
  is not a terminal (or `use_color`'s stream check reports false) and
  verbosity is `Normal`, check `Heartbeat::due` on each tick and, when due,
  emit `render_heartbeat(...)` through `emitter.progress` (which already
  writes a single plain line with no escape byte, satisfying FR-003 by
  construction) and reset the timer. Call `Heartbeat::note_progress` from
  every existing `emitter.progress(...)` call site inside `drive_live`
  (`apply_event`'s stage-matched/stage-exited lines, the launch line, the
  filter-narrowing line) so a run with real progress resets the interval, per
  the Clarifications session.
- [X] T027 [US2] Add a failing-then-passing test exercising `drive_live`'s
  non-terminal path end to end if this project's existing test harness for
  the live driver supports constructing one without a real ETW source
  (matching whatever pattern `crates/fragcap-cli/src/orchestrator.rs`'s
  existing tests, if any, already use for `drive_live`; if none exists,
  extend the pure-function coverage from T023-T024 instead and record in the
  task's own commit message why an end-to-end test was not added, rather
  than silently skipping the intended coverage).
- [X] T028 [US2] Run `cargo test -p fragcap-cli heartbeat`; confirm T023,
  T024 pass.

**Checkpoint**: User Stories 1 and 2 both independently functional; the
terminal and non-terminal branches are mutually exclusive and both tested.

---

## Phase 5: User Story 3 - The other output surfaces are untouched (Priority: P1)

**Goal**: `--json`, `--mode stream --out -`, `--quiet`, `--silent`, and
`extcap` are provably unaffected; the optional `capture.progress` JSON event
exists and is correctly gated.

**Independent Test**: Diff each surface's output against a pre-feature
baseline under identical inputs.

### Tests for User Story 3

- [X] T029 [P] [US3] Add a failing test in `crates/fragcap-cli/src/events.rs`'s
  test module asserting a new `Event::CaptureProgress { .. }` variant renders
  as `"event":"capture.progress"` carrying the fields listed in
  `contracts/capture-progress-event.md`, following the existing
  `SessionComplete` test's shape.
- [X] T030 [P] [US3] Add a failing test in `crates/fragcap-cli/src/orchestrator.rs`'s
  test module (or a new integration test under `crates/fragcap-cli/tests/`)
  asserting that with `Format::Json`, `drive_live`'s tick loop calls
  `emitter.event(&Event::CaptureProgress { .. })` and never calls the
  terminal-redraw or heartbeat code paths (a mock/spy emitter or a captured
  buffer asserting no `\x1b` byte and no plain heartbeat sentence appears).
- [X] T031 [P] [US3] Add a failing test asserting that with `Format::Human`
  and `Verbosity::Quiet` or `Verbosity::Silent`, no redraw and no heartbeat
  line appear on a terminal-backed writer (extending T020/T026's call sites'
  existing verbosity gate with an explicit assertion, matching
  `emit.rs`'s existing `progress_is_suppressed_by_quiet_and_silent` test's
  style).
- [X] T032 [P] [US3] Add a failing test asserting no byte written by this
  slice's new code ever reaches a stdout-backed writer under any
  format/verbosity combination (construct the live-status call sites with
  separate mock stdout/stderr writers and assert stdout stays empty across
  every combination).

### Implementation for User Story 3

- [X] T033 [US3] Add `Event::CaptureProgress` to
  `crates/fragcap-cli/src/events.rs` per `contracts/capture-progress-event.md`:
  the variant, its `kind()` arm (`"capture.progress"`), and its `render`
  match arm. Making T029 pass.
- [X] T034 [US3] In `drive_live`, add the `Format::Json` branch alongside the
  terminal/non-terminal branches already added by T020/T026: on each tick,
  when `emitter`'s format is JSON, build the same `LiveStatusSnapshot` (minus
  `holder_tally`, per the contract) and call
  `emitter.event(&Event::CaptureProgress { .. })`; ensure this branch and the
  human branches are mutually exclusive (an `if/else if/else` over the three
  cases, not three independent `if`s that could all fire). Making T030 pass.
- [X] T035 [US3] Confirm (via T031's test, adding any missing gate) that the
  terminal-redraw and heartbeat call sites added in T020 and T026 already
  check `verbosity == Verbosity::Normal` before doing any work, matching
  `Emitter::progress`'s existing contract; add the check if either call site
  is missing it.
- [X] T036 [US3] Run
  `cargo test -p fragcap-cli --features etw,live` (Windows) or
  `cargo test -p fragcap-cli events orchestrator` (any platform, for the
  parts that do not need the live feature) to confirm T029-T032 pass.
- [X] T037 [US3] Run the full existing `extcap` test suite
  (`cargo test -p fragcap-cli extcap`, or wherever those tests live) to
  confirm zero behavior change; this slice's diff must not touch
  `capture_prerecorded`/`drive` at all (research R-1), so this is a
  regression check, not new coverage.
- [X] T038 [US3] Add the SC-002 standing regression test: drive
  `drive_live`'s non-terminal branch (T026) across several simulated ticks
  with a captured, non-terminal stderr buffer (mixing ordinary progress
  lines and at least one due heartbeat), and assert the entire captured
  buffer contains zero `\x1b` bytes. This is the one test SC-002 explicitly
  names ("verified by a standing regression test that scans the captured
  bytes"); T024's heartbeat-only check and T032's stdout-only check do not
  individually cover the whole non-terminal stream.
- [X] T039 [US1] Add the SC-005 reproduction scenario: construct a
  `LiveStatusSnapshot` sequence where the holder tally's dominant entry
  reaches roughly 90% of admitted packets within the first few ticks after
  volume-gate admission begins (mirroring issue #186's measured run), and
  assert `render_status`'s first frame for that sequence already names the
  dominant image, not only the completion summary. Add the same scenario as
  an explicit numbered step in `quickstart.md`'s manual validation section,
  since the automated version only proves the renderer's behavior, not that
  `drive_live` actually calls it that early in a real run.

**Checkpoint**: All three user stories independently functional and composed
correctly: the human, JSON, and suppressed-verbosity paths are mutually
exclusive and each individually tested; `extcap` and the offline driver are
provably untouched; SC-002 and SC-005 each have a standing test naming them
directly.

---

## Phase 6: Polish & Cross-Cutting Concerns

- [X] T040 [P] Add "Live status block", "Redraw", and "Heartbeat line"
  entries to `docs/glossary/command-line-and-diagnostics.md`, cross-linked
  with the existing "Completion summary" entry, matching that file's
  existing `## Term` / definition / `**See also:**` format (P-6). Regenerate
  `docs/glossary/index.md` via its documented lint-docs.sh mechanism.
- [X] T041 [P] Update the module doc comments touched by this slice
  (`crates/fragcap-core/src/pipeline/live_stats.rs`,
  `crates/fragcap-core/src/pipeline/mod.rs`'s `output_loop` doc comment,
  `crates/fragcap-cli/src/live_status/mod.rs`, `crates/fragcap-cli/src/color.rs`)
  to describe the new behavior accurately.
- [X] T042 [P] Add a changelog fragment
  `changelog.d/S069-live-capture-status.md` (an `added` entry for the live
  status display and heartbeat, a `changed` entry for `use_color`'s new
  stream parameter) per `AGENTS.md`'s changelog-fragment convention.
- [X] T043 Run the quickstart.md automated validation section (items 1-3 at
  minimum on this platform; item 4, the Tier 2 Windows manual run, recorded
  separately per `AGENTS.md`'s verification discipline if this session
  cannot execute it).
- [X] T044 Run the full gate set (`cargo fmt --all -- --check`, `cargo
  clippy --all-targets --all-features -- -D warnings`, `cargo test
  --workspace --locked`, `cargo xtask lint`, `cargo xtask deps`, `cargo
  xtask license`, `cargo xtask ci`) in the foreground and confirm every step
  is green before the slice's pre-push halt. Confirm `Cargo.lock` is
  byte-identical to before this slice's first commit (SC-004).

## Dependencies

- Phase 2 (T002-T010) blocks Phase 3 (US1 needs `LiveStats` and the
  stream-parameterized `use_color`), Phase 4 (US2 needs the stream-parameterized
  `use_color` to detect non-terminal stderr), and Phase 5 (US3's
  `capture.progress` event needs `LiveStatusSnapshot`, defined in Phase 3, and
  US3's stdout-isolation tests need Phase 3/4's call sites to exist).
- Phase 3 (US1) has no dependency on Phase 4; the two branches (terminal
  redraw, non-terminal heartbeat) are mutually exclusive but independently
  implementable and testable.
- Phase 4 (US2) depends only on Phase 2 (the stream predicate), not on Phase
  3's rendering code.
- Phase 5 (US3) depends on Phase 3 for `LiveStatusSnapshot` (T018) and on
  both Phase 3 (T020) and Phase 4 (T026) existing so its non-interference
  tests have real call sites to assert against; its `Event::CaptureProgress`
  work (T033-T034) does not depend on Phase 4 at all.
- Phase 6 depends on Phases 3, 4, and 5 all being complete.

## Parallel execution examples

- T002 (buffer test) and T004 (LiveStats test) are independent, different
  new files.
- T009 (spawn_pipeline threading) and T010 (color stream parameter) are
  independent, different files, both closing out Phase 2.
- Within Phase 3, T011-T017 are independent additions across two new test
  files (`live_status/mod.rs`, `live_status/redraw.rs`) and should be
  written together in one pass.
- Within Phase 5, T029 (events.rs test) and T030-T032 (orchestrator tests)
  are independent of each other.
- T040, T041, T042 (Phase 6) are independent of each other and of T043/T044.

## Implementation strategy

**MVP scope**: User Story 1 alone (Phases 1-3) already delivers the issue's
headline fix: an operator on a terminal sees the status block, including the
per-process breakdown that would have shown the sixteen-minute run's
dominant non-target contributor within seconds. User Story 2 (the
non-terminal heartbeat) and User Story 3 (the JSON event and the explicit
non-interference proofs) are both framed by the issue as non-negotiable
design constraints rather than optional extras, so this plan implements all
three before the slice's verification gate, with the phase boundaries above
marking where a scope cut would land if time ran short. The broader visual
pass (color for warnings/errors generally, thousands separators elsewhere,
`CompletionSummary` restyling) named in the issue's second half stays out of
scope entirely, per the spec's Assumptions section, and is left for a
follow-up issue.
