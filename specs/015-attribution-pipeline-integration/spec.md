# Feature Specification: Attribution Session-to-Pipeline Integration

**Feature Branch**: `feat/attribution-pipeline-integration`

**Created**: 2026-08-10

**Status**: Draft

**Slice**: Follow-up to S13 (filter management), resolving GitHub issues #18 and
#19; specification sections 8.6, 11.2, 11.6, 12.2, and 29 (deviation);
constitution P-1, P-2, P-3, P-4, P-6, and P-9.

**Numbering note**: This is the fifteenth spec directory (`015-`) but is NOT the
roadmap's reserved slice S15 ("Transports and streaming sinks", specification
sections 14.1 to 14.4). It is a paired follow-up to the completed S13, deferred
there and recorded as specification section 29 open items. The `015-` prefix is
a directory ordinal only. The roadmap's future slices S15 through S18 keep their
names and shift to later directory ordinals; `docs/plans/README.md` is updated to
say so.

**Input**: Resolve the two paired follow-ups deferred from S13 (pull request #17)
and recorded as specification section 29 open items in
`changelog.d/S13-filter-management.decisions.md`. (1) Drive the attribution
snapshot refresh from the pipeline control thread, which requires changing
`FlowAttributor::refresh` from `&mut self` to `&self` so it can be called through
the shared `Arc<dyn FlowAttributor>` that section 11.6 mandates for lock-free
resolve; this is an architecture-of-record trait change taken through the
deviation process and promoted to section 29. (2) Restrict the phase-two
narrowing endpoint set to endpoints owned by profiled processes, by joining the
socket table (socket to PID) with the S11/S12 process-tree stage bindings (PID to
profiled) that already reach the packet path through `CaptureSession::role_bindings()`
and the binding publisher. Both are verified at tier 1; the live tier-2 path
remains unexecuted in continuous integration.

## Overview

S13 built phases two and three of the filter lifecycle: it compiles a capture
filter admitting only endpoints belonging to profiled processes and keeps it
current. Two properties it needed were deferred as section 29 open items because
each required a change larger than S13's scope, and S13's tier-1 machinery did
not depend on either (a scripted attributor supplied a controlled, already-correct
endpoint set). This slice closes both. They are paired because they share one
seam, the section 8.6 control thread that reads the attribution snapshot and
drives the filter, and because doing the refresh change first makes the narrowing
change land on a single unified control thread rather than two.

**The attribution snapshot is refreshed during a run, from the pipeline control
thread.** Today `FlowAttributor::refresh` takes `&mut self`, so it cannot be
called through the `Arc<dyn FlowAttributor>` the capture threads hold for
lock-free `resolve` (section 11.6). Nothing in the pipeline refreshes the socket
table during a run: after construction the published `AttributionIndex` is never
replaced, so on the live backend a connection opened after capture starts is
never attributed and never enters the narrowed filter. The CLI worked around this
with a stopgap `RefreshDriver` thread that owns the attributor mutably and lives
outside the pipeline. Changing the signature to `refresh(&self)`, with interior
mutability for the attributor's retention map mirroring the existing arc-swap
publication, lets the section 8.6 control thread drive the refresh on the section
11.2 cadence and triggers while the capture threads resolve concurrently. The
stopgap `RefreshDriver` collapses onto that control thread.

**The narrowing endpoint set is restricted to profiled processes.** Section 12.2
phase two compiles a filter admitting only endpoints "belonging to profiled
processes," but the control thread narrows from `FlowAttributor::active_endpoints()`,
which returns every socket-table endpoint (plus retained), unfiltered by
PID/stage/role, and `AttributionIndex::endpoints` has already dropped the owning
process identifier. On the live backend this compiles every application's sockets
into the filter, which does not remove the background traffic phase-two narrowing
exists to remove. The profiled-PID set is already computed on the packet path:
`CaptureSession` binds PIDs to stages and publishes `(pid, role, stage)` through
the binding publisher, and the role-stamping attributor already joins that
snapshot to resolutions by PID. This slice threads the same profiled-PID set into
the endpoint enumeration so the narrowing input is restricted before the filter
program is compiled.

Two properties shape the slice.

**Correctness of resolve stays lock-free.** The capture thread's per-packet
`resolve` reads the published snapshot wait-free through `arc-swap` (section
11.6). Making `refresh` take `&self` must not introduce a lock on that read path;
the interior mutability added for the retention map lives on the refresh side (the
control thread), not the resolve side. A concurrency test exercises resolve across
a publication to prove the read path is unblocked.

**The narrowing change is a restriction of an existing pure input, not a new
data path.** The filter manager already narrows correctly to whatever endpoint
set it is given (S13). Only the source of that set changes: from "every endpoint
the attributor holds" to "every endpoint owned by a profiled PID." The profiled-PID
set is produced by the session and already published; this slice consumes it, it
does not rebuild stage matching.

## Clarifications

### Session 2026-08-10

Resolved under autopilot from the architecture of record (specification sections
8.3, 8.6, 10, 11.2, 11.6, 12.2, and 29) and the constitution; no operator
escalation was required.

- Q (E-a): Where does the driven refresh run, on the pipeline's own section 8.6
  control thread or on a separate driver? -> A: On the pipeline control thread.
  Section 8.6 places the socket-table refresh on the control thread, and the
  `refresh(&self)` signature is precisely what lets the same `Arc<dyn FlowAttributor>`
  the capture threads resolve through also be refreshed there. The CLI's
  `RefreshDriver`, a separate thread introduced by S13/S14 only because a `&mut`
  refresh could not cross the capture threads, is removed; its schedule logic
  (`RefreshSchedule`, `wants_refresh`, `note_matched_process_start`) moves onto the
  control thread unchanged. Keeping a second driver thread was rejected: it
  duplicates the control thread's role, and the reason it existed (the `&mut`
  signature) is gone.

- Q (E-b): What supplies the interior mutability for the retention map when
  `refresh` becomes `&self`? -> A: The retention map moves behind the same
  publication discipline the index already uses. The refresh side computes the new
  retention state and the new `AttributionIndex` and publishes them together
  through the existing `arc-swap` cell; the map is not read on the resolve path, so
  it needs no separate lock visible to `resolve`. A `Mutex<RetentionMap>` was
  considered and is acceptable because the map is touched only by the single
  control thread driving refresh (never concurrently), but folding the retained
  state into the published snapshot is preferred because it keeps one publication
  point and cannot desynchronize the map from the index it produced. Either way the
  resolve read path is unchanged and stays lock-free (section 11.6). A hand-rolled
  `AtomicPtr` reclamation scheme was rejected: it adds `unsafe` to a workspace that
  has none outside a platform binding, for no benefit over the existing arc-swap.

- Q (E-c): How does the profiled-PID set reach the control thread that enumerates
  endpoints? -> A: Through the same binding snapshot the role-stamping attributor
  already reads. The session publishes `(pid, role, stage)` for every stage-bound
  node through the binding publisher; the narrowing input is the set of PIDs that
  snapshot names. The endpoint enumeration is restricted to endpoints whose owning
  module/PID is in that set. This requires the endpoint enumeration to carry the
  owning identifier far enough to join, rather than dropping it as
  `AttributionIndex::endpoints` does today; the join is by owning module for UDP
  (whose key is the local endpoint alone) and by PID where available, consistent
  with the S10 decision that both tables are read by owning module. Recomputing
  stage matching in the pipeline was rejected (P-3 layering: the session owns stage
  matching; the pipeline consumes its published result).

- Q (E-d): Does restricting the narrowing set change what is captured, or only
  what the kernel filter admits? -> A: Only what the kernel filter admits.
  Userspace attribution runs on every packet regardless of the installed filter
  (S13, FR-005), so restricting the narrowing set is a performance refinement:
  fewer unrelated sockets are compiled into the filter, so less background traffic
  crosses the kernel boundary. Correctness is unchanged; a briefly stale filter is
  still accounted as a filter gap. No new discard class is introduced by either
  change, so no new counter is required (P-4 is satisfied by the existing set).

- Q (E-e): What is the refresh cadence and what triggers an immediate refresh?
  -> A: The section 11.2 cadence and triggers already modeled by `RefreshSchedule`
  are reused unchanged: a periodic due interval, plus an immediate request on a
  matched-process start (`note_matched_process_start`). This slice moves where they
  are driven (onto the control thread), not the policy itself. Widening or changing
  the cadence is out of scope.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A connection opened mid-run becomes attributable (Priority: P1)

On the live backend a game opens a new connection after capture has started.
Because the control thread drives `FlowAttributor::refresh` on the section 11.2
cadence, the socket table is re-read, the new socket is resolved to its owning
process, and the published snapshot is replaced, so the connection is attributed
and, if owned by a profiled process, enters the narrowed filter. Before this
slice the snapshot was frozen at construction and the connection was never seen.

**Why this priority**: Without a driven refresh the live backend attributes only
connections that existed before capture began, which for a launched game client
is often none of them. This is the capability that makes live attribution
function at all; the narrowing refinement (Story 2) builds on it.

**Independent Test**: Drive a stub attributor whose snapshot gains an endpoint on
the second refresh through the pipeline control thread; assert the endpoint is
unresolvable before the refresh and resolvable after, and that it enters the set
handed to the filter manager. No capture driver, elevation, or game.

**Acceptance Scenarios**:

1. **Given** a running capture whose attribution snapshot does not yet contain a
   connection, **When** the connection opens and the control thread's next refresh
   runs, **Then** the connection becomes resolvable on the capture threads' next
   `resolve`.
2. **Given** a matched process starts mid-run, **When** it signals
   `note_matched_process_start`, **Then** an immediate refresh is requested rather
   than waiting for the periodic interval.
3. **Given** the control thread drives refresh, **When** capture threads resolve
   concurrently during a publication, **Then** every resolve is wait-free and none
   blocks behind the refresh (section 11.6).

---

### User Story 2 - The kernel filter narrows to the target's sockets only (Priority: P1)

Once profiled processes are bound and their sockets appear in the attribution
map, fragcap compiles a filter admitting only endpoints owned by those processes,
not every socket on the machine. An unrelated background application's sockets,
present in the same socket table, are excluded from the compiled program, so the
volume crossing the kernel boundary is the target's traffic plus whatever shares
its ports, not all IP traffic.

**Why this priority**: The whole point of phase-two narrowing (section 12.2) is to
stop compiling unrelated sockets into the filter. Narrowing from the full
socket-table endpoint set, as today, admits every application and removes almost
none of the background volume reconnaissance measured at up to ninety-four
percent. Restricting to profiled endpoints is what makes narrowing do its job.

**Independent Test**: Feed the control thread a socket-table snapshot containing
both a profiled PID's endpoints and an unprofiled PID's endpoints, with a binding
snapshot naming only the profiled PID; assert the compiled program admits exactly
the profiled endpoints and excludes the unprofiled ones, on a source double that
records the installed program. No capture driver, elevation, or game.

**Acceptance Scenarios**:

1. **Given** a socket table holding endpoints owned by both a profiled and an
   unprofiled process, **When** the control thread narrows, **Then** the compiled
   filter program admits only the profiled process's endpoints.
2. **Given** a profiled process's endpoints spanning IPv4 and IPv6, **When** the
   filter is compiled, **Then** the program admits both address families and still
   excludes the unprofiled endpoints.
3. **Given** a UDP socket bound to a wildcard address owned by a profiled process,
   **When** the filter is compiled, **Then** it is admitted by the owning-module
   join even though its address does not identify it, consistent with the S10
   owning-module rule.
4. **Given** the operator-facing "filter narrowed to N endpoints" message,
   **When** it is emitted, **Then** N counts only the profiled endpoints actually
   compiled into the program, not the full socket-table set.

---

### User Story 3 - Every FlowAttributor implementor moves to the new signature cleanly (Priority: P2)

The `refresh(&self)` change is an architecture-of-record trait change. Every
implementor and every test double changes with it in the same slice, the
deviation is recorded in specification section 29, and the trait stays
dyn-compatible and `Send + Sync` so the pipeline can still hold `Arc<dyn
FlowAttributor>`.

**Why this priority**: A trait-signature change that is not applied to every
implementor does not compile; recording the deviation is what keeps the
specification and the code from silently diverging (the roadmap's deviation rule).
It is P2 only because it is mechanical once the design (Story 1) is fixed.

**Independent Test**: The workspace compiles and `cargo xtask ci` passes with the
new signature applied to `SocketTableAttributor`, `PublishedResolver`,
`ScriptedAttributor`, the role-stamping attributor, and the test doubles; the
dyn-compatibility and `Send + Sync` compile-time assertions still hold.

**Acceptance Scenarios**:

1. **Given** the new `refresh(&self)` signature, **When** the workspace builds,
   **Then** every `FlowAttributor` implementor and test double compiles against it.
2. **Given** the trait change, **When** the slice commits, **Then** a dated
   decision fragment records it as a specification section 29 deviation.
3. **Given** the trait remains dyn-compatible and `Send + Sync`, **When** the
   pipeline constructs `Arc<dyn FlowAttributor>`, **Then** it compiles and the
   existing compile-time trait assertions pass.

---

### Edge Cases

- The retention map's window origin is unchanged: retention still runs from the
  instant an endpoint was last observed present, not from the refresh that noticed
  it gone. Moving the refresh onto the control thread must not alter the window's
  measurement (a P-9 concern), only who calls it.
- A refresh that finds no change publishes an equal snapshot (or skips
  publication); either way `resolve` is never blocked and no endpoint set churns
  the filter needlessly.
- An empty profiled-PID set (no process bound yet) narrows to nothing new: the
  bootstrap filter remains until the first profiled endpoint appears, exactly as
  S13 specified, because the restricted set is empty rather than "all endpoints."
- A profiled process exits mid-run: its endpoints leave the profiled set on the
  next refresh, and the S10 retention window governs how long its closing
  connections stay attributed, unchanged by this slice.
- The offline replay path drives no live refresh and has no socket table; the
  scripted attributor's `refresh(&self)` is a no-op, so the offline goldens are
  unchanged.
- The CLI `RefreshDriver` removal must not change offline behavior: offline never
  had a live refresh to drive, so collapsing the driver is a no-op there and the
  offline goldens stay byte-identical.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `FlowAttributor::refresh` MUST take `&self`, so it can be called
  through the shared `Arc<dyn FlowAttributor>` the capture threads hold, and the
  change MUST be recorded as a specification section 29 deviation with a dated
  decision fragment.
- **FR-002**: `SocketTableAttributor` MUST refresh through interior mutability
  (its retention map and published index updated behind the existing arc-swap
  publication or an equivalent single-writer discipline), and the per-packet
  `resolve` read path MUST remain lock-free and wait-free (section 11.6).
- **FR-003**: Every `FlowAttributor` implementor and test double MUST move to the
  `refresh(&self)` signature in this slice: `SocketTableAttributor`,
  `PublishedResolver`, `ScriptedAttributor`, the role-stamping attributor, and the
  stub/fixed/panic test doubles. The trait MUST stay dyn-compatible and `Send +
  Sync`.
- **FR-004**: The pipeline control thread (section 8.6) MUST drive
  `FlowAttributor::refresh` on the section 11.2 cadence and triggers, reusing the
  existing `RefreshSchedule` / `wants_refresh` / `note_matched_process_start`;
  the CLI stopgap `RefreshDriver` MUST be removed or reduced to nothing that owns
  a mutable attributor.
- **FR-005**: The phase-two narrowing endpoint set MUST be restricted to endpoints
  owned by profiled processes, joining the socket table (socket to owning
  module/PID) with the session's published stage bindings (PID to profiled) before
  the filter program is compiled by `FilterManager::poll`.
- **FR-006**: The endpoint enumeration feeding the narrowing MUST carry the owning
  identifier far enough to perform the profiled-process join, rather than dropping
  it; the join MUST be by owning module for UDP (local endpoint key) consistent
  with the S10 owning-module rule.
- **FR-007**: The operator-facing "filter narrowed to N endpoints" message MUST
  report the count of profiled endpoints actually compiled into the program, not
  the unfiltered socket-table set, in both the offline and live command paths.
- **FR-008**: A connection opened after capture starts MUST become resolvable
  after the control thread's next refresh, and, if owned by a profiled process,
  MUST enter the narrowed filter on the next permitted reinstall.
- **FR-009**: Neither change MUST introduce a new discard class; the existing
  named counters (kernel, buffer, sink, filter-gap) keep their meanings and the
  pipeline conservation invariant is unaffected (P-4).
- **FR-010**: Neither change MUST introduce a platform-specific dependency into
  `fragcap-core` (P-2), and neither MUST open a process handle or otherwise touch a
  denylisted technique (P-1); profiled-PID information comes from the existing
  session bindings, not from a new process query.
- **FR-011**: The whole slice MUST be verifiable at tier 1 (no capture driver, no
  elevation, no game); the live tier-2 path remains unexecuted in continuous
  integration and MUST be reported as such rather than as verified.
- **FR-012**: Any new term MUST get a glossary entry in the same change (P-6);
  the resolutions MUST be promoted into specification sections 11, 12.2, and 29.

### Key Entities

- **Attribution snapshot**: the immutable `AttributionIndex` published lock-free
  through arc-swap and read wait-free by `resolve`; now replaced during a run by a
  control-thread-driven refresh.
- **Retention map**: the closing-connection grace state the refresh ages and
  rewrites; moved behind interior mutability so `refresh(&self)` can update it.
- **Refresh schedule**: the section 11.2 cadence and trigger policy
  (`RefreshSchedule`), unchanged, now driven from the pipeline control thread.
- **Profiled-PID set**: the PIDs the session has stage-bound, published as
  `(pid, role, stage)` through the binding publisher and consumed here to restrict
  narrowing.
- **Profiled endpoint set**: the socket-table endpoints owned by a profiled PID,
  the restricted input to `FilterManager::poll`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A connection absent from the initial snapshot becomes resolvable
  after a control-thread-driven refresh, verified through the pipeline at tier 1.
- **SC-002**: The compiled filter program admits exactly the profiled process's
  endpoints and excludes an unprofiled process's endpoints sharing the same socket
  table, verified across IPv4, IPv6, and a wildcard UDP bind on a source double.
- **SC-003**: A concurrency test exercises wait-free `resolve` across a refresh
  publication, proving the read path is not blocked by the `&self` refresh
  (section 11.6).
- **SC-004**: The `refresh(&self)` signature is applied to every implementor and
  test double, the workspace builds, and the dyn-compatibility and `Send + Sync`
  compile-time assertions pass.
- **SC-005**: The CLI `RefreshDriver` is removed and refresh is driven from the
  pipeline control thread, with offline goldens byte-identical (offline drives no
  live refresh).
- **SC-006**: `cargo xtask ci` passes (format, clippy, tests, conventions lint,
  dependency direction, license), and `cargo xtask neutral` and `msrv` exit 0.
- **SC-007**: The two resolutions are promoted into specification sections 11,
  12.2, and 29, and the trait change is recorded as a dated section 29 decision.

## Assumptions

- The socket-table attributor (S10) already publishes its index lock-free through
  arc-swap; this slice reuses that publication for the retained state and adds no
  new dependency (`arc-swap` and `windows-sys` are already present).
- The session (S12) already computes and publishes stage bindings as `(pid, role,
  stage)` through the binding publisher, and the role-stamping attributor already
  joins that snapshot by PID; this slice consumes the same published set for
  narrowing and does not rebuild stage matching (P-3).
- The filter manager (S13) narrows correctly to whatever endpoint set it is given;
  only the input source changes, so its debounce, rate limit, and gap accounting
  are unchanged.
- The section 11.2 refresh cadence and triggers (`RefreshSchedule`) are correct as
  built; this slice changes where they are driven, not the policy.
- The offline replay and scripted sources drive no live refresh; their
  `refresh(&self)` is a no-op and the offline goldens are unaffected.
- No new runtime dependency is required; `fragcap-core`'s allowlist is untouched
  (P-2) and no process handle is opened (P-1).
