# Research: Stage Matching and Session Lifecycle

Decisions taken before implementation, each with the alternatives weighed against
the constitution, the architecture of record, and existing code. Recorded for
promotion to the changelog decisions fragment.

## D-1. Stage binding is written onto the process node in `fragcap-core`

**Decision**: `fragcap-core` gains `ProcessTree::bind_stage(id, StageId) -> bool`
that sets the `stage` field S11 reserved on `ProcessNode`. The `fragcap-profile`
matcher decides the binding; a caller applies it through this method.

**Alternatives**: a side-map `NodeId -> StageId` owned by the matcher, leaving
the tree immutable after the fold.

**Why**: `tree.rs` documents the `stage` field as "the place [10.3 and 10.4]
write to", reserved precisely so a consumer already written against the node is
not disturbed. `StageId` already lives in `fragcap-core::attribution`. A side-map
would duplicate node identity tracking and split the node's own state across two
owners, and `descends_from` evaluation would then need the side-map threaded
everywhere the tree already goes. Writing the reserved field is the design the
architecture anticipated. The method is pure data mutation with no platform
surface, so P-2 holds.

## D-2. `descends_from` is evaluated once, on the start event, over current bindings

**Decision**: matching evaluates each start event against the bindings that exist
at that instant; `descends_from` resolves by walking the node's strict ancestry
and testing whether any ancestor node is bound to the named role. No deferred
re-evaluation queue.

**Alternatives**: a work queue that re-evaluates unbound `descends_from` stages
whenever a new binding appears.

**Why**: S11 guarantees causal creation order (a parent's start event precedes its
child's), and a stage that matches an ancestor binds on the ancestor's start, so
the ancestor is already bound when the descendant is evaluated. A re-evaluation
queue would add machinery for a reordering the event source does not produce. A
descendant with genuinely no bound ancestor at its start correctly does not
match. Resolving by walking ancestry and reading each ancestor's `stage()` also
handles more than one process bound to one role without extra bookkeeping.

## D-3. Multiple matching stages bind the first in declaration order

**Decision**: when more than one stage's predicates all hold for one process, the
first stage in profile declaration order binds.

**Why**: section 15.4 validation already makes an ambiguous image match within a
chain an error, so this is the residual case. A total order over declaration
position makes the result deterministic rather than dependent on iteration
happenstance, which is the same property S10's join order exists for.

## D-4. The watching-discard counter is the session's own, not `CaptureStats`

**Decision**: the count of packets discarded before acquisition lives on a
facade-owned `SessionStats { watching_discarded, retained, retained_bytes }`. The
session asserts observed equals retained plus watching-discards.

**Alternatives**: a new field on `fragcap-core::stats::CaptureStats`.

**Why**: the Watching discard happens upstream of the pipeline the S08
conservation identity is asserted over; the discarded packets never enter the
buffer or a sink. Adding a field to `CaptureStats` now would either break that
identity or sit unused until S13/S14 wire the session in. `WatcherReport` and
`SourceStats` set the precedent: a component's own accounting is a separate value
the run assembles alongside the capture's. P-4 requires a named counter that is
surfaced, which `SessionStats.watching_discarded` is; it does not require the
counter live in `CaptureStats`. When S13/S14 wire the session to the pipeline the
run report assembles both.

## D-5. Acquisition timeout and duration are measured from arm

**Decision**: both the acquisition timeout and the duration bound are measured
from the instant the session was armed (entered Watching). The acquisition
timeout is optional; the duration bound is optional.

**Why**: the operator's bound is a wall-clock statement about the whole session
(`--duration 30m`), and measuring from arm gives a single clock origin for both
bounds rather than two. A session that never acquires still ends: by the
acquisition timeout when set, or by the duration bound or an interrupt otherwise.

## D-6. A service process does not keep the all-exited condition from firing

**Decision**: the "all matched processes have exited and no stage remains
awaited" stop condition considers only non-service bound processes for liveness,
and considers a stage awaited only while a non-service stage has bound no process
yet.

**Why**: section 10.4 states a service is never awaited during acquisition,
because waiting on something already running deadlocks. A live platform service
that outlives the session must not keep the session from recognizing that its
gameplay processes have all exited. A service-bound process is still recorded and
attributed; it simply does not gate this particular stop condition.
