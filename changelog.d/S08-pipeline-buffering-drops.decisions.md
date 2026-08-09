### 2026-08-08: The calling thread acquires, and the sink thread is spawned

Specification section 8.6 draws a capture thread and a sink thread without
saying which one is the caller's. `PacketSource` carries no `Send` bound, while
`FlowAttributor`, `ProcessWatcher`, and `Sink` all do, which is the seam's own
record of which components were expected to cross a thread boundary. Moving the
source to a spawned thread would require adding `Send` to a trait
`fragcap-core::traits` documents as intended to reach 1.0.0 unchanged.

Acquiring on the caller's thread needs no such change, and the data flow is
identical.

**This is a deferral with an owner.** Section 12.1 requires one capture handle
and one capture thread per interface, which this arrangement cannot express. S09
will need `PacketSource: Send` and should carry the trait change with the slice
that first requires it, through the deviation process, for promotion to
specification section 29. The limitation is visible today: a `Pipeline` cannot
be moved to another thread, which the slice's own tests had to work around.

### 2026-08-08: A failed sink is retired rather than fatal, reversing this slice's first answer

Recorded as a reversal because the first answer was written into the spec and
then found wrong during planning, and the reasoning is worth keeping.

The first answer read `SinkError::is_countable`, documented in S02 as "whether
the pipeline should count this and carry on rather than stop", as meaning a
non-countable error stops the whole run. Retiring the failed sink and
continuing was rejected on the grounds that the retired sink's missing writes
would be recorded nowhere, `CaptureStats` having no counter for a retired sink.

That does not survive being followed through. Stopping the run does not remove
the packets already buffered or still arriving; it leaves them with nowhere to
go and no counter, which constitution P-4 calls a defect. Both options need the
same counter, and section 12.4 already supplies it: `sink_dropped` is "dropped
by a sink that could not accept", and a failed sink is a sink that cannot
accept.

Retirement therefore needs no new counter, conserves exactly, and keeps a
capture running when one of several outputs dies. `is_countable` still draws
the line; what it decides is whether the sink survives the packet, not whether
the run survives the sink. S15 may revisit when a streaming sink whose failure
is routine exists.

### 2026-08-08: No runtime dependency for the bounded buffer

Section 12.4 requires bounded, drop-oldest, and a producer that never waits,
together. `std::sync::mpsc::channel` is unbounded and cannot drop.
`sync_channel` blocks the producer, which section 12.4 forbids by name, and its
`try_send` fails rather than evicting, which is drop-newest and the wrong
policy. A third-party bounded channel offers the same two shapes and would
still leave the eviction to be written by hand.

The buffer is a `VecDeque` behind a `Mutex` and a `Condvar`. The workspace's
one runtime dependency stays one.

The property claimed is that the producer never waits for the consumer to make
progress, not that it never blocks. The second is false of any shared structure
and would be a claim the code could not honor. The producer's wait is bounded
by a critical section that pushes and at most pops, held by a consumer that is
itself never waiting on anything outside the buffer, so sink slowness is not
expressible as producer latency. A lock-free ring would remove even that wait
and was rejected as materially harder to prove correct for a property the
specification does not ask for.

### 2026-08-08: The terminal item is exempt from the capacity bound

The acquisition side ends by pushing one item carrying its final counters.
Subjecting it to eviction would discard an observed packet to make room for
fragcap's own bookkeeping, and P-4 would then require counting a loss caused by
the tool's shutdown rather than by a slow sink. The queue holds at most capacity
plus one, and the extra item is never a packet.

### 2026-08-08: The eviction count lives in the buffer, not in the producer

So that the consumer can read it however the producer terminated, including an
unwinding panic. A producer-side counter would lose the count in precisely the
case where an operator most needs to know that packets went missing.

### 2026-08-08: A panic is re-raised, never reported as an end reason

The buffer closes when its producer handle drops, which unwinding does, so the
output side observes an ending rather than waiting for a terminal item that will
never arrive. A guard owning both the producer and the output thread's join
handle closes the buffer and then joins it, on every path out of `run`, so the
sinks are drained, flushed, and finished before a panic escapes.

Holding the producer and the join handle separately deadlocks, and did:
locals drop in reverse declaration order, so the guard joined a thread still
waiting on a buffer the producer had not yet closed. The test suite hung until
the two were folded into one guard. Recorded because the ordering is not
obvious from reading either piece alone.

A panic is never converted into an `EndReason`. It is a defect, and filing it
under an accounting category would describe a program that was not running
correctly as though it were. The acquisition side's counters are lost with its
stack, which is a real gap and is documented rather than hidden; the eviction
count survives because it is not kept there.

**The obligation is symmetric, and the first version of this only went one
way.** Review found that a panicking sink unwound the output thread without
telling the acquisition side, which then kept reading until the source closed on
its own. A replay source closes; a live source does not, so `run` would have
acquired forever and never reached the join that re-raises the panic. The
original test used a finite source and could not have caught it. The output
thread now holds a guard that requests the stop however it terminates, and the
test uses a source that never closes, so a regression hangs rather than passing.

### 2026-08-08: An ending acquisition reached on its own outranks a later retirement

Also from review. The end reason was replaced with every-sink-retired whenever
every sink had retired, regardless of what had already ended acquisition. A
source that failed with a `DeviceLost` and a last sink that failed afterwards,
while the output side was still draining, therefore reported the retirement and
buried the device loss, which is the diagnostic an operator most needs.

Retirement now replaces only the stop it requested. An ending acquisition
reached on its own happened first and is the reason. The retirements are
reported either way, in `sink_failures`, so nothing is lost by the narrowing.

### 2026-08-08: The bounded buffer refuses a zero capacity rather than documenting against it

`Pipeline::new` rejects a zero capacity with a named error, and review pointed
out that the crate-private constructor beneath it did not. A zero capacity there
is worse than useless: every push finds the queue full, pops nothing, and still
advances the eviction count, so the buffer grows without bound while reporting
losses that never happened. A counter that lies is the one failure the module
exists to prevent, so the precondition is asserted.

### 2026-08-08: The `malformed` JSON golden was wrong, and driving the writers from the pipeline found it

`fixtures/goldens/malformed.jsonl` claimed `"unattributed":5` for five packets
that produced no flow key. Attribution was never attempted on any of them.
`AttributionState` has distinguished never-attempted from
attempted-and-unresolved since S02, precisely because the two mean different
things to an operator, and `stats.rs` defines `packets_unattributed` as
"retained and marked because attribution did not resolve".

The cause was the S07 corpus helper, which counted with `attribution.is_some()`
and folded the two states together. The writer was faithful; what it was handed
was not. The helper now matches on `attribution_state()`, and the golden's
trailer line is corrected. One field on one line of one golden changed; the
other fifteen goldens reproduce byte for byte through the pipeline, which is
what makes this a finding rather than a format change.

This is the class of defect the end-to-end phase exists to catch, and it is
worth noting that no test caught it for a whole slice: the S07 goldens were
self-consistent, and the wrong number was wrong only against a definition that
lived in another crate.
