# Implementation Plan: Socket Table Attributor

**Slice**: S10 | **Branch**: `feat/socket-table-attributor` |
**Date**: 2026-08-09 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from
`specs/010-socket-table-attributor/spec.md`

## Summary

Implement specification section 11 as the production `FlowAttributor`. A socket
table snapshot is read from the platform, joined against captured flows by
5-tuple under a total and documented order, aged into a retention map that
keeps the tail of a closing connection attributed while marking the answer as
inferred, and published as an immutable value that any number of capture
threads read without locking. The platform half sits behind a feature that is
off by default; every rule is exercised at tier 1 against declared tables and
an injected clock.

The technical approach is settled in [research.md](research.md). Its five
decisions: read both tables by owning module so that both carry a socket
creation instant, name processes by toolhelp enumeration so that no process
handle is opened at all, publish through `arc-swap` because section 11.6 forbids
a reader being blocked by a writer, pin `windows-sys` to the version `pcap`
already resolved so the dependency graph gains nothing, and supply the
publication cell now while leaving the control thread to S13.

## Technical Context

**Language/Version**: Rust 2021, workspace minimum 1.82

**Primary Dependencies**: `arc-swap` (new, runtime), `windows-sys` (new,
runtime, optional and target-gated), `fragcap-core` (existing)

**Storage**: N/A

**Testing**: `cargo test --workspace --locked`, tier 1 throughout; the platform
backend's own tests are tier 2 and do not run in the ordinary check set

**Target Platform**: Windows for the backend; the crate builds anywhere with
the backend absent

**Project Type**: Rust library crate within a workspace

**Performance Goals**: A snapshot costs one to three milliseconds against
roughly 1800 sockets, per Appendix D, against a one-second cadence. A lookup
performs one atomic load and a bounded scan and calls into no operating system
interface.

**Constraints**: A lookup must not block a publication and a publication must
not block a lookup; the ordinary check set must pass with no platform present;
no process handle may be opened.

**Scale/Scope**: Roughly 1800 sockets per snapshot, one snapshot per second,
one process-name enumeration per snapshot.

## Constitution Check

*GATE: passed before Phase 0, re-checked after Phase 1.*

| Principle | Verdict | Basis |
| --- | --- | --- |
| P-1, passive observation | **Pass, and strengthened** | Attribution is reconstructed from the IP Helper socket table and query-only process enumeration, both on the section 19.2 permitted list. Research R-2 chose the toolhelp path specifically because it opens no process handle at all, so the "state the access rights at the call site" rule has nothing to apply to. `cargo xtask lint` gains an assertion that this slice opens none. |
| P-2, core stays neutral | **Pass** | `fragcap-core` gains one trait bound and loses a mutex. It acquires no dependency. Both new dependencies land in `fragcap-attr`, and `windows-sys` is optional and target-gated. |
| P-3, capture and attribution separate | **Pass** | Nothing here acquires a packet. `fragcap-attr` gains no edge to `fragcap-capture`; `cargo xtask deps` proves it. |
| P-4, no silent loss | **Pass** | This slice introduces no discard path. An unresolved lookup returns no attribution, the packet is retained, and the existing `packets_unattributed` counter records it. The distinction between never attempted and attempted and unresolved is preserved (FR-026), which is the specific thing S07 lost once. |
| P-5, compatibility outranks richness | **Not engaged** | No output format changes. SC-013 asserts the corpus goldens are unchanged. |
| P-6, glossary first | **Pass, with work** | Six terms enter with this slice: socket table, socket table entry, attribution index, retention window, refresh trigger, and dual-stack socket. Entries are written in the same change. |
| P-7, wrappers stay thin | **Not engaged** | No wrapper changes. |
| P-8, house standards | **Pass** | UTF-8 without BOM, LF, no dashes, `cargo xtask lint`. |
| P-9, the instrument does not lie | **Pass, and load-bearing** | Three places. Fidelity is supplied by the resolving path and never inferred, so a retained answer is visibly retained. A failed table read leaves the previous snapshot rather than publishing an empty one, because publishing empty would silently unattribute everything after a transient failure. And an attribution is produced when the naming seam supplies nothing, carrying the observed identifier, because the identifier is what was observed. |

**Licensing.** `arc-swap` and `windows-sys` are both MIT or Apache-2.0 across
their graphs, which is inside the allowlist. `windows-sys` at the resolved
version adds no package to `Cargo.lock`; `arc-swap` adds exactly one, with no
dependencies of its own. Neither supplies a capability on the denylist, and
`windows-sys` is a binding rather than a driver.

**Complexity.** No violations to justify. The table below is empty by design.

## Project Structure

### Documentation (this feature)

```text
specs/010-socket-table-attributor/
├── plan.md              # This file
├── spec.md
├── research.md          # Phase 0
├── data-model.md        # Phase 1
├── quickstart.md        # Phase 1
├── contracts/
│   └── attribution-api.md
├── checklists/
│   ├── requirements.md
│   └── attribution.md
└── tasks.md             # Phase 2, /speckit-tasks
```

### Source code

```text
crates/fragcap-core/src/
├── traits.rs                       # FlowAttributor gains Sync
└── pipeline/mod.rs                 # the per-packet attributor mutex goes

crates/fragcap-attr/
├── Cargo.toml                      # arc-swap; windows-sys behind `socket-table`
└── src/
    ├── lib.rs                      # module wiring, narrative
    ├── script.rs                   # unchanged, S04
    ├── scripted.rs                 # unchanged, S04
    ├── seam.rs                     # Clock, SocketTableSource, ProcessNamer
    ├── table.rs                    # SocketTableEntry, SocketTable
    ├── index.rs                    # AttributionIndex, PublishedIndex, matching
    ├── schedule.rs                 # RefreshSchedule
    ├── socket.rs                   # SocketTableAttributor
    └── platform/
        ├── mod.rs                  # cfg gate
        ├── iphelper.rs             # IpHelperTable
        └── toolhelp.rs             # ToolhelpNamer

crates/fragcap/tests/
└── attribution.rs                  # the facade's end-to-end tier 1 test

docs/glossary.md                    # six entries

.github/workflows/platform.yml      # path filters and a build step
xtask/src/lint.rs                   # the OpenProcess assertion
xtask/src/neutral.rs                # fragcap-attr added to the neutral build
```

**Structure Decision**: The attributor lives in `fragcap-attr`, which is where
specification section 8.3 puts it and where `ScriptedAttributor` already is.
The matching logic lives in `index.rs` on an immutable value rather than on the
attributor, because that is what makes every rule in FR-005 through FR-010
testable as a pure function of a declared table and a declared instant.

The end-to-end test lives in the `fragcap` facade for the reason AGENTS.md
records for S06 and S07: the facade is the only crate that legitimately depends
on both sides, and a dev-dependency between siblings would slip past
`cargo xtask deps`, which ignores `[dev-dependencies]` by design.

## Approach, in the order it will be built

**1. The bound, and the mutex.** `FlowAttributor: Send + Sync` in `traits.rs`,
and the pipeline stops locking. This lands first because it is the smallest
change with the widest blast radius, and because a workspace that compiles
after it proves the claim in research R-5 that both existing implementors are
already `Sync`.

**2. The immutable half.** `table.rs` and `index.rs`: entries, tables, the four
exactness ranks, the creation-instant filter, the total order, retention
lookup, and fidelity. Every requirement from FR-001 to FR-010 and FR-018
through FR-023 is a pure function here, tested against declared values.

**3. The schedule.** `schedule.rs`: the interval, the two triggers, and the
rate limit, over an injected instant. FR-011 through FR-017.

**4. Publication.** `PublishedIndex` in `index.rs`, and the concurrency test
that publishes from one thread while several resolve. FR-027 through FR-030.

**5. The attributor.** `socket.rs` composes them and implements
`FlowAttributor`. This is where `refresh` ages retention against the new table,
resolves names, builds an index, and publishes it, and where a read failure
leaves the previous index alone.

**6. The platform backend.** `platform/`, behind the `socket-table` feature,
target-gated. Tier 2; it is compiled in the check set only when the feature is
on, and its behavioral tests need a machine.

The feature is deliberately not `live`. That name is already taken by
`fragcap-capture` and means "links against the npcap import library", which
this backend does not: the IP Helper API ships with the operating system. The
analyze gate found the collision, and the consequence of missing it would have
been a socket table backend that could not be built without a capture driver
software development kit it never calls.

**7. The facade test.** A pipeline driven by a `SocketTableAttributor` over a
declared table, reproducing an attribution end to end with no capture driver.

**8. Glossary, changelog, and the deviation records.**

## Test strategy

Every requirement except FR-034 through FR-036 is tier 1. Three properties are
worth naming because they are the ones a weak test suite would miss.

**Determinism under permutation.** SC-014 resolves the same flow against the
same entries in several orders and asserts one answer. A matcher that iterates
and takes the first hit passes an ordinary test and fails this one.

**No sleeping.** SC-005 drives the whole cadence through the test clock. If any
test in this slice sleeps, the injected clock has not been threaded all the way
through and the deviation that justified it has not paid for itself.

**Concurrency across a publication.** SC-006 spawns readers and a publisher and
asserts that every answer corresponds to some whole index. This is the only
test in the slice that can be flaky, so it runs a bounded number of iterations
with the publisher alternating between two indices whose answers are distinct,
and asserts that each observed answer is one of the two rather than a mixture.

**Fidelity, asserted for the same endpoint across its whole life.** SC-002
takes one endpoint from present, to retained, to expired, in one test rather
than three, because three separate tests can each pass while the transitions
between them are wrong.

## Risks

**The concurrency test is the one that can be flaky.** Mitigated by asserting a
property that holds for any interleaving rather than a specific interleaving,
and by bounding the iteration count so a stuck test fails rather than hangs.

**`arc-swap` is a new runtime dependency and the first the project has taken
for a concurrency property.** The alternative analysis is in research R-3 and
the counter-argument is named there: anyone proposing to remove it must answer
whether a reader may be blocked by a writer, not whether a read lock is fast.

**Two of the three deviations touch a surface intended to reach 1.0.0
unchanged.** The `Sync` bound is a bound rather than a method, and both
existing implementors satisfy it, which is the same argument S09 made and the
reason the change is small. The other two are contents of new types in this
crate and touch no declared surface.

**The platform backend cannot be verified here.** It compiles in continuous
integration when the feature is enabled and it has never read a real table. The
slice will say so rather than implying otherwise, exactly as S09 said its live
source had linked but never run.

**`.github/workflows/platform.yml` is a pinned artifact and this slice changes
it.** The change is small, its reason is that nothing would otherwise ever
compile the new backend, and a dated decision is recorded in `changelog.d`. It
surfaces at the pre-push halt like everything else.

## Complexity Tracking

No constitution violations require justification.
