# Tasks: Pipeline, Buffering, and Drop Accounting

**Slice**: S08

**Branch**: `feat/pipeline-buffering-drops`

**Created**: 2026-08-08

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md),
[data-model.md](data-model.md),
[contracts/pipeline-api.md](contracts/pipeline-api.md),
[quickstart.md](quickstart.md)

Tests are included and are not optional. The slice's deliverable is an
accounting surface, and an accounting surface that is not tested is an
assertion.

Three notes on the shape.

**Phase 2 is the buffer, alone, before anything composes it.** The buffer has
an eviction and ordering contract expressible with no source, no attributor,
and no sink, and a defect there would otherwise surface only as a golden
mismatch three phases later with no obvious cause. It is also the only piece
whose tests can be exhaustive, because it is single-threaded when driven
directly.

**Phase 3 builds the stubs before the pipeline.** Every drop path, every end
reason, and the panic path are reachable only through a source that can fail on
command and sinks that can refuse on command. Writing them first means Phase 4
is written against the tests that will judge it rather than the other way
round.

**Phase 8 is the phase that justifies the slice.** Everything before it proves
the pipeline behaves. Phase 8 proves it produces the same bytes the hand-written
loop produced, which is the check that the composition changed nothing it was
not supposed to.

## Phase 1: Setup

- [X] T001 Create `crates/fragcap-core/src/pipeline/mod.rs` and
  `crates/fragcap-core/src/pipeline/buffer.rs` with SPDX headers and module
  documentation naming specification sections 8.6 and 12.4, and declare
  `pub mod pipeline;` in `crates/fragcap-core/src/lib.rs`. The module uses only
  the standard library, per FR-001
- [X] T002 Add the pipeline re-exports to `crates/fragcap-core/src/lib.rs`
  (`Pipeline`, `PipelineConfig`, `ConfigError`, `StopHandle`,
  `PipelineReport`, `PipelineError`, `EndReason`, `SinkFailure`) and to
  `crates/fragcap/src/lib.rs` under `pub mod core`, matching the existing
  re-export style
- [X] T003 Update the `fragcap-core` crate documentation in
  `crates/fragcap-core/src/lib.rs` to say that `pipeline` is the second module
  that is behavior rather than vocabulary, and why it is here rather than in
  `fragcap-capture`, per plan D-1

## Phase 2: The bounded buffer

Foundational. Everything after this depends on it, and nothing in it depends on
anything else in the slice.

- [X] T004 Define `Item`, the shared state, `Producer`, and `Consumer` in
  `crates/fragcap-core/src/pipeline/buffer.rs` per
  [data-model.md](data-model.md), crate-private, with the eviction count in the
  shared state per research R-5
- [X] T005 Implement `Producer::push` with drop-oldest eviction under the lock,
  advancing the shared eviction count exactly once per eviction, per FR-012 and
  FR-016
- [X] T006 Implement the terminal item's exemption from the capacity bound in
  `Producer::push`, per FR-030 and research R-6
- [X] T007 Implement `Producer::drop` to close the buffer and notify, so the
  consumer observes an ending however the producer terminated, per FR-033a and
  research R-3
- [X] T008 Implement `Consumer::next` to wait on the condvar without spinning
  while the queue is empty and the buffer is open, and to return `None` when
  empty and closed, per FR-015
- [X] T009 [P] Test in `buffer.rs` that a buffer under capacity delivers every
  item in push order, per FR-014
- [X] T010 [P] Test in `buffer.rs` that pushing beyond capacity evicts the
  front, that the survivors keep their relative order, and that the count
  advances by exactly the number evicted, per FR-012, FR-014, FR-016
- [X] T011 [P] Test in `buffer.rs` that a capacity of one works and evicts on
  every push, per the spec's edge cases
- [X] T012 [P] Test in `buffer.rs` that the terminal item is admitted when the
  queue is full, that no packet is evicted to make room for it, and that the
  queue may hold capacity plus one, per FR-030 and research R-6
- [X] T013 [P] Test in `buffer.rs` that dropping the producer while the queue
  still holds items lets the consumer drain them all and only then see `None`,
  per FR-030
- [X] T014 [P] Test in `buffer.rs` that the eviction count is readable through
  the consumer after the producer is gone, per research R-5

## Phase 3: Test stubs

Foundational for every user story phase. No stub is a product type; all live
under `#[cfg(test)]` in `crates/fragcap-core/src/pipeline/mod.rs`.

**Deviation as built.** T017 through T020 name four sink types and landed as one
configurable `StubSink` driven by a `SinkScript`. Four near-identical types
would have differed only in their `write` arm, and a single one makes each test
state the behavior it wants at the call site rather than in a type name. The
behaviors themselves are all present: recording, refusing selected indices,
failing non-countably at an index, and gating on a channel. `StubSink` also
gained flush and finish failure modes, which T067a needed.

- [X] T015 Build `StubSource` in `crates/fragcap-core/src/pipeline/mod.rs`
  yielding a scripted sequence of `RawPacket` values and then a configurable
  terminal outcome (`Closed`, a `DeviceLost`, or a run of `Timeout` before
  either), with a configurable `SourceStats` so the relay can be asserted
- [X] T016 [P] Build `StubAttributor` answering from a map keyed by flow, so a
  test can produce resolved, unresolved, and never-attempted packets in one run
- [X] T017 [P] Build `RecordingSink` capturing every packet it receives, the
  statistics it was finished with, and whether it was flushed before being
  finished
- [X] T018 [P] Build `RefusingSink` returning `SinkError::Full` for a
  predetermined set of packet indices, so `sink_dropped` reaches an exact
  expected value rather than merely a non-zero one, per SC-003
- [X] T019 [P] Build `FailingSink` returning a non-countable error at a
  predetermined index, so retirement is reachable, per FR-027
- [X] T020 [P] Build `GatedSink` that blocks in `write` until the test releases
  a channel it holds, so eviction is forced without a sleep, per research R-7
  and SC-002
- [X] T021 [P] Build `PanickingSource`, a source that panics at a predetermined
  index, so the panic path is reachable, per FR-033b

## Phase 4 (US1, US6): Composition and the run loop

**Goal**: A pipeline built entirely from trait objects runs a source through a
parser and an attributor to a set of sinks.

**Independent test**: Construct from boxed stubs, run, and assert the recording
sink received the scripted packets in order.

- [X] T022 [US1] Define `PipelineConfig` with the section 12.4 default capacity
  of 65,536, the research R-8 default read timeout, and the interface address
  set, in `crates/fragcap-core/src/pipeline/mod.rs`, per FR-010 and FR-008
- [X] T023 [US1] Define `ConfigError` as `#[non_exhaustive]` with
  `ZeroCapacity`, with `Display` and `Error` hand-written to match the
  `error.rs` convention, per FR-011
- [X] T023a [US1] Define `EndReason` as `#[non_exhaustive]` with
  `SourceClosed`, `Stopped`, `SourceFailed`, and `AllSinksRetired`, and
  `SinkFailure` with its index and error, per FR-032 and FR-028. These are
  defined here rather than in Phase 7 because `run` returns them; Phase 7 fills
  in the behavior that produces the last two
- [X] T023b [US1] Define `PipelineReport` with `stats`, `ended`, and
  `sink_failures`, marked `#[must_use]`, and `PipelineError` carrying the whole
  report with `Display` and `Error`, and implement `is_clean` and
  `into_result`, per FR-034, FR-035, FR-036 and plan D-6
- [X] T023c [US1] Define `StopHandle` over `Arc<AtomicBool>` with `stop` and
  `is_stopped`, documenting that stop latency is bounded by the read timeout,
  per FR-029
- [X] T024 [US1] Define `Pipeline` holding a boxed source, a boxed attributor,
  a `Vec<Box<dyn Sink>>`, the config, and the stop handle, with `new`,
  `add_sink`, and `stop_handle`, per FR-002 and FR-003
- [X] T025 [US1] Document the control thread seam at the construction site in
  `mod.rs`, naming S11 and S13 and why the attributor is owned outright for
  now, per FR-007 and plan D-7
- [X] T026 [US1] Implement the acquisition loop per
  [contracts/pipeline-api.md](contracts/pipeline-api.md): stop check, timed
  read, `CapturedPacket` construction, parse, attribution against the packet's
  own timestamp, push. Per FR-005, FR-006, FR-019
- [X] T027 [US1] Implement the output loop draining the buffer and offering
  each packet to every sink in index order, per FR-025
- [X] T028 [US1] Implement `run` to consume the pipeline, acquire on the
  calling thread, and spawn the output thread, per FR-004, FR-033c and plan
  D-1, with a guard that joins the output thread on drop, per research R-3
- [X] T029 [US1] Implement flush-then-finish for every sink, exactly once each,
  after the buffer is drained, per FR-030 and FR-031. Until T038 lands, the
  statistics passed to `finish` are the acquisition side's alone; that is an
  expected intermediate state of this slice rather than a defect, and T046 is
  the test that closes it
- [X] T030 [P] [US1] Test that construction with zero capacity fails with
  `ConfigError::ZeroCapacity` and that construction starts no thread and reads
  no packet, per FR-011
- [X] T031 [P] [US1] Test that a run over a scripted source delivers every
  packet to a recording sink in source order, per FR-014 and FR-038
- [X] T032 [P] [US1] Test that two sinks of different concrete types both
  receive every admitted packet, per FR-025 and US6
- [X] T033 [P] [US1] Test that a run with no sinks completes, reports its
  captured count, and counts nothing as a sink drop, per the spec's edge cases
- [X] T034 [P] [US6] Test that the pipeline is constructible and runnable
  entirely through `Box<dyn PacketSource>`, `Box<dyn FlowAttributor>`, and
  `Box<dyn Sink>` with no concrete type named, per FR-003

## Phase 5 (US2): Drop accounting

**Goal**: Every discard path advances its named counter, and nothing escapes
the accounting.

**Independent test**: Force each path with a stub and assert both the specific
counter and the conservation identity.

- [X] T035 [US2] Wire `packets_captured`, `packets_attributed`, and
  `packets_unattributed` in the acquisition loop, advancing exactly one
  attribution counter when a flow key exists and neither when it does not, per
  FR-019 and FR-020
- [X] T036 [US2] Copy `HeaderParser::stats` into the run's `parse` counters at
  the end of acquisition, per FR-021
- [X] T037 [US2] Copy `PacketSource::stats` into `stats.source` unaltered at
  the end of acquisition, per FR-018
- [X] T038 [US2] Carry the acquisition counters in the terminal item and merge
  them with the buffer's eviction count and the output side's `sink_dropped` in
  the output thread, per FR-023 and plan D-8
- [X] T039 [US2] Advance `sink_dropped` once per `SinkError::Full` from a live
  sink, per FR-017 and FR-026
- [X] T040 [P] [US2] Test that a gated sink and a small capacity drive
  `buffer_dropped` above zero, with the gate released by the test rather than
  by a sleep, per SC-002
- [X] T041 [P] [US2] Test that a refusing sink drives `sink_dropped` to the
  exact expected count, per SC-003
- [X] T042 [P] [US2] Test that with several sinks refusing the same packet,
  `sink_dropped` advances once per refusing sink rather than once per packet,
  per FR-017
- [X] T043 [P] [US2] Write a conservation assertion helper in `mod.rs` checking
  `received + buffer_dropped + refusals == packets_captured` for each sink, and
  call it from every pipeline test in this file, per FR-022, FR-024 and SC-004.
  This helper is the standing enforcement of FR-024: a discard path added later
  with no counter fails here rather than passing quietly
- [X] T044 [P] [US2] Test that a source reporting non-zero `kernel_dropped` and
  `interface_dropped` has both relayed unchanged and folded into no fragcap
  counter, per FR-018
- [X] T045 [P] [US2] Test that a clean run reports zero in every drop counter
  and `lost_anything` is false, per US2
- [X] T046 [P] [US2] Test that `Sink::finish` receives the run's own final
  statistics, that every sink receives the same value, and that the value
  equals the one the report carries, per FR-023
- [X] T047 [P] [US2] Test that a packet producing no flow key advances neither
  attribution counter and still reaches the sink, per FR-020 and FR-039

## Phase 6 (US3): The producer never waits on the sink

**Goal**: A blocked sink cannot stall acquisition.

**Independent test**: Hold a sink and assert acquisition still reached source
exhaustion.

- [X] T048 [US3] Confirm by construction that the acquisition loop's only
  interaction with the output side is `Producer::push`, and document at the
  push site that the wait is bounded by the critical section rather than by
  sink progress, per FR-013 and plan D-2
- [X] T049 [P] [US3] Test that with a gated sink and a source of more packets
  than the buffer holds, acquisition reaches source exhaustion while the sink
  is still held, then release the gate and assert the run completes, per SC-005
- [X] T050 [P] [US3] Test that after eviction the packets the sink does receive
  are in their original relative order, per FR-014 and FR-038

## Phase 7 (US4): Termination

**Goal**: Every ending drains, flushes, finishes, and says why.

**Independent test**: Produce each ending and assert the report and the sink
state.

- [X] T051 [US4] Implement the end-reason determination in `run`, mapping the
  acquisition loop's exit to `SourceClosed`, `Stopped`, or `SourceFailed`, per
  FR-032 and FR-033. The types themselves landed in T023a
- [X] T052 [US4] Implement `SinkError` handling on `flush` and `finish`,
  recording a `SinkFailure` for the sink that produced it, per FR-028a
- [X] T053 [US4] Wire the stop check into the acquisition loop between packets
  so a requested stop ends the run with `Stopped`, per FR-029 and FR-032
- [X] T055 [US4] Implement sink retirement: record a `SinkFailure` on a
  non-countable error, stop offering packets to that sink, and advance
  `sink_dropped` for it on every subsequent packet, per FR-027 and FR-027a
- [X] T056 [US4] Implement `AllSinksRetired`: when every sink has retired, the
  output side requests the stop and keeps draining and counting until the
  terminal item, per FR-027b
- [X] T057 [US4] Implement panic handling: the join-on-drop guard finishes the
  sinks before the panic escapes, and the panic is re-raised rather than
  reported, per FR-033a and FR-033b
- [X] T058 [P] [US4] Test source exhaustion: the reason is `SourceClosed`,
  every buffered packet was written before any sink was finished, per FR-030
- [X] T059 [P] [US4] Test an operator stop: the reason is `Stopped`, the buffer
  was drained, and the run did not report exhaustion, per FR-032
- [X] T060 [P] [US4] Test a terminal source failure: the reason is
  `SourceFailed` naming the error, and the packets already buffered were still
  written, per FR-032
- [X] T061 [P] [US4] Test that one of two sinks failing non-countably retires
  only that sink, that the other keeps receiving, and that the retired one
  accrues a `sink_dropped` per subsequent packet, per FR-027 and FR-027a
- [X] T062 [P] [US4] Test that when every sink retires the reason is
  `AllSinksRetired` and each failure is named with its index, per FR-027b and
  FR-028
- [X] T063 [P] [US4] Test that a recoverable source error neither ends the run
  nor counts as loss, per FR-033
- [X] T064 [P] [US4] Test the degenerate startings: a source closed on the
  first call, and a stop requested before the run begins, per the spec's edge
  cases
- [X] T065 [P] [US4] Test that an acquisition-side panic still finishes every
  sink and still reaches the caller as a panic, per FR-033b and SC-011
- [X] T066 [P] [US4] Test that every sink is flushed before being finished and
  finished exactly once, on every ending, per FR-031
- [X] T067 [P] [US4] Test that the report carries the statistics on the failure
  path as well as the clean path, and that `into_result` errs only on an
  abnormal ending rather than on ordinary drops, per FR-035 and FR-036
- [X] T067a [P] [US4] Test that a sink failing in `flush` or `finish` is
  recorded in the report and does not prevent the remaining sinks from being
  flushed and finished, per FR-028a
- [X] T067b [P] [US4] Test that all four end reasons are produced and named,
  and that a retirement which does not end the run is distinguishable from one
  that does, per SC-006

## Phase 8 (US1, US5): The corpus end to end

**Goal**: The real components, over the real fixtures, reproducing the
committed goldens.

**Independent test**: Run every fixture and compare both outputs byte for byte.

- [X] T068 [US1] Add a pipeline runner helper to
  `crates/fragcap/tests/common/mod.rs` that builds a `Pipeline` from a
  `ReplaySource`, a `ScriptedAttributor`, and both writers over in-memory
  buffers, reusing the existing fixture and interface address helpers
- [X] T069 [US1] Create `crates/fragcap/tests/corpus_pipeline.rs` with module
  documentation explaining why it lives in the facade rather than in a backend
  crate, per FR-042
- [X] T070 [US1] Test that every one of the eight fixtures runs through the
  pipeline and produces pcapng output byte-identical to its committed golden,
  per SC-001
- [X] T071 [US1] Test that every one of the eight fixtures produces JSON Lines
  output byte-identical to its committed golden, per SC-001
- [X] T072 [P] [US5] Test that one pipeline carrying both writers produces both
  outputs from a single pass over the source, per US1 scenario 4
- [X] T073 [P] [US5] Test that running the same fixture twice produces
  byte-identical output, per SC-009
- [X] T074 [P] [US5] Test that the `malformed` fixture's packets, none of which
  produce a flow key, are all written and marked, per FR-039
- [X] T075 [P] [US5] Test that a truncated packet's original length reaches the
  sink unchanged, and that the `fragmented` fixture's three fragments arrive as
  three packets with nothing joined, normalized, or repaired, per FR-037 and
  FR-040
- [X] T076 [P] [US5] Assert the conservation identity across the whole corpus
  run, per FR-022
- [X] T076a [P] [US5] Test that a `PcapngWriter` driven by the pipeline still
  refuses a second `declare_interface`, so the S09 restriction cannot be lifted
  by accident, per FR-043

## Phase 9: Polish and cross-cutting

- [X] T077 [P] Add glossary entries for pipeline, bounded buffer, drop-oldest,
  capture thread, sink thread, and fan-out in `docs/glossary.md`, following the
  section 4.3 template with primary-source references, per FR-041
- [X] T078 [P] Add the drop-oldest cross-reference to the existing
  `Backpressure` entry in `docs/glossary.md`, per FR-041
- [X] T079 [P] Confirm no manifest changed: `git diff main -- Cargo.toml
  Cargo.lock crates/*/Cargo.toml` is empty, per FR-009 and SC-007. Confirm in
  the same pass that `crates/fragcap-core/src/pipeline/` contains no `print!`,
  `println!`, `eprint!`, `eprintln!`, or `dbg!`, per FR-041a, and that no
  benchmark or timing assertion was added, per SC-012
- [X] T080 [P] Update `crates/fragcap-sink/src/lib.rs` module documentation,
  which currently says "The pipeline that drives any of them is S08", to say
  that it now exists
- [X] T081 Update `AGENTS.md` current state: S08 is complete, the loss counters
  carry real values, and record the `PacketSource: Send` finding that S09
  inherits
- [X] T082 Write `changelog.d/S08-pipeline-buffering-drops.added.md` describing
  the pipeline, the buffer, and the accounting
- [X] T083 Write `changelog.d/S08-pipeline-buffering-drops.decisions.md`
  recording plan decisions D-1 through D-9, with the sink retirement reversal
  stated as a reversal
- [X] T084 Run `cargo xtask ci` in the foreground and watch it to completion,
  per SC-008
- [X] T085 Run `cargo xtask neutral` in the foreground, which is the mechanical
  proof that adding threads to `fragcap-core` did not add a platform
  dependency, per SC-010
- [X] T086 Run `cargo xtask msrv` in the foreground and record whether it ran
  or exited 2

## Dependencies

```text
Phase 1 (setup)
   |
Phase 2 (buffer)  <- blocking for everything after
   |
Phase 3 (stubs)   <- blocking for phases 4 to 7
   |
Phase 4 (composition)
   |
   +-- Phase 5 (accounting)
   +-- Phase 6 (no-wait property)
   +-- Phase 7 (termination)
   |
Phase 8 (corpus)  <- needs phases 4, 5, and 7
   |
Phase 9 (polish)
```

Phases 5, 6, and 7 are independent of each other once Phase 4 lands.

The report, end reason, sink failure, and stop handle types are defined in
Phase 4 (T023a to T023c) rather than in Phase 7, because `run` returns them and
the acquisition loop reads the stop flag. Phase 7 supplies the behavior that
produces them. This was the one ordering contradiction the analyze gate found:
as first written, Phase 4 could not compile.

## Parallel opportunities

- T009 through T014: six buffer tests, all in one file, all independent.
- T016 through T021: six stubs, independent of each other once T015 fixes the
  shape.
- T030 through T034, T040 through T047, T058 through T067, T072 through T076:
  every test within a phase is independent of its siblings.
- T077 through T080: four documentation edits in four different files.

## Implementation strategy

The minimum viable increment is Phase 1 through Phase 4 plus T043: a pipeline
that composes the seams and whose conservation identity is asserted. That
alone would be a demonstrable improvement over the hand-written loop, and every
phase after it adds a class of failure the pipeline can be trusted about.

Phase 8 is the phase that can invalidate earlier work, because a golden
mismatch means the composition differs from the loop the goldens came from. It
is deliberately not last: Phase 9 is documentation and gates, so a Phase 8
failure still has room to be fixed without reordering the slice.

Do not change a golden to make Phase 8 pass. A golden that needs changing is
the finding.
