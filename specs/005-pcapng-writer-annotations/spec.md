# Feature Specification: pcapng Writer and Annotation Encoding

**Feature Branch**: `feat/pcapng-writer-annotations`

**Created**: 2026-08-08

**Status**: Draft

**Slice**: S06 (specification sections 13.1, 13.2, 13.3, 13.4; constitution
P-2, P-4, P-5, P-6, P-9)

**Input**: Build the pcapng writer: the block structure of section 13.2, the
attribution annotation of section 13.3, and the fidelity marking of section
13.4, such that an unmodified analyzer opens the result and either displays the
attribution or ignores it.

## Overview

This is the slice where fragcap first produces something. S02 built the
vocabulary, S03 the parser, and S04 the substrate that feeds them. Nothing so
far writes a byte that outlives the process.

The output format is the whole product claim. fragcap exists because standard
capture tooling discards the packet-to-process association below the socket
layer, and the value of recovering it is realized in Wireshark and tshark and
the analysis tooling that already exists, not in tooling fragcap would have to
build. That is constitution P-5, and it is the constraint that decides every
open question in this slice: a format that carries more at the cost of needing
a plugin is a worse format here, at any richness.

So the target is narrow and testable. A file fragcap writes is a valid pcapng
file. An analyzer that has never heard of fragcap opens it, shows the packets,
and shows the attribution as a per-packet comment, because comments are the one
pcapng option every reader already displays. An analyzer that does know about
fragcap parses the same string for structure. Neither needs configuring.

The audience is two readers who never meet. The first is a researcher opening a
capture in Wireshark, who should see `fragcap:pid=7412;proc=eso64.exe;...` in
the packet comment column without having been told to expect it. The second is
S07's JSON Lines writer and S08's pipeline, which need the annotation to be a
value they can construct and compare rather than a string assembled inline at
the point of writing.

Three properties carry the slice.

**Validity is not self-assessed.** A writer that emits what its own reader
accepts has proven nothing. Section 13.1 promises unmodified readers, so the
verification has to reach outside this codebase: the block layout, option
codes, alignment, and length fields are checked against the pcapng structure
itself, and the byte-level expectations are committed as goldens that a human
reviewed once and a machine compares every run after.

**The annotation is a type, not a format string.** Section 13.3 gives a
grammar, and a grammar with one hand-rolled producer and no parser is a grammar
nothing has tested. The encoding is a value with an encoder and a decoder that
round-trip, so that a percent-encoding defect surfaces in a unit test rather
than in somebody's capture six slices later.

**No drop class becomes invisible.** Section 13.2 populates the Interface
Statistics Block from the section 12.4 counters, but pcapng's three standard
fields describe losses upstream of fragcap, and fragcap has two counters of its
own that no standard field expresses. Writing only what fits the standard
fields would satisfy the letter of 13.2 and violate P-4, which is the more
important of the two. The slice records every counter, using the standard
fields where they exist and a declared comment where they do not.

## Clarifications

### Session 2026-08-08

- Q: Which crate owns the writer? → A: `fragcap-sink`, which specification
  section 8.2 defines as the home of sink implementations, and whose module
  documentation already names S06 as the slice that fills it. The writer
  implements the `Sink` trait S02 fixed in core.
- Q: What does the writer write into? → A: Anything implementing
  `std::io::Write`, not a file path. A file is one such target and the CLI will
  supply one, but a test needs an in-memory buffer to compare bytes against a
  golden without touching a filesystem, and section 25.1 requires the tier 1
  tests run with no privilege.
- Q: Which byte order does the writer emit? → A: Little-endian always, never
  host order. pcapng permits either and declares the choice in the Section
  Header Block byte-order magic, so both are valid output. A host-dependent
  writer produces different bytes on different machines for the same input,
  which makes a golden comparison impossible to run on more than one
  architecture and makes a capture non-reproducible. S04 already rejected
  host-dependent reading for the same reason.
- Q: How are fragcap's own drop counters carried, given pcapng has no field for
  them? → A: In an `opt_comment` on the Interface Statistics Block, using the
  same `fragcap:` sentinel grammar as the packet annotation. `isb_ifrecv`,
  `isb_ifdrop`, and `isb_osdrop` carry what they are defined to carry;
  `buffer_dropped` and `sink_dropped` are fragcap's own losses and appear in
  the comment. Silently omitting them would be a P-4 defect, and overloading
  `isb_osdrop` with them would be a P-9 defect, since it would report a fragcap
  loss as an operating system loss.
- Q: Which characters are percent-encoded in an annotation value? → A: The
  three section 13.3 names (semicolon, equals sign, percent sign) plus every
  code point below 0x20 and the code point 0x7F. The three named characters
  break the grammar. The control characters break the containing format, since
  pcapng defines `opt_comment` as a UTF-8 string and a reader that meets a NUL
  or a newline mid-comment behaves unpredictably. Percent-encoding is
  lossless and reversible, so widening it preserves the observation and does
  not conflict with P-9. Recorded for promotion to specification section 29.
- Q: Does the writer emit the section 12.7 session anchor? → A: No. Section
  12.7 says the anchor is written into the capture file, and section 13.2 does
  not list it among the blocks. There is no session in this slice and therefore
  no anchor to record, and inventing a placement now would fix a format
  decision on behalf of the slice that actually has the data. S08 owns capture
  start and supplies it. Recorded as a known gap against section 12.7 rather
  than resolved silently.
- Q: How many interfaces does the writer support? → A: Any number, though the
  fixture corpus exercises one. Section 13.2 assigns interface identifiers in
  the Interface Description Block that every packet block then references, so
  the identifier has to be a real value the writer tracks rather than a
  constant zero. A writer that hardcodes one interface would need its packet
  path rewritten by S09, which is the slice least able to absorb it.
- Q: Is the `iface` annotation key written when there is one interface? → A:
  No. Section 13.3 marks it "when multi-interface", and writing it always would
  add a key to every packet comment in the common case for no information. It
  appears when the writer holds more than one Interface Description Block.
- Q: What does `dir` carry when the pipeline determined no direction? → A:
  `unknown`. Section 13.3 marks `dir` as always present and enumerates `in`,
  `out`, and `local`, but `Direction` in core has exactly two variants and
  `CapturedPacket::direction` is optional, so a fourth state exists in the type
  and has no value in the table. Omitting the key would break the "always"
  guarantee that lets a consumer parse without a presence check. Writing
  `local` would be worse: section 12.6 leaves loopback direction undetermined
  until it can be resolved from the attributed process's endpoint, so `local`
  and "not determined" are different facts, and asserting the first from the
  second is exactly the substitution P-9 forbids. Recorded for promotion to
  specification section 29.
- Q: Then when is `local` written? → A: Not in this slice. It becomes
  reachable when loopback direction resolution lands with the attributor work,
  and the encoder carries the value from the day it is defined so that the
  later slice supplies data rather than extending a grammar. Recorded as a
  known gap rather than left implicit.
- Q: Are `role` and `stage` emitted together or independently? → A:
  Independently, each when present. Section 13.3 marks both "when
  stage-bound", which reads as a pair, but `Attribution` carries them as two
  independent options and its builder sets them separately, so a role without a
  stage is representable and will occur. Treating them as a pair would either
  drop an observed role or fabricate a stage. Recorded for promotion to
  specification section 29.
- Q: How is the nanosecond timestamp converted to the declared microsecond
  resolution? → A: By flooring toward negative infinity, not truncating toward
  zero, so that ordering is preserved across the epoch. The core `Timestamp`
  documentation already names this slice as the single site where the lossy
  narrowing happens, precisely so P-9 compliance has one place to inspect. The
  loss is declared rather than hidden, because the Interface Description Block
  states the resolution the file actually carries.
- Q: What happens to a timestamp that predates the Unix epoch? → A: The writer
  refuses it with a named error. pcapng timestamps are unsigned, so a negative
  value has no representation, and both available workarounds are worse than
  failing: clamping to zero reports an observation that did not happen at that
  time, and wrapping reports one at a time in the far future. Neither is
  recoverable by a reader.
- Q: Which fixtures get golden files? → A: All eight. Selecting a subset
  invites the question of which ones matter, which is answered differently by
  each contributor who asks it. The goldens are produced by a committed
  generator with a drift check in the ordinary gate, which is the pattern S04
  established for the fixture corpus and which works for the same reason: the
  generator is the readable record of what the bytes are supposed to be.
- Q: What if a packet's captured length exceeds the interface's declared snap
  length? → A: It is written exactly as recorded, both lengths intact. S04
  settled this shape for a file that contradicts itself, and the reasoning
  carries: repairing the contradiction would hide a defect in whatever produced
  it. Nothing is discarded, so no counter is owed under P-4.
- Q: What exactly does the Section Header Block comment say? → A:
  `fragcap:profile=0.1.0`. Naming what the comment declares without fixing its
  text leaves a golden comparison pinning bytes nobody chose. Reusing the
  sentinel grammar means a consumer that can read a packet annotation can read
  this too, rather than meeting a second ad hoc format in the same file.
- Q: What timestamp does an Interface Statistics Block carry? → A: The
  timestamp of the last packet written against that interface, or zero when
  none was. The block header carries a timestamp field that has to hold
  something, and the obvious something is a clock reading, which would make
  output differ between runs and break every golden on the second run. Deriving
  it from the data keeps the writer a pure function of its input, which is the
  same property S04 required of reading.
- Q: In what order do annotation keys appear? → A: The order of the section
  13.3 table, with present keys keeping their relative order. Byte-identical
  output requires a fixed order, and the obvious implementation of a key-value
  set does not supply one. Choosing the table's order means the specification
  and the output read the same way.
- Q: Which case are percent-encoded hexadecimal digits written in? → A:
  Uppercase, and the decoder accepts either. Both cases are valid
  percent-encoding, so only fixing one produces a stable golden, and a decoder
  that reads only its own output would fail on a file another tool wrote.
- Q: How is an empty attribution value encoded? → A: As an empty value, with
  the key present. An empty process name is unlikely and representable, and
  omitting the key on encountering one would report that the packet was not
  attributed, which is a different fact about the observation.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - An unmodified analyzer opens the capture (Priority: P1)

A researcher who has never installed anything from this project opens a
fragcap capture in Wireshark. The file opens. The packets are there, with
correct timestamps and lengths. The attribution appears in the packet comment
column.

**Why this priority**: This is the product claim of section 13.1 and the reason
constitution P-5 exists. If it does not hold, every other property of the
writer is irrelevant, because the capture is only worth taking if it remains a
capture file.

**Independent Test**: Write a capture from a fixture, then verify its bytes
against the pcapng block structure independently of the writer's own code: the
Section Header Block byte-order magic and block type, every block's leading and
trailing length agreeing, every option aligned to a 32-bit boundary, and the
option list terminated. A file that satisfies the format's own structural rules
is one a conforming reader can read.

**Acceptance Scenarios**:

1. **Given** a fixture read through the replay source, **When** it is written
   as pcapng, **Then** the output begins with a Section Header Block whose
   byte-order magic is `0x1A2B3C4D` read little-endian.
2. **Given** a written capture, **When** every block is walked by its declared
   length, **Then** the walk consumes the file exactly, with the trailing
   length of each block equal to its leading length.
3. **Given** a written capture, **When** an Enhanced Packet Block is read,
   **Then** its interface identifier names an Interface Description Block that
   appeared earlier in the file.
4. **Given** a packet whose captured length is not a multiple of four, **When**
   it is written, **Then** the packet data is padded to the next 32-bit
   boundary and the padding is not counted in `captured_len`.

---

### User Story 2 - Attribution is visible without special tooling (Priority: P1)

The same researcher reads the comment on a packet and can tell which process
produced it, in which direction it travelled, and how confident fragcap is in
the answer, without consulting documentation for a binary layout.

**Why this priority**: Section 13.3 chose `opt_comment` over a custom option
precisely so that this works with no plugin. The choice is settled in the
specification and is not relitigated here, but it only pays off if the string
written is the string the grammar promises.

**Independent Test**: Encode an attribution covering every key in the section
13.3 table, then decode it and compare against the input. Encoding and decoding
are separate code paths, so a round-trip that holds proves the grammar rather
than proving one function consistent with itself.

**Acceptance Scenarios**:

1. **Given** a packet attributed to pid 7412 running `eso64.exe`, outbound,
   resolved live, **When** its annotation is encoded, **Then** the result is
   `fragcap:pid=7412;proc=eso64.exe;dir=out;attr=live`.
2. **Given** an annotation string produced by the encoder, **When** it is
   decoded, **Then** every key and value equals what was encoded.
3. **Given** a process name containing a semicolon, an equals sign, or a
   percent sign, **When** its annotation is encoded and decoded, **Then** the
   decoded name equals the original exactly.
4. **Given** any annotation, **When** it is encoded, **Then** the string begins
   with the `fragcap:` sentinel and every key is lowercase ASCII.

---

### User Story 3 - Fidelity is recorded, never inferred (Priority: P1)

An analyst discounting inferential attribution can tell, per packet, whether
the endpoint was in the socket table at the time or was resolved from the grace
period map after it had left.

**Why this priority**: Section 13.4 is explicit that `attr` is never inferred
by a consumer, and the distinction is the difference between an observation and
an inference. A retained attribution can be wrong, in the specific case where
an endpoint closed and was reassigned inside the grace period. A consumer that
cannot see which packets are exposed to that cannot correct for it.

**Independent Test**: Write packets in all three attribution states and confirm
each carries the correct `attr` value, and that an unattributed packet carries
no `pid`, `proc`, `role`, or `stage` key at all rather than an empty or
placeholder value.

**Acceptance Scenarios**:

1. **Given** a packet resolved from the live socket table, **When** it is
   written, **Then** its annotation carries `attr=live`.
2. **Given** a packet resolved from the grace period map, **When** it is
   written, **Then** its annotation carries `attr=retained`.
3. **Given** a packet that could not be attributed, **When** it is written,
   **Then** its annotation carries `attr=none` and carries no `pid`, `proc`,
   `role`, or `stage` key.
4. **Given** a packet that could not be attributed, **When** it is written,
   **Then** it is written, and is not dropped or skipped.

---

### User Story 4 - Every drop is in the file (Priority: P1)

An operator who captured for an hour reads the file and can account for the
difference between what the interface received and what the file contains,
broken down by where each packet was lost.

**Why this priority**: Constitution P-4 makes an uncounted discard a defect
rather than an oversight, and section 13.2 puts the counters in the file rather
than only in a console summary. A capture that is quietly short produces
conclusions that are wrong in a way the reader cannot detect.

**Independent Test**: Finish a writer with a statistics snapshot carrying a
non-zero value in every counter, then read the Interface Statistics Block back
and confirm each value appears, in a standard field where one exists and in the
declared comment where none does.

**Acceptance Scenarios**:

1. **Given** a capture with kernel and interface drops, **When** the writer is
   finished, **Then** an Interface Statistics Block per interface carries
   `isb_ifrecv`, `isb_ifdrop`, and `isb_osdrop`.
2. **Given** a capture with fragcap buffer drops or sink drops, **When** the
   writer is finished, **Then** those counts appear in the Interface Statistics
   Block comment under the `fragcap:` sentinel.
3. **Given** a capture with fragcap buffer drops, **When** the file is read,
   **Then** those drops are not reported as `isb_osdrop`.
4. **Given** a writer that is dropped without being finished, **When** the file
   is read, **Then** the packets written so far are readable, since a
   truncated capture is more useful than an unreadable one.

---

### User Story 5 - The same input produces the same file (Priority: P2)

A contributor changing unrelated code runs the test suite and sees a byte-level
diff if the output format moved, on any machine and any architecture.

**Why this priority**: The goldens are the only check that reaches outside this
codebase's own assumptions, and they are worthless if the bytes legitimately
vary between runs. S04 established the same property for reading, and the pair
is what makes S08's end-to-end comparison possible.

**Independent Test**: Write the same fixture twice in one process and compare
the buffers, and compare both against a committed golden file.

**Acceptance Scenarios**:

1. **Given** a fixture, **When** it is written twice, **Then** the two outputs
   are byte-identical.
2. **Given** a fixture, **When** it is written on a big-endian host, **Then**
   the output is byte-identical to the output on a little-endian host.
3. **Given** a committed golden, **When** the writer output diverges from it,
   **Then** the test fails and names the offset of the first differing byte.

---

### User Story 6 - The next slices can build on the annotation (Priority: P2)

The contributor writing S07 emits the same attribution as JSON without
reimplementing the rules for which keys are present, and the contributor
writing S08 constructs an annotation from a `CapturedPacket` without knowing
pcapng.

**Why this priority**: Section 13.5's JSON Lines output carries the same
attribution facts as section 13.3's comment. Two independent derivations of
"which keys are present" would drift, and the drift would be silent because
each would be self-consistent.

**Independent Test**: Construct the annotation value from a `CapturedPacket`
and assert the key set matches the packet's attribution state, with the pcapng
serialization applied as a separate step.

**Acceptance Scenarios**:

1. **Given** a `CapturedPacket`, **When** an annotation is derived from it,
   **Then** the key set follows section 13.3 without the caller choosing.
2. **Given** an annotation value, **When** it is rendered, **Then** rendering is
   a separate operation from deriving, so a second output format reuses the
   derivation.

---

### Edge Cases

- What happens when a process name is not valid UTF-8? The name reaches the
  writer as a Rust string and is therefore already valid UTF-8; the question is
  answered upstream, and the writer does not re-validate.
- What happens when a packet's captured length is zero? It is written, with a
  zero-length data field and the correct padding, because a zero-length
  observation is still an observation.
- What happens when the original length is smaller than the captured length? It
  is written exactly as recorded. S04 established that a file contradicting
  itself is reported rather than repaired, and repairing it here would hide a
  defect in whatever produced it.
- What happens when an annotation exceeds a reasonable comment length? It is
  written. pcapng options carry a 16-bit length, so an option body cannot
  exceed 65,535 bytes; an annotation approaching that is impossible from the
  key set in section 13.3, and the writer errors rather than silently
  truncating if it ever occurs.
- What happens when no interface has been declared and a packet arrives? That
  is a programming error in the caller, not a capture condition, and the writer
  reports it rather than inventing an interface identifier.
- What happens when the underlying writer returns an error mid-block? The error
  is returned. A partially written block makes the file unreadable from that
  point, which is why the finish path is separate and why story 4 scenario 4
  bounds the damage to the blocks already complete.
- What happens when the same interface is declared twice? Each declaration
  produces its own Interface Description Block and its own identifier, because
  pcapng identifies interfaces by declaration order and the writer does not
  deduplicate on the caller's behalf.
- What happens when a packet's captured length exceeds the interface's declared
  snap length? It is written exactly as recorded, with both lengths intact.
  Nothing is discarded, so no counter is owed.
- What happens when a timestamp predates the Unix epoch? The writer returns a
  named error rather than writing the packet, because pcapng cannot represent
  the value and both clamping and wrapping would record a time that was not
  observed.
- What happens to the sub-microsecond part of a timestamp? It is discarded by
  the declared resolution, which the Interface Description Block states, so the
  file does not overstate its own precision. This is the single narrowing site
  in the codebase, named as such by the core timestamp type.
- What happens when the pipeline determined no direction? The packet carries
  `dir=unknown`. It is not guessed, defaulted to inbound, or reported as
  loopback.

## Requirements *(mandatory)*

### Functional Requirements

**Block structure, section 13.2**

- **FR-001**: The writer MUST emit exactly one Section Header Block per file,
  as the first block.
- **FR-002**: The Section Header Block MUST declare `shb_userappl` as
  `fragcap/0.1.0`.
- **FR-003**: The Section Header Block MUST carry an `opt_comment` declaring
  the annotation profile version, as `fragcap:profile=0.1.0`, using the same
  sentinel grammar as every other annotation fragcap writes.
- **FR-004**: The writer MUST emit one Interface Description Block per declared
  interface, before any Enhanced Packet Block referencing it.
- **FR-005**: Each Interface Description Block MUST declare the link type, the
  snap length, the interface name, and the timestamp resolution.
- **FR-006**: The writer MUST assign interface identifiers in declaration
  order, starting at zero, and every Enhanced Packet Block MUST reference a
  valid one.
- **FR-007**: The writer MUST emit one Enhanced Packet Block per packet,
  carrying the interface identifier, the timestamp, the captured length, the
  original length, and the packet data.
- **FR-008**: The writer MUST emit one Interface Statistics Block per declared
  interface when finished, carrying `isb_ifrecv`, `isb_ifdrop`, and
  `isb_osdrop`.
- **FR-008a**: The timestamp of an Interface Statistics Block MUST be the
  timestamp of the last packet written against that interface, or zero when no
  packet was written. The writer MUST NOT read a clock.
- **FR-009**: The writer MUST record timestamps at microsecond resolution and
  declare that resolution in each Interface Description Block, per section
  12.7.
- **FR-009a**: The writer MUST convert nanoseconds to microseconds by flooring
  toward negative infinity, so that a timestamp ordering is preserved by the
  conversion.
- **FR-009b**: The writer MUST refuse a timestamp that predates the Unix epoch
  with a named error, and MUST NOT clamp or wrap it.
- **FR-010**: The writer MUST pad every block body and every option to a 32-bit
  boundary, and MUST NOT count padding in any declared length.
- **FR-011**: Every block MUST carry a trailing total length equal to its
  leading total length.
- **FR-012**: Every option list MUST be terminated with `opt_endofopt`.
- **FR-013**: The writer MUST emit little-endian byte order regardless of host
  architecture.

**Attribution encoding, section 13.3**

- **FR-014**: Every Enhanced Packet Block MUST carry its attribution annotation
  in the `opt_comment` option.
- **FR-015**: An annotation MUST begin with the `fragcap:` sentinel, followed
  by semicolon-separated `key=value` pairs.
- **FR-016**: Annotation keys MUST be lowercase ASCII.
- **FR-016a**: Keys MUST appear in the order of the section 13.3 table: `pid`,
  `proc`, `role`, `stage`, `dir`, `attr`, `iface`. Present keys keep that
  relative order regardless of which are absent.
- **FR-017**: The writer MUST emit `pid` and `proc` when the packet is
  attributed, and MUST omit both when it is not.
- **FR-018**: The writer MUST emit `role` when a role is present and `stage`
  when a stage is present, deciding each independently, and MUST omit either
  when absent.
- **FR-019**: The writer MUST emit `dir` on every packet, with a value of `in`,
  `out`, `local`, or `unknown`.
- **FR-019a**: The writer MUST emit `dir=unknown` when the pipeline determined
  no direction, and MUST NOT emit `local` on that basis.
- **FR-020**: The writer MUST emit `attr` on every packet, with a value of
  `live`, `retained`, or `none`.
- **FR-021**: The writer MUST emit `iface` when more than one interface is
  declared, and MUST omit it otherwise.
- **FR-022**: The writer MUST percent-encode any value containing a semicolon,
  an equals sign, or a percent sign.
- **FR-023**: The writer MUST percent-encode any value containing a code point
  below 0x20 or the code point 0x7F.
- **FR-023a**: Percent-encoded hexadecimal digits MUST be uppercase on output.
  The decoder MUST accept either case, since it reads what other tools write.
- **FR-023b**: An empty value MUST be written as an empty value, not omitted.
  Omitting the key would report a different fact, since absence of `proc`
  means the packet was not attributed.
- **FR-024**: The project MUST provide a decoder for the annotation grammar,
  and encoding followed by decoding MUST yield the original value for every
  input.
- **FR-025**: Deriving which keys an annotation carries MUST be separable from
  rendering it as a string, so that section 13.5's writer reuses the
  derivation.

**Attribution fidelity, section 13.4**

- **FR-026**: `attr=live` MUST indicate the endpoint was present in the socket
  table at resolution time.
- **FR-027**: `attr=retained` MUST indicate resolution from the grace period
  map of section 11.4.
- **FR-028**: `attr=none` MUST indicate the packet could not be attributed.
- **FR-029**: The writer MUST NOT infer, upgrade, or default a fidelity value.
  It records what the pipeline resolved.
- **FR-030**: An unattributed packet MUST be written, never dropped or skipped,
  per constitution P-4.

**Loss accounting, constitution P-4**

- **FR-031**: The writer MUST carry fragcap's own drop counters, which no
  standard Interface Statistics Block field expresses, in an `opt_comment` on
  that block under the `fragcap:` sentinel.
- **FR-032**: The writer MUST NOT report a fragcap loss in a field defined to
  mean an operating system or interface loss.
- **FR-033**: A packet the writer itself refuses MUST be reported to the caller
  as an error rather than silently discarded.

**Placement and dependencies**

- **FR-034**: The writer MUST live in `fragcap-sink` and MUST implement the
  `Sink` trait defined in `fragcap-core`.
- **FR-035**: The writer MUST accept any `std::io::Write` target, not a file
  path only.
- **FR-036**: `fragcap-core` MUST NOT gain a dependency on `fragcap-sink`, and
  the dependency direction check MUST continue to pass.

**Verification**

- **FR-037**: Writing the same input twice MUST produce byte-identical output.
- **FR-038**: The test suite MUST compare writer output against committed
  golden files, and MUST fail with the offset of the first differing byte.
- **FR-038a**: A golden MUST exist for every fixture in the S04 corpus, and a
  drift check in the ordinary gate MUST fail if a committed golden stops
  matching the generator that produced it.
- **FR-039**: The test suite MUST validate output against the pcapng block
  structure by a path independent of the writer's own encoding code.
- **FR-040**: The tier 1 tests MUST run against the committed fixture corpus
  with no capture driver, no elevated privilege, and no game, per section 25.1.

### Key Entities

- **Annotation**: The attribution facts for one packet, as a value: process
  identity when attributed, role and stage when stage-bound, direction,
  fidelity, and interface name when relevant. Derived from a captured packet,
  rendered to the section 13.3 grammar as a separate step.
- **Fidelity**: How attribution was obtained. Live, retained, or none. Recorded
  by the pipeline, never inferred by the writer or by a consumer.
- **Interface declaration**: The link type, snap length, name, and timestamp
  resolution of one capture interface, plus the identifier assigned to it in
  declaration order and referenced by every packet written against it.
- **Writer**: A sink holding an output target, the interfaces declared so far,
  and enough state to emit statistics when finished. Consumed by finishing,
  which is what lets the trailing blocks be written exactly once.
- **Golden**: A committed file of expected output bytes for a given fixture,
  reviewed once by a human and compared mechanically thereafter.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A capture written by fragcap opens in an analyzer that has no
  knowledge of fragcap, with every packet present and its timestamp, captured
  length, and original length matching the input.
- **SC-002**: The attribution for a packet is readable in that analyzer's
  packet comment display without configuration, a plugin, or documentation.
- **SC-003**: Every packet in a written capture carries a direction and a
  fidelity value, with no exceptions and no defaults applied by the reader.
- **SC-004**: An unattributed packet appears in the output, distinguishable
  from an attributed one, in 100 percent of cases.
- **SC-005**: Encoding and decoding an annotation returns the original value
  for every key in the section 13.3 table and for values containing each
  character the grammar reserves.
- **SC-006**: The number of packets an interface received, and the number lost
  at each of the four loss points fragcap can observe, are all recoverable from
  the file alone.
- **SC-007**: Writing the same fixture twice produces identical bytes, on every
  supported architecture.
- **SC-008**: The complete write path runs in the test suite with no capture
  driver, no elevated privilege, and no game running.

## Assumptions

- The fixture corpus committed by S04 is the input to every test in this slice.
  No new fixture is generated, and no fixture is modified.
- Attribution values reaching the writer are already valid UTF-8, because they
  arrive as Rust strings from the types S02 defined. The writer does not
  re-validate them.
- The annotation profile version declared in the Section Header Block is
  `0.1.0`, matching the specification revision that defines the profile. A
  later profile change bumps it.
- Snap length and link type are supplied by the caller declaring the interface.
  The writer does not infer them from packet contents.
- Section 12.4's counters reach the writer through the `CaptureStats` snapshot
  S02 defined, at finish time. The writer does not maintain its own counts of
  losses that occurred upstream of it.
- No dependency is added. pcapng block encoding is length-prefixed binary
  writing over a byte sink, which the standard library covers, and the
  workspace's single external dependency stays single.

## Out of Scope

- **JSON Lines output (section 13.5)**. That is S07. This slice makes the
  annotation derivation reusable so S07 does not restate the key rules, and
  stops there.
- **Statistics reporting (section 13.6)**. The Interface Statistics Block is in
  scope because section 13.2 places it in the file. Console and summary
  reporting are not.
- **The pipeline that drives the writer (sections 8.6, 12.4)**. That is S08.
  This slice provides a sink; nothing here buffers, counts, or fans out.
- **The session anchor (section 12.7)**. There is no session in this slice.
  Recorded as a known gap for S08, which owns capture start, rather than
  resolved by inventing a placement now.
- **Loopback direction resolution (section 12.6)**. The `dir=local` value is
  defined and encodable here, and nothing in this slice produces it, because
  resolving loopback direction requires the attributed process's endpoint. The
  encoder carries the value so the later slice supplies data rather than
  widening a grammar. Recorded as a known gap.
- **Live capture (sections 12.1, 12.2)**. That is S09. Interfaces are declared
  by the caller here.
- **Reading pcapng**. fragcap writes pcapng and reads classic pcap. A reader
  exists in the tests only far enough to validate structure, and is not a
  supported capability.
- **Compression, encryption, and custom blocks**. Section 13.1 rules them out
  on compatibility grounds and section 13.3 rules out custom options for want
  of a Private Enterprise Number.

## Done When

- [ ] `fragcap-sink` contains a pcapng writer implementing the core `Sink`
      trait, writing to any `std::io::Write` target.
- [ ] All four block types of section 13.2 are emitted, correctly aligned,
      length-consistent, and little-endian on every host.
- [ ] The annotation grammar of section 13.3 has an encoder and a decoder that
      round-trip, with percent-encoding covering the reserved characters and
      the control characters.
- [ ] Fidelity is written per section 13.4, with unattributed packets present
      and carrying no identity keys.
- [ ] Every section 12.4 counter is recoverable from the written file, with
      fragcap's own losses distinguishable from upstream losses.
- [ ] Golden files are committed for all eight corpus fixtures, reviewed, and
      compared in the test suite, with a drift check in the ordinary gate.
- [ ] Output is validated against the pcapng structure by a path independent of
      the writer.
- [ ] The full write path is exercised over the S04 fixture corpus at tier 1.
- [ ] `cargo xtask ci` is green, and `neutral` and `msrv` exit 0.
- [ ] A glossary entry exists for every term this slice introduces, per P-6.
- [ ] The section 12.7 gap, the percent-encoding widening, the `dir=unknown`
      value, and the independent treatment of `role` and `stage` are recorded
      as changelog decisions for promotion to specification section 29.
