# Feature Specification: ETW Process Watcher and Tree

**Feature Branch**: `claude/s11-s12-parallel-dev-086100`

**Created**: 2026-08-09

**Status**: Draft

**Slice**: S11 (specification sections 10.1 and 10.2; constitution P-1, P-2,
P-3, P-4, P-6, and P-9)

**Input**: Implement specification sections 10.1 (mechanism) and 10.2 (the
process tree): a `ProcessWatcher` backed by the ETW kernel process provider, a
startup snapshot from query-only enumeration, and the in-memory process tree
keyed by synthetic session-local identifiers that survives operating system
identifier recycling and parent exit. Command lines are recorded verbatim. No
process handle carrying memory-read rights is opened against any target, and no
process is polled. Stage matching, lifecycle classes, session lifecycle, and
stop conditions are sections 10.3 through 10.6 and belong to S12.

## Overview

Nine slices have built a capture tool that has never looked at a process. The
pipeline runs, packets are parsed, both writers write, and every attribution any
of it has ever produced came out of a text file that a test wrote for the
purpose. S11 is where fragcap first observes the machine it is running on.

That framing sets what the slice owns. Attribution answers which process holds a
socket; that is section 11 and it belongs to S10. This slice answers a different
and prior question: which processes exist, which created which, and when. Only
the second question can be answered wrongly in a way that looks like success,
which is why it is worth a slice of its own.

Three properties shape it.

**Ancestry is true only at the instant of creation.** Section 5.3 is exact about
why. A Windows process identifier comes from a reusable pool, and each process
records its creator's identifier without maintaining it. Reading a parent
identifier from a running process therefore yields either nothing or a process
with no relationship to the one under examination. The ETW kernel process
provider emits an event at the moment of creation carrying the parent as
recorded at that instant, and that instant is the only one at which the
relationship is unambiguous. Everything in this slice follows from that single
fact.

**The tree is a value, and that is load-bearing.** The tree is a fold over a
stream of process events. It opens nothing, queries nothing, and touches no
platform interface, so the whole of section 10.2 is testable on any machine with
no elevation and no game. S09 established this shape for interface selection for
exactly the same reason, and S12's stage matching depends on it: a slice that
has to run a game to test `descends_from` is a slice that cannot be tested.

**Missing a process is invisible.** Reconnaissance found chains five and six
levels deep containing pure shims that hold no sockets, and one focal title
running three processes that share the image name `TheDivision2.exe` where only
the last transmits. A watcher that misses the creation of one link in a chain
does not fail. It produces a tree with a hole in it, a session that binds to the
wrong process or to none, an exit code of zero, and a well-formed capture file
containing no gameplay. That is the same failure class as S05's ambiguous match
check and S09's interface selection, and it gets the same treatment here: what
was observed, what was inferred, and what was lost are three different things,
and the tree says which is which.

Two further properties are worth stating because they are where a reasonable
implementation goes wrong.

**Polling is not a fallback.** Section 10.1 refuses polling categorically, and
the reason is this project's whole purpose rather than a performance preference:
a transient launcher whose lifetime is shorter than the poll interval is exactly
the thing fragcap exists to catch. Appendix D.1 records that an unprivileged
process telemetry source exists on the platform, and Appendix D.4 records what
it cost the reconnaissance harness that used it, which is that a chain member
living under one second could have gone unobserved. Offering that as a quiet
degraded mode would produce the failure above under a name that sounds like
success. The absence of elevation is reported as the absence of elevation.

**A command line is a tree field and is recorded verbatim.** Section 10.2 and
constitution P-9 settle this together, and the reasoning is already written down
at length in section 10.2. What S11 adds is a constraint on how one is
obtained: the ETW start event carries it, and no code path in this slice opens a
handle carrying memory-read rights against a running process to recover one that
ETW did not supply. Where a command line is genuinely unavailable, the tree
records that it is unavailable rather than recording an empty one.

## Clarifications

### Session 2026-08-09

- Q: Does the process tree live in `fragcap-core` or in `fragcap-attr`? -> A:
  `fragcap-core`. The tree is a pure fold over `ProcessEvent` values and holds
  no platform interface, which is the same shape `interface::select` has carried
  since S09 and for the same reason: section 10.2's whole content becomes
  testable at tier 1, on any machine, with no elevation. `fragcap-attr` keeps
  the watcher, which is the part that touches ETW. Putting the tree beside the
  watcher would gate every test of ancestry, retention, and identifier recycling
  behind an elevated Windows session, and would leave S12's stage matching with
  nowhere to be tested either, because a matcher is a decision over a tree.

- Q: Does S11 implement an unprivileged fallback for machines where ETW cannot
  be consumed? -> A: No. Section 10.1 forbids polling, and the failure it
  forbids is the one fragcap exists to prevent. A watcher that cannot start
  reports that it cannot start, naming elevation as the missing precondition,
  and the caller fails the run rather than continuing with a watcher that will
  silently miss transient members of the launcher chain. This is a scope
  decision the operator can see in their own invocation, which section 10.2 and
  P-9 both permit, rather than an alteration of what was observed, which neither
  does.

- Q: The startup snapshot reports processes whose creation was not observed. How
  does the tree distinguish their ancestry from ancestry it saw happen? -> A:
  Provenance is carried on the node, not derived. A node created from an
  observed start event carries creation-time ancestry, which section 5.3 says is
  unambiguous. A node created from the startup snapshot carries snapshot
  ancestry, which is a parent identifier read from a running process and which
  section 5.3 says may name an unrelated process or nothing at all. The two are
  not interchangeable and a consumer must be able to tell them apart. This is
  the lesson S06 learned about attribution fidelity, which was initially derived
  from whether an attribution existed and which review caught claiming a live
  socket-table hit for a resolution that came from a text file.

- Q: What happens when ETW reports that it lost events? -> A: The count is
  recorded, surfaced in the run's statistics, and the tree is marked as
  incomplete for the remainder of the session. A lost start event is not a lost
  packet: a packet's loss is local to that packet, while a start event's loss
  removes a node and silently reparents or orphans everything beneath it. P-4
  requires the counter. P-9 requires that a tree which may have a hole in it not
  present itself as a complete one.

- Q: Is the channel between the ETW consumer and the tree bounded, with
  drop-oldest, as section 12.4 specifies for packets? -> A: No. It is unbounded,
  and the asymmetry is deliberate. Section 12.4's bounded drop-oldest buffer
  exists because packets arrive faster than they can be written and losing an
  old one costs one packet. Process events arrive in the thousands over a
  session rather than the millions, and losing one costs a subtree. There is
  therefore no discard path here to count, which is the correct way to satisfy
  P-4 for this stream rather than adding a counter to a bound that should not
  exist.

- Q: `ProcessWatcher::subscribe` takes `&self` and returns a receiver. What does
  a second call return? -> A: An independent receiver that observes every event
  published after that call. The watcher fans out. A single-consumer
  interpretation would make the pipeline's control thread and any diagnostic
  consumer mutually exclusive, and the signature in section 8.5 already implies
  otherwise by taking `&self` rather than consuming or mutably borrowing.

- Q: What identifies a node, given that operating system identifiers recycle? ->
  A: A synthetic session-local identifier, never reused within a session, is the
  node's identity. The pair of operating system identifier and start timestamp
  is the lookup key from the operating system's vocabulary into the tree, and it
  resolves to the node live at the relevant timestamp. Both are section 10.2 as
  written; recording it here because the distinction between an identity and a
  lookup key is the part an implementation collapses.

- Q: In which order does the watcher subscribe to the provider and take the
  startup snapshot? -> A: Subscribe first, then snapshot. The two orders fail
  differently and only one failure is recoverable. Subscribing first can report
  one process twice, once as an event and once in the snapshot, and FR-033
  already requires the tree to reconcile that into a single node in either
  arrival order. Snapshotting first leaves a window in which a process created
  after the snapshot and before the subscription is reported by neither source,
  and nothing downstream can detect that it is missing. A duplicate is visible
  and fixable; a gap is neither, and a gap in a launcher chain is precisely the
  failure this slice exists to prevent.

- Q: What start timestamp does a node get when the startup snapshot found the
  process and the platform does not report when it started? -> A: The snapshot
  reads the start time through the same query-only access rights the enumeration
  already requires, which the platform supplies without any memory right. Where
  it is genuinely unavailable, the node records the start time as unknown and
  orders before every observed event in the session, and a lookup by identifier
  and time resolves to it only when no node with a known start time covers that
  time. It is never given a fabricated start time. Substituting the session's
  own start would be the comfortable untruth P-9 forbids, and would also be
  wrong in the one direction that matters, because it would sort a process that
  has been running for hours after one that started a second ago.

- Q: Where do the watcher's counters live, given that `CaptureStats` carries the
  capture's accounting? -> A: In a watcher-owned report, alongside the
  per-interface `SourceStats` shape S09 established, and not folded into
  `CaptureStats`. Section 12.4's conservation identity is asserted over
  `CaptureStats` in every pipeline test, and putting quantities into it that are
  not packets would weaken the one assertion that catches an uncounted discard
  path. The tree separately reports whether it is possibly incomplete, because
  that is a property a consumer of the tree must be able to see while holding
  only the tree.

- Q: Does the tree retain every process on the machine, or only ones that look
  relevant? -> A: Every process it observes, unfiltered. Section 10.2 retains
  exited nodes for the session and gives no basis for excluding a live one, and
  the only available basis would be the capture scope of section 15.2, which is
  S12's and does not exist yet. Filtering would also break `descends_from`
  through any excluded process, which is the predicate reconnaissance showed is
  required rather than advisory. The cost is real on a working machine and is
  made observable rather than estimated: the tree reports how many nodes it
  retains, so an operator sees the growth during the session instead of reading
  section 10.2's "few kilobytes" and hoping it still holds.

- Q: Does the watcher take the machine-wide kernel trace session, or its own? ->
  A: Its own. fragcap MUST NOT contend for a session that exists once per
  machine, and MUST NOT stop, reconfigure, or take over a session it did not
  create. Taking the singleton would make fragcap fail whenever any other tool
  is already tracing, and taking it by force would make fragcap the tool that
  silently breaks the operator's other instrumentation, which is a P-9 failure
  aimed at somebody else's output. Which concrete provider and session type
  satisfies this is a research question for the plan phase; what the
  specification settles is that the answer must be a private session.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Observe a launcher chain from before it begins (Priority: P1)

An operator starts fragcap before starting a game. fragcap attaches to the
kernel process provider, takes one snapshot of what is already running, and
then observes every process creation and exit on the machine for the duration of
the session. When the operator launches the title through its platform client,
every member of the resulting chain is observed at its moment of creation, with
its parent as recorded at that instant, its image path, and its command line.

**Why this priority**: This is the slice's reason to exist and the mechanism
section 10.1 specifies. Nothing else in the slice has a subject until process
events are arriving.

**Independent Test**: On an elevated Windows session, start the watcher, spawn a
short-lived child process from the test itself, and assert that the start event
names the test process as its parent and carries the child's image path and
command line, and that the exit event follows.

**Acceptance Scenarios**:

1. **Given** an elevated session, **When** the watcher starts, **Then** it
   subscribes to the kernel process provider and begins receiving start and exit
   events for every process on the machine.
2. **Given** a running watcher, **When** a process is created, **Then** a start
   event is published carrying the new identifier, the creating process's
   identifier as recorded at creation, the image path, the command line, and the
   time.
3. **Given** a running watcher, **When** a process exits, **Then** an exit event
   is published carrying the identifier and the time.
4. **Given** a process that exists before the watcher starts, **When** the
   startup snapshot is taken, **Then** it appears in the watcher's snapshot with
   the ancestry the platform reports for it and with its start time where the
   platform supplies one.
5. **Given** a process that lives for less than the interval any polling
   implementation would use, **When** it is created and exits, **Then** both its
   events are observed, because the watcher does not poll.
6. **Given** a watcher starting up, **When** a process is created during
   startup, **Then** it is reported by the event stream, by the snapshot, or by
   both, and never by neither, because the subscription precedes the snapshot.
7. **Given** another tool already holding a machine-wide trace session, **When**
   the watcher starts, **Then** it starts its own session and does not stop,
   reconfigure, or take over the other one.

---

### User Story 2 - Ancestry after recycling and parent exit (Priority: P1)

An analyst asks which process created the game client, across a chain whose
middle members have already exited and whose operating system identifiers have
been reassigned to unrelated processes. The tree answers from what it observed
at creation time. Exited nodes are still present, identifier reuse does not
merge two unrelated processes into one node, and a lookup by operating system
identifier at a given time resolves to the process that was live then.

**Why this priority**: The tree is what the rest of the project consumes.
Section 5.4's observed chains are five and six levels deep with transient
members, so a tree that cannot answer after a parent exits cannot answer at all
for the topologies fragcap was built for.

**Independent Test**: Fold a declared sequence of process events into a tree and
assert ancestry, retention, and recycling behavior. No elevation, no ETW, and no
game, because the tree is a decision over values.

**Acceptance Scenarios**:

1. **Given** a chain of start events, **When** ancestry is queried for the last
   member, **Then** the full path to the root is returned in creation order.
2. **Given** a node whose parent has exited, **When** ancestry is queried,
   **Then** the parent is still present and still named, because exited nodes
   are retained for the session.
3. **Given** two processes that share an operating system identifier and whose
   lifetimes do not overlap, **When** both have been observed, **Then** they are
   two nodes with distinct synthetic identifiers, and neither inherits the
   other's children.
4. **Given** an operating system identifier and a timestamp, **When** the tree
   is queried, **Then** it resolves to the node live at that timestamp, or to
   nothing if no node was.
5. **Given** a synthetic identifier issued in a session, **When** any later node
   is created, **Then** that synthetic identifier is never issued again in the
   same session.
6. **Given** a start event whose parent identifier names no node in the tree,
   **When** it is folded in, **Then** the node is recorded with its ancestry
   unresolved rather than attached to an arbitrary node or discarded.
7. **Given** an exit event for an identifier with no live node, **When** its
   start event arrives afterwards, **Then** the two are joined into one node,
   because the fold does not assume timestamp order.
8. **Given** an exit event for which no start event ever arrives, **When** the
   session ends, **Then** it is counted as unmatched and surfaced, and no node
   is fabricated for it.
9. **Given** any node, **When** it is read, **Then** it carries the operating
   system identifier, the parent's synthetic identifier where resolved, the
   image path, the command line, the start timestamp, and the exit timestamp
   where the process has exited.

---

### User Story 3 - Exercise the tree with no game (Priority: P1)

A contributor working on a laptop with no capture driver, no elevation, and no
game installed runs the whole of the tree's behavior, including the launcher
chains reconnaissance actually observed, from a declared script of process
events.

**Why this priority**: Specification section 25.1's testability claim is a
standing commitment, not a per-slice courtesy, and S12's stage matching is
unbuildable without it. It shares P1 with the two stories above because the
slice is not complete if section 10.2 is only verifiable on an elevated Windows
machine.

**Independent Test**: Replay the two chains recorded in Appendix D, including
the one where three processes share an image name, through the scripted watcher
and assert the resulting tree.

**Acceptance Scenarios**:

1. **Given** a declared script of process events, **When** it is played through
   the scripted watcher, **Then** the watcher publishes exactly those events, in
   order, and the tree built from them matches the tree built from the same
   events arriving live.
2. **Given** the ESO chain from Appendix D, **When** it is replayed, **Then**
   the tree reports five levels from the shell to the client.
3. **Given** the Division 2 chain from Appendix D, **When** it is replayed,
   **Then** the tree holds three distinct nodes sharing the image name
   `TheDivision2.exe`, distinguishable by ancestry.
4. **Given** the scripted watcher, **When** the ordinary check set runs on a
   machine with no elevation and no capture driver, **Then** every test of
   section 10.2 executes.

---

### User Story 4 - Be told what the watcher could not see (Priority: P2)

An operator runs fragcap without elevation, or on a machine where the trace
session cannot start, or through a period where the kernel dropped events. In
each case they are told specifically what happened, and in the third case the
run continues with the tree honestly marked as possibly incomplete.

**Why this priority**: The three conditions have different causes and different
remedies, and all three produce a capture that looks fine. P-4 and P-9 both bind
here, and section 26.4 requires an error to state what was attempted, what
happened, and what to do next.

**Independent Test**: Start the watcher unelevated and assert the error names
elevation. Drive the tree's lost-event path from a synthetic report and assert
the count is surfaced and the tree reports itself incomplete.

**Acceptance Scenarios**:

1. **Given** a session without the privilege the trace session requires,
   **When** the watcher is started, **Then** it fails with an error naming the
   missing privilege and what to do about it, and does not fall back to polling.
2. **Given** a trace session that cannot start for a reason other than
   privilege, **When** the watcher is started, **Then** the platform's own
   reason is relayed rather than replaced by a generic message.
3. **Given** a session in which the kernel reported lost events, **When**
   statistics are read, **Then** the lost count is present and distinct from
   every packet counter.
4. **Given** a session in which any event was lost, **When** the tree is read,
   **Then** it reports itself as possibly incomplete, and a consumer can tell
   that from the tree rather than only from the statistics.
5. **Given** a watcher that fails after the session has started, **When** the
   failure occurs, **Then** it is reported with the platform's reason and the
   run does not report the session as having completed normally.

---

### Edge Cases

- What happens when the startup snapshot and the event stream both report the
  same process, because it was created after the subscription and before the
  snapshot? The tree must hold one node, not two, and the node must prefer
  creation-time ancestry over snapshot ancestry. This is the expected case
  rather than a rare one, because FR-007 subscribes first precisely so that this
  overlap exists in place of a gap.
- What happens when the two sources report that process in either order,
  because the snapshot is a batch and the event stream is not? The same single
  node must result whichever arrives first.
- What happens when the platform reports no start time for a process the
  snapshot found? The node records it as unknown and orders before every
  observed event, so it is never selected over a process whose start was
  observed.
- What happens when a process exits before the tree has folded in its start
  event? The exit must not be discarded as unmatched merely because the fold has
  not caught up. A trace consumer delivers from several buffers and is not
  obliged to order events by timestamp across them, so this is an ordinary case
  rather than a pathological one, and an unmatched exit is only unmatched at the
  end of the session.
- What happens when the parent identifier in a start event names a process that
  exited and whose identifier was reassigned before the event was folded in? The
  parent must resolve by the pair of identifier and time, not by identifier
  alone, or the new node attaches to an unrelated subtree.
- What happens when a command line is unavailable for a process the snapshot
  found? The tree records it as unavailable. It does not record an empty command
  line, and it does not open a handle carrying memory-read rights to recover
  one.
- What happens when a command line contains non-UTF-8 sequences or is extremely
  long? It is recorded as observed, without truncation or normalization, per
  P-9.
- What happens to a process created before the session and still running at its
  end? It appears in the tree with snapshot ancestry and no exit timestamp.
- What happens when the same image name appears at several depths? Nothing
  special; the tree stores nodes, not names. This is the case section 15.4's
  ambiguity check and section 10.3's `descends_from` exist for, and it is
  recorded here because the tree is what makes both answerable.
- What happens on a non-Windows target? `fragcap-attr` must still build, with
  the ETW watcher absent rather than stubbed into something that compiles and
  reports an empty machine.
- What happens when the session runs long enough for tens of thousands of
  processes to come and go? Exited nodes are retained for the session by section
  10.2, so memory grows with the number of processes observed, and that growth
  must be bounded by a documented per-node cost rather than by discarding nodes.

## Requirements *(mandatory)*

### Functional Requirements

**The watcher, section 10.1**

- **FR-001**: The system MUST provide a `ProcessWatcher` implementation backed
  by the ETW kernel process provider, in `fragcap-attr`.
- **FR-002**: The watcher MUST receive a start event for every process created
  and an exit event for every process terminated, system wide, for the duration
  of the session.
- **FR-003**: A start event MUST carry the new process identifier, the creating
  process's identifier as recorded at the instant of creation, the image path,
  the command line, and the time.
- **FR-004**: An exit event MUST carry the process identifier and the time.
- **FR-005**: The watcher MUST consume a trace session it creates for itself. It
  MUST NOT contend for a session that exists once per machine, and MUST NOT
  stop, reconfigure, or take over a session it did not create.
- **FR-006**: The watcher MUST take one snapshot of already-running processes at
  startup, so that targets running before fragcap started are present.
- **FR-007**: The watcher MUST subscribe before taking the startup snapshot,
  never the reverse, so that a process created while the watcher is starting is
  reported twice rather than not at all.
- **FR-008**: The startup snapshot MUST use query-only process enumeration, and
  every process handle it opens MUST state its requested access rights
  explicitly at the call site.
- **FR-009**: The startup snapshot MUST NOT open a handle against any target
  process, for a start time or for anything else, and MUST record a start time
  it therefore does not have as unknown rather than fabricating one.

  This requirement originally permitted `OpenProcess` with
  `PROCESS_QUERY_LIMITED_INFORMATION`, which constitution P-1 does allow. It was
  narrowed during integration with S10, whose process enumeration had already
  made the stronger argument and backed it with a lint: P-1's requirement that a
  handle state its rights exists because a handle request is a thing a reviewer
  has to check, and opening nothing removes the thing to check rather than
  documenting it. The cost is that a process found already running has no start
  time, which FR-024 already gives a defined meaning.
- **FR-010**: The system MUST NOT poll for processes, and MUST NOT read a parent
  identifier from a running process for any purpose other than the startup
  snapshot, where it is recorded as snapshot ancestry per FR-022.
- **FR-011**: The system MUST NOT provide a polling fallback when the trace
  session cannot be consumed. Absence of the required privilege MUST be reported
  as such.
- **FR-012**: `subscribe` MUST return an independent receiver on each call, each
  observing every event published after that call.
- **FR-013**: The channel between the trace consumer and its subscribers MUST
  NOT discard events. It is unbounded rather than bounded drop-oldest, because
  losing a start event loses a subtree while losing a packet loses a packet.
- **FR-014**: The watcher MUST report the count of events the kernel itself
  reported losing, in a report the watcher owns, and that count MUST be
  surfaced in the run's output distinct from every packet counter.
- **FR-015**: The watcher's report MUST NOT be folded into `CaptureStats`.
  Section 12.4's conservation identity is asserted over `CaptureStats`, and
  quantities that are not packets MUST NOT enter it.
- **FR-016**: The watcher MUST relay the platform's own reason when a trace
  session cannot start, rather than replacing it with a generic message.
- **FR-017**: The ETW watcher MUST sit behind a Cargo feature that is off by
  default, so that building the workspace and running the ordinary check set
  requires neither elevation nor a Windows machine.
- **FR-018**: `fragcap-attr` MUST build for a target with no process telemetry
  backend, with the ETW watcher compiled out rather than replaced by a stub that
  reports an empty machine.

**The tree, section 10.2**

- **FR-019**: The system MUST maintain an in-memory process tree keyed by a
  synthetic session-local identifier, in `fragcap-core`, built by folding
  `ProcessEvent` values. It MUST hold no platform interface and MUST perform no
  I/O.
- **FR-020**: A synthetic identifier MUST NOT be reused within a session.
- **FR-021**: Each node MUST record the operating system identifier, the
  parent's synthetic identifier where resolved, the image path, the command
  line, the start timestamp, and the exit timestamp where applicable.
- **FR-022**: Each node MUST record the provenance of its ancestry: observed at
  creation, or read from the startup snapshot. The two MUST NOT be
  interchangeable, and provenance MUST be carried on the node rather than
  derived from whether a parent resolved.
- **FR-023**: A node MUST be resolvable from the pair of operating system
  identifier and timestamp, returning the node live at that timestamp or
  nothing.
- **FR-024**: A node whose start time is unknown MUST order before every
  observed event in the session, and a resolution by identifier and time MUST
  select it only when no node with a known start time covers that time.
- **FR-025**: Two processes sharing an operating system identifier whose
  lifetimes do not overlap MUST be two nodes, and neither MUST inherit the
  other's children.
- **FR-026**: A parent identifier in a start event MUST be resolved by the pair
  of identifier and time, not by identifier alone.
- **FR-027**: Exited nodes MUST be retained for the session, so that packets
  arriving after a process terminates remain attributable.
- **FR-028**: The tree MUST retain every process it observes, unfiltered.
  Deciding which processes matter is section 15.2's capture scope and belongs to
  S12, and a tree that discarded a process would break `descends_from` through
  it for every descendant.
- **FR-029**: The tree MUST report how many nodes it retains, so that the cost
  of FR-027 and FR-028 is observable during a session rather than estimated.
- **FR-030**: A start event whose parent resolves to no node MUST produce a node
  with unresolved ancestry, recorded as such, rather than a node attached to an
  arbitrary parent or a discarded event.
- **FR-031**: The fold MUST NOT assume events arrive in timestamp order. An exit
  event matching no live node MUST be held against a start event that has not
  yet arrived, and MUST be counted as unmatched and surfaced only once the
  session ends with no start event for it. No node MUST be fabricated for one
  either way.
- **FR-032**: The tree MUST report the ancestry path of any node, in creation
  order, including nodes that have exited.
- **FR-033**: The tree MUST reconcile a process reported by both the startup
  snapshot and the event stream into one node, in either arrival order, and that
  node MUST carry creation-time ancestry.
- **FR-034**: The tree MUST report whether any event is known to have been lost
  during the session, so that a consumer can tell an incomplete tree from a
  complete one while holding only the tree.
- **FR-035**: Command lines MUST be recorded verbatim. No path may alter,
  truncate, mask, normalize, or withhold one.
- **FR-036**: Where a command line is genuinely unavailable, the tree MUST
  record it as unavailable rather than as empty.
- **FR-037**: No code path in this slice may open a process handle carrying
  memory-read rights in order to obtain a command line or any other field.
- **FR-038**: The tree MUST make the image file name derivable from the image
  path, because section 10.3's `exe` predicate matches the file name while
  `path_contains` and `path_regex` match the full path, and S12 needs both from
  one recorded value.

**Offline testability, section 25.1**

- **FR-039**: The system MUST provide a scripted `ProcessWatcher` that publishes
  a declared sequence of process events, mirroring the scripted attributor S04
  built for the same reason.
- **FR-040**: The declared sequence MUST be expressible without a Windows
  machine, without elevation, and without a game, and MUST be able to express
  both launcher chains recorded in Appendix D.
- **FR-041**: A tree built from scripted events MUST be indistinguishable from a
  tree built from the same events arriving from ETW, so that a test passing
  against a script is one the real watcher has to satisfy.

**Constitutional constraints**

- **FR-042**: No technique on the section 19.3 denylist may be used. Process
  observation uses ETW kernel providers and query-only enumeration, which are
  the permitted entries in section 19.2, and nothing else.
- **FR-043**: `fragcap-core` MUST NOT acquire a platform-specific dependency, an
  I/O crate, or a telemetry library as a result of this slice.
- **FR-044**: No attribution logic may enter the watcher, and no packet
  acquisition may enter it either. The watcher names neither `PacketSource` nor
  `FlowAttributor` in any signature.
- **FR-045**: Every discard path introduced by this slice MUST have a named
  counter that is surfaced in the run's output. Where this slice deliberately
  has no discard path, that MUST be a property of the design rather than an
  uncounted discard.
- **FR-046**: Every term introduced by this slice MUST receive a glossary entry
  in the same change.
- **FR-047**: Any dependency added MUST carry a license from the allowlist in
  the constitution's licensing section, across its whole graph, and MUST declare
  a minimum toolchain no higher than the workspace minimum.
- **FR-048**: `cargo xtask deps` MUST continue to pass, with no edge from
  `fragcap-attr` to any sibling crate.

**Scope boundary**

- **FR-049**: This slice MUST NOT implement stage matching, lifecycle classes,
  session lifecycle, or stop conditions. Sections 10.3 through 10.6 belong to
  S12. The tree MUST carry a place for a node's matched stage without deciding
  what goes in it.
- **FR-050**: This slice MUST NOT wire the watcher into the capture pipeline's
  control thread. Section 8.6's control thread arrives with the slices that own
  its other occupants.

### Key Entities

- **Process event**: An observed change in the set of running processes. A start
  carries creation-time ancestry, an image path, a command line, and a time; an
  exit carries an identifier and a time.
- **Process record**: A process as the startup snapshot found it. Distinct from
  a process event because its ancestry was read rather than observed.
- **Process node**: One process in the tree. Operating system identifier,
  synthetic identifier, resolved parent, image path, command line, start and
  exit timestamps, ancestry provenance, and a reserved place for the stage S12
  will bind.
- **Synthetic process identifier**: The session-local identity of a node, never
  reused within a session. Distinct from the operating system identifier, which
  is.
- **Process tree**: The set of nodes and the ancestry relation over them, plus
  whether any event is known to have been lost.
- **Ancestry provenance**: Whether a node's parent was observed at creation or
  read from the startup snapshot. Carried, not derived.
- **Watcher report**: What the watcher itself observed about its own operation:
  events the kernel reported losing, and whether the session is still running.
  Separate from `CaptureStats` because it counts things that are not packets.
- **Process script**: A declared sequence of process events, replayable through
  the scripted watcher.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: On an elevated Windows session, a process the test itself spawns
  is observed at creation with the test process named as its parent, and its
  exit is observed, with no polling anywhere in the path.
- **SC-002**: Both launcher chains recorded in Appendix D replay through the
  scripted watcher into trees of the observed depth, five levels and seven
  levels respectively, verified with no elevation and no game. Specification
  section 5.4's prose says six for the second; its own diagram and Appendix
  D.3's topology both list seven processes, and this criterion follows the two
  that agree. Recorded as a deviation below.
- **SC-003**: The three Division 2 processes sharing one image name are three
  distinct nodes distinguishable by ancestry, which is the property section 15.4
  requires and section 10.3's `descends_from` is built on.
- **SC-004**: A process whose identifier is reassigned after it exits produces
  two nodes, and a query by identifier and timestamp resolves each to the
  process live at that time.
- **SC-005**: Ancestry is answerable for a node whose entire parent chain has
  exited.
- **SC-006**: Every node's ancestry provenance is readable, and a node built
  from the startup snapshot is distinguishable from one built from a start event
  without inspecting anything but the node.
- **SC-007**: A session in which events were lost reports the loss in its
  statistics and reports the tree as possibly incomplete, and neither is
  inferable only from the other.
- **SC-008**: Without the privilege the trace session requires, the watcher
  fails with an error naming that privilege, and no polling path exists to fall
  back to, verified by inspection of the code as well as by the test.
- **SC-009**: `cargo xtask ci` passes on a machine with no elevation, no capture
  driver, and no game, because the ETW watcher is behind a feature that is off
  by default.
- **SC-010**: `fragcap-core` builds for a target with no process telemetry
  backend, and `fragcap-attr` builds there with the ETW watcher absent.
- **SC-011**: Every test of section 10.2 runs at tier 1.
- **SC-012**: No source file in the workspace opens a process handle carrying
  memory-read rights, verified mechanically alongside the existing transmit-call
  check.
- **SC-013**: A command line observed by the watcher reaches the tree byte for
  byte, including one containing characters outside ASCII and one longer than
  any plausible buffer an implementation might have chosen.
- **SC-014**: A process reported by both the startup snapshot and the event
  stream produces exactly one node whichever source the tree folds first, and
  that node carries creation-time ancestry in both orders.
- **SC-015**: A node whose start time is unknown is never selected by a
  resolution that a node with a known start time covers, verified over a case
  where both nodes share an operating system identifier.
- **SC-016**: The watcher's lost-event count is readable from the watcher's own
  report and appears nowhere in `CaptureStats`, verified by the conservation
  identity continuing to hold unchanged.
- **SC-017**: A fold of an event count at session scale, taken as ten times the
  larger reconnaissance session, completes and reports a retained node count
  equal to the number of distinct processes observed, with no node discarded.
- **SC-018**: `cargo xtask deps` and `cargo xtask license` continue to pass.

## Assumptions

- Windows is the only platform with a process telemetry backend in this slice.
  The seam this slice fills is the one a later platform backend will fill.
- The ETW kernel process provider carries the image path and the command line on
  its start event, as section 10.1 states and as the reconnaissance sessions
  relied on when they scanned 3,694 command lines.
- Consuming a kernel ETW session requires administrative privilege, per
  constraint C-4 and section 19.5. Appendix D.1's correction to section 19.5
  concerns the capture handle, not the trace session.
- Tests requiring an elevated trace session are tier 2 by section 25.2 and run
  on the Windows runner rather than in the ordinary check set. As of S09 the
  `platform` workflow has real triggers but has never completed.
- Stage matching consumes this tree rather than the other way round. S12 depends
  on S11 and not the reverse, so the tree is designed without reference to the
  profile schema beyond reserving a place for a matched stage.
- The pipeline's control thread of section 8.6 does not exist yet. S08 built the
  capture and sink threads; the control thread arrives when the filter manager
  and the attributor have somewhere to be assembled, which is S13 and S14.
- The unprivileged process telemetry source Appendix D.1 records is a property
  of the platform observed during reconnaissance, not a commitment fragcap makes
  to use it. Appendix D.4 records what it cost the harness that did.
- S10 is in development in parallel and shares the `fragcap-attr` crate. This
  slice adds a module beside the socket table attributor and does not modify it.

## Dependencies

- **S02** supplies `ProcessEvent`, `ProcessRecord`, the `ProcessWatcher` trait,
  and `Timestamp`, all of which this slice extends or implements.
- **S04** supplies the scripted attributor whose shape the scripted watcher
  mirrors.
- **Section 5.3 and section 5.4** supply the reason creation-time ancestry is
  the only reliable kind, and the observed chain depths this slice is verified
  against.
- **Appendix D** supplies both focal titles' process topologies, which are the
  fixtures for the tree.
- **S10** is not a dependency in either direction. It answers which process
  holds a socket; this slice answers which processes exist and which created
  which.

## Deviations Recorded By This Slice

Each is recorded here and promoted to specification section 29 at the next
version, per the constitution's deviation rule.

- **A command line on the start event.** `ProcessEvent::Started` as S02 declared
  it carries an image and a parent and no command line. Section 10.1 states the
  event carries one and section 10.2 makes it a tree field, so the variant gains
  it. The enum is `#[non_exhaustive]`, which permits new variants but not new
  fields on an existing one, so this is a breaking change to the variant and is
  recorded rather than made quietly. S02 anticipated it in the module's own
  documentation.
- **An availability state for a command line.** Section 10.2 lists the command
  line as a tree field without qualification. A process the startup snapshot
  finds may not yield one without a handle carrying memory-read rights, which
  P-1 forbids, so the field admits an unavailable state. Recording that a value
  is unavailable is not withholding it, and P-9's declared-omission rule is
  satisfied by making the absence visible rather than by substituting an empty
  string.
- **Ancestry provenance on the node.** Section 10.2 lists the tree's node fields
  and does not include provenance, because it does not address the startup
  snapshot's weaker ancestry. Section 5.3 establishes that the two kinds differ
  in reliability, so the difference is carried.
- **A watcher-owned report beside the capture's statistics.** Section 26.2 lists
  what runtime statistics carry and names only packet quantities, because it was
  written before there was anything else to count. The watcher's lost-event
  count belongs in the run's output and does not belong in `CaptureStats`, so it
  arrives as its own report rather than as a widening of that structure.
  Recorded because section 26.2's list is the architecture of record for what an
  operator sees during a run.
- **The observed parent identifier survives failing to resolve.** Section 10.2's
  node fields list a parent and say nothing about a parent that names no node in
  the tree. The identifier is kept on the node anyway, because it is an
  observation, and P-9 does not permit discarding one merely because nothing
  downstream could use it. Found while implementing rather than while planning.
- **Section 5.4 says the Division 2 chain is six levels; it is seven.** Section
  5.4's own diagram lists seven processes and Appendix D.3's topology lists the
  same seven. Only the prose sentence between them says six. Found by writing
  the chain out as a test, which is the argument for writing it out as a test.
- **The image field is a path, and the file name is derived.** S02's
  `ProcessEvent::Started` names its field `image` and its tests use a bare file
  name. Sections 10.2 and 10.3 require the full image path, with the file name
  as the subject of one predicate and the path as the subject of two others.
  The field is settled as the path.
