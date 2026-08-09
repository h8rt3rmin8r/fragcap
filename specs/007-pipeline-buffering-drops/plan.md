# Implementation Plan: Pipeline, Buffering, and Drop Accounting

**Branch**: `feat/pipeline-buffering-drops` | **Date**: 2026-08-08 | **Spec**:
[spec.md](spec.md)

**Input**: Feature specification from
`/specs/007-pipeline-buffering-drops/spec.md`

**Slice**: S08 (specification sections 8.6 and 12.4)

## Summary

Add a `pipeline` module to `fragcap-core` that composes a `PacketSource`, a
`FlowAttributor`, the S03 header parser, and a set of `Sink` values, and runs
them across two threads with a bounded drop-oldest buffer between them. The
module is what finally produces the `CaptureStats` values the writers have been
handed by hand since S06.

Two decisions shape everything else.

**The calling thread is the capture thread.** `PacketSource` carries no `Send`
bound, and the other three cross-thread traits do. Rather than change a trait
the project intends to carry to 1.0.0 unchanged, `run` acquires on the thread
it was called from and spawns the sink thread. The data flow is identical to
section 8.6; only which thread is borrowed and which is spawned differs, and
the choice costs nothing here. It does not stay free: section 12.1 puts one
thread on each interface, so S09 will need `PacketSource: Send` and should
carry that trait change with the slice that first requires it.

**A failed sink is retired, not fatal.** Every packet after the failure counts
one `sink_dropped` for the retired sink, which is exactly what section 12.4
defines that counter as. The run ends when the source ends or when every sink
has retired. This is a correction to the spec's first answer, made during
planning and written back into the spec; the reasoning is recorded in the
clarification and in research decision R-4.

No runtime dependency is added. The buffer is a `VecDeque` behind a `Mutex` and
a `Condvar`, because no channel in the standard library or outside it offers
drop-oldest, and every candidate would still have left the eviction to be
written by hand.

## Technical Context

**Language/Version**: Rust, edition 2021. Toolchain 1.96.0; minimum 1.82.

**Primary Dependencies**: None added. `std::thread`, `std::sync::{Arc, Mutex,
Condvar}`, `std::sync::atomic::AtomicBool`, `std::collections::VecDeque`. The
workspace stays at one runtime dependency (`bytes`) and one dev-dependency
(`serde_json`).

**Storage**: None. Sinks own their destinations.

**Testing**: `cargo test --workspace --locked`. Three layers: unit tests on the
buffer in `fragcap-core`, pipeline tests in `fragcap-core` driven by in-crate
stubs that can be made to fail on demand, and the corpus end-to-end test in the
`fragcap` facade comparing against the S06 and S07 goldens.

**Target Platform**: Any target the standard library supports. Threads and
mutexes are not platform-specific in the sense P-2 uses the term; `cargo xtask
neutral` proves it.

**Project Type**: Library crate within a Cargo workspace.

**Performance Goals**: None set, deliberately. The only source available is a
file replay, so a measurement here would describe disk read speed. Section 12.4
states a structural property rather than a number, and the property is asserted
directly.

**Constraints**: The producer never waits on sink progress. Order is preserved.
Output stays byte-identical to the committed goldens. No test may depend on a
sleep or on a particular thread interleaving.

**Scale/Scope**: One new module in one crate, two files. Default buffer 65,536
packets. Largest fixture is 400 packets, so the corpus never evicts.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Applies how | Status |
| --- | --- | --- |
| P-1 Passive observation | The pipeline opens no handle and reads no process. It moves packets between components. | Pass, not engaged |
| P-2 Core stays platform-neutral | The pipeline lives in `fragcap-core` and uses only `std` concurrency. No dependency added. `cargo xtask neutral` and the `core builds without a capture backend` workflow prove it. | Pass |
| P-3 Capture and attribution separate | The pipeline holds both behind trait objects and neither names the other. The corpus test lives in the facade so no dev-dependency edge is created between siblings. | Pass |
| P-4 No silent loss | The point of the slice. Eviction advances `buffer_dropped`, every refusal and every write a retired sink did not receive advances `sink_dropped`, backend counters are relayed unaltered, and the conservation identity is asserted in every pipeline test. | Pass, and load-bearing |
| P-5 Compatibility outranks richness | No format change. The goldens are expected to reproduce unchanged, which is the assertion. | Pass, not engaged |
| P-6 Glossary first | Six entries added: pipeline, bounded buffer, drop-oldest, capture thread, sink thread, fan-out. `Backpressure` gains a cross-reference. | Pass |
| P-7 Wrappers stay thin | No wrapper touched. | Pass, not engaged |
| P-8 House standards | `CONVENTIONS.md` applies; `cargo xtask lint` enforces it. | Pass |
| P-9 The instrument does not lie | The pipeline alters no field, preserves order, and retains packets with no flow key. Drop-oldest is a declared omission counted under P-4, and the loss of acquisition-side counters during a panic is named rather than hidden. | Pass |

Post-design re-check: unchanged. No principle required a justification, and the
Complexity Tracking table below is empty.

## Project Structure

### Documentation (this feature)

```text
specs/007-pipeline-buffering-drops/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/
│   └── pipeline-api.md  # Phase 1 output
├── checklists/
│   ├── requirements.md
│   └── accounting.md
├── spec.md
└── tasks.md             # /speckit-tasks output
```

### Source Code (repository root)

```text
crates/
├── fragcap-core/
│   ├── src/
│   │   ├── lib.rs               # module declaration and re-exports
│   │   ├── pipeline/
│   │   │   ├── mod.rs           # Pipeline, config, report, end reason, run loop
│   │   │   └── buffer.rs        # bounded drop-oldest buffer, producer and consumer
│   │   ├── stats.rs             # unchanged
│   │   └── traits.rs            # unchanged
│   └── tests/
│       └── no_alloc.rs          # unchanged
└── fragcap/
    └── tests/
        ├── common/mod.rs        # gains a pipeline runner helper
        ├── goldens.rs           # unchanged assertions, now driven by the pipeline
        └── corpus_pipeline.rs   # new: corpus end to end through the real pipeline

docs/
└── glossary.md                  # six new entries, one cross-reference

changelog.d/
├── S08-pipeline-buffering-drops.added.md
└── S08-pipeline-buffering-drops.decisions.md
```

**Structure Decision**: A new `pipeline` module inside `fragcap-core`, split
into `mod.rs` for the composition and `buffer.rs` for the queue. The split is
worth making because the buffer has an eviction and ordering contract testable
on its own, with no source, attributor, or sink involved, and a bug there would
otherwise only be visible through the whole pipeline.

The corpus test is a new file in the facade rather than an extension of
`goldens.rs`, so that the existing golden assertions stay readable as what they
are: a statement about the writers. `corpus_pipeline.rs` is a statement about
the composition, and it asserts against the same goldens.

## Design decisions

**D-1. The calling thread acquires; the sink thread is spawned.** Section 8.6
draws a capture thread and a sink thread without saying which is the caller's.
`PacketSource` has no `Send` bound while `FlowAttributor`, `ProcessWatcher`,
and `Sink` all do, so moving the source to a spawned thread would require
adding `Send` to a trait that `traits.rs` documents as intended to reach 1.0.0
unchanged. Acquiring on the caller's thread needs no such change. It also
happens to make the panic path simpler: an unwinding caller drops the producer,
which closes the buffer, and a join-on-drop guard makes the sink thread finish
its work before the panic escapes.

This is a deferral, not a dodge. Section 12.1 requires one capture thread per
interface, so S09 cannot use this arrangement and will need `PacketSource:
Send`. Recorded here so that slice finds the reasoning rather than rediscovering
it, and recorded for promotion to specification section 29.

**D-2. The buffer is a `VecDeque` behind `Mutex` and `Condvar`.** Research R-1
records the alternatives and why each fails. The property the code claims is
"the producer never waits for the consumer to make progress", not "the producer
never blocks", because the second is false of any shared structure and would be
a claim the implementation could not honor. The distinction is the substance of
the claim: the producer's wait is bounded by a critical section that pushes and
possibly pops, held by a consumer that is itself never waiting on anything
outside the buffer.

**D-3. Eviction is counted inside the buffer, not on the capture thread.** The
count lives in the shared state, so the consumer can read it even if the
producer died unwinding. Keeping it in the producer's stack frame would lose it
in exactly the case where the operator most needs to know packets went missing.

**D-4. The terminal item is exempt from the capacity bound.** When the
acquisition side finishes, it pushes one terminal item carrying its final
counters. Subjecting that to eviction would discard an observed packet to make
room for fragcap's own bookkeeping, which would be a loss caused by the tool's
shutdown rather than by a slow sink. The queue may therefore momentarily hold
capacity plus one, and the extra item is never a packet.

**D-5. `sink_dropped` counts per refusal, and a retired sink keeps accruing.**
Section 12.4 defines the counter as "dropped by a sink that could not accept".
A sink that has failed cannot accept, so every subsequent packet it does not
receive is one of these. This is what makes retirement conserve exactly and is
why no new counter is needed. Counting per packet rather than per refusal would
report one loss where three outputs are short and would shrink as the fan-out
widened.

**D-6. The report is unconditional and the `Result` is derived from it.** `run`
returns a `#[must_use] PipelineReport` carrying the statistics, the end reason,
and any sink failures. `into_result` converts it for callers that want failure
to propagate. A bare `Result<CaptureStats, PipelineError>` would discard the
accounting on the error path, which is the one path where P-4 matters most.

**D-7. The capture side owns the attributor outright.** Section 8.6 has the
control thread publishing a snapshot the capture thread reads. That needs a
publisher, a snapshot type, and `Sync` on the attributor, none of which exist
before S10. `FlowAttributor: Send` is exactly the bound that permits owning it
on the acquisition side, and the seam is documented at the construction site.

**D-8. Statistics reach the sink through the buffer, not through shared
atomics.** The terminal item carries the acquisition side's final counters. The
sink thread adds the buffer's eviction count and its own refusals and passes
the sum to `Sink::finish`. Reading shared atomics at an arbitrary instant could
hand `finish` a value that was never true of any moment, which is a P-9 problem
rather than only a race.

**D-9. No test depends on timing.** The drop tests use a gate the test itself
controls rather than a sleep: a stub sink that blocks on a channel the test
releases, and a stub sink that refuses a predetermined set of indices. The
assertions are the conservation identity and exact counts, both of which hold
under every interleaving.

## Complexity Tracking

No constitution violations. Table intentionally empty.
