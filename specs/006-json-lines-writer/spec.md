# Feature Specification: JSON Lines Writer

**Feature Branch**: `feat/json-lines-writer`

**Created**: 2026-08-08

**Status**: Draft

**Slice**: S07 (specification section 13.5; constitution P-2, P-4, P-5, P-6,
P-9)

**Input**: Build the JSON Lines writer: one object per packet, one object per
line, a header object and a trailer object, and a payload-free mode, reusing
the attribution derivation S06 exposed rather than restating which keys are
present.

## Overview

S06 gave fragcap an output format. This slice gives it a second one, for
consumers that do not read pcapng: shell pipelines, log shippers, and anything
that would rather match a line than parse a block structure.

The two formats carry the same facts. That is the point of the slice, and it is
where it can go wrong. Section 13.3's annotation and section 13.5's JSON object
both answer "which process produced this packet, in which direction, and how
sure are we", and if each derives that answer independently the two will
disagree eventually, silently, because each will be internally consistent. S06
anticipated this and split deriving an `Annotation` from rendering it. This
slice is the test of whether that split was real: if the JSON writer has to
restate any presence rule, the split failed and both writers are now wrong in
different ways.

The audience is a researcher with a shell, not a Rust programmer. A line is
self-contained by design, which is the property that makes `grep` and `jq` work
on it and which drives most of the differences from the pcapng profile.

Three properties carry the slice.

**Every line is valid JSON, checked by something that is not this writer.** A
hand-written serializer that satisfies a hand-written validator has proven that
two functions agree. Section 13.5 promises consumers with off-the-shelf JSON
parsers, so the verification has to reach a real one.

**Numbers are exact, unconditionally.** The timestamp in section 13.5's example
carries microsecond resolution, and the obvious implementation is a float. A
float is not wrong today: it renders present-era timestamps correctly. It is
correct only under a magnitude bound it does not state, silently losing whole
microseconds once microseconds-since-epoch exceed a 53-bit significand. A
capture format outlives the reasoning that says its numbers are small enough,
so this one uses integer arithmetic and needs no such reasoning.

**Loss is in the stream.** The trailer object carries the same accounting the
Interface Statistics Block carries, so a consumer reading only the JSON stream
can tell whether the capture is short, and by how much, and where.

## Clarifications

### Session 2026-08-08

- Q: Which crate owns the writer? → A: `fragcap-sink`, beside the pcapng
  writer. Section 8.2 places sink implementations there and the crate's module
  documentation already names S07.
- Q: Does this slice add a JSON dependency? → A: No runtime dependency. The
  writer is hand-rolled, and `serde_json` is added as a dev-dependency to
  validate the output in tests. Writing JSON is a materially smaller problem
  than reading it: escape a string, format an integer, emit a delimiter. The
  reason to hand-roll is not aversion to dependencies but that the exact byte
  shape matters here, and a general-purpose serializer would have to be
  configured into producing it and then trusted to keep producing it. The
  reason to take the dev-dependency is the opposite: verification is worth more
  the less it shares with the thing it verifies, and a third-party parser is
  the strongest independent check available, considerably stronger than the
  structural validator S06 had to hand-write for pcapng.
- Q: How is the timestamp rendered? → A: As a JSON number, built by integer
  arithmetic from the nanosecond timestamp: whole seconds, a decimal point, and
  exactly six digits of microseconds. Never through a float. The float path was
  measured rather than assumed wrong, and it renders present-era timestamps
  correctly; what it does not do is stay correct, because it holds
  microseconds-since-epoch exactly only up to a 53-bit significand and rounds
  silently past it. Integer arithmetic is exact at every magnitude and costs
  the same four lines. This is the same single-narrowing-site discipline S06
  applied, landing on the same value: microseconds, matching section 12.7.
- Q: Which keys does a packet object carry, and in what order? → A: The order
  of section 13.5's example: `ts`, `iface`, `pid`, `proc`, `role`, `stage`,
  `dir`, `attr`, `proto`, the endpoint pair, `len`, `orig_len`, `data`. Present
  keys keep that relative order. JSON object order is semantically irrelevant
  and byte-identical output requires one anyway, and a fixed order is what lets
  a human reading a stream find a field by position.
- Q: `stage` does not appear in the section 13.5 example. Is it written? → A:
  Yes, when present. The example shows a packet that has no stage rather than a
  key set that excludes one; section 13.3 carries `stage` and both formats
  carry the same facts. Omitting it would mean the JSON stream could not
  express a stage-bound attribution that the pcapng output could.
- Q: Is `iface` written on every record, given S06 writes it only in a
  multi-interface capture? → A: Yes, on every record. This is a deliberate
  divergence between the two formats and the reason is structural. In pcapng
  the file already carries exactly one Interface Description Block, so the key
  would repeat information the container holds; in JSON Lines a line is
  self-contained by design, which is the property that makes the format worth
  having, and a consumer that split the stream would otherwise lose the
  interface. Section 13.5's example shows it unconditionally. The shared
  derivation supports both because interface is a parameter to it rather than
  a rule inside it.
- Q: Does the header object carry the section 12.7 session anchor? → A: No.
  Section 13.5 says the header declares the fragcap version, the session
  anchor, and the interface set. There is no session in this slice and
  therefore no anchor, exactly as in S06, and inventing a shape for it now
  would fix a format decision on behalf of S08, which owns capture start. The
  header carries version and interface set, and the gap is recorded rather
  than papered over.
- Q: How are header and trailer distinguished from packet records? → A: By a
  `type` key that packet records do not carry, per section 13.5. `type` is the
  first key in both, so a consumer can dispatch on the first field without
  parsing the whole object.
- Q: Which case is hex encoded in? → A: Lowercase, following section 13.5's
  example. S06 percent-encodes in uppercase, following the convention for that
  encoding. The inconsistency is only apparent: each follows its own format's
  convention, and both are fixed so goldens are stable.
- Q: Section 13.5 shows `src` and `dst`, but `FlowKey` carries `local` and
  `remote`. Which is written? → A: `src` and `dst` when the direction is known,
  and `local` and `remote` when it is not. `FlowKey` normalized endpoint
  position deliberately, so wire order is not stored and can only be recovered
  by combining the key with the direction: outbound means source is local,
  inbound means source is remote. When direction is `unknown`, which this slice
  produces for every loopback packet and every packet with no flow key, wire
  order is genuinely not known, and emitting `src` and `dst` anyway would pick
  one at random and present it as observed. Emitting the normalized pair under
  its own names says exactly what is known: these two endpoints, this one is
  local, and which sent this particular packet was not determined. A consumer
  can dispatch on which key set is present. Recorded for promotion to
  specification section 29.
- Q: What does a packet with no flow key carry? → A: No `proto`, and neither
  endpoint pair. The keys are absent rather than null, because absent means "not
  resolved" consistently across this format, and a null would invite a consumer
  to treat it as a value.
- Q: What does payload-free mode omit? → A: The `data` key entirely, not an
  empty string. Section 13.5 calls it a metadata-only stream; an empty `data`
  would be indistinguishable from a zero-length packet, which is a real
  observation this format must be able to record.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A researcher greps a capture (Priority: P1)

Someone with a shell and no Rust toolchain pipes a fragcap JSON stream into
`jq` and asks which processes sent traffic to a given address. It works with
the tools already on their machine.

**Why this priority**: This is the entire reason section 13.5 exists. pcapng
serves analyzers; this serves pipelines.

**Independent Test**: Write a stream from a fixture and parse every line with a
third-party JSON parser, asserting the parse succeeds and the values match what
the packet carried.

**Acceptance Scenarios**:

1. **Given** a capture, **When** it is written as JSON Lines, **Then** every
   line parses as a JSON object with an off-the-shelf parser.
2. **Given** a stream, **When** lines are counted, **Then** there is exactly
   one line per packet plus a header and a trailer, and no enclosing array.
3. **Given** a packet with a process name containing a quote, a backslash, or a
   control character, **When** it is written, **Then** the line still parses
   and the name round-trips exactly.
4. **Given** any record, **When** it is read, **Then** it is a complete object
   on a single line, with no embedded newline.

---

### User Story 2 - Both formats agree (Priority: P1)

The contributor comparing a pcapng capture against the JSON stream of the same
input finds the same attribution facts in both.

**Why this priority**: Two derivations of the same rules drift silently. S06
split derivation from rendering specifically so this slice would not need its
own copy, and an untested split is an assumed one.

**Independent Test**: For every fixture, derive the annotation once and assert
the JSON record and the pcapng comment carry identical process, role, stage,
direction, and fidelity values.

**Acceptance Scenarios**:

1. **Given** a fixture, **When** both writers run over it, **Then** for every
   packet the `pid`, `proc`, `role`, `stage`, `dir`, and `attr` values agree.
2. **Given** an unattributed packet, **When** both writers run, **Then**
   neither emits identity keys and both record `attr` as `none`.
3. **Given** the JSON writer, **When** it decides which keys to emit, **Then**
   it calls the shared derivation rather than inspecting the packet itself.

---

### User Story 3 - Timestamps are exact (Priority: P1)

An analyst correlating a fragcap stream against another log finds the
microsecond values are the ones that were recorded, not values near them.

**Why this priority**: A timestamp that is approximately right is the kind of
defect that survives review and corrupts a correlation months later.
Constitution P-9 makes recording an approximation of an observation a defect
rather than a rounding convenience.

**Independent Test**: Write timestamps chosen to be unrepresentable in binary
floating point and assert the emitted text is exact to the microsecond.

**Acceptance Scenarios**:

1. **Given** a timestamp of 1,754,500,000.123456 seconds, **When** it is
   written, **Then** the emitted text is exactly `1754500000.123456`.
2. **Given** any whole second, **When** it is written, **Then** six decimal
   places are still emitted, so field widths do not vary.
3. **Given** a sub-microsecond component, **When** it is written, **Then** it
   is floored, consistently with the declared microsecond resolution.
4. **Given** a timestamp before the Unix epoch, **When** it is written,
   **Then** the writer reports a named error rather than emitting a negative
   or wrapped value.

---

### User Story 4 - The stream accounts for itself (Priority: P1)

An operator consuming only the JSON stream can tell whether the capture lost
anything, and where.

**Why this priority**: Constitution P-4. A consumer of this format may never
see the pcapng file or the console summary, so the accounting has to be in the
stream or it is not available to them at all.

**Independent Test**: Finish a writer with a statistics snapshot carrying a
non-zero value in every counter and assert each appears in the trailer.

**Acceptance Scenarios**:

1. **Given** a finished stream, **When** the last line is read, **Then** it is
   a trailer object carrying every section 12.4 counter.
2. **Given** a trailer, **When** it is read, **Then** it is distinguishable
   from a packet record by its `type` key.
3. **Given** a capture that dropped nothing, **When** the trailer is read,
   **Then** the counters are present and zero rather than omitted.

---

### User Story 5 - Payload-free mode (Priority: P2)

An operator running a long capture for flow analysis takes a metadata-only
stream at a fraction of the volume.

**Why this priority**: Section 13.5 names it, and it is the difference between
a stream that can run for hours and one that cannot. Not P1 because the
attribution facts, not the payload, are what this project exists to record.

**Independent Test**: Write the same fixture with and without payloads and
assert the records are identical except for the presence of `data`.

**Acceptance Scenarios**:

1. **Given** payload-free mode, **When** a packet is written, **Then** the
   `data` key is absent entirely.
2. **Given** payload-free mode, **When** a zero-length packet is written,
   **Then** it is still distinguishable from a suppressed payload, because
   `len` is still present and `data` is absent in both modes for different
   reasons that `len` disambiguates.
3. **Given** either mode, **When** records are compared, **Then** every key
   other than `data` is identical.

---

### User Story 6 - The same input produces the same stream (Priority: P2)

A contributor changing unrelated code sees a byte-level diff if the format
moved.

**Why this priority**: The same reasoning as S06. The goldens are the check
that outlives the author.

**Independent Test**: Write the same fixture twice and compare, then compare
against a committed golden.

**Acceptance Scenarios**:

1. **Given** a fixture, **When** it is written twice, **Then** the two streams
   are byte-identical.
2. **Given** a committed golden, **When** output diverges, **Then** the test
   fails and names the first differing line.

---

### Edge Cases

- What happens when a process name contains a double quote or a backslash? It
  is escaped per JSON string rules and round-trips exactly.
- What happens when a process name contains a control character? It is escaped
  with the `\uXXXX` form, because JSON forbids a literal control character in a
  string.
- What happens when a process name contains a character outside the Basic
  Multilingual Plane? It is emitted as UTF-8 directly. JSON permits it, and
  escaping it would produce a surrogate pair for no benefit.
- What happens to a packet with no flow key? `proto` and both endpoint pairs
  are absent. It is still written, with its attribution keys and lengths.
- What happens when the original length is smaller than the captured length? It
  is written exactly as recorded. Consistent with S04 and S06: a contradiction
  is reported, not repaired.
- What happens to a zero-length payload? `data` is an empty string in payload
  mode. Distinct from payload-free mode, where the key is absent.
- What happens if the writer is dropped without finishing? The lines already
  written are complete and parseable; the trailer is absent, and its absence is
  how a consumer knows the stream was truncated.

## Requirements *(mandatory)*

### Functional Requirements

**Stream shape, section 13.5**

- **FR-001**: The writer MUST emit newline-delimited JSON, one object per line,
  with no enclosing array.
- **FR-002**: The writer MUST emit exactly one object per packet.
- **FR-003**: The writer MUST emit one header object before any packet record.
- **FR-004**: The writer MUST emit one trailer object when finished.
- **FR-005**: The header and trailer MUST carry a `type` key, and packet
  records MUST NOT carry one.
- **FR-006**: `type` MUST be the first key of the header and the trailer.
- **FR-007**: The header MUST declare the fragcap version and the interface
  set.
- **FR-008**: Every record MUST occupy exactly one line, with no embedded
  newline in any value.
- **FR-009**: Every line MUST be terminated with a single line feed, including
  the last.

**Packet records**

- **FR-010**: Keys MUST appear in the order `ts`, `iface`, `pid`, `proc`,
  `role`, `stage`, `dir`, `attr`, `proto`, then either `src` and `dst` or
  `local` and `remote`, then `len`, `orig_len`, `data`, with present keys
  keeping that relative order.
- **FR-011**: `ts` MUST be a JSON number carrying whole seconds and exactly six
  fractional digits.
- **FR-012**: `ts` MUST be produced by integer arithmetic and MUST NOT pass
  through a floating point value at any point.
- **FR-013**: `ts` MUST floor sub-microsecond components, consistently with the
  declared resolution.
- **FR-014**: A timestamp predating the Unix epoch MUST produce a named error
  rather than a negative or wrapped value.
- **FR-015**: `iface` MUST be present on every packet record.
- **FR-016**: `pid` and `proc` MUST be present exactly when the packet is
  attributed, and MUST be absent otherwise.
- **FR-017**: `role` and `stage` MUST each be present when the corresponding
  value is, decided independently of each other.
- **FR-018**: `dir` and `attr` MUST be present on every packet record.
- **FR-019**: `proto` MUST be present when the packet has a flow key, and MUST
  be absent otherwise, never null.
- **FR-019a**: When the packet has a flow key and a known direction, the writer
  MUST emit `src` and `dst` in wire order, derived from the direction: outbound
  means source is the local endpoint, inbound means source is the remote one.
- **FR-019b**: When the packet has a flow key and no known direction, the
  writer MUST emit `local` and `remote` instead of `src` and `dst`, and MUST
  NOT guess wire order.
- **FR-019c**: A record MUST NOT carry both key pairs.
- **FR-020**: `len` MUST carry the captured length and `orig_len` the original
  length, both exactly as recorded.
- **FR-021**: `data` MUST carry the payload hex-encoded in lowercase.

**Shared derivation**

- **FR-022**: The writer MUST obtain which attribution keys are present from
  the derivation S06 exposed, and MUST NOT restate the presence rules.
- **FR-023**: For any packet, the `pid`, `proc`, `role`, `stage`, `dir`, and
  `attr` values MUST equal those the pcapng writer emits for the same packet.

**Payload-free mode**

- **FR-024**: In payload-free mode the writer MUST omit the `data` key
  entirely, not emit an empty or null value.
- **FR-025**: Payload-free mode MUST change no other key.

**Escaping and encoding**

- **FR-026**: String values MUST escape the double quote, the backslash, and
  every code point below 0x20, per JSON string rules.
- **FR-027**: Control characters without a short escape MUST use the `\uXXXX`
  form.
- **FR-028**: Characters above 0x7F MUST be emitted as UTF-8 rather than
  escaped.
- **FR-029**: Every emitted line MUST parse as a JSON object under a
  third-party parser.

**Loss accounting, constitution P-4**

- **FR-030**: The trailer MUST carry every section 12.4 counter.
- **FR-031**: Counters MUST be present and zero rather than omitted when
  nothing was lost.
- **FR-032**: An unattributed packet MUST be written, never dropped or
  skipped.
- **FR-033**: A record the writer refuses MUST be reported to the caller as an
  error rather than silently discarded.

**Placement and dependencies**

- **FR-034**: The writer MUST live in `fragcap-sink` and implement the `Sink`
  trait.
- **FR-035**: The writer MUST accept any `std::io::Write` target.
- **FR-036**: The slice MUST add no runtime dependency to any crate.
- **FR-037**: A JSON parser MAY be added as a dev-dependency for verification,
  and MUST NOT be reachable from any non-test code path.

**Verification**

- **FR-038**: Writing the same input twice MUST produce byte-identical output.
- **FR-038a**: The writer MUST read no clock, no environment variable, no
  locale, and no host property. Every byte MUST be a function of the packets,
  the interface set, the payload mode, and the statistics snapshot.
- **FR-038b**: The interface set in the header MUST be emitted in declaration
  order, from an ordered collection, so the header does not vary between runs.
- **FR-038c**: Every trailer counter MUST come from the supplied statistics
  snapshot, never sampled or recomputed by the writer.
- **FR-039**: The test suite MUST compare output against committed goldens and
  fail with the first differing line.
- **FR-040**: A golden MUST exist for every fixture in the S04 corpus.
- **FR-041**: The tier 1 tests MUST run with no capture driver, no elevated
  privilege, and no game.

### Key Entities

- **Packet record**: One observation as a JSON object: when, on what interface,
  by which process, in which direction, with what confidence, over which flow,
  at what length, with what bytes.
- **Header record**: Declares the writer version and the interface set, once,
  before any packet.
- **Trailer record**: Carries final statistics, once, after every packet. Its
  absence means the stream was truncated.
- **Payload mode**: Whether `data` is written. A property of the writer, fixed
  at construction, not a per-record decision.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Every line of every stream this slice produces parses with an
  off-the-shelf JSON parser, in 100 percent of cases.
- **SC-002a**: A record never asserts wire order that was not determined; the
  key names distinguish an observed direction from a normalized position.
- **SC-002**: The attribution facts in a JSON stream and in the pcapng capture
  of the same input agree for every packet, with no exceptions.
- **SC-003**: A microsecond timestamp is emitted exactly, including values that
  binary floating point cannot represent.
- **SC-004**: An unattributed packet appears in the stream, distinguishable
  from an attributed one, in 100 percent of cases.
- **SC-005**: Every section 12.4 counter is recoverable from the stream alone.
- **SC-006**: Payload-free mode changes exactly one key.
- **SC-007**: Writing the same fixture twice produces identical bytes.
- **SC-008**: The write path runs in the test suite with no capture driver, no
  elevated privilege, and no game.

## Assumptions

- The S04 fixture corpus is the input to every test. No fixture is added or
  modified.
- Attribution values are already valid UTF-8, arriving as Rust strings.
- The interface set is supplied by the caller at construction, as in S06.
- Section 12.4 counters reach the writer through the `CaptureStats` snapshot at
  finish time.
- `serde_json` is acceptable under the constitution's license allowlist, being
  dual MIT and Apache-2.0.

## Out of Scope

- **The pipeline (section 8.6)**. S08. This slice provides a sink.
- **Transports and streaming destinations (section 14)**. S15. This writer
  targets a `Write`, and what that write goes to is not its concern.
- **Statistics reporting beyond the trailer (section 13.6)**. The console
  summary is S14.
- **The session anchor (section 12.7)**. Recorded as a known gap, as in S06.
  S08 owns capture start.
- **Reading JSON Lines**. fragcap writes this format. The parser in the tests
  is a verification tool, not a capability.
- **A JSON schema document**. Useful, and belongs with the documentation site
  in S18.

## Done When

- [ ] `fragcap-sink` contains a JSON Lines writer implementing `Sink` over any
      `std::io::Write`.
- [ ] Header, packet, and trailer records are emitted per section 13.5, with a
      fixed key order.
- [ ] Timestamps are exact to the microsecond and never pass through a float.
- [ ] The attribution keys come from S06's shared derivation, with a test
      asserting both formats agree for every fixture.
- [ ] Payload-free mode omits exactly one key.
- [ ] Every section 12.4 counter is recoverable from the trailer.
- [ ] Every line is validated by a third-party JSON parser in the test suite.
- [ ] Goldens committed for all eight fixtures, with a drift check in the gate.
- [ ] No runtime dependency added; the dev-dependency is test-only.
- [ ] `cargo xtask ci` green, `neutral` and `msrv` exit 0.
- [ ] A glossary entry exists for every term this slice introduces.
- [ ] The section 12.7 gap, the `iface` divergence from S06, and the
      `src`/`dst` versus `local`/`remote` resolution are recorded as changelog
      decisions.
