# Feature Specification: Replay Source and Fixture Corpus

**Feature Branch**: `feat/replay-source-fixtures`

**Created**: 2026-08-08

**Status**: Draft

**Slice**: S04 (specification sections 25.1, 25.3; constitution P-1, P-3, P-4,
P-6, P-9)

**Input**: Build the tier 1 test substrate: a replay packet source, a scripted
attributor, and the committed fixture corpus that make the pipeline a
deterministic function from fixture input to output.

## Overview

Specification section 25.1 makes a claim the project has been spending
architectural effort on since S01: that the entire pipeline runs with no
capture driver, no elevated privilege, and no game. S02 built the seam that
makes the claim possible and S03 built the first thing behind it. Neither
tested the claim, because there was nothing to feed.

This slice is what makes it true. It delivers three things that only mean
something together: a source that reads packets out of a recorded file, an
attributor that returns predetermined answers, and a corpus of files worth
reading. With them, every slice from S06 onward is testable the day it is
written rather than the day a Windows runner and a capture driver are
available.

The audience is a contributor writing S06, S07, S08, S13, or S16. What they
need is not a clever replay source. It is a substrate they can trust without
reading its source: feed it the same fixture twice and get the same packets,
and if it ever skips something, hear about it.

Two properties carry that trust, and both are worth more than the reading code.

**Determinism is the product.** A test whose input varies between runs is a
test whose failures cannot be reproduced, and a golden-output comparison
against a nondeterministic source is worse than no test at all. The replay
source must be a pure function from file bytes to a packet sequence, on every
platform and every run.

**A fixture must be legible.** A committed binary blob that nobody can read is
a test input nobody can review, and section 25.3 requires these be reviewed
before they land. The corpus is therefore produced by a generator whose source
is the readable record of what each fixture contains, and a check mode proves
the committed bytes still match the generator that claims to describe them.

There is also a rule here that outranks both, and it is the reason the corpus
is synthetic. Section 25.3 forbids any fixture containing traffic from a real
game session, because such captures carry account identifiers, session tokens,
and addresses. Every fixture in this slice is generated from constants written
into the generator, and the addresses used are the documentation ranges
reserved for exactly this purpose.

## Clarifications

### Session 2026-08-08

- Q: Which capture file format do the fixtures use? → A: Classic pcap, which is
  what section 25.3 names. The pcapng that fragcap writes is S06's concern, and
  reading it is nobody's yet.
- Q: Where does the replay source live? → A: `fragcap-capture`, which
  specification section 8.2 defines as the home of packet source backends,
  live capture and replay alike.
- Q: Is the scripted attributor in this slice or a later one? → A: This one.
  Section 25.3 requires every fixture be paired with an attribution script, so
  the format must exist here regardless, and a format with no consumer is a
  format nothing has verified. It lives in `fragcap-attr` per section 8.2.
- Q: What format is an attribution script? → A: A minimal line-oriented text
  format parsed by hand, adding no dependency.
- Q: How is a committed binary fixture kept honest? → A: A generator produces
  the corpus deterministically, and a check mode regenerates and compares, so a
  hand-edited or drifted fixture fails rather than passing quietly.
- Q: What does the replay source do when asked to install a filter? → A:
  Records it and applies nothing, documented as such. It does not fail, because
  a pipeline that sets a filter unconditionally must still run over fixtures.
- Q: Where do the reader's own skip counters live? → A: A replay statistics
  type in `fragcap-capture`, not new fields on the shared source statistics
  type.
- Q: Does `burst.pcap` contain more packets than the pipeline's buffer holds?
  → A: No. It carries a sustained burst, and the test that needs overflow
  configures a smaller buffer.
- Q: How does a script identify a flow, given that a UDP flow has no fixed
  remote endpoint? → A: By the attribution key the flow key already derives:
  protocol and local endpoint for UDP, both endpoints for TCP. The script
  reuses that asymmetry rather than inventing a parallel one.
- Q: How does the attributor know what time it is, given that the resolve
  method takes no time parameter? → A: The caller tells it, through a method on
  the scripted attributor rather than on the seam. The seam is not changed.
- Q: What time base do script windows use? → A: Absolute nanoseconds since the
  Unix epoch, matching the packet timestamps the fixtures carry, with comments
  permitted so the generator can annotate them readably.
- Q: How small is small? → A: 64 KiB per fixture and 256 KiB for the corpus
  including its scripts, asserted rather than judged.
- Q: Where does the generator live, and how is a fixture regenerated? → A: In
  the corpus test target of `fragcap-capture`, regenerated by setting an
  environment variable when running that test. Not in `xtask`, and not in any
  crate that ships.

All were resolved under the autopilot decision policy rather than escalated.
Six have consequences outside this slice and are set out here.

**The attribution script format.** The alternatives were TOML, which the
profile schema in S05 will need anyway, and a minimal format parsed by hand.
TOML is the better format and the worse choice here, because adopting it means
adopting a parser and its proc-macro dependencies now, on behalf of a slice
that has not yet made that decision on its own merits. S05 owns the profile
schema and should choose its parser against the profile's requirements, not
inherit one chosen for a test fixture.

The counter-argument is that two text formats in one repository is a smell, and
it would be, if this were a user-facing format. It is not: an attribution script
is a test input that ships beside the fixture it describes, is written only by
the generator, and is read only by the scripted attributor. It is deliberately
trivial, and the moment it wants nesting or types it should become TOML rather
than growing.

**`burst.pcap` and the buffer.** Section 25.3 says this fixture exercises a
"sustained rate exceeding buffer capacity". The buffer in section 12.4 holds
65,536 packets, so a fixture that genuinely exceeds it holds more than that and
runs to several megabytes, which contradicts the same section's requirement
that each fixture be small.

The property under test is backpressure, and backpressure is a relationship
between a rate and a capacity, not a property of a file. A fixture carrying a
sustained burst plus a test that configures a small buffer exercises the same
code path, deterministically, in kilobytes. The fixture therefore delivers the
sustained rate and S08 supplies the capacity. Recorded for promotion to
specification section 29, because it narrows what section 25.3 says the fixture
does.

**Filters on a replay source.** A replay source has no kernel to install a
filter into. Failing the call would break any pipeline that sets a filter
before reading, and silently accepting it would let a test believe filtering
happened when it did not. The source therefore records the program, applies
nothing, and says so in its documentation and in this spec. Software filtering
over a replay source, if it is ever wanted, is S13's to decide.

**The seam has no clock, and it must not grow one.** The flow attributor's
resolve method takes a flow key and nothing else. A real attributor needs no
time parameter, because it answers from a socket table that is already current;
"now" is implicit in the data it reads. A scripted attributor has no such
source of now, and port reuse is exactly the case where the answer depends on
it.

The tempting fix is to add a timestamp parameter to the seam. That is refused.
S02 fixed these five traits as the part of the surface intended to survive to
1.0.0 unchanged, and widening one of them to accommodate a test double would
be paying an architectural cost for a testing convenience. It would also make
every real implementation take a parameter it does not want.

Instead the scripted attributor carries a caller-set clock as its own method,
outside the seam. The caller knows the packet's timestamp, sets it, then
resolves. A script with no time windows answers the same at any time, so a
caller that never sets a clock still works. This keeps the asymmetry where it
belongs: it is a property of the double, not of the interface.

**Scripts key on the attribution key, not the flow key.** A script entry names
a protocol, a local endpoint, and optionally a remote one, which is exactly the
shape specification section 8.4 defines for matching a socket table: both
endpoints for TCP, the local endpoint alone for UDP.

Reusing it rather than keying on the full flow key has two consequences worth
having. A UDP entry cannot accidentally require a remote endpoint the platform
would never supply, so the script cannot express an attribution the real
attributor could not make. And the scripted attributor exercises the same
matching shape S10 will implement, including the wildcard bind allowance, so a
test written against the double is a test the real thing has to satisfy.

**Where the generator lives.** Three homes were considered. Putting it in
`xtask` matches the precedent section 25.4 sets for regenerating golden files,
and was declined because `xtask` today has no dependencies at all, deliberately,
and building capture files means either duplicating the frame construction that
already exists or taking an edge from `xtask` into the product graph. Either
costs more than the symmetry is worth.

Putting it in `fragcap-capture` proper was declined because it would ship a
fixture generator to every consumer of the crate for the benefit of this
repository's own tests.

It therefore lives in the corpus test target, which is compiled only when tests
run and is not part of the published crate. Regeneration is an environment
variable on that test rather than a separate command, which is less
discoverable than a subcommand and is documented in `quickstart.md` for that
reason.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - The pipeline runs without a capture driver (Priority: P1)

A contributor writing S06, S07, or S08 needs packets to feed their code, on
their own machine, with no driver installed, no elevation, and no game running.

**Why this priority**: This is the claim section 25.1 makes and the return on
the architecture the project has been paying for since S01. Every slice after
this one depends on it.

**Independent Test**: Read a fixture end to end on a machine with no capture
driver and confirm the packets arrive with the timestamps and bytes the
generator put in.

**Acceptance Scenarios**:

1. **Given** a fixture file, **When** a replay source reads it to exhaustion,
   **Then** it yields exactly the packets the file contains, in file order,
   each carrying the timestamp and the captured and original lengths the file
   records.
2. **Given** an exhausted replay source, **When** it is read again, **Then** it
   reports the terminal closed condition, which a caller can tell apart both
   from a timeout, which means keep going, and from a failure, which means
   something broke.
3. **Given** a replay source, **When** it is asked for its link type, **Then**
   it reports the one the file declares, so the header parser is told what it
   is parsing rather than guessing.
4. **Given** a replay source that has read a fixture, **When** its statistics
   are read, **Then** they report the count it actually delivered.

---

### User Story 2 - The same fixture produces the same run (Priority: P1)

A contributor comparing output against a golden file needs the input to be
identical every time, on every platform.

**Why this priority**: A golden comparison against a varying input is not a
test. Determinism is also what makes a failure reproducible, and an
irreproducible failure in the pipeline is one nobody can fix.

**Independent Test**: Read the same fixture twice in one process and on two
platforms, and compare the full packet sequences for equality.

**Acceptance Scenarios**:

1. **Given** a fixture, **When** it is read twice, **Then** both readings yield
   identical packet sequences, including timestamps, lengths, and bytes.
2. **Given** a fixture written in the opposite byte order, **When** it is read,
   **Then** it yields the same packets as the native-order file, because
   endianness is a property of the file rather than of the packets.
3. **Given** a fixture recording timestamps at microsecond resolution and one
   recording them at nanosecond resolution, **When** each is read, **Then**
   both yield the timestamps their files record, without a resolution
   conversion silently rounding one of them.
4. **Given** any fixture, **When** it is read on a different platform, **Then**
   the packet sequence is unchanged.

---

### User Story 3 - A malformed file says what it skipped (Priority: P1)

An operator or contributor pointing the replay source at a damaged or truncated
file needs to know that something was skipped, and what.

**Why this priority**: Constitution P-4 makes an uncounted discard a defect. A
replay source that silently stops at a truncated record turns a damaged fixture
into a passing test over fewer packets than intended, which is the failure mode
hardest to notice and most damaging to trust in every test built on it.

**Independent Test**: Read a deliberately damaged file and confirm the named
counter for the damage advanced and the undamaged packets still arrived.

**Acceptance Scenarios**:

1. **Given** a file whose header is not a recognized capture file, **When** it
   is opened, **Then** opening fails with a named cause rather than yielding
   zero packets.
2. **Given** a file whose final record is truncated part way through, **When**
   it is read, **Then** every complete record before it is delivered and the
   truncated one advances a named counter.
3. **Given** a record declaring a captured length larger than the file
   contains, **When** it is read, **Then** it advances a named counter distinct
   from the truncation counter and reading stops rather than continuing into
   whatever follows.
4. **Given** a record whose captured length exceeds its original on-wire
   length, **When** it is read, **Then** it advances a named counter, because
   a record claiming to hold more bytes than were on the wire contradicts
   itself.
5. **Given** any damaged file, **When** reading finishes, **Then** the counters
   distinguish each cause rather than reporting one aggregate.

---

### User Story 4 - Attribution is scriptable, including over time (Priority: P1)

A contributor testing the attribution join, port reuse, or retained attribution
needs an attributor that returns a stated answer for a stated flow at a stated
time, without a socket table.

**Why this priority**: Section 25.1 names the scripted attributor as half of
what makes the pipeline testable. Port reuse in particular, where the same
local port belongs to different processes at different times, cannot be tested
any other way without a live machine and a stopwatch.

**Independent Test**: Write a script giving one flow two owners at two times,
and confirm the attributor returns each within its own window.

**Acceptance Scenarios**:

1. **Given** a script naming a flow and an owner, **When** the attributor is
   asked to resolve that flow, **Then** it returns the named owner.
2. **Given** a script naming a flow with no owner, **When** the attributor is
   asked, **Then** it returns nothing, which is attempted and unresolved rather
   than an error.
3. **Given** a script giving one flow different owners in two time windows,
   **When** the attributor is asked at a time in each window, **Then** it
   returns the owner for that window.
4. **Given** a flow the script does not mention, **When** the attributor is
   asked, **Then** it returns nothing rather than guessing.
5. **Given** a script, **When** the attributor is asked for its active
   endpoints, **Then** it reports the endpoints the script declares, so the
   retention behavior of section 11.4 has something to be tested against later.

---

### User Story 5 - A fixture can be reviewed (Priority: P2)

A reviewer seeing a binary file in a pull request needs to know what is in it
without a packet analyzer, and needs confidence that the committed bytes are
what the description claims.

**Why this priority**: Section 25.3 requires fixtures be reviewed before they
land, and CONTRIBUTING makes the same demand. A binary nobody can read is
reviewed in name only. It is P2 rather than P1 because the corpus is usable
without it, and untrustworthy without it in a way that only shows up later.

**Independent Test**: Regenerate the corpus and confirm the committed bytes are
unchanged; then alter one fixture by hand and confirm the check fails.

**Acceptance Scenarios**:

1. **Given** the generator, **When** it is run, **Then** it writes every
   fixture deterministically, producing byte-identical output on every run and
   platform.
2. **Given** the committed corpus, **When** the check mode runs, **Then** it
   confirms every committed fixture matches what the generator produces.
3. **Given** a fixture altered by hand, **When** the check mode runs, **Then**
   it fails and names the fixture that drifted.
4. **Given** the generator source, **When** a reviewer reads it, **Then** the
   contents of each fixture are stated there in readable form rather than only
   as bytes.

---

### User Story 6 - The corpus covers what it claims (Priority: P2)

A contributor relying on the corpus needs each fixture to actually exercise the
condition section 25.3 says it does.

**Why this priority**: A corpus that silently stops covering a condition leaves
every test built on it passing for the wrong reason. This is the failure that
makes a test suite worthless while looking healthy.

**Independent Test**: For each fixture, assert the property it exists to
exercise, so a generator change that drops the property fails here rather than
in a distant slice.

**Acceptance Scenarios**:

1. **Given** each of the eight fixtures section 25.3 names, **When** the corpus
   is checked, **Then** every one exists and is readable.
2. **Given** the fragmented fixture, **When** it is parsed, **Then** it
   contains both an initial and a non-initial fragment.
3. **Given** the loopback fixture, **When** it is parsed, **Then** it produces
   at least one packet whose direction is undetermined because both endpoints
   are local.
4. **Given** the malformed fixture, **When** it is parsed, **Then** it reaches
   more than one distinct parse rejection cause.
5. **Given** the IPv6 fixture, **When** it is parsed, **Then** at least one
   packet carries an extension header chain.
6. **Given** any fixture, **When** it is inspected, **Then** every address is
   from the documentation ranges or is a loopback address, and every payload
   byte is the filler pattern, which is how "carries nothing real" becomes a
   property a test can check rather than a judgment a reviewer has to make.

### Edge Cases

- What happens when the file is empty, or shorter than a capture file header?
  Opening fails with a named cause, distinct from a file that opens and holds
  no packets.
- What happens when the file declares a link type fragcap does not parse? It is
  read and reported as-is. Deciding what to do with an unparseable
  encapsulation belongs to the parser, which already counts it, and refusing to
  read the file would make that path untestable.
- What happens when a record's captured length is zero? It is delivered as a
  zero-length packet, because a zero-length record is well-formed and dropping
  it would be a silent loss.
- What happens when a record's timestamp goes backwards relative to the one
  before it? It is delivered unchanged and not reordered. Reordering would be
  an alteration, and a capture containing an out-of-order timestamp is a fact
  about the capture that the operator should see.
- What happens when the file declares a snapshot length smaller than a record's
  captured length? The record is delivered and a named counter advances,
  because the file contradicts itself and the operator should know, but the
  bytes are real and discarding them would lose an observation.
- What happens when the same script names one flow twice in overlapping time
  windows? The script fails to load with a named cause rather than one window
  silently winning.
- What happens when a script references a fixture that does not exist, or a
  fixture has no script? Both are reported by the corpus check rather than
  discovered by a later slice.
- What happens when a fixture is regenerated on a platform with different line
  endings? The generator writes bytes, not text, so line endings do not arise
  for the capture files. The scripts are text and are written with the line
  endings the repository requires.

## Requirements *(mandatory)*

### Functional Requirements

The capture file reader.

- **FR-001**: The reader MUST read classic pcap files, the format section 25.3
  names for the corpus.
- **FR-002**: The reader MUST accept files in both byte orders, determined from
  the file's own magic number rather than from the reading host.
- **FR-003**: The reader MUST accept both microsecond and nanosecond timestamp
  resolutions, determined from the magic number, and MUST record each
  timestamp at the resolution the file declares without rounding.
- **FR-004**: The reader MUST report the link type the file declares.
- **FR-005**: The reader MUST reject a file whose magic number is not a
  recognized capture file, with a named cause, rather than yielding an empty
  packet sequence.
- **FR-006**: The reader MUST deliver each record's captured bytes unmodified,
  and MUST carry the original on-wire length separately from the captured
  length so truncation stays self-describing.
- **FR-007**: The reader MUST NOT reorder, alter, or drop a record for being
  unusual. An out-of-order timestamp, a zero-length record, and an unparseable
  link type are all delivered as they are.

Reader accounting.

- **FR-008**: A record truncated part way through MUST advance a named counter,
  and every complete record before it MUST still be delivered.
- **FR-009**: A record declaring a captured length the file cannot supply MUST
  advance a named counter distinct from the truncation counter.
- **FR-010**: A record whose captured length exceeds its original on-wire
  length MUST advance a named counter, because it contradicts itself, and MUST
  still be delivered with both lengths exactly as the file records them. The
  reader MUST NOT reconcile the two by adjusting either, because the
  contradiction is the observation and repairing it would hide the defect in
  whatever wrote the file.
- **FR-011**: A record whose captured length exceeds the file's declared
  snapshot length MUST advance a named counter and MUST still be delivered.
- **FR-012**: Reader counters MUST live in their own type rather than as new
  fields on the shared source statistics type, and MUST be separately named per
  cause rather than aggregated.
- **FR-013**: Any total exposed over the reader counters MUST be derived from
  the named counters rather than stored.

The replay source.

- **FR-014**: The replay source MUST live in `fragcap-capture` and MUST
  implement the packet source seam without altering it.
- **FR-015**: The replay source MUST yield the file's packets in file order and
  MUST report exhaustion as the terminal closed condition, distinguishable both
  from a timeout, after which a capture loop continues, and from a failure. It
  MUST NOT report exhaustion as a timeout, which would make a pipeline spin
  forever on a finished file.
- **FR-016**: The replay source MUST report the file's link type and MUST
  report the count it delivered as the number of frames received.
- **FR-016a**: The backend drop counts a source reports MUST be zero for a
  replay source, because there is no kernel and no interface to have dropped
  anything. The reader's own skip counters MUST NOT be reported there:
  presenting fragcap's accounting as a backend's observation is the folding
  S02 kept these two types apart to prevent.
- **FR-017**: The replay source MUST accept a filter program without failing
  and MUST NOT apply it. Its documentation MUST state that a replay source does
  not filter, so no caller mistakes acceptance for application.
- **FR-018**: The replay source MUST contain no attribution logic, per
  constitution P-3.
- **FR-019**: Reading a fixture twice MUST yield identical packet sequences,
  and the same fixture MUST yield the same sequence on every platform.

The scripted attributor.

- **FR-020**: The scripted attributor MUST live in `fragcap-attr` and MUST
  implement the flow attributor seam without altering it.
- **FR-021**: An attribution script MUST be able to declare, for a flow, an
  owner, no owner, and different owners in different time windows.
- **FR-021a**: A script entry MUST identify a flow by protocol, local endpoint,
  and optionally remote endpoint, and the attributor MUST match it through the
  attribution key the flow key derives. It MUST NOT be possible to write an
  entry requiring a remote endpoint for a UDP flow, because the platform never
  supplies one and specification section 8.4 forbids inventing one.
- **FR-021b**: Matching MUST honor the wildcard bind allowance for UDP, so the
  double and the real attributor S10 builds agree on what resolves.
- **FR-022**: The attributor MUST return the owner whose window contains the
  time asked about, and MUST return nothing for a flow the script does not
  mention.
- **FR-022a**: The attributor MUST take the current time from its caller
  through a method of its own, and the flow attributor seam MUST NOT be
  widened to carry a timestamp. An entry with no window MUST match at any time,
  so a caller that never sets a clock still resolves.
- **FR-022b**: Script times MUST be absolute, on the same base as the packet
  timestamps the fixtures carry, so a script and its fixture cannot disagree
  about when something happened.
- **FR-023**: The attributor MUST report the active endpoints the script
  declares.
- **FR-024**: A script declaring overlapping windows for one flow MUST fail to
  load with a named cause rather than resolving the ambiguity silently.
- **FR-025**: The script format MUST add no dependency to the workspace, and
  MUST permit comments so a generated script can carry a readable annotation of
  the times it states numerically.
- **FR-026**: The scripted attributor MUST contain no packet acquisition, per
  constitution P-3.

The fixture corpus.

- **FR-027**: The corpus MUST contain the eight fixtures section 25.3 names,
  under `fixtures/`, committed, each exercising the condition that section
  states for it.
- **FR-028**: Every fixture MUST be synthetic, generated from constants in the
  generator. No fixture may contain traffic captured from a real game session.
- **FR-029**: No fixture may contain an address attributable to a real
  operator. Every address MUST come from one of two sets, enumerated here so
  the rule is checkable rather than judged: the ranges reserved for
  documentation, and the loopback addresses. Loopback is included because the
  loopback fixture cannot exercise its stated condition without it, and a
  loopback address identifies no operator. Link layer addresses MUST likewise
  be drawn from a stated constant set rather than from any real adapter.
- **FR-029a**: Fixture payload bytes MUST be a documented deterministic filler
  pattern rather than arbitrary content. This is what makes "contains no
  account identifier or session token" a property a test can check: a payload
  that is not the filler pattern fails, and no test has to recognize what a
  session token looks like.
- **FR-030**: Every fixture MUST be paired with an attribution script.
- **FR-031**: Each fixture MUST be at most 64 KiB, and the corpus including its
  scripts at most 256 KiB. Both MUST be asserted rather than judged.
- **FR-032**: A generator MUST produce the whole corpus deterministically,
  byte-identically on every run and platform. It MUST NOT ship in any published
  crate, and MUST NOT add a dependency to `xtask`.
- **FR-032a**: No fixture's content may derive from the wall clock, the
  filesystem, the environment, or any other ambient input. Every byte MUST come
  from a constant in the generator, including the timestamp base, because a
  fixture that varies is a golden comparison that cannot be trusted and a
  failure that cannot be reproduced.
- **FR-033**: A check mode MUST regenerate the corpus and compare it against
  what is committed, failing and naming what differs. It MUST cover the
  attribution scripts as well as the capture files, because a script that has
  drifted from its fixture misattributes exactly as quietly as a fixture that
  has drifted from its generator. It MUST run as part of the ordinary test
  suite, so drift is caught by the standard gate rather than by remembering to
  run something.
- **FR-034**: The corpus check MUST report a fixture with no script and a
  script with no fixture.
- **FR-035**: Each fixture's stated condition MUST be asserted by a test, so a
  fixture that stops exercising its condition fails here rather than in a later
  slice.

Hygiene.

- **FR-036**: Every term this slice introduces MUST have a glossary entry in
  `docs/glossary.md` in this same change, per P-6.
- **FR-037**: Every public item MUST carry documentation stating what it
  represents, and any item whose behavior a later slice completes MUST name
  that slice.
- **FR-038**: Any divergence from the architecture of record discovered here
  MUST be recorded in the slice for promotion to specification section 29.

### Key Entities

- **Capture file reader**: Turns the bytes of a recorded capture file into a
  sequence of packets, reporting what it could not read rather than stopping
  quietly.
- **Replay source**: A packet source backed by a file rather than an interface.
  The half of the section 25.1 claim that supplies packets.
- **Scripted attributor**: A flow attributor backed by a declared script rather
  than a socket table. The half that supplies owners.
- **Attribution script**: What the scripted attributor returns, for which flow,
  in which time window. The time dimension is what makes port reuse testable.
- **Fixture**: One small, committed, synthetic capture file that exists to
  exercise one stated condition.
- **Fixture corpus**: The eight fixtures together, plus their scripts, plus the
  generator that produces them and the check that proves they still match it.
- **Replay statistics**: One named counter per way the reader declined to
  deliver a record as-is, kept separate from the counters a live backend
  reports.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The pipeline shape can be exercised end to end over a fixture
  with no capture driver, no elevation, and no game, demonstrated by a test
  that reads a fixture, parses each packet, and resolves each flow against a
  script.
- **SC-002**: Reading any fixture twice yields identical packet sequences,
  asserted for every fixture in the corpus rather than one representative.
- **SC-003**: A file in either byte order and at either timestamp resolution
  yields the same packets, asserted by generating the same fixture four ways
  and comparing.
- **SC-004**: Every reader rejection and skip cause is reachable by a
  constructed file, and a test asserts that exactly the corresponding counter
  advanced.
- **SC-005**: No record is dropped for being unusual: a zero-length record, an
  out-of-order timestamp, and an unparseable link type all arrive.
- **SC-006**: A script resolves one flow to two different owners in two time
  windows, and to nothing outside them.
- **SC-006a**: A UDP script entry resolves a datagram whose local address is a
  specific interface address against a wildcard bind, matching the rule
  specification section 8.4 states, so the double cannot disagree with the
  attributor S10 builds.
- **SC-006b**: The flow attributor seam is unchanged by this slice, verified by
  the trait definition carrying no timestamp parameter after it.
- **SC-007**: The corpus contains all eight named fixtures, each paired with a
  script, verified by a check that fails on a missing pair either way.
- **SC-008**: Each fixture's stated condition is asserted by a test, with no
  fixture whose condition is assumed rather than checked.
- **SC-009**: Regenerating the corpus reproduces the committed bytes exactly,
  and altering a fixture by hand makes the check fail.
- **SC-010**: Every fixture is at most 64 KiB and the corpus at most 256 KiB,
  asserted by a test rather than by inspection.
- **SC-011**: No fixture contains an address outside the documentation ranges
  and the loopback addresses, and no fixture payload departs from the filler
  pattern, both asserted by a test over the whole corpus rather than by
  inspection.
- **SC-012**: `fragcap-core` still builds for a target with no capture backend,
  and the dependency direction check still passes with the two crates this
  slice fills.
- **SC-013**: Every term introduced has a glossary entry.
- **SC-014**: The full local gate set passes: format, lint, tests, repository
  conventions, dependency direction, and per-crate licensing.

## Assumptions

- Specification sections 25.1 and 25.3 are the architecture of record. Where
  they are silent, this slice decides and records the decision for promotion to
  section 29.
- The corpus serves later slices, so a fixture may exercise a condition no test
  in this slice consumes. Section 25.3 lists fixtures for buffering and port
  reuse, which S08 and S10 use; this slice builds them and asserts their
  contents without building the consumers.
- The scripted attributor's time dimension is driven by the caller, which knows
  the packet timestamp. This slice provides the mechanism; wiring it to the
  packet being attributed is the pipeline's job in S08. The caller is
  single-threaded for tier 1 purposes, so the clock needs no synchronization.
  If S08 publishes the attributor across threads it inherits that problem, and
  it is the same problem a real attributor already has.
- Golden output files, specification section 25.4, are not part of this slice.
  They compare pipeline output, and there is no pipeline until S08.
- The replay source reads a whole file. Streaming a file being written
  concurrently is not a case the corpus or the tier 1 tests need.
- `.gitignore` already excludes capture files globally and re-includes those
  under `fixtures/`. This slice is the first to rely on that, so the
  re-inclusion is verified rather than assumed.

## Out of Scope

- Live packet acquisition. S09.
- The real socket table attributor. S10. This slice's attributor answers from a
  script and never reads a socket table.
- Reading pcapng. fragcap writes it in S06; nothing needs to read it.
- Writing any capture file from the pipeline. S06 and S07. The generator writes
  fixture files, which is a build-time concern rather than a capture sink.
- Golden output comparison. S08, once there is pipeline output to compare.
- The pipeline, its buffer, and drop accounting. S08. This slice supplies the
  input that pipeline is tested with.
- Software filtering over a replay source. S13 owns filters and may decide it.
- Property-based fuzzing of the parser, specification section 25.5. It belongs
  with the parser rather than with the corpus.

## Done When

- Every requirement above is satisfied and traceable to a test or a check.
- The full local gate set passes in the foreground, watched to completion.
- The glossary carries an entry for every term introduced.
- Deviations from the architecture of record are recorded in the slice for
  promotion to specification section 29.
- A changelog fragment exists describing the change.
