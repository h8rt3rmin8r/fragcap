# Feature Specification: Socket Table Attributor

**Feature Branch**: `feat/socket-table-attributor`

**Created**: 2026-08-09

**Status**: Draft

**Slice**: S10 (specification section 11; constitution P-1, P-2, P-3, P-4, P-6,
and P-9)

**Input**: Implement specification section 11 (flow attribution) as the
production `FlowAttributor` in `fragcap-attr`: a socket table snapshot joined
against captured flows by 5-tuple, the section 11.2 cadence with its two
on-demand triggers, the section 11.4 retention window with its fidelity
marking, the section 11.5 rule that an unattributed packet is retained and
counted, and the section 11.6 publication contract. The Windows backend reads
the IP Helper API extended tables behind an off-by-default feature, in the same
shape S09 used for `live`.

## Overview

The project is named for attribution and has never attributed anything. Nine
slices have built a capture tool whose every attribution to date came from
`ScriptedAttributor`, which reads a text file that a test wrote. That is the
right thing to have built first, and specification section 25.1 is explicit
about why: a scripted attributor is what makes the pipeline a deterministic
function from fixture to output, testable with no driver, no elevation, and no
game. But it means the seam has never been asked a question it could get wrong.

S10 is the first attributor that can be wrong, and the shape of the slice
follows from that.

**Attribution is a join, and the join has a race.** Section 11.1 makes it a
two-stage lookup: endpoint to process identifier through the socket table, then
process identifier to image name and role through the process tree. The socket
table is sampled rather than subscribed, so between two samples a socket can
open, carry traffic, and close, and the packets on it are unattributed. Section
11.3 accepts that rather than eliminating it, and bounds it: gameplay
connections are held for minutes to hours, and Appendix D measured 12,249
closed connections across both focal titles with **none under 250 ms**. The
exposure is real and it is small, and the slice's job is to keep it that way
rather than to pretend it is zero.

**The cost of the wrong access path is three orders of magnitude.** Section
11.2 records that a table-interface snapshot of roughly 1800 sockets costs one
to three milliseconds, and that the object-model projection of the same data
costs 1400 to 2000. A one-second cadence is affordable by a factor of several
hundred through the first path and impossible through the second. This is
written into the specification because the convenient interface is the one an
implementation reaches for, and reaching for it produces the conclusion that
polling is unworkable, which is false.

**Attribution that resolves is not the same as attribution that is observed.**
Section 11.4 retains an endpoint for a grace period after it leaves the table,
because a closing connection produces final packets processed after the socket
is gone, and discarding attribution at that instant would leave the tail of
every connection unattributed. A retained answer can be wrong in one specific
way: the endpoint closed and its port was reassigned inside the window.
`Fidelity::Retained` exists so a consumer can see which answers carry that
exposure. It was added in S06 after review caught the pcapng writer inferring
`Live` from the mere existence of an attribution, and this slice is the first
that produces both values for real. P-9 makes the distinction binding rather
than advisory.

**What this slice does not know.** Stage two of the section 11.1 lookup is the
process tree, and the process tree is S11. This slice resolves the process
identifier from the table and takes the image name from an injected seam, so
that S11 replaces a default rather than restructures an attributor. Role and
stage stay `None` here; S12 fills them.

**The publication mechanism was deliberately deferred to this slice.** S08 left
the attributor behind a mutex that every capture thread locks per packet, and
said so in `pipeline/mod.rs`: building the section 11.6 publication mechanism
then would have fixed the snapshot's shape before this slice knew what a socket
table snapshot costs to publish. It now knows. Section 11.6 requires that the
control thread build a new immutable map per refresh and publish it atomically,
and that the capture thread read it without blocking packet acquisition. Making
that true is part of this slice.

## Clarifications

### Session 2026-08-09

- Q: Should the socket creation timestamp that Appendix D found exposed on the
  TCP table be used to narrow the section 11.3 race window in this slice, or
  deferred? -> A: Used. It is the difference between a correct and an incorrect
  answer under port reuse, not an optimization. A socket created after a
  packet's timestamp cannot have owned that packet, and rejecting it is the
  only mechanism available that distinguishes the previous owner of a reused
  port from the current one. Section 11.3's race window narrows to sockets that
  both open and close inside one interval, which is what Appendix D D.1 already
  claims for it. UDP carries no creation time and takes no such filter, which
  is a per-protocol asymmetry recorded rather than papered over.

- Q: Does a dual-stack socket, meaning an IPv6 wildcard bind receiving IPv4
  traffic, match in this slice? `AttributionKey::local_matches_bind` names S10
  as its owner and Appendix D found no focal title relying on it. -> A: Yes,
  matched, and only for UDP, which is the protocol whose key is the local
  endpoint alone and therefore the only one taking the wildcard allowance at
  all. A bind of `[::]:P` matches an observed IPv4 local endpoint on port `P`.
  Refusing it would make a whole class of sockets silently unattributable, and
  a silent unattributable class is worse than a rare false positive that
  `Fidelity` cannot mark: the port must still match, and the alternative is
  every packet on such a socket counted as unresolved. Appendix D's finding
  means the rule is unexercised by the focal titles rather than wrong.

- Q: How does an `&self` lookup trigger the section 11.2 unseen-endpoint
  snapshot, given that `FlowAttributor::resolve` cannot mutate? -> A: The
  lookup records a request rather than performing a refresh. An unresolved
  lookup on an endpoint the current snapshot does not carry sets a shared
  request flag, subject to the 200 ms rate limit; the owner acts on it at its
  next opportunity. This keeps acquisition free of table reads, which section
  11.6 requires for the same reason it requires lock-free reads, and it makes
  the trigger observable in a test without a real table or a real clock.

- Q: What supplies time, given that a one-second cadence, a 200 ms rate limit,
  and a thirty-second retention window are all untestable at tier 1 against a
  real clock? -> A: An injected clock, defaulting to the system clock. Every
  cadence and retention rule is then a pure function of declared instants, and
  the whole of section 11.2 and 11.4 is exercised in microseconds with no
  sleeping and no flakiness. A test that sleeps for thirty seconds to prove a
  thirty-second window is a test nobody runs.

- Q: Does removing the per-packet attributor mutex from the pipeline belong to
  this slice? -> A: Yes. S08 deferred it here by name, section 11.6 is one of
  this slice's specification sections, and the mutex is the thing section 11.6
  forbids. It requires adding `Sync` to `FlowAttributor`, recorded as a
  deviation in the same shape S09 used when it added `Send` to `PacketSource`.

- Q: When several table entries match one flow, what is the total order that
  picks the winner? "Prefer the more exact match" does not settle it, and the
  specification requires the same table and the same flow always produce the
  same answer. -> A: A four-rank exactness ladder, then two total tiebreaks
  inside a rank. Ranks, most exact first: a TCP entry matching both endpoints;
  an entry matching the local address and port exactly; an entry bound to a
  wildcard of the same address family on that port; an entry bound to an IPv6
  wildcard matching an IPv4 local endpoint. Inside a rank, prefer the entry
  with the latest creation instant that is still at or before the packet's
  instant, because that is the socket that most plausibly owned the packet
  under port reuse; entries with no reported creation instant sort below any
  that has one. If two remain indistinguishable, the lower process identifier
  wins, which decides nothing meaningful and is there only so the order is
  total rather than dependent on table iteration order.

- Q: Where do the interval, the retention period, and the rate limit come from?
  -> A: Plain configuration values carried on the attributor, with the
  specification's defaults. Not the profile. S05's `capture` section accepts
  exactly five keys and refuses unknown ones, and it refuses them deliberately:
  a key with no consumer is a key whose behavior is untested and whose meaning
  is set by whoever first reads it. S14 owns adding keys to that schema when it
  owns a command line that can set them. Introducing three here would put
  unexercised keys into a schema whose whole value is that it rejects what it
  does not understand.

- Q: Does the default process namer resolve an image name eagerly, once per
  refresh, or lazily on each unresolved identifier? -> A: Eagerly, during the
  refresh, and only for the identifiers the table actually reported. The names
  become part of the immutable published snapshot. Lazy resolution would put an
  operating system call on the acquisition path for the first packet of every
  new process, which is precisely the stall FR-017 and section 11.6 forbid, and
  it would do it at the least convenient moment: the start of a session, when
  the most sockets are opening at once.

- Q: The retention window is measured from what origin? -> A: The instant the
  endpoint was last observed present in a table, not the instant of the refresh
  that first noticed it gone. Those differ by up to one interval, and measuring
  from the later one would make a thirty-second window silently thirty-one
  seconds. The distinction is small and the reason to be exact about it is not:
  a retained answer carries a marked risk of being wrong, and quietly widening
  the window that produces them widens that risk without saying so.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Learn which process a captured flow belonged to (Priority: P1)

An operator captures traffic while a game is running and wants each flow in the
output to name the process that produced it, rather than an address and a port
they must correlate by hand.

**Why this priority**: This is the project's reason to exist. Every other story
here refines an answer this one produces.

**Independent Test**: Drive the attributor over a synthetic socket table
declaring known endpoints and known owners, resolve a set of flows against it,
and assert each resolves to the declared owner. Requires no elevation, no
capture driver, and no game.

**Acceptance Scenarios**:

1. **Given** a socket table carrying a TCP entry for local endpoint L and
   remote endpoint R owned by process P, **When** a flow keyed on L and R is
   resolved, **Then** the answer names P and reports fidelity `Live`.
2. **Given** a socket table carrying a UDP entry bound to local endpoint L
   owned by process P, **When** a datagram flow with local endpoint L and any
   remote endpoint is resolved, **Then** the answer names P, because a UDP
   table entry carries no remote and section 8.4 forbids inventing one.
3. **Given** a socket table carrying no entry matching a flow, **When** that
   flow is resolved, **Then** the answer is unresolved, the packet is retained,
   and the unattributed counter advances.
4. **Given** a resolved process identifier, **When** the answer is built,
   **Then** it carries the process image name supplied by the naming seam, and
   role and stage are absent because no profile has been matched yet.

---

### User Story 2 - Keep the tail of a connection attributed (Priority: P1)

An operator captures a session through to a clean disconnect and expects the
final packets of each connection to name the same process as the rest of it,
not to appear as an unattributed tail.

**Why this priority**: Without retention, every connection in every capture
ends in a run of unattributed packets, which is both wrong and conspicuous.

**Independent Test**: Declare an endpoint present in one snapshot and absent
from the next, resolve flows on it at instants inside and outside the grace
period, and assert on both the answer and its fidelity.

**Acceptance Scenarios**:

1. **Given** an endpoint that was present and has left the table, **When** a
   flow on it is resolved at an instant inside the retention window, **Then**
   the answer names the previous owner and reports fidelity `Retained`.
2. **Given** the same endpoint, **When** a flow on it is resolved at an instant
   after the retention window has elapsed, **Then** the answer is unresolved.
3. **Given** an endpoint present in the current table, **When** it is resolved,
   **Then** fidelity is `Live` and never `Retained`, so that the two are never
   conflated in output.
4. **Given** an endpoint that left the table and then reappeared under a new
   owner inside the window, **When** a flow on it is resolved, **Then** the
   live entry wins over the retained one, because the table is evidence and
   retention is inference.

---

### User Story 3 - Attribute a socket that opened moments ago (Priority: P2)

An operator starts a capture, launches a title, and expects the flows the
client opens on startup to be attributed rather than lost to the interval
between two snapshots.

**Why this priority**: A game session's first seconds are when the most sockets
open at once, and a fixed poll with no triggers is at its worst exactly there.

**Independent Test**: Advance an injected clock and declare table contents per
instant, then assert that an unresolved lookup on an unseen endpoint records a
refresh request, that the request is rate limited, and that a declared process
start records one immediately.

**Acceptance Scenarios**:

1. **Given** the default cadence, **When** the interval elapses, **Then** the
   next refresh is due.
2. **Given** an unresolved lookup on an endpoint the current snapshot does not
   carry, **When** the lookup happens, **Then** a refresh is requested.
3. **Given** a refresh request has just been recorded, **When** further
   unresolved lookups occur inside 200 ms, **Then** no additional request is
   recorded, so a burst of unattributable traffic cannot drive the table read
   rate.
4. **Given** 200 ms has elapsed since the last triggered request, **When**
   another unresolved lookup on an unseen endpoint occurs, **Then** a request
   is recorded again.
5. **Given** a process start matching a profile stage is reported, **When** it
   is reported, **Then** a refresh is requested immediately and without regard
   to the rate limit, because a newly matched process is about to open sockets.

---

### User Story 4 - Capture at rate while attribution is refreshing (Priority: P2)

An operator captures a busy interface and expects the attribution refresh not
to be visible as a periodic stall in packet acquisition.

**Why this priority**: Section 11.6 exists for this, and the pipeline currently
violates it: every capture thread takes the same mutex per packet.

**Independent Test**: Resolve concurrently from several threads while the
snapshot is replaced, and assert that every answer is internally consistent and
that no reader observes a partially built map.

**Acceptance Scenarios**:

1. **Given** several capture threads resolving flows, **When** a refresh
   publishes a new snapshot, **Then** each reader sees either the whole old
   snapshot or the whole new one, and never a mixture.
2. **Given** a lookup in progress, **When** a refresh publishes, **Then** the
   lookup is not blocked by the publication and the publication is not blocked
   by the lookup.
3. **Given** the pipeline, **When** several capture threads resolve, **Then**
   no lock is taken per packet on the attribution path.

---

### User Story 5 - Run the whole check set on a machine with no platform (Priority: P2)

A contributor on any machine runs the repository's check set and expects it to
pass without a capture driver, without elevation, and without the platform this
slice's backend targets.

**Why this priority**: S09 established this property and it is cheap to lose.
An attributor that only compiles on Windows would take the project's whole
offline testing claim with it.

**Independent Test**: `cargo xtask ci` on a machine with no npcap and no
Windows-only build inputs.

**Acceptance Scenarios**:

1. **Given** a machine without the platform, **When** the check set runs,
   **Then** it passes, because the platform backend is behind a feature that is
   off by default.
2. **Given** the default feature set, **When** the crate is built for a target
   with no platform backend, **Then** it builds with the backend absent rather
   than stubbed into something that compiles and lies.

---

### Edge Cases

- What happens when the socket table reports two entries that both match one
  flow? The match must be deterministic and documented rather than dependent on
  table order, because a nondeterministic attribution is one that changes
  between runs over the same traffic.
- What happens when a port is reused by a different process inside the
  retention window? A live entry beats a retained one. When only the retained
  entry exists, the answer carries `Retained` so that the exposure is visible
  in the output rather than hidden.
- What happens when a socket's creation time is later than the packet's
  timestamp? That socket cannot have owned that packet and is not matched.
- What happens when the table read fails? The failure is reported, the previous
  snapshot stays published rather than being replaced by an empty one, and the
  run continues. Replacing a good snapshot with an empty one on a transient
  failure would silently unattribute every subsequent packet.
- What happens when a process identifier resolves to no image name? The
  attribution still names the process identifier, because the identifier is
  what was observed. Reporting nothing would discard an observation, which P-9
  forbids.
- What happens when a process identifier is reused by the operating system? The
  answer is the owner the table reported at the instant it was read; nothing
  here claims more than that.
- What happens when the same local port is bound by both a TCP and a UDP
  socket, owned by different processes? They are distinct entries and never
  collide, because the protocol participates in the key.
- What happens when an IPv6 wildcard bind exists alongside a specific IPv4
  bind on the same port? The specific bind wins, because it is the more exact
  match.
- What happens when the table is empty? Every lookup is unresolved, every
  packet is retained and counted, and nothing errors.
- What happens when retention is configured to zero? An endpoint that leaves
  the table is immediately unresolvable, and no answer ever carries `Retained`.

## Requirements *(mandatory)*

### Functional Requirements

**The socket table snapshot, section 11.1**

- **FR-001**: The system MUST represent a socket table snapshot as an immutable
  value mapping an endpoint, meaning the tuple of protocol, local address, and
  local port, to the owning process identifier.
- **FR-002**: A snapshot MUST carry, for a TCP entry, both the local and the
  remote endpoint, and MUST NOT carry a remote endpoint for a UDP entry.
- **FR-003**: A snapshot MUST carry the socket creation instant for any entry
  whose platform reports one, and MUST distinguish "not reported" from any
  particular instant.
- **FR-004**: A snapshot MUST be constructible from declared contents, so that
  every rule below is testable without a platform.

**Matching, sections 8.4 and 11.1**

- **FR-005**: A TCP flow MUST match a TCP entry on both endpoints exactly.
- **FR-006**: A UDP flow MUST match a UDP entry on the local endpoint alone,
  and MUST NOT be matched against any remote endpoint.
- **FR-007**: A UDP flow MUST match an entry bound to a wildcard address on the
  same port, including an IPv6 wildcard bind against an IPv4 local endpoint.
- **FR-008**: When more than one entry matches, the system MUST rank candidates
  by exactness, most exact first: both endpoints matched; local address and
  port matched exactly; a wildcard bind of the same address family on that
  port; an IPv6 wildcard bind against an IPv4 local endpoint.
- **FR-008a**: Within one exactness rank, the system MUST prefer the entry
  whose creation instant is the latest of those at or before the packet's
  instant, and MUST rank an entry with no reported creation instant below any
  entry that has one.
- **FR-008b**: The resulting order MUST be total, so that the same table and
  the same flow always produce the same answer regardless of the order the
  platform reported its rows in. Any residual tie MUST be broken by a declared
  rule rather than by iteration order.
- **FR-009**: An entry whose creation instant is later than the packet's
  instant MUST NOT match that packet. This applies to both protocols: the
  platform reports a creation instant on both the TCP and the UDP table when
  the table is requested by owning module rather than by owning process
  identifier, which planning found and Appendix D attributes to TCP alone.
- **FR-010**: Matching MUST use the packet's own instant and never the present
  moment.

**Cadence, section 11.2**

- **FR-011**: The refresh interval MUST default to one second and MUST be
  configurable.
- **FR-011a**: The interval, the retention period, and the rate limit MUST be
  carried as plain configuration on the attributor and MUST NOT be added to the
  profile schema in this slice, which accepts a closed key set and refuses
  unknown keys.
- **FR-012**: The system MUST report when a refresh is due, as a function of an
  injected clock rather than by sleeping.
- **FR-013**: A reported process start matching a profile stage MUST request an
  immediate refresh.
- **FR-014**: An unresolved lookup on an endpoint absent from the current
  snapshot MUST request a refresh.
- **FR-015**: Refresh requests arising from FR-014 MUST be rate limited to at
  most one per two hundred milliseconds, and the limit MUST be configurable.
  The limit MUST hold under concurrent requests from several capture threads:
  exactly one caller may claim each window.
- **FR-015a**: A refresh MUST NOT discard a request recorded against the index
  that refresh published. Where the two cannot be distinguished, an extra
  refresh is preferred to a missed one.
- **FR-016**: The rate limit MUST NOT apply to requests arising from FR-013.
- **FR-017**: A lookup MUST NOT read the socket table, enumerate a process, or
  open a handle. Requesting a refresh records a request and performs none of
  those. A lookup MAY read the injected clock, and only on the path that
  records a request, because the rate limit of FR-015 bounds a wall-clock cost
  and cannot be measured in capture time.

**Retention, section 11.4**

- **FR-018**: An endpoint that disappears from the table MUST remain resolvable
  for a grace period defaulting to thirty seconds, and the period MUST be
  configurable.
- **FR-018a**: The grace period MUST be measured from the instant the endpoint
  was last observed present in a table, and MUST NOT be measured from the
  refresh that first observed its absence.
- **FR-018b**: Retention MUST resolve a flow by the same matching rules and the
  same order as the current table, including the wildcard and dual-stack
  allowances of FR-007. A socket retained under a wildcard bind MUST still match
  the concrete local addresses it matched while live.
- **FR-018c**: Retention MUST record every socket separately rather than one per
  endpoint. Several sockets can occupy one local endpoint, and keeping only one
  would discard the others' attribution when they close.
- **FR-018d**: A retained record MUST carry the image name resolved while the
  socket was live, and that name MUST NOT be re-resolved from the process
  identifier afterwards. The owning process may have exited, and the platform
  reuses identifiers.
- **FR-019**: An answer resolved from a retained endpoint MUST carry fidelity
  `Retained`; an answer resolved from the current table MUST carry `Live`.
- **FR-020**: A live entry MUST take precedence over a retained entry for the
  same endpoint.
- **FR-021**: A retained endpoint MUST become unresolvable once the grace
  period has elapsed, measured against the packet's instant.
- **FR-022**: Retention MUST NOT grow without bound: entries past the grace
  period MUST be discarded on refresh.
- **FR-023**: `active_endpoints` MUST report the endpoints of the current
  snapshot together with those still inside the retention window.

**Unattributed packets, section 11.5**

- **FR-024**: An unresolved lookup MUST return no attribution rather than a
  fabricated one, and MUST NOT be an error.
- **FR-025**: An unresolved lookup MUST NOT cause a packet to be discarded on
  attribution grounds.
- **FR-026**: A packet carrying no flow key MUST NOT be counted as attempted
  and unresolved, preserving the distinction `AttributionState` has held since
  S02.

**Publication, section 11.6**

- **FR-027**: A refresh MUST build a new snapshot and publish it as a whole; a
  reader MUST never observe a partially built snapshot.
- **FR-028**: Lookups MUST NOT block a publication, and a publication MUST NOT
  block lookups.
- **FR-029**: Several capture threads MUST be able to resolve concurrently
  without taking a mutual exclusion lock on the attribution path.
- **FR-030**: A failed table read MUST leave the previously published snapshot
  in place and MUST be reported rather than swallowed.

**The process naming seam, section 11.1 stage two**

- **FR-031**: Resolving a process identifier to an image name MUST go through
  an injectable seam, so that the process tree of S11 replaces a default rather
  than restructures the attributor.
- **FR-032**: An attribution MUST be produced even when the seam supplies no
  name, carrying the observed process identifier.
- **FR-033**: The default naming implementation MUST use query-only process
  enumeration and MUST NOT open a process handle carrying memory rights.
- **FR-033a**: Names MUST be resolved during refresh, for the identifiers the
  table reported, and carried in the published snapshot. A lookup MUST NOT
  resolve a name on the acquisition path.

**Platform backend and portability**

- **FR-034**: The platform backend MUST read the operating system's extended
  TCP and UDP tables through the platform's table interface, and MUST NOT use
  the object-model projection of the same data.
- **FR-035**: The platform backend MUST be behind a cargo feature that is off
  by default, so the ordinary check set passes without the platform.
- **FR-036**: The crate MUST build for a target with no platform backend, with
  the backend absent rather than replaced by something that compiles and
  reports fabricated contents.
- **FR-037**: `fragcap-core` MUST acquire no platform dependency from this
  slice.
- **FR-038**: The attributor MUST contain no packet acquisition, and no
  dependency edge from `fragcap-attr` to `fragcap-capture` may be introduced.

**Vocabulary**

- **FR-039**: Every term this slice introduces MUST have a glossary entry in
  the same change, per constitution P-6.

### Key Entities

- **Socket table snapshot**: The contents of the operating system socket table
  at one instant, as an immutable value. Endpoints, owners, and for TCP the
  creation instant.
- **Socket table entry**: One row. Protocol, local endpoint, remote endpoint
  for TCP only, owning process identifier, creation instant when reported.
- **Retention map**: Endpoints that have left the table, with their last known
  owner and the instant they were last seen.
- **Attribution index**: The published, immutable structure a lookup reads. The
  current snapshot, the retention map, and the image names resolved for the
  identifiers both mention, so that a lookup reads only this value.
- **Refresh schedule**: When the next refresh is due, and whether one has been
  requested. Interval, rate limit, last refresh instant, last request instant.
- **Socket table source**: The seam that produces a snapshot. Implemented over
  the platform table interface, and over declared contents for tests.
- **Process namer**: The seam that turns a process identifier into an image
  name. Query-only, replaced by the process tree in S11.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Every matching rule in FR-005 through FR-010 is verified against
  a declared socket table, with no elevation, no capture driver, and no game.
- **SC-002**: A flow resolved against a table entry present at the packet's
  instant reports fidelity `Live`; the same flow resolved after the entry has
  left the table and inside the grace period reports `Retained`; after the
  grace period it is unresolved. All three are asserted for the same endpoint
  in one test.
- **SC-003**: A port reused by a second process inside the retention window
  resolves to the second process, not the first, whenever the second is present
  in the table.
- **SC-004**: A socket whose creation instant is later than a packet's instant
  never attributes that packet, verified for both protocols, and an entry with
  no reported creation instant still matches rather than being excluded.
- **SC-005**: The full section 11.2 cadence, meaning the interval, the process
  start trigger, the unseen endpoint trigger, and the 200 ms rate limit, is
  verified against an injected clock, and no test in the slice sleeps.
- **SC-006**: Concurrent resolution from several threads across a publication
  yields only whole snapshots, verified under a test that publishes repeatedly
  while readers resolve.
- **SC-007**: The pipeline resolves attribution without taking a mutual
  exclusion lock per packet, and the conservation identity established in S08
  continues to hold: for every sink, received plus `buffer_dropped` plus
  refusals equals `packets_captured`.
- **SC-008**: A failed table read leaves the previously published snapshot
  resolving exactly as it did before the failure, and reports the failure.
- **SC-009**: An unresolved lookup advances the unattributed counter and no
  packet is dropped, with the existing distinction between attempted and never
  attempted preserved.
- **SC-010**: `cargo xtask ci` passes on a machine with neither the capture
  driver nor the platform backend enabled.
- **SC-011**: `cargo xtask deps` passes, with `fragcap-core` taking no platform
  dependency and no edge from `fragcap-attr` to any sibling crate.
- **SC-012**: `cargo xtask lint` passes, including the check that no fragcap
  source names a prohibited call, and no process handle in this slice requests
  memory rights.
- **SC-013**: The existing corpus goldens are unchanged by this slice, because
  nothing here alters what the scripted attributor answers or what either
  writer emits for it.
- **SC-014**: A table whose rows are presented in a different order produces an
  identical answer for every flow, verified by resolving against the same
  declared entries permuted.
- **SC-016**: A socket bound to a wildcard address, including an IPv6 wildcard
  answering IPv4 traffic, resolves the same flows after it leaves the table as
  it did while present, differing only in fidelity.
- **SC-017**: Given the same set of sockets, the retained path and the live path
  select the same owner for a flow. A connection does not change owner at the
  instant its socket closes.
- **SC-018**: Several concurrent connections sharing one local endpoint each
  resolve to their own owner after all of them close.
- **SC-019**: An image name known while a socket was live survives the exit of
  the process that held it, and is not replaced when the platform reuses that
  process identifier.
- **SC-020**: Under concurrent requests from several threads, exactly one is
  accepted per rate-limit window.
- **SC-015**: No lookup reads the socket table, enumerates a process, or opens
  a handle. Every name, owner, and instant a lookup can return is present in
  the published snapshot before the lookup begins. The one exception is the
  injected clock, read only on the path that records a refresh request, which
  returns nothing to the caller.

## Assumptions

- The platform with a socket table backend in this slice is Windows, through
  the IP Helper API. Section 9.4's other backends are later work and fill the
  same seam.
- The process tree of S11 is the eventual stage-two lookup. Until it exists,
  the default namer is query-only enumeration, which constitution P-1 permits
  explicitly.
- Role and stage remain absent on every attribution this slice produces.
  Section 15's profile roles are matched in S12, and producing a role here
  would require guessing at a mapping that slice owns.
- The control thread of section 8.6 does not exist yet. This slice supplies an
  attributor whose refresh belongs on a control thread and whose lookups are
  safe from many capture threads, and S13 arranges the thread itself.
- Tests requiring the platform backend are tier 2 by section 25.2. The
  `platform` workflow exists for them and has never turned green; nothing here
  reports it as passing.
- Appendix D's measurement of one to three milliseconds per snapshot is taken
  as the basis for the one-second default. Nothing in this slice re-measures
  it.
- Section 11.3's acceptance threshold, the master specification's own SC-7 and
  not one of this document's SC-001 through SC-015, sets 99 percent of packets
  belonging to profiled processes. It cannot be evaluated in this slice,
  because evaluating it requires a profile, a process tree, and a live game. It
  is S17's to measure.

## Dependencies

- **S02** supplies `FlowAttributor`, `Attribution`, `Fidelity`, `Endpoint`,
  `FlowKey`, and `AttributionKey`, including the wildcard bind rule that names
  this slice as the owner of dual-stack handling.
- **S03** supplies the flow keys this attributor is asked about.
- **S04** supplies `ScriptedAttributor`, which stays as the tier 1 attributor
  for pipeline and corpus tests and is not replaced by this slice.
- **S08** supplies the pipeline whose per-packet attributor mutex this slice
  removes, and which deferred the section 11.6 mechanism here by name.
- **Q-1, Q-2, and Q-3** are resolved. Appendix D supplies the snapshot cost,
  the absence of a UDP remote endpoint, the presence of a creation timestamp,
  and the connection lifetime distribution that bounds section 11.3.

## Deviations Recorded By This Slice

- **Adding `Sync` to `FlowAttributor`.** Specification section 8.5 declares the
  trait with neither `Send` nor `Sync`; S09 added `Send` to `PacketSource`
  through this same process. Section 11.6 requires that several capture threads
  resolve concurrently without locking, which cannot be expressed while the
  pipeline must hold the attributor behind a mutex to share it. Recorded here
  and promoted to specification section 29.
- **A socket creation instant on the socket table entry.** Section 11 describes
  the snapshot as a map from endpoint to owning process identifier and says
  nothing about creation time. Appendix D D.1 found the platform exposes it and
  states that it narrows the section 11.3 race window; carrying it is what
  makes that true. Recorded here and promoted to specification section 29.
- **The UDP table also reports a socket creation instant.** Appendix D D.1
  records the creation timestamp as a property of the TCP table, and the
  narrowing of the section 11.3 race window as following from it. Planning
  found the platform reports it on both tables, when each is requested by
  owning module rather than by owning process identifier. This matters more for
  UDP than for TCP: a UDP key is the local endpoint alone, so it is the weaker
  of the two joins and the one where a reused port is least distinguishable.
  Recorded here and promoted to Appendix D and specification section 29.
- **An injected clock on the attributor.** Section 11.2 states a cadence
  without saying where time comes from. Every cadence and retention rule is
  otherwise untestable at tier 1, which specification section 25.1 requires.
  Recorded here and promoted to specification section 29.
