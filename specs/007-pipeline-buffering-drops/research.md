# Phase 0 Research: Pipeline, Buffering, and Drop Accounting

**Slice**: S08 | **Date**: 2026-08-08 | **Plan**: [plan.md](plan.md)

The specification carried no unresolved `NEEDS CLARIFICATION` marker into this
phase. What follows are the decisions the plan rests on, each with the
alternatives that were evaluated and the reason they lost. They are recorded
here so that a later reader can tell a considered choice from an accident.

## R-1. The buffer mechanism

**Decision**: A `std::collections::VecDeque<Item>` behind a `std::sync::Mutex`,
paired with a `std::sync::Condvar` for the consumer's wait. Capacity is checked
on push; when full, the front is popped and discarded before the new item is
pushed to the back.

**Rationale**: Section 12.4 requires three properties together: bounded,
drop-oldest, and a producer that never waits on the consumer. Nothing available
supplies all three, so the eviction has to be written by hand regardless, and
once it is written by hand the surrounding structure is a queue and a lock.

**Alternatives considered**:

| Candidate | Why it fails |
| --- | --- |
| `std::sync::mpsc::channel` | Unbounded. Cannot drop, so a slow sink becomes unbounded memory growth rather than counted loss. |
| `std::sync::mpsc::sync_channel` with `send` | Bounded and blocks the producer, which is the one behavior section 12.4 forbids by name. |
| `std::sync::mpsc::sync_channel` with `try_send` | Bounded and non-blocking, but fails rather than evicting. That is drop-newest, and section 12.4 chose drop-oldest with a stated reason. |
| `crossbeam-channel` bounded | Same two shapes as the standard library. Would add a runtime dependency and still leave eviction to be written. |
| A lock-free ring | Removes even the bounded critical-section wait. Materially harder to prove correct, and the property it buys is not the one the specification asks for. Reconsider only if measurement in S09 shows the lock is a real cost. |

## R-2. Which thread acquires

**Decision**: `run` acquires packets on the thread it was called from and
spawns the sink thread.

**Rationale**: `PacketSource` has no `Send` bound. `FlowAttributor`,
`ProcessWatcher`, and `Sink` all do, which is the seam's own statement about
which components were expected to cross a thread boundary. Moving the source to
a spawned thread would require adding `Send` to a trait that `traits.rs`
documents as intended to reach 1.0.0 unchanged, and the deviation process
exists for exactly that kind of change. Nothing in this slice needs it.

**Consequence recorded for S09**: Section 12.1 requires one capture handle and
one capture thread per interface. That arrangement cannot borrow the caller's
thread, so S09 will need `PacketSource: Send`. The trait change belongs to the
slice that first requires it, with the deviation recorded for promotion to
specification section 29.

**Alternatives considered**: Adding `Send` to `PacketSource` now, so that the
threading matches section 8.6's diagram literally. Rejected because it changes
the architecture of record to buy a cosmetic correspondence, and because doing
it now would land the change in a slice with no test that needs it.

## R-3. The panic path

**Decision**: The buffer closes when its producer handle is dropped, which
unwinding does. The sink thread therefore observes an ending regardless of how
the acquisition side terminated. A guard holding the sink thread's join handle
joins it on drop, so the sinks are drained, flushed, and finished before the
panic escapes `run`. The panic is then re-raised rather than converted into an
end reason.

**Rationale**: The failure mode being designed against is a deadlock in which
the sink thread waits forever for a terminal item that a panicking producer
will never send. Tying the buffer's closure to handle drop makes that
impossible without any panic-specific code path, which is the property worth
having: the correctness does not depend on remembering to handle the panic
case. Re-raising rather than reporting keeps a defect filed as a defect;
converting it into an `EndReason` would file it under an accounting category
that means something else, and P-4's counters would then be describing a
program that was not running correctly.

**Known gap, named rather than hidden**: the acquisition side's counters live
in its stack frame and are lost when it unwinds. The output files therefore
carry only what the sink side counted. This is stated in the specification's
edge cases. The eviction count is deliberately not in that frame; see R-5.

**Alternatives considered**: `catch_unwind` around the acquisition loop, so the
counters could be recovered. Rejected: it requires `UnwindSafe` bounds on trait
objects the seams do not carry, and it converts a defect into a recoverable
condition, which is the outcome the decision above rejects.

## R-4. Sink failure policy

**Decision**: A sink returning an error for which `SinkError::is_countable` is
false is retired. Every subsequent packet advances `sink_dropped` once for that
sink. Other sinks keep receiving. The run ends when the source ends or when
every sink has retired. Every sink is flushed and finished, retired or not.

**Rationale**: This decision was made twice, and the first answer was wrong in
a way worth recording. The first answer was that a non-countable error stops
the run, on the reasoning that retiring a sink would leave its missing writes
recorded nowhere, since `CaptureStats` has no counter for a retired sink.

Following that through shows it does not hold. Stopping the run does not remove
the packets that were already buffered or that arrive during the wind-down; it
just leaves them with nowhere to go and no counter, which P-4 calls a defect.
Both options need the same counter, and section 12.4 already supplies it:
`sink_dropped` is "dropped by a sink that could not accept", and a failed sink
is a sink that cannot accept. With that reading, retirement conserves exactly,
needs no new counter, and keeps a capture running when one of several outputs
dies, which is what a capture tool should do.

`SinkError::is_countable` still draws the line. What it decides is whether the
sink survives the packet, not whether the run survives the sink.

**Alternatives considered**: Adding a `sink_retired` counter to `CaptureStats`.
Rejected: it would be a second name for a loss `sink_dropped` already names,
and two counters for one cause is what the stats module's own documentation
warns against.

## R-5. Where the eviction count lives

**Decision**: In the buffer's shared state, not in the producer's stack.

**Rationale**: The consumer can then read it regardless of how the producer
terminated, including an unwinding panic. Keeping it producer-side would lose
the count in precisely the situation where an operator most needs to know that
packets went missing.

**Alternatives considered**: An `AtomicU64` beside the buffer. Equivalent in
effect and one more thing to keep consistent with the queue it describes; the
count is already only touched under the lock that guards the eviction.

## R-6. The terminal item and the capacity bound

**Decision**: The terminal item carrying the acquisition side's final counters
is pushed without regard to capacity.

**Rationale**: Evicting an observed packet to make room for a bookkeeping
marker would be loss caused by fragcap's own shutdown. P-4 would then require
counting it, and the operator would be reading a drop counter that describes
the tool rather than the capture. Exempting one item is a smaller anomaly than
that, and the queue holds at most capacity plus one, with the extra never a
packet.

## R-7. Test strategy for the drop paths

**Decision**: Three layers, none of them timing-dependent.

1. Buffer unit tests in `fragcap-core`, single-threaded, asserting eviction,
   ordering, the count, and the terminal item's exemption directly.
2. Pipeline tests in `fragcap-core` with in-crate stubs: a source that yields a
   scripted sequence and then `Closed`, an attributor that answers from a map,
   and sinks that can be told to refuse specific indices, to fail
   non-countably at a specific index, or to block until the test releases them.
3. The corpus end-to-end test in the `fragcap` facade, comparing both writers'
   output against the committed goldens.

**Rationale**: A drop test that sleeps is a test that passes on a fast machine
and fails on a loaded one, and the failure looks like a defect in the code. The
blocking sink is released by the test itself, so the interleaving is fixed by
construction rather than hoped for. The assertion that survives every
interleaving is the conservation identity, which is why it is asserted in every
pipeline test rather than in one dedicated to it.

**Alternatives considered**: `loom` for exhaustive interleaving exploration.
Rejected for this slice: it would be a dev-dependency, the structure under test
is a single mutex-guarded queue rather than a lock-free algorithm, and the
properties that matter are expressible as invariants that hold without needing
to enumerate schedules. Worth revisiting if a lock-free ring is ever adopted
per R-1.

## R-8. Default read timeout

**Decision**: 100 milliseconds.

**Rationale**: The replay source ignores the timeout entirely, so the value is
inert for every test in this slice and matters only to S09. It sets the upper
bound on how long a stop request waits, which the specification requires be
stated rather than hidden. 100 milliseconds is short enough that a stop feels
immediate to an operator and long enough that an idle capture is not spinning
through wakeups.

**Alternatives considered**: Requiring the caller to supply it with no default.
Rejected: every caller would pick a number for the same reason and the default
documents the reasoning once.
