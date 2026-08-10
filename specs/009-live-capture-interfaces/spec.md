# Feature Specification: Live Capture Source and Interfaces

**Feature Branch**: `feat/live-capture-interfaces`

**Created**: 2026-08-09

**Status**: Draft

**Slice**: S09 (specification sections 12.1 and 12.2; constitution P-1, P-2,
P-3, P-4, P-6, P-9, and the licensing section)

**Input**: Implement specification sections 12.1 (interface selection) and 12.2
(kernel filtering strategy) in `fragcap-capture`: a live `PacketSource` backed
by the platform capture driver, interface enumeration and the three-step
selection precedence, per-interface handles and threads, interface identifiers
carried on captured packets into output, and the bootstrap phase of the filter
lifecycle. npcap is detected, never bundled, downloaded, installed, or
vendored. Add `Send` to `PacketSource` through the deviation process, and lift
the single-interface restriction both writers carry.

## Overview

Eight slices have built a capture tool that has never captured anything. The
pipeline runs, the parser parses, both writers write, and every packet that has
ever passed through any of it came out of a file that this repository generated
for the purpose. S09 is where fragcap first reads from a network interface.

That framing matters for what the slice owns. The pipeline does not change
shape here, because S08 built it to a specification that already anticipated
this. What changes is that the thing feeding it becomes real, and three
consequences follow that the offline path was able to defer.

**A capture has more than one interface.** Section 12.1 requires one handle and
one thread per interface, all feeding one bounded buffer, and packets carrying
an interface identifier preserved into output. Nothing in the current type
vocabulary can express that. `RawPacket` records what a frame was and when, and
says nothing about where it arrived, because until now there was exactly one
where. Both writers refuse a second interface declaration for precisely this
reason, and their refusal is a placeholder S09 removes by supplying what was
missing rather than by relaxing a check.

**A source is opened on a thread that is not the caller's.** `PacketSource` is
the only one of the four behavioral traits without a `Send` bound. S08 chose
that deliberately: it acquired on the calling thread and spawned only the sink
thread, so that a trait intended to survive to 1.0.0 unchanged did not have to
change to make one slice work. Section 12.1's per-interface thread ends that
arrangement. The bound is added here through the deviation process, recorded,
and promoted to specification section 29, because a trait in section 8.5 is the
architecture of record and not a local edit.

**Choosing wrongly is invisible.** Interface selection is the first place in
this project where fragcap makes a decision the operator did not make and
cannot see the consequences of. Selecting an interface the traffic is not on
produces a run that exits zero, writes a well-formed capture file, and contains
nothing. That is the same failure class S05 built its ambiguity check for, and
it gets the same treatment: the selection is reported, and the reasoning behind
each inclusion and exclusion is available rather than implied.

Two further properties shape the slice.

**Loopback is captured, and the reason changed.** Reconnaissance refuted
assumption A-5: the launcher-to-client handoff is not visible on loopback in
either focal title. Loopback capture survives on the strength of a different
measurement from the same sessions, which is that both titles use loopback
heavily for intra-process communication, one moving 5.4 MB of it in twenty
minutes. Excluding it would leave a visible and unexplained gap in the record
of what a process did, which is a P-9 problem and not merely an incomplete one.

**The bootstrap filter is permissive on purpose.** Section 12.2 phase one
admits IPv4 and IPv6 and nothing else, and discards in userspace, because no
attribution exists yet and narrowing before it does would discard traffic in
the kernel with no way to know what was lost. S09 owns phase one and the
mechanism that installs a filter on a live handle. Phases two and three, which
compile a narrowed filter from the attribution map and maintain it under
debounce and rate limits, belong to S13 and depend on a socket table attributor
that S10 has not built yet.

## Clarifications

### Session 2026-08-09

- Q: Where does the interface identifier live in the type vocabulary? -> A: On
  `CapturedPacket`, not optional, attached by the pipeline at the lift from
  `RawPacket`. `RawPacket` stays interface-free because a source knows only its
  own interface, so putting the identifier there would make every source
  including the replay source invent one, and would repeat a per-run constant
  on every packet. Non-optional rather than optional because every packet did
  arrive somewhere; an absent identifier would be a claim that a packet came
  from nowhere, which is the kind of comfortable untruth P-9 forbids. It also
  makes the writers' existing refusal of an undeclared interface reachable
  rather than vestigial.

- Q: Does the pipeline take several sources, or does a multiplexing source wrap
  them and present one? -> A: The pipeline takes several sources directly and
  spawns a thread for each. A multiplexer is the tempting answer because it
  leaves the pipeline untouched, and it is wrong twice: it would need its own
  fan-in buffer, duplicating the bounded buffer section 12.4 already specifies
  as the single one, and `next_packet` returns a `RawPacket` that by the answer
  above carries no interface identifier, so the multiplexer would have to
  invent a side channel to carry what the pipeline is about to attach anyway.

- Q: What must a machine have to build the workspace and run the ordinary check
  set? -> A: Neither the capture driver nor its software development kit. The
  live source sits behind a Cargo feature that is off by default, so `cargo
  xtask ci` on any machine builds and tests without either. The `platform`
  workflow enables the feature and is the only place tier 2 tests run. This
  keeps the constitution's no-vendoring rule from turning into a build-time
  requirement every contributor has to satisfy.

- Q: Where do the loopback and broad capture settings reach selection from,
  given that `fragcap-profile` is a sibling of `fragcap-capture`? -> A: From
  the caller, as plain values. Specification section 8.3 forbids a sibling
  edge, `cargo xtask deps` enforces it mechanically, and the facade is the only
  crate that legitimately holds both. Selection therefore takes an inventory
  and a settings value it is given, which is also what makes it testable
  without a machine.

- Q: What ends a run when one capture thread fails? -> A: The interface retires
  and the run continues; the run ends when every source has retired or the stop
  handle is set. This is the same answer S08 reached for a failed sink, for the
  same reason: ending the whole capture because one interface disappeared would
  discard observations still arriving on the others. Retirement is recorded
  with the interface named and the reason, and surfaced in the run's report. It
  advances no drop counter, because nothing was observed and then discarded.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Capture without naming an interface (Priority: P1)

An operator on a machine with the capture driver installed starts fragcap
without saying which interface to watch. fragcap selects the interface carrying
the default route, adds the loopback adapter because the profile asked for it,
opens a handle on each, and produces packets into the existing pipeline. The
run ends with statistics that include what the driver itself reported losing.

**Why this priority**: This is the slice's reason to exist. Every other story
here is a refinement of a capability that does not exist until this one works,
and it is the first time the project's central claim is exercised against a
real network rather than a fixture.

**Independent Test**: On a Windows machine with the driver present, run a
capture over self-generated loopback traffic and assert that the packets the
test sent appear in the output with their own timestamps and lengths. No game
and no profiled process is required.

**Acceptance Scenarios**:

1. **Given** a machine with the capture driver installed and a profile
   requesting loopback capture, **When** a capture is started with no interface
   named, **Then** the interface carrying the default route and the loopback
   adapter are both selected and both opened.
2. **Given** an open live source, **When** traffic arrives on the interface,
   **Then** each frame is yielded with the timestamp the driver supplied and
   the original on-wire length, whether or not a snapshot length truncated it.
3. **Given** a capture that has ended, **When** the statistics are read,
   **Then** `kernel_dropped` and `interface_dropped` carry the driver's own
   numbers, relayed unaltered and not folded into any fragcap counter.
4. **Given** a live source that has not seen a frame within the read timeout,
   **When** the capture loop polls it, **Then** it reports nothing rather than
   an error, and the loop continues.

---

### User Story 2 - Capture several interfaces and tell them apart (Priority: P1)

An operator watches more than one interface in a single run. Each interface is
captured on its own handle and its own thread, all of them feed one bounded
buffer, and every packet in the output names the interface it arrived on. The
capture file declares each interface with its own link type, and an unmodified
analyzer attributes each packet to the right one.

**Why this priority**: Section 12.1 requires it, the default selection in story
one already produces two interfaces, and both writers currently refuse the
second. Without this the first story cannot ship in its stated form.

**Independent Test**: Drive the pipeline from two replay sources declared as
distinct interfaces with different link types, and assert that the pcapng
output declares two interfaces, that each packet block references the correct
one, and that the JSON Lines output names the interface on every record.

**Acceptance Scenarios**:

1. **Given** two interfaces selected for a run, **When** the capture starts,
   **Then** each is opened on its own handle and read on its own thread, and
   both deliver into the same bounded buffer.
2. **Given** a packet acquired on a named interface, **When** it reaches a
   sink, **Then** it carries the identity of the interface it arrived on.
3. **Given** a capture holding more than one interface, **When** the pcapng
   writer declares them, **Then** each carries its own link type and snapshot
   length, and each packet block references the interface it arrived on.
4. **Given** a capture holding exactly one interface, **When** the output is
   written, **Then** the per-packet interface key is omitted, matching section
   13.3 and the behavior established in S06.
5. **Given** two interfaces with different link types, **When** frames from
   both are parsed, **Then** each is parsed against its own interface's link
   type rather than a capture-wide one.

---

### User Story 3 - Control which interfaces are watched (Priority: P2)

An operator overrides the automatic choice, either by naming interfaces
explicitly or by asking for broad capture. fragcap applies the section 12.1
precedence, excludes virtual interfaces from automatic selection while leaving
them explicitly selectable, and reports what it chose and what it passed over.

**Why this priority**: The automatic path in story one is one of three
precedence steps, and the other two are what an operator reaches for when the
automatic choice is wrong. Reporting the decision is what makes a wrong
automatic choice recoverable rather than mysterious.

**Independent Test**: Run the selection over a synthetic inventory of
interfaces covering every case (default route, loopback, virtual, down, no
address) and assert the chosen set and the recorded reason for every inclusion
and exclusion. No driver is required, because selection is a decision over an
inventory rather than an act on a handle.

**Acceptance Scenarios**:

1. **Given** interfaces named explicitly, **When** selection runs, **Then**
   exactly those are selected, including any that automatic selection would
   have excluded as virtual.
2. **Given** no explicit names and a profile not requesting broad capture,
   **When** selection runs, **Then** the interface carrying the default route
   is selected, plus the loopback adapter if and only if the profile requests
   loopback capture.
3. **Given** no explicit names and a profile requesting broad capture, **When**
   selection runs, **Then** every interface that is up, has an address, and is
   not virtual is selected.
4. **Given** an interface identified as virtual, **When** automatic selection
   runs, **Then** it is excluded and the exclusion is recorded with its reason.
5. **Given** a name that matches no enumerated interface, **When** selection
   runs, **Then** the run fails with a named error listing the names available,
   rather than silently capturing on fewer interfaces than asked for.
6. **Given** any completed selection, **When** the operator reads the run's
   report, **Then** every enumerated interface appears with either its
   selection or the reason it was passed over.

---

### User Story 4 - Be told when the capture driver is absent (Priority: P2)

An operator without the capture driver, or with it installed without the
options fragcap needs, gets a specific statement of what is missing and where
to obtain it. fragcap never downloads it, never installs it, and never invokes
an installer.

**Why this priority**: The constitution's licensing section makes detection
rather than installation binding, and a capture tool that fails with an opaque
error on the most common first-run condition is one an operator abandons. The
`doctor` command that presents this to a user belongs to S14; the capability it
presents belongs here, because it is the live source that knows.

**Independent Test**: Interrogate the detection path on a machine without the
driver and assert that it reports absence with the official download location,
and that no process is spawned and nothing is written outside the capture
output.

**Acceptance Scenarios**:

1. **Given** a machine with no capture driver, **When** a live capture is
   attempted, **Then** it fails with an error naming the driver, its absence,
   and the official download location.
2. **Given** a machine with the driver present, **When** detection runs,
   **Then** the driver's version is reported.
3. **Given** any detection outcome, **When** the code path is inspected,
   **Then** it neither downloads, installs, nor invokes an installer, and the
   repository contains no driver binaries, installers, or software development
   kit files.
4. **Given** an interface that disappears mid-capture, **When** the capture
   thread next reads from it, **Then** the run reports the device as lost with
   the interface named, and the remaining interfaces continue.

---

### Edge Cases

- What happens when the machine has no default route? Automatic selection has
  no interface to choose, and the run must say so rather than capture nothing
  successfully.
- What happens when the loopback adapter is absent because the driver was
  installed without loopback support? The requested loopback capture cannot be
  satisfied, and the missing installation option is named specifically rather
  than reported as a generic failure.
- What happens when two interfaces report the same name? Interface identity
  must remain unambiguous within a run even when the platform's names are not
  unique.
- What happens when an interface is up and has an address but produces no
  traffic for the whole run? It appears in the output's interface declarations
  with zero packets, because it was watched and reporting otherwise would
  understate what was observed.
- What happens when one capture thread fails and others continue? The failure
  must be attributable to its interface, and the run must not report the whole
  capture as failed while other interfaces are still delivering.
- What happens when the buffer fills because one interface is far busier than
  the others? Eviction is drop-oldest across the shared buffer, as S08 built
  it, and the resulting `buffer_dropped` count is the capture's, not one
  interface's.
- What happens when a snapshot length truncates a frame? The original on-wire
  length is preserved and the packet reports itself as truncated, exactly as
  the replay path already does.
- What happens on a non-Windows target? The crate must still build, with the
  live source absent rather than stubbed into something that compiles and lies.

## Requirements *(mandatory)*

### Functional Requirements

**Interface enumeration and identity, section 12.1**

- **FR-001**: The system MUST enumerate the machine's capture-capable
  interfaces, reporting for each a stable identifier, a human-readable name, a
  link type, its addresses, whether it is up, whether it is a loopback adapter,
  and whether it is virtual.
- **FR-002**: The system MUST assign each selected interface an identifier that
  is unique within a run and stable for the run's duration, independent of
  whether the platform's own names are unique.
- **FR-003**: Interface enumeration MUST NOT require the capture driver to open
  a handle, so that an operator can be told what exists before anything is
  captured.
- **FR-004**: The system MUST classify an interface as virtual using a
  documented rule, and MUST record the rule's verdict rather than applying it
  silently.

**Interface selection, section 12.1**

- **FR-005**: Selection MUST apply the section 12.1 precedence in order:
  explicitly named interfaces first; otherwise the default-route interface plus
  the loopback adapter when loopback capture is requested; otherwise, when
  broad capture is requested, every interface that is up, has an address, and
  is not virtual.
- **FR-006**: Explicitly named interfaces MUST be selectable even when
  automatic selection would exclude them as virtual.
- **FR-007**: An explicitly named interface that matches nothing enumerated
  MUST produce a named error listing the available interfaces, which the caller
  surfaces as a failed run rather than capturing on fewer interfaces than
  asked for.
- **FR-008**: Automatic selection MUST exclude virtual interfaces.
- **FR-009**: Selection MUST produce, for every enumerated interface, either
  its inclusion or a named reason for its exclusion, and that record MUST be
  available to the run's report.
- **FR-010**: Selection MUST be a decision over an inventory value rather than
  an act performed on a live machine, so that it is testable without a capture
  driver.
- **FR-011**: A selection that would choose no interfaces at all MUST produce a
  named error rather than an empty selection, so that the caller cannot open a
  capture that exits successfully having watched nothing.
- **FR-012**: Selection MUST take the loopback and broad capture settings from
  its caller as plain values. `fragcap-capture` MUST NOT depend on
  `fragcap-profile`, which is its sibling under specification section 8.3.

**The live source, sections 12.1 and 12.7**

- **FR-013**: The system MUST provide a `PacketSource` implementation backed by
  the platform capture driver, one instance per interface, in
  `fragcap-capture`.
- **FR-014**: The live source MUST yield each frame with the timestamp the
  capture driver supplied, unaltered.
- **FR-015**: The live source MUST report the original on-wire length
  separately from the bytes retained, so that a snapshot length is
  self-describing.
- **FR-016**: The live source MUST report a read timeout as no packet rather
  than as an error, so that the capture loop continues.
- **FR-017**: The live source MUST relay the driver's own dropped-frame counts
  into `kernel_dropped` and `interface_dropped` unaltered, and MUST NOT fold
  them into any fragcap counter.
- **FR-018**: The live source MUST report the link type of its own interface.
- **FR-019**: The live source MUST report an interface that has disappeared as
  a lost device, naming the interface, and MUST NOT report it as a recoverable
  timeout.
- **FR-020**: The live source MUST accept a snapshot length and a promiscuous
  mode setting at open time.
- **FR-021**: `fragcap-capture` MUST build for a target with no capture
  backend, with the live source compiled out rather than replaced by a stub
  that appears to work.
- **FR-022**: The live source MUST sit behind a Cargo feature that is off by
  default, so that building the workspace and running the ordinary check set
  requires neither the capture driver nor its software development kit.

**The `Send` bound and per-interface threads, section 12.1**

- **FR-023**: `PacketSource` MUST require `Send`, and the change MUST be
  recorded as a deviation in this slice and promoted to specification section
  29.
- **FR-024**: The pipeline MUST accept more than one packet source directly and
  MUST run each on its own thread. It MUST NOT reach multi-interface capture
  through a multiplexing source that fans several sources into one, because
  that would introduce a second buffer where section 12.4 specifies one.
- **FR-025**: All capture threads MUST deliver into the single bounded buffer
  S08 built, retaining its drop-oldest semantics and its conservation identity.
- **FR-026**: A packet MUST be parsed against the link type of the interface it
  arrived on, rather than a capture-wide link type.
- **FR-027**: The failure of one capture thread MUST retire that interface and
  MUST NOT end the run while another interface is still delivering. The run
  MUST end when every source has retired or the stop handle is set.
- **FR-028**: A retirement MUST be recorded with the interface named and the
  reason given, and MUST be surfaced in the run's report. It MUST NOT advance a
  drop counter, because nothing was observed and then discarded.
- **FR-029**: The run MUST remain able to report which interface each counter
  movement originated from where the counter is per-interface, and MUST NOT
  claim per-interface precision for counters that are capture-wide.

**Interface identity in output, sections 12.1 and 13.3**

- **FR-030**: `CapturedPacket` MUST carry the identity of the interface the
  packet was acquired on, not optionally, attached by the pipeline at the lift
  from `RawPacket`. `RawPacket` MUST remain interface-free, because a source
  knows only its own interface.
- **FR-031**: The pcapng writer MUST accept more than one interface
  declaration, each with its own link type and snapshot length, and MUST
  reference the correct one from each packet block.
- **FR-032**: The pcapng writer MUST stop refusing a second interface
  declaration, and the refusal MUST be replaced by working support rather than
  by removing the check.
- **FR-033**: The JSON Lines writer MUST name the interface on every record in
  a multi-interface capture.
- **FR-034**: Each writer MUST keep the single-interface behavior it already
  has, because SC-005 requires byte-identical output for that case. The two
  differ and the difference is deliberate: the pcapng writer omits the
  annotation `iface` key in a single-interface capture, per section 13.3,
  because a packet block already references its interface numerically and the
  key would be redundant. The JSON Lines writer names the interface on every
  record regardless, because a JSON Lines record is consumed one line at a time
  and a line that does not say where its packet came from is not independently
  interpretable.

  This requirement originally asserted that both writers omit the key. That was
  false about the JSON writer, and the committed goldens are what caught it
  during implementation.
- **FR-035**: A packet naming an interface that was never declared MUST
  continue to be refused rather than written against a fabricated declaration.

**The bootstrap filter, section 12.2**

- **FR-036**: The system MUST install a bootstrap filter admitting IPv4 and
  IPv6 traffic and nothing else on each live handle before any packet is
  delivered.
- **FR-037**: Installing a filter MUST be possible on an already-open live
  handle, so that S13 can narrow one without reopening.
- **FR-038**: A filter the backend rejects MUST be reported as a rejected
  filter with the backend's own detail, and MUST NOT silently leave the
  previous filter installed.
- **FR-039**: The system MUST NOT implement filter narrowing or maintenance in
  this slice; phases two and three of section 12.2 belong to S13.
- **FR-040**: Userspace scope decisions MUST remain independent of the
  installed filter, so that correctness never depends on filter freshness.

**Capture driver detection, constitution licensing section**

- **FR-041**: The system MUST detect the capture driver's presence and version
  at runtime.
- **FR-042**: The system MUST report absence with the official download
  location.
- **FR-043**: The system MUST NOT download, install, or invoke an installer for
  the capture driver, under any code path.
- **FR-044**: The repository MUST contain no capture driver binaries,
  installers, or software development kit files, and no build step may commit
  them.
- **FR-045**: When a required non-default installation option is absent, the
  system MUST name that option specifically rather than reporting a generic
  failure.

**Constitutional constraints**

- **FR-046**: No technique on the section 19.3 denylist may be used. Packet
  acquisition uses the NDIS capture driver and nothing else.
- **FR-047**: `fragcap-core` MUST NOT acquire a platform-specific dependency, a
  capture library, or an I/O crate as a result of this slice.
- **FR-048**: No packet acquisition code may enter an attributor, and no
  attribution logic may enter a packet source.
- **FR-049**: Every discard path introduced by this slice MUST have a named
  counter that is surfaced in statistics.
- **FR-050**: Every term introduced by this slice MUST receive a glossary entry
  in the same change.
- **FR-051**: Any dependency added MUST carry a license from the allowlist in
  the constitution's licensing section.

**Found in review of pull request 12**

- **FR-052**: Each packet source MUST carry the address set of the interface it
  captures on, and header parsing MUST use that set rather than a run-wide one.
  Specification section 12.6 determines direction by matching against "the
  address set of the capturing interface", and section 8.4 places the flow key's
  local endpoint by the same test. A shared set cannot express this on a
  multi-homed machine: one interface's addresses reject every other interface's
  traffic, and their union assigns a direction and a local endpoint to a packet
  observed on an adapter that does not hold the matched address.
- **FR-053**: A capture thread that panics MUST wind down the other capture
  threads before the panic reaches the caller. Every capture thread holds a
  producer, so a surviving one keeps the bounded buffer open, the output thread
  waits on it, and the run hangs instead of reporting a defect.
- **FR-054**: Where a backend cannot honour the timeout passed to
  `next_packet`, it MUST document that and MUST make the value that does govern
  reachable, rather than silently substituting a different one.

### Key Entities

- **Interface inventory**: What the machine reports about its capture-capable
  interfaces at a moment in time. A value, so that selection over it is a pure
  decision testable without a driver.
- **Interface record**: One entry in the inventory. Identifier, name, link
  type, addresses, up state, loopback flag, virtual flag.
- **Interface identifier**: The run-scoped identity assigned to a selected
  interface, carried on every packet acquired from it and preserved into
  output.
- **Selection outcome**: The chosen interfaces plus, for every interface not
  chosen, the named reason. Exists so that an unexpectedly empty capture is
  diagnosable from the run's own report.
- **Live source**: A `PacketSource` bound to one open handle on one interface.
- **Capture driver report**: Presence, version, and the state of the
  installation options fragcap requires.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: On a machine with the capture driver installed, a capture over
  traffic the test itself generates yields every frame the test sent, with the
  timestamps and lengths the driver reported.
- **SC-002**: Interface selection produces the documented result for every case
  in the inventory matrix (explicitly named, default route, loopback present
  and absent, virtual, down, no address, broad capture), verified without a
  capture driver.
- **SC-003**: Every enumerated interface appears in the run's report either as
  selected or with a named reason for exclusion, with no interface unaccounted
  for.
- **SC-004**: A capture holding two interfaces opens in an unmodified analyzer
  with both declared, and every packet attributed to the interface it arrived
  on.
- **SC-005**: A capture holding one interface produces output byte-identical to
  what S06 and S07 produce today for the same input, so that multi-interface
  support costs the single-interface case nothing.
- **SC-006**: The conservation identity S08 established continues to hold with
  several capture threads running: for every sink, received plus
  `buffer_dropped` plus refusals equals `packets_captured`.
- **SC-007**: The driver's reported drop counts appear in the run's statistics
  distinct from every fragcap counter, and changing one does not change the
  other.
- **SC-008**: On a machine without the capture driver, the failure names the
  driver, its absence, and where to obtain it, within the first attempted
  capture.
- **SC-009**: `fragcap-core` builds for a target with no capture backend, and
  `fragcap-capture` builds there with the live source absent.
- **SC-010**: The repository contains no capture driver binary, installer, or
  software development kit file, verified mechanically.
- **SC-011**: `cargo xtask ci` passes on a machine with neither the capture
  driver nor its software development kit installed, because the live source is
  behind a feature that is off by default.
- **SC-012**: A run in which one of several interfaces fails part way through
  continues delivering from the others, ends when the last has retired, and
  names the failed interface and the reason in its report.
- **SC-013**: `cargo xtask deps` continues to pass, with no edge from
  `fragcap-capture` to any sibling crate.

## Assumptions

- The capture driver targeted on Windows is npcap, per specification section
  20.2 and the constitution's licensing section. No other driver is supported
  in this slice.
- Windows is the only platform with a live source in this slice. Section 28's
  Linux and macOS backends are later work, and the seam this slice fills is the
  one they will fill too.
- The profile supplies whether loopback capture and broad capture are
  requested. `fragcap-profile` exists as of S05 and is the natural source of
  those settings, but it is a sibling of `fragcap-capture` and section 8.3
  forbids the edge, so the facade translates the profile into the plain values
  selection takes. Wiring the command line to them is S14's work.
- Tests requiring a capture driver are tier 2 by specification section 25.2 and
  run on the Windows runner rather than in the ordinary check set. The
  `platform` workflow exists for this and has never run.
- The `doctor` command that presents driver detection to an operator is S14.
  This slice supplies the capability and not the presentation.
- The session anchor of section 12.7 is not part of this slice. It is written
  into the capture file, which makes it the writer's concern, and it has no
  consumer until correlation with external event logs is built.
- Filter compilation beyond the fixed bootstrap program belongs to S13.
  Whatever this slice needs to express the bootstrap filter is the minimum that
  installs it, not a general compiler.
- The pipeline's existing shape survives. This slice widens its source side to
  several sources and changes nothing about the buffer, the sink thread, the
  drop accounting, or the retirement of a failed sink.

## Dependencies

- **S03** supplies header parsing and the link types a live interface will
  report.
- **S08** supplies the pipeline, the bounded buffer, and the drop accounting
  this slice feeds.
- **S05** supplies the profile that carries the loopback and broad capture
  settings.
- **S06 and S07** supply the writers whose single-interface restriction this
  slice lifts.
- **Q-5** is resolved. Reconnaissance refuted assumption A-5 and supplied the
  measurement that keeps loopback capture in scope for a different reason.

## Deviations Recorded By This Slice

- **Adding `Send` to `PacketSource`.** Specification section 8.5 declares the
  trait without it, and S08 relied on its absence. Section 12.1 requires one
  capture thread per interface, which cannot be satisfied without it. Recorded
  here and promoted to specification section 29.
- **An interface identifier on the captured packet.** Section 8.4's packet
  vocabulary predates any capture with more than one interface. Section 12.1
  requires the identifier be preserved into output, which requires it be
  carried. Recorded here and promoted to specification section 29.
- **Per-interface source statistics.** Each handle has its own driver buffer,
  so `kernel_dropped` was always a per-interface quantity; there has simply
  never been a second interface to reveal it. `CaptureStats::source` becomes a
  computed sum over per-interface entries rather than a stored field, which
  keeps `stats.rs`'s standing rule that no aggregate is stored and lets a
  kernel drop name the buffer that is undersized. Found during planning rather
  than before it. Recorded here and promoted to specification section 29.
