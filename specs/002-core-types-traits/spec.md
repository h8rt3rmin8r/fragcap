# Feature Specification: Core Types and Traits

**Feature Branch**: `feat/core-types-traits`

**Created**: 2026-08-08

**Status**: Draft

**Slice**: S02 (specification sections 8.4, 8.5; constitution P-2, P-3, P-4,
P-6, P-9)

**Input**: Fill the `fragcap-core` skeleton with the type and trait vocabulary
every later slice is written against.

## Overview

S01 built eight crates that compile and check. Every one of them is empty. This
slice writes the vocabulary into `fragcap-core` that the other seven are
expressed in.

It adds no behavior. Nothing captures a packet, resolves an attribution, parses
a header, or writes a file when this slice lands. What changes is that sixteen
later slices stop having to invent the shape of the thing they are implementing,
because the shape is fixed, documented, and checked.

That makes the audience a contributor writing a later slice, not an operator
running a capture. The measure of success is whether S03 through S18 can be
written against these types without renegotiating them, and whether a type that
would permit a constitution violation is impossible to construct rather than
merely discouraged.

Two properties make this slice worth spending care on rather than transcribing.

**The seams outlive the implementations.** `PacketSource` and `FlowAttributor`
are the seam that makes the whole pipeline testable offline, with no capture
driver, no elevation, and no game running. If the seam is shaped wrong here,
every test in section 25 inherits the mistake, and the cost of fixing it grows
with each slice that has been written against it.

**Some constitution principles are enforceable by type.** P-4 says every
discarded packet is counted in a named counter. A statistics type carrying a
single `dropped: u64` satisfies the letter and defeats the purpose, because it
cannot say which discard path fired. A type with one named field per discard
path makes the principle structural: adding a discard path without a counter
stops compiling. The same logic applies to P-9 and to the protocol asymmetry in
section 8.4.

## Clarifications

### Session 2026-08-08

- Q: Does this slice introduce the workspace's first external dependencies, and
  is that in scope? → A: Yes, and yes. The dependency set is chosen here and
  deliberately kept minimal.
- Q: Must the four behavioral traits be usable as trait objects? → A: Yes. The
  pipeline in section 8.6 owns them across thread boundaries and fans out to
  several sinks, which requires dynamic dispatch.
- Q: Does `orig_len` being separate from the captured data length mean
  truncation is permitted? → A: Truncation of stored bytes is permitted and is
  the operator's choice via snapshot length. Losing the fact that truncation
  happened is not.
- Q: Must the flow key and attribution key be usable as hash map keys?
  → A: Yes. Both are lookup keys by construction, so equality and hashing are
  part of their contract rather than an implementation convenience.
- Q: Does the vocabulary distinguish "attribution was not attempted" from
  "attribution was attempted and did not resolve"? → A: Yes, and it needs no
  new field. The distinction is derivable from the flow key and attribution
  together.
- Q: What resolution does the timestamp carry? → A: One canonical internal
  resolution of nanoseconds since the Unix epoch. Output-format resolution is
  declared at the output boundary, not carried per packet.
- Q: Are the error types opaque or enumerated? → A: Enumerated with named
  variants, and extensible without a breaking change.
- Q: Do backend-reported counters and fragcap's own pipeline counters share
  fields? → A: No. They are separate types and separate fields.

All were resolved under the autopilot decision policy rather than escalated.
Rationale follows; the concrete type-level consequences are carried into
`plan.md` and `research.md`.

**External dependencies.** The architecture of record writes `Bytes` and
`Timestamp` as if they were given. They are not; something has to back them.
The alternatives are to define everything locally over the standard library, or
to take a small number of well-established crates. This slice takes the second
option for the packet payload type and the first for everything else.

The reasoning is that a packet payload is cloned into a bounded buffer, handed
to a sink thread, and fanned out to several sinks. A payload type with cheap
reference-counted clones is the difference between one copy per packet and
several, and that is the hot path of the whole program. A timestamp, by
contrast, is a fixed-point integer count whose required semantics are dictated
by the pcapng format rather than by any crate, and wrapping the standard
library is both smaller and more accurate than adopting a date-time library.

This is also the moment two checks stop being decorative. `cargo xtask msrv`
and the `audit` workflow both pass vacuously today because the dependency graph
is empty, a fact S01 recorded rather than hid. From this slice on, they
constrain something real, and the dependency license allowlist in `deny.toml`
gets its first actual subject.

**Trait objects.** Section 8.6 puts the `PacketSource` on a capture thread, the
`FlowAttributor` and `ProcessWatcher` on a control thread, and fans the buffer
out to a file sink, a stream sink, and a ring buffer. Sinks are therefore a
heterogeneous collection selected at runtime from command line arguments, which
is dynamic dispatch by definition. `Sink::finish` taking `self: Box<Self>` in
the architecture of record is already an admission of this: that signature
exists precisely so a boxed trait object can be consumed. The traits are
therefore constrained to remain dyn-compatible, and that constraint is worth
stating as a requirement because it is easy to break later by adding a generic
method.

**Truncation and P-9.** `RawPacket` carries both the captured bytes and an
original length, and those two can differ. That is not a licence to alter an
observation. The operator chooses a snapshot length; that is scope, which P-9
explicitly permits, and it is visible in their own invocation. What P-9
prohibits is the record failing to say that it happened. Keeping the original
length alongside the possibly shorter payload is what makes truncation
self-describing rather than silent, and it is why the two fields are separate
rather than one field and an assumption.

**Keys are keys.** The flow key indexes the attribution lookup on the capture
thread, and the attribution key is matched against socket table entries. Both
are used as map keys in the slices that follow, so equality and hashing belong
to the contract stated here rather than being added later by whoever needs them
first. Stating it now also prevents a field being added to either type that
cannot participate in a stable hash.

**Attribution state is already expressible.** The architecture of record gives
`CapturedPacket` an optional flow and an optional attribution. Read together,
those two fields already separate the three states that matter: no flow key
means attribution was never attempted, because there was nothing to attempt it
with; a flow key with no attribution means it was attempted and did not resolve;
a flow key with an attribution means it resolved.

The alternative considered was an explicit three-state enum in place of the
optional attribution. It was rejected on two grounds. It would deviate from the
architecture of record for information that is already recoverable, and it would
add a discriminant to a struct that exists once per packet on the hot path. The
requirement this produces is therefore documentation and a test that pins the
mapping, not a new field. P-4's requirement that unattributed packets be
"retained and marked" is met by the absent attribution plus the named counter
that records the cause.

**Timestamp resolution.** The pcapng format records a timestamp as a 64-bit
integer count whose unit is declared per interface. Carrying that declared
resolution on every packet would push format concerns into core, and would mean
arithmetic between two timestamps had to consult both their resolutions.

One canonical internal resolution avoids that. Nanoseconds since the Unix epoch
is finer than any capture backend supplies, so converting inward loses nothing,
and the output layer converts outward once at the boundary where the format's
resolution is declared anyway. This keeps core free of format knowledge, which
P-2 requires, and keeps P-9's prohibition on rounding checkable: there is one
conversion site to inspect rather than one per packet.

**Enumerated errors.** A caller has to act differently on different failures. A
packet source that times out with no traffic is a normal condition and the
capture loop continues; a packet source whose device has disappeared is not, and
the loop must stop and report. An opaque error type forces that decision to be
made by inspecting a message string, which is not a contract.

Named variants also give the statistics counters something to correspond to. P-4
wants one named counter per discard cause, and the causes are exactly the error
variants that can arise on the discard paths, so enumerating one makes the other
reviewable. The error enums are declared extensible so a later slice can add a
variant without breaking every caller, which matters because S09, S15, and S16
each add failure modes that cannot be enumerated now.

**Backend counters stay separate from pipeline counters.** A capture backend
reports its own counts: frames it received, frames it dropped for want of buffer
space, frames the interface dropped before the backend saw them. fragcap's
pipeline separately drops from its bounded ring. These are different facts with
different remedies, and an operator seeing one blended number cannot tell
whether to raise a buffer size, shorten a snapshot length, or accept the loss.

Blending them would also be a P-9 problem rather than only a usability one. The
backend's counts are observations that fragcap is reporting on behalf of another
component; folding fragcap's own accounting into them alters what that component
said. The source statistics type therefore carries the backend's counts, the
capture statistics type carries fragcap's, and neither reaches into the other.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Write a later slice against a fixed vocabulary (Priority: P1)

A contributor picks up S03, S06, or S10 and needs to know what a flow key is,
what a packet looks like before and after attribution, and what contract their
implementation has to satisfy. They read one crate and find out, without
inferring it from the specification or from a sibling slice's code.

**Why this priority**: This is the entire value of the slice. Every other story
here is a property of doing this correctly.

**Independent Test**: Take the trait definitions alone, write a stub
implementation of each in a test, and confirm the pipeline shape in section 8.6
can be expressed against them without adding or changing a method.

**Acceptance Scenarios**:

1. **Given** the core crate, **When** a contributor looks for the type
   representing a packet that has been attributed, **Then** exactly one such
   type exists, its optional fields say what may be absent, and its
   documentation names the slice that populates each.
2. **Given** the core crate, **When** a contributor writes a test-only
   implementation of each behavioral trait, **Then** every trait can be
   implemented without reference to any platform capability.
3. **Given** the core crate, **When** a contributor stores those
   implementations behind pointers as the pipeline requires, **Then** each trait
   is usable as a trait object.

---

### User Story 2 - The protocol asymmetry cannot be papered over (Priority: P1)

A contributor implementing socket table attribution in S10 reaches the point
where a UDP entry has no remote endpoint. The obvious shortcut is to invent one,
which produces confident wrong attributions. The vocabulary makes the shortcut
unavailable.

**Why this priority**: The architecture of record states this MUST NOT be
papered over, and states why: honest coarse attribution beats confident wrong
attribution. A rule that lives only in prose is a rule that gets violated by
someone who did not read that paragraph. Encoding it in the type moves it from
prose to structure.

**Independent Test**: Attempt to construct an attribution key for a UDP flow
that carries a remote endpoint, and confirm the vocabulary offers no way to do
it.

**Acceptance Scenarios**:

1. **Given** a TCP flow key, **When** its attribution key is derived, **Then**
   the result carries both endpoints, matching what the TCP socket table
   supplies.
2. **Given** a UDP flow key, **When** its attribution key is derived, **Then**
   the result carries the local endpoint only, and there is no variant that
   would carry a remote endpoint for UDP.
3. **Given** a UDP socket bound to a wildcard address, **When** attribution is
   attempted for a datagram on a specific interface address, **Then** the
   vocabulary supports matching against the wildcard as well as the specific
   address.

---

### User Story 3 - A discard path needs a counter (Priority: P1)

A contributor in a later slice adds a reason a packet gets dropped. The
statistics type forces them to name it, and the compiler notices if they do not.

**Why this priority**: Constitution P-4 calls an uncounted discard a defect
rather than an oversight. This is the one class of defect the constitution
singles out as corrupting the output's meaning rather than merely being
recoverable, so it deserves structural enforcement rather than review attention.

**Independent Test**: Read the statistics types and confirm each discard
category has its own named field, and that a test can assert on an individual
category rather than only on a total.

**Acceptance Scenarios**:

1. **Given** the statistics types, **When** a reader looks for how many packets
   were dropped, **Then** they find counts broken out per named cause rather
   than one aggregate.
2. **Given** the statistics types, **When** a total is needed, **Then** it is
   derived from the named counters rather than stored separately where the two
   could disagree.
3. **Given** an unattributed packet, **When** it flows through the vocabulary,
   **Then** it is representable as retained-and-marked rather than only as
   dropped.

---

### User Story 4 - Core stays portable, mechanically (Priority: P2)

A contributor adds a dependency to the core crate that only builds on Windows.
The project rejects the change rather than discovering it when a second platform
is attempted.

**Why this priority**: P-2 exists because platform leakage into core is cheap to
introduce, expensive to remove, and invisible until someone tries the second
platform. S01 already built both checks that catch it; this slice is the first
one that gives them something to catch.

**Independent Test**: Build the core crate for a target with no capture backend
and confirm it compiles; add a platform-specific dependency to its manifest and
confirm the dependency direction check rejects it.

**Acceptance Scenarios**:

1. **Given** the core crate with its new dependency set, **When** it is built
   for a target where no capture backend exists, **Then** it compiles.
2. **Given** the core crate, **When** the dependency direction check runs,
   **Then** it confirms core has no platform-specific dependency, no I/O crate,
   and no capture library.
3. **Given** the new dependency set, **When** the license audit runs, **Then**
   every dependency's license is on the allowlist.

### Edge Cases

- What happens when a packet is captured but no flow key can be derived,
  because the headers are truncated or the protocol is neither TCP nor UDP? The
  vocabulary must represent a packet with no flow rather than forcing a
  fabricated one.
- What happens when a flow key is derived but no attribution resolves? The
  packet is retained and marked, per P-4. A flow key present with no attribution
  is exactly that state, and it is distinguishable from a packet that had no
  flow key to attempt attribution with.
- What happens when the same process name appears on every packet of a long
  flow? The representation must not allocate per packet.
- What happens when a UDP socket is bound to a wildcard address and a datagram
  arrives on a specific interface address? Both must be matchable.
- What happens when captured bytes are shorter than the original length? The
  record must still state the original length, so truncation is visible.
- What happens when a contributor adds a method to a behavioral trait that
  makes it generic? Dyn compatibility breaks and the pipeline stops being
  expressible. The slice must have a test that fails in that case.

## Requirements *(mandatory)*

### Functional Requirements

Types, from specification section 8.4.

- **FR-001**: `fragcap-core` MUST expose a protocol type distinguishing exactly
  TCP and UDP, with no third variant, because the socket table join is defined
  for those two only.
- **FR-002**: `fragcap-core` MUST expose a flow key carrying the protocol and
  two endpoints, where the local endpoint is by definition the endpoint on the
  capturing host. Its documentation MUST state that this normalization is what
  makes one flow one key.
- **FR-003**: The flow key MUST expose a derivation of the subset matchable
  against a socket table entry, returning both endpoints for TCP and the local
  endpoint alone for UDP.
- **FR-004**: The attribution key type MUST have no representation carrying a
  remote endpoint for UDP. Preventing the fabrication described in section 8.4
  MUST be a property of the type rather than a documented warning.
- **FR-005**: The vocabulary MUST support matching a UDP local endpoint against
  a wildcard bind address as well as a specific interface address.
- **FR-006**: `fragcap-core` MUST expose a direction type with inbound and
  outbound variants, carried per packet rather than as a property of the flow.
- **FR-007**: `fragcap-core` MUST expose an attribution carrying a process
  identifier, a process name, an optional role, and an optional stage
  identifier. Process name and role MUST use a shared reference-counted
  representation so that repeating them across every packet of a flow costs no
  per-packet allocation.
- **FR-008**: `fragcap-core` MUST expose a raw packet carrying an observation
  timestamp, the captured bytes, and the original on-wire length as a field
  distinct from the captured length.
- **FR-009**: `fragcap-core` MUST expose a captured packet carrying the raw
  packet's fields plus an optional flow key, an optional direction, and an
  optional attribution, so that a packet with none of them resolved is
  representable.
- **FR-010**: The captured packet MUST distinguish three attribution states
  without adding a field: no flow key means never attempted, a flow key with no
  attribution means attempted and unresolved, and a flow key with an attribution
  means resolved. This mapping MUST be documented on the type and pinned by a
  test.
- **FR-011**: The timestamp type MUST carry one canonical resolution of
  nanoseconds since the Unix epoch, MUST NOT carry a per-packet output
  resolution, and MUST NOT round, normalize, or reorder an observation.
- **FR-012**: `fragcap-core` MUST expose the supporting types the above require:
  a stage identifier, a link type, and an endpoint.
- **FR-013**: The flow key and the attribution key MUST support equality and
  hashing so both can be used as map keys, and every field either type carries
  MUST participate in a stable hash.

Traits, from specification section 8.5.

- **FR-014**: `fragcap-core` MUST declare a packet source trait with methods to
  take the next packet within a timeout, set a filter program, report
  statistics, and report its link type.
- **FR-015**: `fragcap-core` MUST declare a flow attributor trait, bounded to be
  sendable across threads, with methods to resolve a flow key, refresh its
  view, and list active endpoints.
- **FR-016**: `fragcap-core` MUST declare a process watcher trait, bounded to be
  sendable across threads, with methods to subscribe to process events and to
  take a snapshot of process records.
- **FR-017**: `fragcap-core` MUST declare a sink trait, bounded to be sendable
  across threads, with methods to write a captured packet, flush, and finish by
  consuming an owned pointer to itself together with capture statistics.
- **FR-018**: `fragcap-core` MUST declare a dissector trait with no
  implementations, so that the seam's shape is fixed before any protocol work
  begins.
- **FR-019**: The packet source, flow attributor, process watcher, and sink
  traits MUST remain usable as trait objects, and the slice MUST include a test
  that fails if any of them stops being.
- **FR-020**: No trait may combine packet acquisition with attribution. The
  dependency direction check MUST continue to pass, and the two traits MUST NOT
  reference each other.

Errors and statistics.

- **FR-021**: `fragcap-core` MUST expose one error type per behavioral trait
  that produces failures: a source error, an attribution error, and a sink
  error. Each MUST integrate with the standard error trait so a caller can
  report a cause chain.
- **FR-022**: Each error type MUST enumerate named variants rather than being
  opaque, MUST let a caller distinguish a recoverable condition such as a
  timeout from a terminal one, and MUST be extensible with new variants without
  breaking existing callers.
- **FR-023**: Statistics types MUST carry one named counter per discard cause.
  A single aggregate count of discards MUST NOT be the only representation.
- **FR-024**: Backend-reported counts and fragcap's own pipeline counts MUST
  live in separate types with separate fields, and neither MUST be folded into
  the other.
- **FR-025**: Where a total is exposed, it MUST be derived from the named
  counters rather than stored independently, so the two cannot disagree.
- **FR-026**: The vocabulary MUST represent an unattributed packet as retained
  and marked. It MUST NOT require dropping a packet that failed attribution.

Constraints and hygiene.

- **FR-027**: `fragcap-core` MUST acquire no platform-specific dependency, no
  I/O crate, and no capture library, and MUST continue to build for a target
  with no capture backend.
- **FR-028**: Every external dependency added by this slice MUST carry a license
  on the allowlist in specification section 20.4, and the dependency audit MUST
  pass against the real graph.
- **FR-029**: No type or trait in this slice may offer an operation whose
  purpose is to alter, mask, truncate, reorder, or withhold an observation in
  the capture path, per P-9.
- **FR-030**: Every term this slice introduces MUST have a glossary entry in
  `docs/glossary.md` in this same change, per P-6.
- **FR-031**: Every public item MUST carry documentation stating what it
  represents and, where the item is a seam that a later slice fills, naming that
  slice.

### Key Entities

- **Protocol**: Which of the two transport protocols a flow uses. Determines
  how the socket table join is performed, and nothing else.
- **Flow key**: The identity of a conversation, normalized so the capturing
  host's endpoint is always in the same position. One conversation, one key.
- **Attribution key**: The part of a flow key that a socket table can actually
  answer. Deliberately narrower for UDP than for TCP, because the platform
  supplies less.
- **Direction**: Which way an individual packet travelled. A property of the
  packet, not of the flow, because the flow key already normalized position.
- **Attribution**: The process a flow belongs to, plus its role and launch stage
  when known. Names are shared rather than copied per packet.
- **Raw packet**: An observation as acquired: when, what bytes, and how long the
  frame was on the wire before any snapshot limit applied.
- **Captured packet**: A raw packet plus whatever the pipeline resolved about
  it. Every added field is optional, because resolution can fail and a failure
  must not discard the observation.
- **Source statistics**: What the capture backend itself reports, carried
  unaltered because it is that component's observation rather than fragcap's
  accounting.
- **Capture statistics**: What fragcap's own pipeline counted, including one
  named counter per reason fragcap discarded a packet. Kept separate from source
  statistics so an operator can tell where loss happened.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A contributor can express the full pipeline shape of
  specification section 8.6 against these traits without adding or altering a
  method, demonstrated by a test that wires stub implementations together.
- **SC-002**: Every type and trait named in specification sections 8.4 and 8.5
  exists in the core crate, verified item by item against those sections, with
  zero omissions.
- **SC-003**: Attempting to represent a UDP attribution key that carries a
  remote endpoint fails, and a test asserts the TCP and UDP derivations
  independently.
- **SC-004**: Every discard cause the vocabulary admits has its own named
  counter, and a test asserts on an individual cause rather than a total.
- **SC-010**: The three attribution states are distinguishable from a captured
  packet alone, demonstrated by a test that constructs each and asserts which
  state it reads as.
- **SC-011**: Backend-reported counts and pipeline counts cannot be confused:
  they live in separate types, and no field of either is a sum across both.
- **SC-005**: The core crate builds for a target with no capture backend, and
  the dependency direction check confirms it carries no platform dependency.
- **SC-006**: The dependency audit passes against a non-empty dependency graph,
  making it a meaningful check for the first time.
- **SC-007**: The minimum supported toolchain check constrains a real
  dependency set, and the declared minimum is either confirmed or revised with
  the reason recorded.
- **SC-008**: Every term introduced has a glossary entry, verified by reading
  the change's new public items against `docs/glossary.md`.
- **SC-009**: The full local gate set passes: format, lint, tests, repository
  conventions, dependency direction, and per-crate licensing.

## Assumptions

- The type and trait signatures in specification sections 8.4 and 8.5 are the
  architecture of record and are transcribed rather than redesigned. Where a
  signature is incomplete, because it names a type the specification does not
  define, the gap is filled here and recorded as a deviation for promotion to
  specification section 29.
- Concrete backing choices, specifically what represents a timestamp and a byte
  payload, belong to `plan.md` and `research.md` rather than to this spec. This
  document constrains their required properties.
- No behavior is implemented. Stub implementations exist only in tests, and only
  to prove the seams are expressible.
- Test-only implementations of the traits are the deliverable's own proof, not a
  replay source or a scripted attributor. Those are S04's work.
- The declared minimum supported toolchain may need to rise once real
  dependencies are present. Discovering that is an expected outcome of this
  slice, not a failure of it.
- The reconnaissance gate is closed, so nothing in this slice waits on
  Q-1 through Q-6.

## Out of Scope

- Any packet acquisition, live or replayed. S04 and S09.
- Any header parsing or flow key derivation from bytes. S03.
- Any attribution logic or socket table access. S10.
- Any process watching or process tree. S11.
- Any output writing, pcapng or otherwise. S06 and S07.
- Any pipeline, buffering, or drop accounting behavior. S08. This slice defines
  the counters; it does not increment them.
- Any dissector implementation. The trait is declared empty on purpose.
- Any command line surface. S14.

## Done When

- Every requirement above is satisfied and traceable to a test or a check.
- The full local gate set passes in the foreground, watched to completion.
- The glossary carries an entry for every term introduced.
- Deviations from the architecture of record are recorded in the slice for
  promotion to specification section 29.
- A changelog fragment exists describing the change.
