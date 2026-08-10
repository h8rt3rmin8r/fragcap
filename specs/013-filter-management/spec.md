# Feature Specification: Filter Management

**Feature Branch**: `feat/filter-management`

**Created**: 2026-08-10

**Status**: Draft

**Slice**: S13 (specification sections 12.2 and 12.3; constitution P-2, P-3,
P-4, P-6, and P-9)

**Input**: Implement specification sections 12.2 (kernel filtering strategy: the
three-phase filter lifecycle) and 12.3 (filter correctness and filter-gap
accounting). Phase one, bootstrap, was installed by S09. This slice adds phase
two, narrowing (compile a capture filter admitting only the endpoints belonging
to profiled processes, and install it on each live handle) and phase three,
maintenance (recompile and reinstall as the endpoint set changes, debounced by
two seconds and rate limited to one reinstallation per five seconds per handle).
The narrowed filter is a performance optimization and never the sole determinant
of what is captured: userspace attribution runs on every packet regardless.
Packets a stale narrowed filter briefly excludes are accounted as filter gaps,
counted and surfaced. Filter compilation, the maintenance policy, and the
control-thread orchestration are pure over core types and testable offline
against a source double that records the programs it was asked to install; the
socket-table endpoint source (S10), the bootstrap filter and its application on a
live handle (S09), and the profile schema (S05) are consumed, not rebuilt. No
operator-facing filter flag or profile key is introduced; that is S14.

## Overview

Twelve slices have built a capture tool that selects interfaces (S09), opens a
handle per interface with a bootstrap filter admitting `ip or ip6` (S09),
attributes flows to processes from the live socket table (S10), and drives a
capture session (S12) through a buffered pipeline (S08). Every packet crossing
the kernel boundary today is read into userspace and, in bootstrap, most are
discarded there because they belong to no profiled process. Reconnaissance found
that on an ordinary machine a single unrelated background process accounted for
up to ninety-four percent of captured bytes, so the interval before the filter
narrows is a cost to minimize, not a theoretical concern.

S13 adds phases two and three of the filter lifecycle. It derives the set of
endpoints belonging to profiled processes from the live attribution map, compiles
a capture filter admitting only those endpoints, installs it on each live handle,
and keeps it current as connections open and close, all without ever letting the
kernel filter become the authority on what is captured.

Two properties shape the slice.

**The kernel filter is an optimization, never the authority.** Userspace
attribution runs on every packet regardless of the installed filter (section
12.3). A stale filter that over-admits is cleaned up correctly in userspace; a
stale filter that under-admits briefly is counted as a filter gap and surfaced.
Correctness never depends on filter freshness. This is what makes the whole slice
testable offline: the narrowing decision is a pure function over an endpoint set,
the maintenance schedule is arithmetic over instants, and the gap accounting is a
set difference, none of which needs a live driver.

**Compilation and policy are pure over core types.** Turning a set of endpoints
into a filter program, and deciding when to recompile and reinstall under a
debounce and a per-handle rate limit, open nothing and touch no platform
interface. So the whole of section 12.2's strategy is tested on any machine. Only
application, compiling the program text onto an npcap handle, is platform bound,
and S09 already built it. A capture filter expression is just text to
`fragcap-core`; only `fragcap-capture` knows the text is npcap's grammar.

## Clarifications

### Session 2026-08-10

Resolved under autopilot from the architecture of record (sections 8.3, 8.6,
11.6, 12.1 through 12.3) and the constitution; no operator escalation was
required.

- Q (D-a): What feeds the narrowing filter, the process tree's flow set that the
  section 8.6 diagram draws, or the attribution map that the section 12.2 prose
  names? -> A: The live attribution map, read through
  `FlowAttributor::active_endpoints()`, which the socket-table attributor (S10)
  publishes lock-free as the seam reserved for this slice. Section 12.2 is
  explicit that "the attribution map is the only reliable source, which is why
  phase two depends on it rather than on traffic inspection." The diagram's "flow
  set" and the prose's "attribution map" denote the same endpoint set; the
  divergence in rendering is recorded as a deviation candidate for specification
  section 29. **Known limitation (review of pull request 17):**
  `active_endpoints()` currently reports every socket-table endpoint, not only
  those owned by profiled processes, and nothing in the pipeline drives the
  refresh that keeps the live snapshot current (`FlowAttributor::refresh` is `&mut
  self` and cannot be called through the shared `Arc`). Restricting the set to
  profiled endpoints (a join with the S11/S12 process-tree stage bindings) and
  driving the refresh (which needs a `refresh(&self)` signature) are the
  session-to-pipeline integration required before the live backend narrows
  correctly; both are recorded as section 29 open items. The tier-1 machinery is
  verified against a controlled endpoint set and does not depend on them.
- Q (D-b): How is a filter gap counted, given fragcap never sees a packet the
  kernel filter excluded? -> A: The `filter_gaps` counter counts gap occurrences,
  not packets. A gap occurrence is an endpoint that is active in the attribution
  map while a narrowed filter that does not admit it is installed on a handle, for
  the interval from the endpoint appearing in the map until the reinstall that
  admits it. fragcap counts these occurrences (a set difference between the wanted
  endpoint set and the installed program, per handle) the first poll an endpoint is
  excluded, once per episode, independent of whether a reinstall ever follows, so an
  endpoint that closes before settling or is still excluded when capture ends is
  counted. It does not fabricate a count of the packets the kernel excluded,
  because those packets are never delivered to fragcap and inventing a number for
  them would violate P-9. This is the honest reading of section 12.3's "counted as
  a filter gap and reported in statistics"; the prose says "packets," so the unit
  choice is recorded as a deviation candidate for section 29. Bootstrap admits
  everything, so the first narrowing per handle excludes only unwanted traffic and
  records no gap; gaps arise only in phase three, when an endpoint appears while a
  strictly narrowed filter is installed.
- Q (D-c): How does the control thread install a filter on a source whose handle
  is owned by, and only safe to touch from, its own capture thread (a `pcap`
  handle is not `Sync`)? -> A: A per-source `std::sync::mpsc` channel from the
  control thread to each capture thread. The control thread sends the current
  desired `FilterProgram`; each capture thread, between reads, drains its receiver
  to the latest value and calls `set_filter` on its own handle. Only the owning
  thread touches the handle, so `PacketSource` stays `!Sync` and the trait gains
  no bound (P-3). The considered alternative, an `arc-swap` cell per source
  (mirroring S10's lock-free attribution snapshot), was rejected because it would
  require widening `fragcap-core`'s dependency allowlist, which is deliberately
  just `bytes` and is a P-2 guard; the filter slot is checked between reads, off
  the per-packet path, so section 11.6's lock-free mandate (which exists for the
  per-packet attribution read) does not extend to it, and a std channel with a
  non-blocking drain is the smaller commitment. A `Mutex` was rejected for
  introducing a lock where a channel needs none.
- Q (D-d): What grammar does the compiled filter use, and how are endpoints
  composed? -> A: A libpcap filter expression (npcap's grammar; specification
  appendix E), built as the union over the endpoint set: each endpoint contributes
  a clause constraining protocol, host address, and port, ORed together, spanning
  IPv4 and IPv6 by address family. A wildcard bind (`0.0.0.0` or `::`, the address
  a UDP game socket is commonly reported under) drops the host constraint and
  admits by protocol and port alone, because a `host 0.0.0.0` clause matches no
  real packet and would silently exclude the socket's whole traffic. Over-admission
  of traffic that shares a target's ports, or a wildcard's port, is expected and
  left to userspace attribution (section 12.2), not tightened further in the
  kernel. An endpoint set that is non-empty compiles to a
  strictly narrowed program; once narrowing has begun, an endpoint set that
  transiently empties keeps the last narrowed program rather than reverting to the
  bootstrap admit-all, because reverting would re-flood the boundary for endpoints
  that are gone (the S10 retention window keeps closing endpoints present for its
  grace period, so a truly empty set mid-session is rare). Generating the text is
  string building over core types; a rejected program maps to the existing
  `SourceError::FilterRejected` and is surfaced, never swallowed.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Narrow the kernel filter to profiled endpoints (Priority: P1)

Once the attribution map holds endpoints belonging to profiled processes, fragcap
compiles a capture filter admitting only those endpoints and installs it on each
live handle, so the volume crossing the kernel boundary drops from all IP traffic
to the target's traffic plus whatever shares its ports. The endpoint set comes
from the attribution map, never from observed name resolution, because gameplay
endpoints are reached by address with no preceding name lookup in both focal
titles.

**Why this priority**: Bootstrap-phase volume is dominated by unrelated
background traffic (up to ninety-four percent in reconnaissance). Narrowing is the
capability that makes a capture on a working machine tractable, and every later
concern (maintenance, gap accounting) is a refinement of it. It is the core of the
slice.

**Independent Test**: Feed a scripted attributor a set of active endpoints, run
the filter manager, and assert the compiled program admits exactly those
endpoints across IPv4 and IPv6 and is installed on a source double that records
it. No capture driver, elevation, or game.

**Acceptance Scenarios**:

1. **Given** a bootstrap filter (`ip or ip6`) installed on each handle, **When**
   the attribution map first contains one or more profiled endpoints, **Then**
   fragcap compiles a filter admitting only those endpoints and installs it on
   each live handle.
2. **Given** a profiled endpoint set spanning both an IPv4 and an IPv6 address,
   **When** the filter is compiled, **Then** the program admits both address
   families.
3. **Given** an endpoint whose port is also used by a non-target process, **When**
   the narrowed filter is installed, **Then** the shared-port traffic is admitted
   at the kernel and discarded by userspace attribution, not tightened away in the
   kernel.
4. **Given** an empty profiled endpoint set, **When** narrowing has not yet begun,
   **Then** the bootstrap filter remains installed and no narrowed program is
   compiled.

---

### User Story 2 - Keep the filter current without thrashing capture (Priority: P1)

As connections open and close the endpoint set changes, and fragcap recompiles and
reinstalls to track it. Because installing a filter briefly interrupts capture on
that handle and endpoint sets churn during connection establishment, recompilation
is debounced by two seconds and reinstallation is rate limited to one per five
seconds per handle. Correctness does not depend on the filter being fresh: a
briefly stale filter is admitted, and the traffic it wrongly excludes is accounted.

**Why this priority**: Without maintenance the narrowed filter goes stale the
moment a new connection opens, and either it is never updated (traffic silently
lost) or it is updated on every change (capture thrashed by constant
reinstallation). The debounce and rate limit are what make narrowing survivable on
live traffic.

**Independent Test**: Drive the maintenance policy with a scripted clock and a
changing endpoint set; assert that reinstalls respect the two-second debounce and
the one-per-five-seconds-per-handle rate limit, and that rapid churn coalesces
into a single reinstall.

**Acceptance Scenarios**:

1. **Given** a narrowed filter installed on a handle, **When** the endpoint set
   changes and less than two seconds have elapsed since the change settled,
   **Then** no reinstall occurs yet (debounce).
2. **Given** a reinstall just occurred on a handle, **When** the endpoint set
   changes again within five seconds, **Then** the reinstall for that handle is
   deferred until the rate-limit interval elapses.
3. **Given** several endpoint-set changes within the debounce window, **When** the
   window elapses, **Then** they coalesce into a single recompilation and
   reinstall.
4. **Given** an endpoint that closes, **When** the S10 retention window still
   holds it, **Then** it remains in the endpoint set until retention lapses and is
   not removed from the filter mid-teardown.

---

### User Story 3 - Never let the kernel filter decide correctness, and account every gap (Priority: P1)

The narrowed filter is only ever an optimization. Userspace attribution runs on
every packet regardless of the installed filter, and the capture-scope decision is
made there. When phase three leaves a filter briefly stale, it may exclude packets
fragcap wanted; each such occurrence is counted as a filter gap and surfaced in
statistics, so a capture that was briefly short says so rather than reading as
clean.

**Why this priority**: A capture tool that let a stale kernel filter silently
determine scope would lose traffic without saying so, which is the exact failure
P-4 forbids. Making the filter advisory and counting its gaps is what keeps the
capture's meaning trustworthy.

**Independent Test**: Drive the control thread with an endpoint set that gains an
endpoint while a strictly narrowed filter is installed; assert the `filter_gaps`
counter records the occurrence, that the counter is distinct from the kernel,
buffer, and sink drop counters, and that the pipeline conservation invariant still
holds.

**Acceptance Scenarios**:

1. **Given** any installed filter, narrowed or bootstrap, **When** a packet is
   delivered, **Then** userspace attribution runs on it and the capture-scope
   decision is made in userspace, not by the filter.
2. **Given** a strictly narrowed filter installed on a handle, **When** a new
   profiled endpoint appears before the next permitted reinstall, **Then** a filter
   gap is recorded for that occurrence and surfaced in statistics.
3. **Given** the first narrowing after bootstrap, **When** it installs, **Then** it
   records no gap, because bootstrap admitted everything and the narrowing excludes
   only unwanted traffic.
4. **Given** a completed capture, **When** statistics are reported, **Then**
   `filter_gaps` is reported separately from `kernel_dropped`, `buffer_dropped`,
   and `sink_dropped`, and the conservation invariant (observed equals retained
   plus every named discard, per sink) is unaffected by it.

---

### Edge Cases

- Bootstrap to first narrowing records no filter gap: bootstrap admits `ip or
  ip6`, so the first narrowed program excludes only traffic fragcap did not want.
- A handle whose reinstall is deferred by the rate limit continues to capture on
  its currently installed program; the deferral is not a loss and advances no drop
  counter, only (when it excludes a newly wanted endpoint) the filter-gap counter.
- An endpoint set that transiently empties after narrowing has begun keeps the last
  narrowed program rather than reverting to bootstrap admit-all.
- A `set_filter` rejection is handled by phase: a rejection at bootstrap (in
  `LiveSource::open`, before capture starts) retires the interface, which is
  existing S09 behavior. A rejection during phase-three maintenance is non-fatal:
  the capture thread keeps the prior installed program and continues, because
  correctness never depends on filter freshness and retiring the interface would
  lose all its subsequent traffic to spare a failed optimization. It advances no
  drop counter (nothing was observed and then discarded). Because fragcap generates
  the program from a fixed grammar this path is defensive and cannot occur in
  normal operation; a dedicated operator-facing diagnostic for it is S14's
  (`doctor` and the command-line surface), not this slice's.
- A capture with no live source (a replay run) installs no kernel filter; the
  filter manager is inert because the source double either records or ignores the
  program, and correctness is unaffected since attribution runs regardless.
- Per-handle install state differs across handles under the rate limit: gap
  accounting is per handle and aggregates into the capture-wide `filter_gaps`.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST derive the narrowing endpoint set from the
  attribution map via `FlowAttributor::active_endpoints()`, never from observed
  name resolution or traffic inspection. (Restricting that set to endpoints owned
  by profiled processes on the live backend, and driving the periodic refresh that
  keeps it current, are the session-to-pipeline integration recorded as section 29
  open items; see the Clarifications known-limitation note. This slice's machinery
  and its tier-1 verification narrow whatever set the attributor reports.)
- **FR-002**: The system MUST compile an endpoint set into a capture filter
  program admitting exactly the union of those endpoints, spanning IPv4 and IPv6,
  as a pure function over core types.
- **FR-003**: The system MUST install the compiled program on each live handle;
  the bootstrap filter (`ip or ip6`) installed by S09 remains until the first
  narrowing.
- **FR-004**: The system MUST recompile and reinstall as the endpoint set changes,
  debounced by two seconds and rate limited to one reinstallation per five seconds
  per handle.
- **FR-005**: The narrowed filter MUST NOT be the sole determinant of what is
  captured; userspace attribution MUST run on every packet regardless of the
  installed filter, and the capture-scope decision MUST be made there.
- **FR-006**: A packet a stale narrowed filter briefly excludes MUST be accounted
  as a filter gap, counted in the named `filter_gaps` counter and surfaced in
  statistics (P-4). The counter counts gap occurrences (a wanted endpoint not yet
  admitted by the installed program), not fabricated kernel-excluded packet counts
  (P-9).
- **FR-007**: Over-admission of traffic sharing a target's ports MUST be accepted
  and resolved by userspace attribution, not tightened further in the kernel.
- **FR-008**: Filter compilation and the maintenance policy MUST be pure over core
  types, opening nothing, and MUST be testable at tier 1 against a source double
  that records installed programs, with no capture driver, no elevation, and no
  game.
- **FR-009**: The control thread MUST install filters without merging
  `PacketSource` and `FlowAttributor` and without adding a `Sync` bound to
  `PacketSource`; each source's handle MUST be touched only by its owning thread
  (P-3).
- **FR-010**: Filter management MUST NOT introduce a platform-specific dependency
  into `fragcap-core`; only `fragcap-capture` may treat the program text as
  npcap/libpcap syntax (P-2).
- **FR-011**: No operator-facing filter flag or profile key MUST be introduced;
  the narrowed filter derives solely from the attribution map. Any operator
  override is S14's to add.
- **FR-012**: Every term this slice introduces MUST have a glossary entry in the
  same change (P-6), including `Filter gap` (resolving the existing dangling
  reference from `Bootstrap filter`), `Filter manager`, `Filter program`, and the
  narrowing and maintenance phases.
- **FR-013**: The pipeline conservation invariant MUST continue to hold;
  `filter_gaps` is distinct from the kernel, buffer, and sink drop counters and is
  not a discard of a packet fragcap observed.

### Key Entities

- **Filter program**: the compiled capture filter handed to a `PacketSource`
  (`FilterProgram`, grown from the S09 stub).
- **Endpoint set**: the addresses, ports, and protocols admitted by a narrowed
  filter, derived from the attribution map.
- **Filter manager**: the control-thread component that reads the endpoint set,
  compiles a program, and schedules per-handle reinstalls under the debounce and
  rate limit.
- **Filter gap**: an endpoint active in the attribution map while a narrowed
  filter that does not admit it is installed on a handle.
- **Per-source filter channel**: the `std::sync::mpsc` channel through which the
  control thread hands the current program to a capture thread.
- **Narrowing (phase two) and maintenance (phase three)**: the two filter
  lifecycle phases this slice adds to the S09 bootstrap.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An endpoint set compiles to a filter program that admits exactly
  those endpoints across IPv4 and IPv6, verified by a test per composition case.
- **SC-002**: The maintenance policy honors the two-second debounce and the
  one-per-five-seconds-per-handle rate limit, verified over a scripted clock,
  including that rapid churn coalesces into a single reinstall.
- **SC-003**: The control thread narrows from bootstrap on the first endpoints and
  reinstalls on change, verified end to end at tier 1 against a source double that
  records the sequence of installed programs.
- **SC-004**: Filter gaps are counted and surfaced, distinct from the three drop
  counters, and the pipeline conservation invariant is unaffected, verified by a
  gap test driven through the pipeline.
- **SC-005**: The whole slice is exercised at tier 1 with no capture driver, no
  elevation, and no game, and `fragcap-core` still builds for a target with no
  capture backend (`cargo xtask neutral`).
- **SC-006**: `cargo xtask ci` passes (format, clippy, tests, conventions lint,
  dependency direction, and license), and `cargo xtask neutral` and `msrv` exit 0.

## Assumptions

- `FlowAttributor::active_endpoints()` (S10) is the endpoint source and is already
  published lock-free; the control thread reads it without blocking captures.
- The bootstrap filter and its application on a live handle
  (`PacketSource::set_filter`, `LiveSource::install_filter`) are S09 and are
  consumed, not rebuilt.
- Per-source publication of the current program uses `std::sync::mpsc`, so no new
  dependency is introduced and `fragcap-core`'s dependency allowlist (currently
  just `bytes`) is untouched (P-2).
- The replay and scripted sources expose `set_filter` observably (recording the
  programs they are asked to install) for tier-1 tests; installing a program on a
  live npcap handle is tier 2 and not required to complete this slice.
- The profile schema (S05) has no filter keys and none are added here; any
  operator-facing filter option is S14's.
- The `filter_gaps` counter already exists on the statistics record (reserved by
  an earlier slice) and is populated here; the three drop counters keep their
  existing meanings.
