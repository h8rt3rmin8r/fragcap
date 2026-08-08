# Research: pcapng Writer and Annotation Encoding

**Slice**: S06

**Created**: 2026-08-08

**Feature**: [spec.md](spec.md)

This document resolves the unknowns the plan's technical context raised, in the
form the plan template asks for: decision, rationale, alternatives considered.
Every decision here is a consequence of the specification, the constitution, or
a property the slice has to hold; none is a preference.

## R-1: The pcapng structures this slice writes

**Decision**: Four block types and the option codes below, all values written
little-endian, all lengths in bytes.

Common block framing, which every block shares:

| Field | Width | Meaning |
| --- | --- | --- |
| Block type | 4 | Identifies the block |
| Block total length | 4 | Whole block including both length fields |
| Body | variable | Type-specific, padded to a 32-bit boundary |
| Block total length | 4 | Repeated, equal to the leading value |

Block types:

| Block | Type value |
| --- | --- |
| Section Header | `0x0A0D0D0A` |
| Interface Description | `0x00000001` |
| Interface Statistics | `0x00000005` |
| Enhanced Packet | `0x00000006` |

Option framing, which every option shares: a 2-byte code, a 2-byte length
carrying the value length before padding, the value, then padding to a 32-bit
boundary. An option list ends with `opt_endofopt`, code 0, length 0.

| Option | Code | Block | Value |
| --- | --- | --- | --- |
| `opt_endofopt` | 0 | all | empty |
| `opt_comment` | 1 | all | UTF-8 text |
| `shb_userappl` | 4 | Section Header | UTF-8 text |
| `if_name` | 2 | Interface Description | UTF-8 text |
| `if_tsresol` | 9 | Interface Description | 1 byte |
| `isb_ifrecv` | 4 | Interface Statistics | u64 |
| `isb_ifdrop` | 5 | Interface Statistics | u64 |
| `isb_osdrop` | 7 | Interface Statistics | u64 |

Block bodies:

- **Section Header**: byte-order magic `0x1A2B3C4D`, major version 1, minor
  version 0, section length as a signed 64-bit value, then options. Section
  length is written as -1, meaning unspecified, because the writer streams and
  does not know the total when it writes the header.
- **Interface Description**: link type as u16, a reserved u16 written as zero,
  snap length as u32, then options.
- **Enhanced Packet**: interface identifier u32, timestamp upper 32 bits,
  timestamp lower 32 bits, captured length u32, original length u32, packet
  data padded to a 32-bit boundary, then options.
- **Interface Statistics**: interface identifier u32, timestamp upper 32 bits,
  timestamp lower 32 bits, then options.

**Rationale**: These are the format's own definitions, not choices. They are
recorded here because the plan has to name them somewhere a reviewer can check,
and because the encoder's correctness is entirely a matter of getting them
right.

**Alternatives considered**: None. The format is the format.

**How this is verified rather than trusted**: See R-6. Reciting structures from
a specification is exactly the sort of claim that reads as authoritative and
turns out to be one option code off, so the slice does not rest on this table
being right. It rests on a reader that has never seen fragcap accepting the
output.

**Verified, 2026-08-08, before the plan committed to it.** A 384-byte probe
file was assembled by hand from exactly the table above, containing all four
block types, and read with Wireshark 4.6.3 tooling that has no knowledge of
this project. Every structure resolved:

| Claim | Evidence from `capinfos` and `tshark` |
| --- | --- |
| Block framing and lengths | File parsed, 1 packet, 384 bytes, no error |
| Section Header, `shb_userappl` 4 | `Capture application: fragcap/0.1.0` |
| Section Header, `opt_comment` 1 | `Capture comment: fragcap:profile=0.1.0` |
| Interface Description, `if_name` 2 | `Name = probe0` |
| Interface Description, `if_tsresol` 9, value 6 | `Time precision = microseconds (6)`, `Time ticks per second = 1000000` |
| Interface Description, link type and snaplen | `Encapsulation = Ethernet (1 - ether)`, `Capture length = 65535` |
| Enhanced Packet, lengths and timestamp | `60 bytes on wire, 60 bytes captured`, epoch `1754650000.000000` |
| Enhanced Packet, `opt_comment` 1 | `fragcap:pid=7412;proc=eso64.exe;dir=out;attr=live` |
| Interface Statistics block parsed | `Number of stat entries = 1` |

The annotation string came back byte for byte, which is section 13.3 and
constitution P-5 demonstrated rather than asserted, on the reader the claim is
about. The probe is a throwaway and is not committed; what it bought was
certainty that the implementation phase starts from a correct table rather than
discovering an option code by debugging.

## R-2: Timestamp encoding under a declared resolution

**Decision**: Declare `if_tsresol` as 6, meaning microseconds. Convert the
core nanosecond timestamp by flooring toward negative infinity. Split the
resulting unsigned 64-bit microsecond count into an upper and a lower 32-bit
half. Refuse a value that predates the Unix epoch.

**Rationale**: Specification section 12.7 requires microsecond resolution in
the file and requires the interface to declare the resolution it carries, so
the file never overstates its own precision. The core `Timestamp` type's
documentation names this slice as the single place the narrowing happens,
specifically so that P-9 compliance has one site to inspect rather than one per
call site; honoring that is what keeps the property true.

Flooring rather than truncating toward zero matters only for pre-epoch values,
which this slice refuses outright, but it is the correct rule regardless
because it preserves ordering, and a conversion that can reorder two
observations is a conversion that alters the record.

**Alternatives considered**:

- **Declare nanosecond resolution (`if_tsresol` 9) and lose nothing.** The
  format permits it and every modern reader supports it. Rejected because
  section 12.7 fixes microseconds, and the specification is the architecture of
  record. If nanosecond resolution is wanted, that is a specification change
  with a recorded decision, not a writer making its own call.
- **Clamp a pre-epoch timestamp to zero.** Rejected under P-9: it records an
  observation at a time it did not happen, and a reader cannot tell that it
  was clamped.
- **Wrap a pre-epoch timestamp into the unsigned range.** Rejected for the same
  reason and more strongly, since it places the observation in the year 586524
  rather than merely early.

## R-3: Where the annotation is built, and by what type

**Decision**: An `Annotation` value in `fragcap-sink`, derived from a
`CapturedPacket` by one function and rendered to the section 13.3 grammar by
another. A decoder parses the grammar back into the same value.

**Rationale**: FR-025 requires the derivation be reusable by S07, whose JSON
Lines output carries the same facts in a different syntax. Two independent
derivations of "which keys are present" would drift, and the drift would be
silent, because each would be internally consistent. Separating derivation from
rendering makes the shared part shareable.

The decoder exists because a grammar with one producer and no consumer has
never been tested. FR-024 requires the round trip, and a round trip through two
code paths written from the same table is a real check; a round trip through
one function is a tautology.

**Alternatives considered**:

- **Format the string inline at the point of writing the packet block.**
  Rejected: it makes the key-presence rules unreachable from S07 and untestable
  without constructing a whole pcapng file to inspect one comment.
- **Put the annotation type in `fragcap-core`.** Tempting, since both S06 and
  S07 need it and core is where shared vocabulary lives. Rejected for this
  slice because the type is an output-format concern and core is the crate the
  constitution keeps narrow. If S07 finds it genuinely needs the type rather
  than the rules, promoting it is a small change made with two consumers in
  view rather than one imagined.

## R-4: Determinism, and the inputs that threaten it

**Decision**: The writer reads no clock, no environment, no locale, and no
host property. Every byte it emits is a function of the packets, the interface
declarations, and the statistics snapshot it was given.

Four specific inputs are fixed because each would otherwise vary:

| Input | Fixed as |
| --- | --- |
| Byte order | Little-endian, regardless of host |
| Interface Statistics Block timestamp | Last packet written on that interface, or zero |
| Annotation key order | The section 13.3 table order |
| Percent-encoding hexadecimal case | Uppercase on output, either accepted on input |

**Rationale**: FR-037 requires byte-identical output and FR-038 pins it with
committed goldens. A golden comparison is worth exactly as much as the
determinism behind it, and each of these four is individually sufficient to
break it. The Interface Statistics Block timestamp is the one worth naming
twice: the block header carries a timestamp field that has to hold something,
the obvious something is the current time, and a writer that reads the clock
there produces goldens that pass on the first run and fail forever after. The
likely response to that failure is deleting the goldens, which removes the only
check that reaches outside this codebase's own assumptions.

S04 met the same class of problem from the reading side and answered it the
same way, by naming the ambient inputs rather than instructing the implementer
to be deterministic.

**Alternatives considered**:

- **Write in host byte order, which pcapng permits and declares.** Rejected:
  valid output, useless goldens, and a capture whose bytes depend on the
  machine that took it.
- **Use the real time in the Interface Statistics Block and exclude that field
  from golden comparison.** Rejected: an exclusion list is a place for the next
  nondeterministic field to hide, and the field has a meaningful data-derived
  value available at no cost.

## R-5: Carrying fragcap's own loss counters

**Decision**: `isb_ifrecv`, `isb_ifdrop`, and `isb_osdrop` carry the upstream
counters they are defined to carry. fragcap's own `buffer_dropped` and
`sink_dropped` go in an `opt_comment` on the same block, under the `fragcap:`
sentinel, in the same grammar as every other annotation.

**Rationale**: Section 13.2 says the Interface Statistics Block is populated
from the section 12.4 counters, and section 12.4 has counters that pcapng has
no field for. Writing only the three that fit satisfies section 13.2 as
written and violates P-4, which makes an uncounted, unsurfaced discard a
defect. Between a specification sentence and a constitution principle, the
constitution wins, and in this case both can be satisfied at once.

Overloading `isb_osdrop` with fragcap's losses was considered and rejected
under P-9: that field means the operating system dropped the packet, and a
fragcap buffer drop reported there is a false statement about where the loss
happened, which a reader has no way to detect or correct.

**Alternatives considered**:

- **A pcapng custom option.** Rejected on the same grounds section 13.3
  rejected it for the packet annotation: custom options require a Private
  Enterprise Number this project does not hold.
- **Report the counters only in the console summary.** Rejected: the file
  outlives the run, and a capture that is quietly short is the defect class P-4
  exists to prevent.

## R-6: Verifying the output against a reader that never heard of fragcap

**Decision**: Two independent checks, at different costs.

The mandatory one, which runs in the ordinary gate on every machine, is a
structural validator in the test suite: it walks the file by its declared
block lengths, confirms the walk consumes the file exactly, confirms leading
and trailing lengths agree, confirms option alignment and termination, and
confirms every packet block references an interface that was declared earlier.
It is written against the format, not against the writer's encoding functions,
so it cannot inherit an encoding mistake.

The stronger one, which is documented in the quickstart and not wired into the
gate, is tshark. Wireshark 4.6.3 is present on the development machine, reads
the file with no configuration, and displays the annotation in the packet
comment column. This is the actual claim of section 13.1 and constitution P-5,
tested by the actual population it claims to serve.

**Rationale**: A writer verified only by its own reader has proven that two
functions agree, which is not the property section 13.1 promises. The
structural validator is the strongest check that can run everywhere, and it is
genuinely independent. tshark is stronger still but cannot be a gate: the
continuous integration runners are not guaranteed to have Wireshark installed,
and the constitution is explicit that a check which did not run must never look
like one that passed. Making it a documented manual step keeps that honest.

**Alternatives considered**:

- **Add Wireshark to the continuous integration image and gate on tshark.**
  Rejected for this slice. It changes a pinned workflow artifact, adds an
  install step to every run for one slice's benefit, and the structural
  validator already catches the defects a format writer actually produces. Left
  as a recorded option for S18, which owns analyzer integration and has other
  reasons to want tshark present.
- **Vendor a pcapng reader crate to validate with.** Rejected: it adds a
  dependency to validate output the project could validate itself, and a reader
  crate that shares a lineage with the writer's assumptions is not the
  independent check it appears to be.

## R-7: Dependencies

**Decision**: None added. The workspace stays at one external dependency.

**Rationale**: Writing pcapng is length-prefixed binary output over a byte
sink. The standard library covers `u16`, `u32`, `u64`, and `i64`
little-endian conversion through `to_le_bytes`, and covers everything else
through `std::io::Write`. Percent-encoding of a seven-key grammar over a known
character set is a short loop, and pulling in a general-purpose encoder would
mean adopting its escaping tables and its idea of which characters are
reserved, neither of which matches section 13.3.

**Alternatives considered**: A pcapng crate. Rejected: the writer is the
product of this slice, the format decisions are specification decisions rather
than library ones, and a crate that made a different choice about section
length, byte order, or comment encoding would have to be fought rather than
used.
