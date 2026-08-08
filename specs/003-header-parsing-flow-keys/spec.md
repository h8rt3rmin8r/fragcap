# Feature Specification: Header Parsing and Flow Keys

**Feature Branch**: `feat/header-parsing-flow-keys`

**Created**: 2026-08-08

**Status**: Draft

**Slice**: S03 (specification sections 12.5, 12.6; constitution P-2, P-4, P-6,
P-9)

**Input**: Derive a `FlowKey` and a `Direction` from a captured frame's link,
network, and transport headers, without copying and without allocating.

## Overview

S02 fixed the shape of a flow key and left every `CapturedPacket` with
`flow: None` and `direction: None`. This slice is the one that fills them, and
it is the first slice in the project that computes anything.

The input is a byte slice and a link type. The output is, for a frame fragcap
understands, the identity of the conversation the frame belongs to and which
way it travelled. For a frame fragcap does not understand, the output is the
absence of both, together with a named counter saying which of a dozen distinct
reasons applied.

That second half is the part worth spending care on. A header parser is not
hard to write; a header parser that is honest about what it failed to parse is
a different exercise. Constitution P-4 requires a named counter per discard
cause, and P-9 requires that fragcap never quietly produce a tidier answer than
the observation supports. A parser satisfies both only if every path that ends
without a flow key ends in a distinct, surfaced counter, and if no path ever
guesses.

Two consequences of that shape the requirements below.

**Ambiguity is a result, not a failure.** Loopback traffic has a local source
and a local destination, so specification section 12.6's rule returns both
answers at once. The correct output is not a coin flip and not silence: it is a
flow key with `direction = None` and an ambiguity counter. Section 12.6
resolves that case from the attributed process's endpoint in a later slice,
which is only possible if this slice reports it rather than hiding it.

The case where the address set matches *neither* endpoint is a different
outcome, not a variant of the same one, and it produces no flow key at all.
Section 8.4 defines the key's local field as the endpoint on the capturing
host, and there is no such endpoint here. Its counter is separate because its
remedy is separate: an ambiguity means loopback, a missing local endpoint means
a stale address set or traffic that was never ours.

**Not reassembling is a commitment, not an omission.** Section 12.5 states that
fragcap attributes the first IP fragment from its transport header and
subsequent fragments by their fragment identifier and address pair, and that it
does not reassemble. Attributing a subsequent fragment therefore requires
remembering what the first fragment resolved to. That memory is bounded and it
evicts, and both the eviction and the unmatched fragment it may later cause are
named counters, because a fragment fragcap could not match is a flow key it
declined to invent rather than a packet it quietly mislabelled.

The audience is a contributor writing S04, S06, S08, or S09. The measure of
success is whether the pipeline can be assembled over fixtures without any of
those slices renegotiating what parsing returns.

## Clarifications

### Session 2026-08-08

- Q: Which crate owns header parsing? → A: `fragcap-core`, in a new `parse`
  module.
- Q: `LinkType::NULL` is documented in S02 as having no link layer header. Is
  that correct? → A: No. Code 0 is BSD loopback encapsulation, which carries a
  four byte address family field. Code 101 is the one with no link layer
  header. The S02 documentation is corrected in this slice.
- Q: Are VLAN tagged Ethernet frames parsed? → A: No. Specification section
  12.5 enumerates what fragcap parses and does not name them. They land in a
  named counter, which is what makes the gap visible if it ever fires.
- Q: How is a subsequent IP fragment attributed, given that fragcap does not
  reassemble? → A: Through a bounded fragment identity table populated by first
  fragments, with drop-oldest eviction and named counters for eviction and for
  unmatched fragments.
- Q: What does the parser return when the interface address set matches both
  endpoints? → A: No direction, plus an `ambiguous` counter distinct from the
  counter for matching neither endpoint.
- Q: Who owns the interface address set? → A: The caller. This slice consumes a
  set it is handed and never queries the platform for one, because querying is
  platform work and constitution P-2 keeps it out of core.
- Q: Is "allocation-free" asserted or merely intended? → A: Asserted, by a test
  running the parser under a counting global allocator.
- Q: Do parse outcome counters live in `CaptureStats` or in their own type?
  → A: Their own type, held by `CaptureStats` by value, matching how
  `SourceStats` is already held.
- Q: What rule assigns the local and remote positions when direction cannot be
  determined? → A: A flow key requires at least one endpoint in the interface
  address set. When both are in it, the loopback case, a canonical ordering
  picks the local position. When neither is, there is no flow key at all.
- Q: What are the two stated bounds, the fragment identity table's capacity and
  the extension header chain limit? → A: 256 entries and 8 headers.
- Q: Is the parser a free function or a value the caller owns? → A: A value the
  caller owns and calls through a mutable reference, holding the address set,
  the fragment table, and the counters.
- Q: What stops a stale fragment identity from producing a wrong attribution?
  → A: The identity includes the protocol number, and an entry is removed when
  its datagram's last fragment is seen. Identifier reuse within the table's
  lifetime remains possible and is recorded as a known limitation rather than
  claimed away.
- Q: Does `CapturedPacket` gain a field carrying the parse rejection cause?
  → A: No. The parse result carries it, the counters aggregate it, and the
  per-packet field is left to the slice that demonstrates it needs one.

All were resolved under the autopilot decision policy rather than escalated.
The rationale for each is carried into `plan.md`; the three with consequences
outside this slice are summarized here.

**Where parsing lives.** The alternatives were `fragcap-core` and
`fragcap-capture`. Section 12.5 says the capture thread parses, which reads
like an argument for the capture crate, but the capture thread belongs to the
pipeline, and specification section 8.2 places the pipeline in
`fragcap-core`. Putting the parser in `fragcap-capture` would require core to
depend on it, which inverts the one-directional dependency graph in section
8.3 and would fail the mechanical check that enforces it. Parsing is also pure
computation over a byte slice with no I/O and no platform surface, so it
carries nothing that P-2 excludes from core. Both the replay source in S04 and
the live source in S09 feed the same pipeline, so a single home is correct
regardless.

**The corrected link type documentation.** S02 documented code 0 as having no
link layer header. The shared libpcap and pcapng registry assigns that meaning
to code 101 and assigns code 0 to BSD loopback encapsulation, which prefixes a
four byte address family value. The error was harmless while nothing parsed,
and stops being harmless in exactly this slice. Correcting it here rather than
working around it is the P-9 answer: a comment that misdescribes an
encapsulation is a small lie that a later parser inherits.

**No local endpoint means no flow key.** Specification section 8.4 defines the
flow key's local field as the endpoint on the capturing host. When the
interface address set matches neither endpoint, no such endpoint exists, and
the three ways to respond are to put an arbitrary endpoint in the local
position, to key on wire order, or to produce no key.

Wire order was rejected outright: it produces two keys for one conversation and
destroys the normalization the key exists for. Between the other two, an
arbitrarily assigned local position is a field asserting something untrue about
the world, which is the same class of fabrication section 8.4 prohibits for UDP
remote endpoints, and it buys nothing: a packet with no local endpoint has no
local socket, so no socket table lookup could ever resolve it. The key would be
unusable as well as false.

Producing no key is therefore both the honest answer and the useful one. The
packet is still retained, still written with full fidelity, and the counter
fires once per packet, so a stale or empty address set announces itself loudly
instead of yielding a capture full of keys that resolve to nothing. The
loopback case is different and keeps its key, because there both endpoints
genuinely are local and the only open question is which one to write down.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A conversation gets an identity (Priority: P1)

A contributor assembling the pipeline over a fixture capture needs each frame
turned into the identity of the conversation it belongs to, so that the
attributor has something to look up and the writer has something to annotate.

**Why this priority**: Without it there is no attribution and no annotation.
Every slice downstream of S03 on the critical path is blocked on this one
capability.

**Independent Test**: Hand the parser a synthetic Ethernet frame carrying an
IPv4 TCP segment and confirm the returned flow key names the same protocol,
addresses, and ports the frame carries, with the capturing host's endpoint in
the local position.

**Acceptance Scenarios**:

1. **Given** an Ethernet frame carrying IPv4 TCP, **When** it is parsed against
   an address set containing the frame's source address, **Then** the result
   carries a flow key whose protocol is TCP, whose local endpoint is the source
   address and port, whose remote endpoint is the destination address and port,
   and whose direction is outbound.
2. **Given** the same frame parsed against an address set containing the
   frame's destination address instead, **Then** the local and remote endpoints
   are swapped relative to the wire order and the direction is inbound.
3. **Given** an Ethernet frame carrying IPv6 UDP, **When** it is parsed,
   **Then** the result carries a UDP flow key with the IPv6 endpoints.
4. **Given** a frame on a raw IP link type, with no link layer header at all,
   **When** it is parsed, **Then** the network header is read from the first
   byte and the same flow key is produced as for the equivalent Ethernet frame.
5. **Given** an IPv6 packet whose transport header sits behind a chain of
   extension headers, **When** it is parsed, **Then** the chain is walked and
   the transport ports are read from the correct offset.

---

### User Story 2 - A frame fragcap cannot parse says why (Priority: P1)

An operator sees a capture in which some packets carry no attribution. They
need to know whether that is one cause or several, and which, because the
remedy differs: an unexpected encapsulation, a protocol outside the supported
set, a truncated snapshot length, and a malformed header are four different
problems.

**Why this priority**: Constitution P-4 makes an uncounted discard path a
defect rather than an oversight, and P-9 makes a parser that quietly returns
nothing a worse failure than one that returns nothing loudly. This is also the
property that makes the parser debuggable at all: without it, the only symptom
of a parser bug is a lower attribution rate.

**Independent Test**: Construct one frame per rejection cause, parse each, and
confirm the corresponding named counter advanced and no other counter did.

**Acceptance Scenarios**:

1. **Given** an Ethernet frame carrying an EtherType fragcap does not parse,
   **When** it is parsed, **Then** no flow key is produced and the counter for
   an unsupported EtherType advances.
2. **Given** an IPv4 packet whose protocol is neither TCP nor UDP, **When** it
   is parsed, **Then** no flow key is produced and the counter for an
   unsupported transport protocol advances.
3. **Given** a frame truncated part way through its transport header by a
   snapshot length, **When** it is parsed, **Then** no flow key is produced and
   the counter for a short header advances, distinct from the malformed header
   counter.
4. **Given** an IPv4 header declaring a header length below the legal minimum,
   **When** it is parsed, **Then** no flow key is produced and the counter for
   a malformed network header advances.
5. **Given** a frame on a link type fragcap does not parse, **When** it is
   parsed, **Then** no flow key is produced and the counter for an unsupported
   link type advances.
6. **Given** any frame that produced no flow key, **When** it continues through
   the pipeline, **Then** it is retained and marked rather than dropped, and no
   drop counter advances.

---

### User Story 3 - Direction is honest about loopback (Priority: P1)

A contributor working on loopback capture needs to know that fragcap did not
silently pick a direction for a packet whose source and destination are both
local.

**Why this priority**: Specification section 12.6 states that loopback matches
both tests and is resolved from the attributed process's endpoint, which is a
later slice. A parser that resolves the ambiguity by preferring one branch of
its own conditional would produce a direction that is right half the time and
carries no indication of which half. That is precisely the class of quiet
distortion P-9 exists to prevent.

**Independent Test**: Parse a frame whose source and destination are both in
the interface address set and confirm no direction is returned and the
ambiguity counter advanced.

**Acceptance Scenarios**:

1. **Given** a frame whose source and destination are both in the interface
   address set, **When** it is parsed, **Then** a flow key is produced, no
   direction is produced, and the ambiguous direction counter advances.
2. **Given** a frame whose source and destination are both absent from the
   interface address set, **When** it is parsed, **Then** no flow key is
   produced, no direction is produced, and a counter distinct from the
   ambiguity counter advances.
3. **Given** an ambiguous frame, **When** its flow key is inspected, **Then**
   the local and remote positions are assigned by a documented, deterministic
   rule, so that both directions of one loopback conversation produce one key
   rather than two.
4. **Given** an interface address set that changes, **When** subsequent frames
   are parsed against the new set, **Then** direction reflects the new set,
   with no cached state from the old one.

---

### User Story 4 - Fragments attributed without reassembly (Priority: P2)

A contributor captures traffic containing IP fragments and needs the
non-initial fragments associated with the same conversation as the first,
without fragcap reassembling anything.

**Why this priority**: Specification section 12.5 requires it explicitly and
states why reassembly is refused: it would destroy the on-wire fidelity that
makes the capture worth taking. The capability is genuinely lower priority than
the three above, because the focal titles' traffic is predominantly
unfragmented, but the refusal to reassemble is not negotiable and the
alternative to this story is silently unattributed fragments.

**Independent Test**: Parse a first fragment carrying a UDP header, then a
subsequent fragment of the same datagram, and confirm both yield the same flow
key while the payload of neither is modified or joined.

**Acceptance Scenarios**:

1. **Given** an IPv4 first fragment carrying a complete transport header,
   **When** it is parsed, **Then** a flow key is produced normally and the
   fragment identity is remembered.
2. **Given** a subsequent IPv4 fragment of that datagram, **When** it is
   parsed, **Then** the same flow key is produced, derived from the remembered
   identity rather than from a transport header the fragment does not carry.
3. **Given** a subsequent fragment whose first fragment was never seen, **When**
   it is parsed, **Then** no flow key is produced and the unmatched fragment
   counter advances.
4. **Given** an IPv6 packet carrying a fragment extension header with a
   non-zero offset, **When** it is parsed, **Then** it is treated as a
   subsequent fragment by the same rule as IPv4.
5. **Given** more distinct fragmented datagrams in flight than the fragment
   identity table holds, **When** the table is full, **Then** the oldest entry
   is evicted, the eviction counter advances, and no packet is dropped.
6. **Given** any fragment, **When** it is written out, **Then** its bytes are
   exactly as captured, because nothing reassembled them.

---

### User Story 5 - Parsing costs nothing per packet (Priority: P2)

A contributor profiling the capture thread needs parsing to read fields in
place rather than copy the frame, and to allocate nothing per packet.

**Why this priority**: Specification section 12.5 states the requirement
directly. It is P2 rather than P1 because a correct parser that allocates is
still a correct parser, while an allocation-free parser that misparses is
worthless, so correctness is sequenced first. It is not P3 because this code
runs once per packet on the hot path and retrofitting the property later means
rewriting the module.

**Independent Test**: Run the parser over a corpus of frames under a counting
global allocator and confirm zero allocations occurred.

**Acceptance Scenarios**:

1. **Given** a corpus of frames covering every supported combination, **When**
   each is parsed under an allocation-counting allocator, **Then** the
   allocation count is zero.
2. **Given** a frame, **When** it is parsed, **Then** the frame's bytes are
   neither copied into an owned buffer nor modified.
3. **Given** the parse result, **When** it is inspected, **Then** it borrows
   nothing from the frame, so the caller may drop or forward the frame
   independently of the result.

### Edge Cases

- What happens when a frame is shorter than the link layer header it claims?
  No flow key, short header counter, packet retained.
- What happens when an IPv4 total length field exceeds the captured bytes?
  Parsing continues from the captured bytes only, because the declared length
  describes the wire and the capture may legitimately be truncated. Reading
  past the captured bytes is never attempted.
- What happens when an IPv4 header length field is legal but points past the
  end of the captured bytes? Short header counter, not malformed. A legal
  header length on a snapshotted frame is truncation, and calling it
  malformation would send an operator looking for a broken sender.
- What happens when an IPv6 extension header chain is longer than any
  legitimate packet's? The walk is bounded at eight headers and terminates, and
  the chain limit counter advances. An unbounded walk over attacker-controlled
  bytes on the capture thread is a denial of service against the capture, not
  merely a parse bug. The walk additionally refuses any declared length that
  would not advance the cursor, which the defined length encodings make
  unreachable but which is kept so that termination does not depend on them.
- What happens when an IPv6 chain ends in an encrypted payload, so there are no
  readable ports? No flow key, unsupported transport counter, packet retained.
- What happens when an IPv6 chain ends in the no-next-header value? No flow
  key, and it is counted separately from an unsupported transport, because it
  is a well-formed packet that legitimately has no transport rather than one
  fragcap declined to parse.
- What happens when a TCP or UDP header is present but truncated before both
  port fields? Short header counter. Ports are never inferred.
- What happens when a UDP header declares a length shorter than its own header?
  Malformed transport header counter. The ports are still physically present,
  but a header that contradicts itself is not a trustworthy source for them.
- What happens when the interface address set is empty, or is stale after an
  address change? No packet gets a flow key, and the counter for matching
  neither endpoint advances once per packet. That is deliberately loud: the
  alternative is a capture full of keys that no socket table lookup could ever
  resolve, which looks like working attribution that happens to find nothing.
- What happens when a frame's source and destination addresses are identical?
  It is a loopback case by the section 12.6 rule and is reported ambiguous.
- What happens to a promiscuously captured frame between two other hosts? It
  has no local endpoint, so it gets no flow key and is counted. The frame is
  still written out in full.
- What happens to the fragment identity table when a capture runs for hours?
  It is bounded by entry count, not by time, and evicts oldest first. Bounding
  by count rather than by age is what makes the memory ceiling a stated number
  rather than a function of traffic.

## Requirements *(mandatory)*

### Functional Requirements

Placement and shape.

- **FR-001**: Header parsing MUST live in `fragcap-core`, MUST add no
  dependency to that crate, and MUST leave the dependency direction check
  passing unchanged.
- **FR-002**: The parser MUST accept a byte slice and a link type and MUST NOT
  require, hold, or acquire any platform resource.
- **FR-003**: The parse result MUST own its contents rather than borrowing from
  the input slice, so the caller may forward or drop the frame independently.
- **FR-004**: Parsing MUST perform no heap allocation, and the slice MUST
  include a test that fails if it does.
- **FR-005**: Parsing MUST NOT modify the input bytes and MUST NOT copy the
  frame into an owned buffer.

Link layer.

- **FR-006**: The parser MUST handle the Ethernet link type, reading the
  EtherType field and dispatching on it to the IPv4 and IPv6 network parsers.
- **FR-007**: The parser MUST handle the raw IP link type, treating the first
  byte of the frame as the first byte of the network header and dispatching on
  the IP version field.
- **FR-008**: The parser MUST handle the BSD loopback link type, reading its
  four byte address family field. It MUST accept the field in either byte
  order, because the field is host ordered and a capture may be read on a host
  of the opposite order, and MUST accept the known address family values for
  IPv4 and IPv6.
- **FR-009**: The parser MUST NOT parse VLAN tagged Ethernet frames in this
  slice. Such a frame MUST yield no flow key and MUST advance the unsupported
  EtherType counter.
- **FR-010**: Any other link type MUST yield no flow key and MUST advance the
  unsupported link type counter.
- **FR-011**: The documentation on the link type constant for code 0 MUST be
  corrected to describe BSD loopback encapsulation, and the constant for code
  101 MUST be the one described as carrying no link layer header.

Network layer.

- **FR-012**: The parser MUST handle IPv4, validating the version and header
  length fields, skipping options by the declared header length, and reading
  the protocol and address fields.
- **FR-013**: The parser MUST handle IPv6, reading the fixed header's next
  header and address fields.
- **FR-014**: The parser MUST walk the IPv6 extension header chain to reach the
  transport header, handling at minimum hop-by-hop options, routing,
  destination options, fragment, and authentication headers, each advanced by
  its own length encoding.
- **FR-015**: The extension header walk MUST be bounded at eight headers and
  MUST terminate on any chain that would exceed that, advancing a named
  counter. It MUST also terminate on a declared header length that would not
  advance the cursor.
- **FR-016**: An IPv6 chain terminating in the no-next-header value MUST yield
  no flow key and MUST advance a counter distinct from the unsupported
  transport counter.
- **FR-017**: A network header that is malformed, meaning its own fields
  contradict each other, MUST yield no flow key and MUST advance the malformed
  network header counter. A header whose fields are internally legal but whose
  declared extent exceeds the captured bytes is truncated rather than
  malformed, and MUST advance the short header counter instead. The distinction
  is that the first indicates a broken sender or a parser bug and the second
  indicates a snapshot length, which are different remedies.

Transport layer.

- **FR-018**: The parser MUST read the source and destination ports from TCP
  and UDP headers, and MUST require both port fields to be present in the
  captured bytes before producing a flow key.
- **FR-019**: A transport protocol other than TCP or UDP MUST yield no flow key
  and MUST advance the unsupported transport counter.
- **FR-020**: The parser MUST NOT infer, default, or zero a port that is not
  physically present in the captured bytes.

Fragments.

- **FR-021**: An initial fragment, meaning a packet that is a fragment by
  FR-026 and whose fragment offset is zero, MUST be parsed normally from its
  transport header, and its fragment identity MUST be recorded. A packet that
  is not a fragment at all MUST NOT be recorded, even though its offset is also
  zero; recording unfragmented traffic would fill the table with entries no
  second fragment will ever match and evict the entries that matter.
- **FR-021a**: An initial fragment whose transport header cannot be parsed MUST
  NOT record an identity, because there is no flow key to record. Its
  subsequent fragments are consequently unmatched and counted as such, which is
  the correct outcome: the datagram's ports were never observed.
- **FR-022**: A non-initial fragment MUST be resolved to the flow key recorded
  for its fragment identity. For IPv4, fragment identity is the address pair,
  the protocol number, and the sixteen bit identification field, matching the
  reassembly key the IPv4 standard defines; omitting the protocol number would
  collide across protocols. For IPv6, it is the address pair and the thirty two
  bit identification field from the fragment extension header, matching the
  reassembly key the IPv6 standard defines, which carries no protocol number.
- **FR-022a**: A non-initial fragment's direction MUST be determined from its
  own addresses by the ordinary rule, not inherited from the recorded entry,
  because every fragment carries the full address pair and the interface
  address set may have changed since the first fragment was seen.
- **FR-023**: A non-initial fragment whose identity is not recorded MUST yield
  no flow key and MUST advance the unmatched fragment counter.
- **FR-024**: The fragment identity table MUST hold at most 256 entries, MUST
  evict oldest first when full, and MUST advance a named counter on each
  eviction.
- **FR-024a**: An entry MUST be removed when the last fragment of its datagram
  is observed, meaning a fragment with a non-zero offset and no more-fragments
  flag, so that an entry does not outlive the datagram it describes.
- **FR-025**: The parser MUST NOT reassemble fragments, MUST NOT join payloads,
  and MUST NOT alter any fragment's bytes.
- **FR-026**: An IPv4 packet is a fragment when its more-fragments flag is set
  or its fragment offset is non-zero. An IPv6 packet is a fragment when its
  extension header chain contains a fragment header. Both MUST be recognized.

Direction.

- **FR-027**: The parser MUST accept an interface address set supplied by the
  caller and MUST NOT query the platform for one.
- **FR-028**: A packet whose source address is in the set and whose destination
  address is not MUST be outbound. A packet whose destination is in the set and
  whose source is not MUST be inbound.
- **FR-029**: A packet whose source and destination are both in the set MUST
  yield a flow key with no direction and MUST advance an ambiguous direction
  counter. The assignment of its endpoints to the local and remote positions
  MUST follow a documented deterministic rule, so that both halves of one
  loopback conversation produce one key rather than two.
- **FR-030**: A packet whose source and destination are both absent from the
  set MUST yield no flow key and no direction, and MUST advance a counter
  distinct from the ambiguous one. The parser MUST NOT place an endpoint that
  is not on the capturing host into the local position of a flow key.
- **FR-031**: The parser MUST be a value the caller owns, holding the address
  set, the fragment identity table, and the counters. It MUST NOT require
  interior mutability or locking, because the capture thread owns exactly one
  and does not share it.
- **FR-032**: The interface address set MUST be replaceable by the caller
  between packets through a single operation, and the parser MUST hold no
  cached derivation of a previous set.

Counters.

- **FR-033**: Every path that ends without a flow key MUST advance exactly one
  named counter, and the causes MUST be separately named rather than
  aggregated.
- **FR-034**: Parse counters MUST live in their own type, held by value inside
  the capture statistics type, matching how backend statistics are already
  held.
- **FR-035**: Any total exposed over the parse counters MUST be derived from
  the named counters rather than stored separately.
- **FR-036**: No parse outcome may cause a packet to be dropped. A packet with
  no flow key MUST be representable as retained and marked, and no drop counter
  may advance for a parse reason.
- **FR-036a**: The parse result MUST carry the rejection cause for the caller
  that wants it. The captured packet type MUST NOT gain a field storing that
  cause, because the architecture of record does not have one and the counters
  answer the diagnostic question without a per-packet cost.

Hygiene.

- **FR-037**: Every term this slice introduces MUST have a glossary entry in
  `docs/glossary.md` in this same change, per P-6.
- **FR-038**: Every public item MUST carry documentation stating what it
  represents, and any item whose behavior a later slice completes MUST name
  that slice.
- **FR-039**: Any divergence from the architecture of record discovered here
  MUST be recorded in the slice for promotion to specification section 29.

### Key Entities

- **Parse outcome**: What the parser concluded about one frame. Either a flow
  key with an optional direction, or a named reason no flow key was produced.
  Never silence.
- **Parse rejection cause**: The specific reason a frame produced no flow key.
  One per counter, and the set is closed and enumerated.
- **Interface address set**: The addresses belonging to the capturing host,
  supplied by the caller, against which a packet's endpoints are tested to
  determine direction.
- **Fragment identity**: What associates the non-initial fragments of one
  datagram with its first. Defined by the reassembly key of the address family
  in question, which differs between the two: see FR-022.
- **Fragment identity table**: The bounded, evicting memory from a fragment
  identity to the protocol and port pair the first fragment carried, which lets
  a non-initial fragment be attributed without reassembly. It stores the ports
  rather than an assembled flow key, so that direction and the local position
  are recomputed for every fragment as FR-022a requires.
- **Parse statistics**: One named counter per parse rejection cause and per
  undetermined direction cause, carried alongside the existing capture
  counters rather than folded into them.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Every combination of supported link type, network protocol, and
  transport protocol produces the correct flow key, verified by a test per
  combination with no combination untested.
- **SC-002**: Every enumerated parse rejection cause is reachable by a
  constructed frame, and a test asserts that exactly the corresponding counter
  advanced.
- **SC-003**: Parsing a corpus covering every supported combination performs
  zero heap allocations, measured under a counting allocator.
- **SC-004**: A loopback frame yields a flow key, no direction, and an
  ambiguity count. A frame matching neither endpoint yields no flow key and a
  different count. A test asserts the two outcomes differ in both respects.
- **SC-005**: Both directions of one conversation, and both directions of one
  loopback conversation, produce a single flow key, verified by inserting both
  into a map and asserting one entry.
- **SC-006**: A subsequent IP fragment resolves to its first fragment's flow
  key, and an orphaned subsequent fragment resolves to none and is counted.
- **SC-007**: An adversarial IPv6 extension header chain terminates the walk
  within the stated bound, verified by a test whose failure mode would
  otherwise be a hang.
- **SC-008**: No packet is dropped for any parse reason, verified by asserting
  the drop counters are unchanged across the full rejection corpus.
- **SC-009**: `fragcap-core` still builds for a target with no capture backend,
  and the dependency direction check still passes.
- **SC-010**: Every term introduced has a glossary entry, verified by reading
  the change's new public items against `docs/glossary.md`.
- **SC-011**: The full local gate set passes: format, lint, tests, repository
  conventions, dependency direction, and per-crate licensing.

## Assumptions

- Specification sections 12.5 and 12.6 are the architecture of record and are
  implemented rather than redesigned. Where they are silent, this slice decides
  and records the decision for promotion to section 29.
- The interface address set is a set of addresses without ports. Section 12.6
  matches on address alone, and the port-based loopback resolution it describes
  is a later slice's work over the attributed endpoint.
- Refreshing the address set on change notifications is platform work owned by
  S09 and S13. This slice provides the seam that a refresh writes into, and
  requires only that replacement is possible between packets.
- The focal titles' traffic is predominantly unfragmented, per the
  reconnaissance findings. Fragment handling is required for correctness rather
  than for throughput, which is why the fragment table is small and bounded
  rather than tuned.
- Frames are parsed one at a time with no cross-frame state other than the
  fragment identity table. Any future cross-frame inference is a different
  slice with a different justification.
- The reconnaissance gate is closed, so nothing here waits on Q-1 through Q-6.

### Known limitation: fragment identifier reuse

The IPv4 fragment identifier is sixteen bits. A host that fragments heavily can
reuse an identifier for the same address pair and protocol before the earlier
entry has been removed, and a subsequent fragment of the new datagram would
then resolve to the earlier datagram's flow key, meaning the wrong ports and
possibly the wrong process.

Removing an entry when its datagram's last fragment is observed shortens the
window substantially, and the 256 entry bound caps how long an unfinished entry
can survive. Neither eliminates the case, and it is not detectable from the
capture, so it cannot be counted. It is therefore stated here rather than
claimed away, and recorded for promotion to specification section 29.

An expiry timer would narrow it further and is rejected in this slice: a clock
in `fragcap-core` is a platform surface that constitution P-2 excludes, and the
packet timestamp cannot substitute, because a replay source's timestamps run at
whatever rate the fixture was recorded at.

## Out of Scope

- Any packet acquisition, live or replayed. S04 and S09.
- Any attribution or socket table access. S10. This slice produces the key that
  attribution looks up; it performs no lookup.
- Resolving loopback direction from the attributed process's endpoint. That
  requires an attribution, so it follows S10 and belongs with the loopback work
  in S13.
- Querying, enumerating, or watching interface addresses. S09 and S13.
- IP reassembly, in this or any slice. Refused by section 12.5.
- VLAN tagged frames, tunnelling protocols, and encapsulations beyond the three
  link types named above.
- Protocol dissection beyond the transport ports. The dissector seam stays
  empty.
- The bounded ring, drop accounting, and the pipeline that calls this parser.
  S08. This slice defines the parse counters; S08 wires them into a running
  capture.

## Done When

- Every requirement above is satisfied and traceable to a test or a check.
- The full local gate set passes in the foreground, watched to completion.
- The glossary carries an entry for every term introduced.
- Deviations from the architecture of record are recorded in the slice for
  promotion to specification section 29.
- A changelog fragment exists describing the change.
