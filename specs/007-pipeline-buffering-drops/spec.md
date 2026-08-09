# Feature Specification: Pipeline, Buffering, and Drop Accounting

**Feature Branch**: `feat/pipeline-buffering-drops`

**Created**: 2026-08-08

**Status**: Draft

**Slice**: S08 (specification sections 8.6 and 12.4; constitution P-2, P-3,
P-4, P-6, P-9)

**Input**: Build the three-thread capture pipeline from specification section
8.6 and the bounded buffer with drop accounting from section 12.4. Compose the
replay source, the header parser, the scripted attributor, and both writers
into something that actually runs, and make the drop counters carry real
values.

## Overview

Every part of fragcap's offline path now exists and nothing joins them. S03
parses a frame into a flow key, S04 replays a capture file and answers
attribution from a script, S06 and S07 write two output formats. What is
missing is the thing that reads from one end and writes to the other, and it is
missing in a way that has been quietly costing the project accuracy: the
`CaptureStats` value handed to `Sink::finish` today is a snapshot a test
composed by hand. The counters are real types with real names and no producer.

This slice supplies the producer. That is the whole point, and it is why the
slice is named for accounting rather than for plumbing.

Three properties carry it.

**The capture thread never waits on the sink.** Specification section 12.4 is
explicit about why: blocking a capture thread stalls the kernel buffer behind
it and converts a fragcap drop, which is visible and controllable, into a
kernel drop, which is neither. The bounded buffer therefore drops the oldest
packet to admit the newest and never applies backpressure upstream. The
producer waits only for the buffer's own brief critical section, never for a
sink to make progress, and the distinction between those two waits is the
property that has to be tested rather than asserted.

**Every packet is accounted for exactly once.** The conservation identity is
the strongest statement constitution P-4 can be given: the number of packets
the pipeline accepted equals the number written plus the number the buffer
dropped plus the number sinks refused. A test that asserts a counter is
non-zero proves a counter is reachable. A test that asserts the identity holds
proves nothing escaped the accounting, which is the actual requirement, and it
holds under every thread interleaving rather than under the convenient one.

**Order survives.** Constitution P-9 forbids reordering an observation, and a
concurrent pipeline is the first place in this project where reordering could
happen by accident rather than by decision. Packets leave the buffer in the
order they entered it, and the end-to-end run over the corpus reproduces the
committed goldens byte for byte, which is the check that would catch a
reordering nobody thought to look for.

The slice deliberately builds two of section 8.6's three threads. The control
thread owns the `ProcessWatcher`, the process tree, and the filter manager,
none of which exist before S11 and S13. Its seam is named and left unfilled,
because inventing a snapshot-publication mechanism now would fix its shape
before the slice that knows what a socket table snapshot costs to publish.

## Clarifications

### Session 2026-08-08

- Q: Which crate owns the pipeline? -> A: `fragcap-core`, in a new `pipeline`
  module. Specification section 8.2 places the pipeline there, the crate's own
  module documentation already anticipates it, and the parser it drives was put
  there in S03 for the same reason. Constitution P-2 permits it because
  threads, channels, and mutexes are standard library facilities with no
  platform surface.
- Q: Does this slice add a runtime dependency? -> A: No. The buffer is
  hand-rolled over `std::sync::Mutex`, `std::sync::Condvar`, and
  `std::collections::VecDeque`. The alternatives were examined rather than
  dismissed. `std::sync::mpsc::channel` is unbounded and cannot drop.
  `std::sync::mpsc::sync_channel` is bounded and blocks the producer, which is
  the one behavior section 12.4 forbids; its `try_send` fails rather than
  evicting, which is drop-newest and the wrong policy. A third-party bounded
  channel has the same two shapes and would not supply drop-oldest either, so
  the dependency would buy nothing and would still leave the eviction to be
  written by hand. The workspace's one runtime dependency stays one.
- Q: Does the capture thread block on the buffer lock, and does that violate
  the no-backpressure rule? -> A: It acquires a lock whose critical section is
  a push and a possible pop, and it never waits on sink progress. The rule
  section 12.4 states is that a slow sink must not stall capture, and a lock
  held for a bounded number of instructions by a consumer that is not itself
  waiting on anything cannot express sink slowness. A lock-free ring would
  remove even that wait and was rejected: it is materially harder to prove
  correct, and the property being bought is not the one the specification asks
  for. The requirement is stated as "never waits for a sink to make progress"
  rather than "never blocks", because the second is false of any shared
  structure and would be a claim the code could not honor.
- Q: What happens when a sink returns an error that is not `Full`? -> A: That
  sink is retired. Every subsequent packet advances `sink_dropped` for the
  retired sink exactly as a `Full` would, the other sinks keep receiving, and
  the run ends when the source ends or when every sink has retired, whichever
  comes first. The first non-countable failure is named in the report, and a
  retired sink is still flushed and finished so its output is terminated and
  carries the final accounting.

  This answer was reached twice. The first reading of
  `SinkError::is_countable`, which S02 documented as "whether the pipeline
  should count this and carry on rather than stop", was that a non-countable
  error stops the whole run, and that retiring the sink was the unsafe option
  because a retired sink's missing writes would be recorded nowhere.

  That reasoning does not survive being followed through. Stopping the run does
  not avoid the accounting problem; it relocates it. Packets already buffered
  and packets still arriving have to go somewhere, and "nowhere, because we
  stopped" is a discard path with no counter, which P-4 calls a defect. Once
  both options are made to conserve, the counter each needs is the same one,
  and section 12.4 already defines it: `sink_dropped` is "dropped by a sink
  that could not accept", and a sink that has failed is precisely a sink that
  cannot accept. Retiring therefore needs no new counter, conserves exactly,
  and keeps a capture running when one of several outputs dies, which is what a
  capture tool should do. `is_countable` still decides the split; what it
  decides is whether the sink survives the packet, not whether the run survives
  the sink.
- Q: When several sinks refuse the same packet, how many times does
  `sink_dropped` advance? -> A: Once per refusal, so three sinks refusing one
  packet advances it by three. The counter measures writes that did not happen,
  and section 12.4 names its remedy as a slow consumer downstream of fragcap.
  Counting per packet would report one loss where three outputs are short, and
  would make the number smaller as the fan-out widened, which is backwards.
- Q: Who owns the `FlowAttributor` while the control thread does not exist? ->
  A: The capture thread owns it outright, moved in at construction. Section 8.6
  has the control thread publishing a snapshot the capture thread reads without
  blocking, and that arrangement needs a publisher, a snapshot type, and a
  reason for the attributor to be `Sync`, none of which exist yet. The trait is
  `Send`, which is exactly the bound that permits moving it to the capture
  thread, and `FlowAttributor` is part of the surface intended to reach 1.0.0
  unchanged, so adding `Sync` to it would be a change to the architecture of
  record rather than a slice-local convenience. The seam is documented at the
  construction site and filled by S10 or S13.
- Q: How does the sink thread learn the final statistics, given the capture
  thread produces most of them? -> A: Through the buffer. The buffer carries a
  sequence of packets followed by exactly one terminal item that carries the
  capture thread's final counters. The sink thread drains until it sees the
  terminal item, adds its own `sink_dropped` to what it received, and passes
  the sum to `Sink::finish`. The alternative, shared atomics read at an
  arbitrary instant, would let `finish` be handed a value that was never true
  of any moment, which is a P-9 problem rather than only a race.
- Q: What does `run` return? -> A: A `PipelineReport` carrying the final
  `CaptureStats` and an `EndReason`, marked `#[must_use]`, with an
  `into_result` method for callers who want the ordinary `Result` shape. A bare
  `Result<CaptureStats, PipelineError>` was the first shape considered and
  discarded: on the error path the accounting is exactly what an operator most
  needs, and a shape that discards the counters when the run fails contradicts
  P-4 at the one moment P-4 matters most. Making the report unconditional makes
  it impossible to learn the outcome without also being handed the numbers.
- Q: What is the default buffer capacity, and is it configurable? -> A: 65,536
  packets, per section 12.4, configurable through the pipeline's configuration
  and never zero. A zero-capacity buffer would drop every packet, which is a
  configuration that can only be a mistake; it is rejected at construction
  rather than honored.
- Q: How is a stop requested? -> A: A cloneable stop handle backed by an
  `AtomicBool`, checked by the capture thread between packets. It is
  cooperative and never interrupts a blocking source call, so a source waiting
  on its timeout finishes that wait first. The bound on stop latency is
  therefore the source's own timeout, which is the caller's choice and is
  documented rather than hidden.
- Q: Is the corpus end-to-end test in the facade or in core? -> A: The facade,
  for the reason S06 and S07 both landed on. The facade is the only crate that
  legitimately depends on `fragcap-capture`, `fragcap-attr`, and
  `fragcap-sink`, and a dev-dependency between any two of those would create
  exactly the edge P-3 exists to prevent while passing `cargo xtask deps`,
  which ignores dev-dependencies by design. Core keeps unit tests that drive
  the pipeline with in-crate stubs.
- Q: Does this slice change either writer? -> A: No. Both still declare one
  interface and refuse a second, because `CapturedPacket` still carries no
  interface identifier and `Sink::write` still has nowhere to pass one. S09
  brings both. The goldens are therefore expected to reproduce unchanged, and
  that is the assertion: if driving the writers from a real pipeline instead of
  a hand-written loop changes a byte, something about the loop was load-bearing
  and undocumented.
- Q: Does the pipeline emit the section 12.7 session anchor? -> A: No. The
  anchor needs a capture driver clock reading, which arrives with S09, and both
  writers already record the gap rather than papering over it. This slice
  establishes capture start as a concept the pipeline owns and leaves the
  anchor's shape to the slice that can populate it honestly.
- Q: What happens if the acquisition thread panics? The output side is waiting
  for a terminal item that will never arrive. -> A: The buffer closes when its
  producer handle is dropped, which unwinding does, so the consumer observes an
  ending regardless of how the producer terminated and cannot wait forever. The
  run then drains, flushes, and finishes every sink with the statistics the
  consumer holds, and re-raises the panic once that cleanup is done. A panic is
  a defect in whatever panicked, and the two tempting alternatives both hide
  it: converting it into an end reason files a defect under an accounting
  category that means something else, and letting it escape before the sinks
  are finished leaves an unterminated output file. Re-raising after cleanup is
  what `std::thread::scope` does and for the same reason.
- Q: Does `run` block, or spawn and return a handle? -> A: It consumes the
  pipeline and blocks until the run ends. A caller wanting it in the background
  spawns a thread and keeps the stop handle, which is two lines and needs
  nothing from this slice. Providing both shapes now would be designing an
  interface for S14 before S14 exists to say what it needs, and the blocking
  shape is the one the other would be built from anyway.
- Q: Which terms need glossary entries? -> A: Pipeline, bounded buffer,
  drop-oldest, capture thread, sink thread, and fan-out. The existing
  `Backpressure` entry gains a cross-reference to drop-oldest, because it
  currently describes the general concept and this slice is what fixes fragcap's
  answer to it.
- Q: Does this slice add logging or any observability beyond the counters? ->
  A: No. The counters and the end reason are the whole observability surface,
  and they are structured data a caller reads rather than text a human greps. A
  logging facade is a workspace-wide dependency decision, it would be the second
  runtime dependency, and it belongs to S14, which is the first slice with an
  operator to log to and a place to put the output.
- Q: Does this slice set a throughput or latency target? -> A: No target and no
  benchmark. The only source available is a file replay, so any number measured
  here would describe disk read speed rather than capture, and publishing it
  would invite the comparison it cannot support. Section 12.4 states no
  throughput target either; what it states is the structural property, that a
  slow sink must not stall capture, and that property is asserted directly in
  User Story 3 rather than through a number standing in for it. S09 owns the
  first honest measurement.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - The corpus runs through the real pipeline (Priority: P1)

A contributor points the pipeline at a fixture and gets the same output files
the hand-written test loop produced, without writing a loop.

**Why this priority**: This is the slice. Until the composition exists, every
consumer of fragcap's seams has to write its own capture loop, and each one is
a place the accounting can differ.

**Independent Test**: Run every fixture in the corpus through the pipeline with
a replay source, the real parser, a scripted attributor, and both writers, and
compare the output against the committed goldens.

**Acceptance Scenarios**:

1. **Given** a fixture and its script, **When** the pipeline runs to
   completion, **Then** the pcapng output is byte-identical to the committed
   golden.
2. **Given** the same fixture, **When** the pipeline runs, **Then** the JSON
   Lines output is byte-identical to the committed golden.
3. **Given** all eight fixtures, **When** each is run, **Then** every one
   completes and no fixture requires a special case in the caller.
4. **Given** one pipeline, **When** it is given both writers at once, **Then**
   both outputs are produced from a single pass over the source.

---

### User Story 2 - Nothing is lost without being counted (Priority: P1)

An operator reading the trailing statistics can account for every packet the
pipeline accepted.

**Why this priority**: Constitution P-4, and the reason the slice exists. The
counters have had names since S02 and no producer; a counter that cannot be
made non-zero is indistinguishable from one that is wired up wrong.

**Independent Test**: Force each discard path with a purpose-built stub, and
assert both that the specific counter advanced and that the conservation
identity still holds.

**Acceptance Scenarios**:

1. **Given** a buffer smaller than the number of packets and a sink held from
   draining, **When** the run completes, **Then** `buffer_dropped` is non-zero.
2. **Given** a sink that refuses selected packets, **When** the run completes,
   **Then** `sink_dropped` equals the number of refusals.
3. **Given** any run and any sink, **When** the run completes, **Then** the
   packets that sink received plus `buffer_dropped` plus that sink's refusals
   equals `packets_captured`.
4. **Given** a run where nothing was refused and nothing was evicted, **When**
   the statistics are read, **Then** every drop counter is zero and
   `lost_anything` is false.
5. **Given** a source reporting non-zero `kernel_dropped`, **When** the run
   completes, **Then** the value is relayed unchanged and is not folded into
   any fragcap counter.
6. **Given** a completed run, **When** `Sink::finish` is called, **Then** it
   receives the run's own final statistics rather than a default or partial
   value.

---

### User Story 3 - A slow sink does not stall capture (Priority: P1)

A capture continues at source speed while a sink is blocked, losing buffered
packets by eviction rather than losing them in the kernel.

**Why this priority**: Section 12.4's stated reason for drop-oldest. Getting
this wrong produces a pipeline that looks correct in every test and converts
fragcap drops into kernel drops the moment it meets real traffic.

**Independent Test**: Hold a sink from draining, run a source to exhaustion,
and assert the capture side finished while the sink was still held.

**Acceptance Scenarios**:

1. **Given** a sink that is not draining, **When** the source has more packets
   than the buffer holds, **Then** the capture side still reaches source
   exhaustion.
2. **Given** a full buffer, **When** a new packet arrives, **Then** the oldest
   buffered packet is evicted and counted, and the new packet is admitted.
3. **Given** a full buffer, **When** a new packet arrives, **Then** the
   producer does not wait for the consumer to remove anything.
4. **Given** a buffer that has evicted packets, **When** the remaining packets
   are drained, **Then** they arrive in their original relative order.

---

### User Story 4 - Shutdown is orderly (Priority: P1)

A run that ends, for any reason, leaves complete output files and a statistics
value that describes what happened.

**Why this priority**: A capture tool that loses its buffered tail on shutdown
loses exactly the packets around whatever the operator stopped to look at.

**Independent Test**: End a run each of the four ways and assert in every case
that the buffer was drained, every sink was flushed and finished, and the
reason is named.

**Acceptance Scenarios**:

1. **Given** a source that reaches exhaustion, **When** the run ends, **Then**
   every packet still in the buffer is written before any sink is finished.
2. **Given** a stop request, **When** the capture side observes it, **Then** it
   stops reading, the buffer is drained, and the reason is reported as a stop
   rather than as exhaustion.
3. **Given** a source that fails terminally, **When** the run ends, **Then**
   the failure is named in the report and the packets already buffered are
   still written.
4. **Given** two sinks and one that fails with a non-countable error, **When**
   the run continues, **Then** the other sink still receives every subsequent
   packet and the retired one accrues a `sink_dropped` for each.
5. **Given** every sink retired, **When** the run ends, **Then** the reason is
   reported as such and each failure is named with its sink index.
6. **Given** any ending, **When** the report is read, **Then** it carries both
   the final statistics and the reason, and the reason distinguishes a clean
   end from every other kind.

---

### User Story 5 - Order and content survive the crossing (Priority: P2)

A researcher comparing pipeline output against the source file finds the same
packets, in the same order, unaltered.

**Why this priority**: Constitution P-9. Not P1 only because the golden
comparison in User Story 1 would catch a gross violation; this story is what
catches a subtle one.

**Independent Test**: Run a fixture whose packets are individually
distinguishable and compare the written sequence against the source sequence.

**Acceptance Scenarios**:

1. **Given** a source of distinguishable packets and a buffer large enough for
   all of them, **When** the run completes, **Then** the sink received them in
   source order with nothing missing.
2. **Given** a packet whose original length exceeds its captured length,
   **When** it crosses the pipeline, **Then** both lengths arrive at the sink
   unchanged.
3. **Given** a packet that produced no flow key, **When** it crosses the
   pipeline, **Then** it is written and marked rather than dropped.
4. **Given** the same fixture run twice, **When** the outputs are compared,
   **Then** they are byte-identical.

---

### User Story 6 - The pipeline composes seams only (Priority: P2)

A contributor adds a new source, attributor, or sink and runs it through the
pipeline without changing the pipeline.

**Why this priority**: Constitution P-3 and specification section 8.7. The
extension points are only real if something composes them generically.

**Independent Test**: Construct the pipeline entirely from trait objects and
run it.

**Acceptance Scenarios**:

1. **Given** a boxed source, a boxed attributor, and boxed sinks, **When** the
   pipeline is constructed, **Then** it compiles and runs with no knowledge of
   the concrete types.
2. **Given** the pipeline's public surface, **When** it is inspected, **Then**
   no source names an attributor and no attributor names a source.
3. **Given** several sinks of different concrete types, **When** they are
   given to one pipeline, **Then** each receives every admitted packet.

---

### Edge Cases

- What happens when the buffer capacity is zero? Construction fails with a
  named error. A buffer that drops everything can only be a mistake, and
  honoring it would produce a run whose statistics were correct and whose
  output was empty.
- What happens when the buffer capacity is one? It works, and every packet
  arriving while one is buffered evicts it. Permitted, because it is the
  degenerate case of a legitimate setting rather than a contradiction, and it
  is the easiest configuration in which to test eviction.
- What happens when there are no sinks at all? The run proceeds and every
  packet is drained and discarded with nothing written. This is not counted as
  a sink drop, because no sink refused anything; it is the operator's declared
  scope, which P-9 distinguishes from altering an observation. The report says
  how many packets were captured, so the emptiness is visible.
- What happens when a sink panics? The pipeline does not catch it. A panicking
  sink is a defect in that sink, and converting it into a counted drop would
  hide the defect behind an accounting category that means something else. The
  panic reaches the caller once the run has finished unwinding.
- What happens when the acquisition side panics? The buffer closes as its
  producer handle is dropped during unwinding, so the output side observes an
  ending rather than waiting for a terminal item that will never come. The
  buffer is drained, every sink is flushed and finished with the statistics the
  output side holds, and the panic is re-raised afterwards. The output files are
  well formed and short, and the panic says why.
- What happens to the statistics the acquisition side had accumulated when it
  panics? They are lost with its stack, and the output files therefore carry
  only what the output side counted. This is a real gap and it is named rather
  than hidden: a panic is a defect, and a defect that also corrupts the
  accounting is a stronger reason to fix it than one that does not.
- What happens when the source returns a timeout? The capture side continues,
  because `SourceError::is_recoverable` says so. It is not counted as loss,
  because nothing was lost.
- What happens when the source returns `Closed` on the very first call? The run
  ends cleanly with zero packets, every sink is still opened and finished, and
  the output is a well-formed empty capture.
- What happens when a stop is requested before the run starts? The run ends
  immediately with the stop reason and produces well-formed empty output.
- What happens when a stop is requested while a source call is blocked? The
  stop takes effect after that call returns. Cooperative by design; the latency
  bound is the caller's own timeout.
- What happens when the same packet is refused by one sink and accepted by
  another? The refusal is counted once, the accepting sink keeps its copy, and
  the conservation identity is asserted per sink rather than globally.
- What happens to `packets_attributed` and `packets_unattributed` for a packet
  with no flow key? Neither advances. Attribution was not attempted, which
  section 8.4 and the S02 vocabulary already distinguish from attempted and
  unresolved.

## Requirements *(mandatory)*

### Functional Requirements

**Composition, section 8.6**

- **FR-001**: The pipeline MUST live in `fragcap-core` and take no
  platform-specific dependency, no I/O crate, and no asynchronous runtime.
- **FR-002**: The pipeline MUST be constructible from a boxed `PacketSource`, a
  boxed `FlowAttributor`, and a collection of boxed `Sink` values.
- **FR-003**: The pipeline MUST NOT require any sink, source, or attributor to
  name another's concrete type.
- **FR-004**: The pipeline MUST run acquisition and output on separate threads,
  per section 8.6.
- **FR-005**: The pipeline MUST perform header parsing on the acquisition side,
  before the packet enters the buffer.
- **FR-006**: The pipeline MUST perform attribution lookup on the acquisition
  side, using the packet's own timestamp rather than the present moment.
- **FR-007**: The pipeline MUST document the control thread seam at the
  construction site, and MUST NOT require a `ProcessWatcher` or a filter
  manager to exist.
- **FR-008**: The pipeline MUST accept the interface address set that direction
  determination requires, per section 12.6.
- **FR-009**: The pipeline MUST NOT introduce a new runtime dependency.

**The bounded buffer, section 12.4**

- **FR-010**: The buffer MUST be bounded, with a default capacity of 65,536
  packets.
- **FR-011**: The capacity MUST be configurable, and a capacity of zero MUST be
  rejected at construction with a named error.
- **FR-012**: When the buffer is full, it MUST evict the oldest buffered packet
  to admit the newest.
- **FR-013**: The producer MUST NOT wait for the consumer to remove an item.
- **FR-014**: The buffer MUST deliver items in the order they were admitted.
- **FR-015**: The consumer MUST wait without spinning when the buffer is empty
  and the run has not ended.

**Drop accounting, section 12.4 and constitution P-4**

- **FR-016**: Each eviction MUST advance `buffer_dropped` by exactly one.
- **FR-017**: Each `SinkError::Full` returned by a sink MUST advance
  `sink_dropped` by exactly one, counted once per refusing sink rather than
  once per packet.
- **FR-018**: The pipeline MUST relay `kernel_dropped` and `interface_dropped`
  from the source unaltered, and MUST NOT fold them into any counter of its
  own.
- **FR-019**: The pipeline MUST advance `packets_captured` once per packet
  accepted from the source.
- **FR-020**: The pipeline MUST advance exactly one of `packets_attributed` or
  `packets_unattributed` for a packet that produced a flow key, and neither for
  a packet that did not.
- **FR-021**: The pipeline MUST accumulate the parse counters from the header
  parser into the run's statistics.
- **FR-022**: For every sink, the packets that sink received plus
  `buffer_dropped` plus that sink's refusals MUST equal `packets_captured`.
- **FR-023**: The statistics passed to `Sink::finish` MUST be the run's own
  final values, including counters produced on the output side.
- **FR-024**: The pipeline MUST NOT add a discard path that has no named
  counter.

**Fan-out and sink failure**

- **FR-025**: Every admitted packet MUST be offered to every sink that has not
  been retired.
- **FR-026**: A `SinkError::Full` MUST be counted and the sink MUST remain in
  service.
- **FR-027**: A sink error for which `is_countable` is false MUST retire that
  sink, and MUST NOT by itself end the run while another sink remains in
  service.
- **FR-027a**: Every packet admitted after a sink is retired MUST advance
  `sink_dropped` once for that retired sink.
- **FR-027b**: The run MUST end when every sink has been retired.
- **FR-027c**: The run MUST report every-sink-retired as the end reason only
  when that is what ended acquisition. An ending acquisition reached on its own
  first, by exhaustion or by a terminal source failure, MUST be reported
  instead, with the retirements still named in the report.
- **FR-028**: Every sink MUST be flushed and finished, retired or not, and
  every non-countable failure MUST be named in the report alongside the index
  of the sink that produced it.
- **FR-028a**: An error returned by `flush` or `finish` MUST be recorded in the
  report as a failure of that sink. It cannot retire the sink, because there
  are no further writes, and it MUST NOT prevent the remaining sinks from being
  flushed and finished.

**Shutdown**

- **FR-029**: The pipeline MUST provide a cloneable stop handle that ends the
  run cooperatively.
- **FR-030**: On any ending, every packet remaining in the buffer MUST be
  offered to the sinks before any sink is finished.
- **FR-031**: On any ending, every sink MUST be flushed and then finished
  exactly once.
- **FR-032**: The run MUST report why it ended, distinguishing source
  exhaustion, an operator stop, a terminal source failure, and every sink
  having retired.
- **FR-033**: A recoverable source error MUST NOT end the run and MUST NOT be
  counted as loss.
- **FR-033a**: The output side MUST observe an ending when the acquisition side
  terminates for any reason, including an unwinding panic, and MUST NOT wait
  indefinitely for a terminal item.
- **FR-033a1**: The acquisition side MUST observe an ending when the output side
  terminates for any reason, including an unwinding panic, and MUST NOT continue
  acquiring indefinitely. The obligation is symmetric with FR-033a and matters
  for the same reason: a source that never closes on its own, which is every
  live source, would otherwise keep the run alive forever and the panic would
  never reach the caller.
- **FR-033b**: When the acquisition side panics, the run MUST drain, flush, and
  finish every sink before the panic reaches the caller, and MUST re-raise the
  panic rather than reporting it as an end reason.
- **FR-033c**: The run MUST consume the pipeline and block until the run ends.

**Reporting**

- **FR-034**: The run MUST return a report carrying both the final statistics
  and the end reason, and the report MUST be marked so that discarding it is a
  warning.
- **FR-035**: The report MUST carry the statistics regardless of how the run
  ended.
- **FR-036**: The report MUST offer a conversion to an ordinary `Result` for
  callers that want failure to propagate.

**Fidelity, constitution P-9**

- **FR-037**: The pipeline MUST NOT alter any observed field of a packet.
- **FR-038**: The pipeline MUST NOT reorder packets relative to their arrival
  from the source.
- **FR-039**: A packet with no flow key MUST be written and marked, never
  dropped.
- **FR-040**: The pipeline MUST NOT reassemble, normalize, or repair anything
  the source produced.

**House rules**

- **FR-041**: The glossary MUST gain entries for pipeline, bounded buffer,
  drop-oldest, capture thread, sink thread, and fan-out in this change, and the
  existing `Backpressure` entry MUST cross-reference drop-oldest, per
  constitution P-6.
- **FR-041a**: The pipeline MUST NOT emit log output and MUST NOT introduce a
  logging facade. The statistics and the end reason are its whole reporting
  surface.
- **FR-042**: The end-to-end corpus test MUST live in the `fragcap` facade, and
  no dev-dependency between `fragcap-capture`, `fragcap-attr`, and
  `fragcap-sink` may be introduced.
- **FR-043**: Neither writer's one-interface restriction may be lifted or
  worked around by this slice.

### Key Entities

- **Pipeline**: Owns a source, an attributor, a parser, and a set of sinks, and
  runs them to a defined ending. The unit of composition specification section
  8.6 describes.
- **Pipeline configuration**: Buffer capacity, the source read timeout, and the
  interface address set. Everything an operator can vary that does not change
  which components are attached.
- **Bounded buffer**: A fixed-capacity queue with drop-oldest eviction, a
  non-waiting producer, and a waiting consumer. Carries packets and exactly one
  terminal item.
- **Stop handle**: A cloneable request to end the run, observed cooperatively
  by the acquisition side.
- **Pipeline report**: The final statistics and the reason the run ended,
  returned together so neither can be read without the other.
- **End reason**: Source exhaustion, an operator stop, a terminal source
  failure, or every sink having retired. The vocabulary the report answers in.
- **Sink failure record**: The index of a sink that returned a non-countable
  error and the error it returned. Carried by the report independently of the
  end reason, because a sink can retire without ending the run.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All eight corpus fixtures run end to end through the pipeline and
  reproduce both committed goldens byte for byte.
- **SC-002**: A test forces `buffer_dropped` above zero by holding a sink, and
  the assertion does not depend on a sleep or on thread scheduling.
- **SC-003**: A test forces `sink_dropped` to an exact expected value by
  refusing a known set of packets.
- **SC-004**: The conservation identity holds in every test that runs the
  pipeline, including those that force drops.
- **SC-005**: A capture whose sink never drains still reaches source exhaustion
  on the acquisition side.
- **SC-006**: Each of the four end reasons is produced by a test and named in
  the report, and a retirement that does not end the run is exercised
  separately.
- **SC-007**: The workspace still has exactly one runtime dependency after this
  slice.
- **SC-008**: `cargo xtask ci` passes, including the dependency direction check
  and the conventions linter.
- **SC-009**: Running the same fixture twice produces byte-identical output.
- **SC-010**: `fragcap-core` still builds for a target with no capture backend.
- **SC-011**: An acquisition-side panic is exercised by a test that asserts the
  output side ended, the sinks were finished, and the panic still reached the
  caller.
- **SC-011a**: An output-side panic is exercised against a source that never
  closes on its own, so a test that hangs is the failure. A finite source cannot
  detect this and must not be used for it.
- **SC-012**: No test in this slice asserts a throughput or latency number, and
  no benchmark is added.

## Assumptions

- The replay source signals exhaustion with `SourceError::Closed` and never
  with `Ok(None)`, as S04 established and documented.
- The scripted attributor resolves against the packet's timestamp, as S04
  established.
- Both writers already produce byte-stable output for a given input, as S06 and
  S07 established with their goldens, so a change in the goldens indicates a
  change in the driving loop rather than in a writer.
- The corpus fixtures are small enough that a default-capacity buffer never
  evicts during the golden comparison, so the goldens test the composition
  rather than the eviction policy.
- The operator, not the pipeline, decides how many sinks to attach and what
  they write to.
- No packet in the corpus carries an interface identifier, because the type
  does not have one until S09.

## Out of Scope

- Live capture, multiple interfaces, and the interface identifier on a packet
  (S09). Both writers keep their one-interface restriction.
- The control thread's contents: the process watcher and process tree (S11),
  stage matching and session lifecycle (S12), and the filter manager (S13).
  Only the seam is established.
- The section 12.7 session anchor, which needs a capture driver clock.
- Ring mode and triggers (S16), transports and streaming sinks (S15), and the
  command line interface (S14).
- Any change to the five behavioral traits in `fragcap-core`. The design
  deliberately avoids needing one; see the plan's note on `PacketSource` and
  `Send`, which S09 will have to resolve when it puts one thread on each
  interface.
- Logging, tracing, and any observability surface other than the statistics and
  the end reason. S14 owns the first one, because it is the first slice with an
  operator to report to.
- Throughput and latency targets, and any benchmark. A file replay measures
  disk read speed, and S09 owns the first honest measurement.
- A non-blocking or spawning form of the run entry point. A caller that wants
  one spawns a thread and keeps the stop handle.

## Done When

- [ ] The pipeline runs all eight corpus fixtures end to end and reproduces
      both committed goldens byte for byte.
- [ ] `buffer_dropped` and `sink_dropped` are each demonstrated non-zero by a
      test that forces the condition rather than constructs the value.
- [ ] The conservation identity is asserted in every pipeline test.
- [ ] All four end reasons are exercised, and an acquisition-side panic is
      exercised separately.
- [ ] The workspace runtime dependency count is unchanged.
- [ ] The glossary carries entries for pipeline, bounded buffer, drop-oldest,
      capture thread, sink thread, and fan-out.
- [ ] `cargo xtask ci` passes in the foreground, watched to completion.
