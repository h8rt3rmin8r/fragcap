# Research: Header Parsing and Flow Keys

**Slice**: S03

**Created**: 2026-08-08

Findings behind the decisions in [plan.md](plan.md). Each entry records what
was chosen, why, and what else was evaluated. Wire format details are recorded
here rather than only in code comments, because a parser written from memory is
the most common way a subtly wrong offset gets in.

## R-1. Link layer encapsulations and their codes

**Decision**: Handle three codes. 1 is Ethernet, with a fourteen byte header
whose last two bytes are the EtherType. 101 is raw IP, with no link layer
header at all, so the first byte of the frame is the first byte of the network
header. 0 is BSD loopback encapsulation, with a four byte host-order address
family field ahead of the network header.

**Rationale**: These are the three the S02 `LinkType` constants name, and they
are what specification section 12.5's "Ethernet and raw IP link types" resolves
to against the shared libpcap and pcapng registry.

The registry is also what showed the S02 documentation to be wrong. Code 0 was
described there as having no link layer header, which is code 101's property.
The two are easy to confuse because both deliver an IP header near the start of
the frame, and a parser built on the confusion reads the IP version nibble out
of the low byte of an address family value, where it would find 2 for IPv4
traffic and reject every packet as an unsupported version. The failure would
have been silent, counted, and attributed to the wrong cause.

**Alternatives considered**: Handling only Ethernet, on the grounds that the
npcap loopback adapter presents a synthetic Ethernet header on Windows and the
focal titles are Windows-only. Rejected because the replay source in S04 reads
fixture files that may carry any of the three, and because a link type fragcap
declines is a counted rejection whose cause an operator then has to diagnose.

## R-2. Distinguishing the two byte orders of the loopback address family

**Decision**: Read the four bytes both as little-endian and as big-endian, and
accept the reading that matches a known address family value. Known values are
2 for IPv4, and 10, 23, 24, 28, and 30 for IPv6.

**Rationale**: The field is documented as host order, and a capture file
carries no record of the capturing host's order, so a reader on the opposite
order sees the value byte-swapped. Consumers of libpcap files handle this by
accepting both.

The set of IPv6 values is larger than one because the constant differs by
platform: 10 on Linux, 23 on Windows, 24 on FreeBSD, 28 on OpenBSD and macOS,
30 on other Darwin-derived systems. Accepting the union is what makes a fixture
recorded anywhere readable.

Ambiguity was checked rather than assumed. Every known value is small, so its
byte-swapped form is a large value with its low bytes zero, and no such value
is in the known set. A four byte field therefore has at most one valid reading,
so accepting both orders resolves rather than guesses.

**Alternatives considered**: Accepting native order only, which would silently
misparse any fixture recorded on the opposite order. Inferring order from the
following IP version nibble, which would work but makes the address family
field decorative and would misfire on a value fragcap does not recognize.

## R-3. IPv4 header fields the parser needs

**Decision**: Read the version and header length from the first byte, the
protocol and the fragment flags and offset from the fixed positions, and the
address pair from the last eight bytes of the fixed header. Skip options by the
declared header length rather than assuming twenty bytes.

**Rationale**: The header length field is in units of four octets and its
minimum legal value is five. A value below five is self-contradictory, because
the fixed header alone is twenty octets, and is the cleanest available signal of
a malformed header as distinct from a truncated one. A value whose implied
offset exceeds the captured bytes is the truncated case.

Assuming twenty bytes would be correct for the overwhelming majority of traffic
and wrong in a way that reads plausible header bytes as ports, which is worse
than rejecting.

**Corrected in review of pull request 6.** The first draft of this entry said
the total length is not used to bound reads at all, on the grounds that a
snapshot length legitimately makes the captured bytes shorter than it. That
reasoning is sound for one direction and was wrongly generalized to both.

The datagram extent is the smaller of the declared length and the captured
length. When the declared length is larger, the capture was truncated and the
captured length wins, which is the case the original reasoning covered. When it
is smaller, the excess is not the datagram at all: Ethernet pads every frame
below its sixty byte minimum, and some senders append trailing data.

The concrete failure that reasoning permitted: an IPv4 datagram declaring TCP
and carrying no TCP header, padded to sixty bytes, yielded a flow key with both
ports read from the padding. Both were zero, so the key looked plausible. That
is a fabricated observation, which constitution P-9 forbids outright, and it
would have attributed a packet to whatever process happened to hold port zero
semantics downstream.

A declared length of zero is a third case and is not an error. Large send
offload leaves the field for the network adapter to fill in, after the point
the capture is taken, so outbound traffic captured on the sending host reads
zero routinely. That is ordinary on the focal platform, where captures are
taken on the sending host by definition, and treating it as malformed would
discard real game traffic. The captured length is the only information
available, so it is used.

A non-zero declared length below the header's own length is self-contradictory
and is the malformed case.

## R-4. IPv6 extension header chain encodings

**Decision**: Walk the chain, handling five header types by their own length
encodings, and treat two values as terminal.

| Next header | Meaning | Advance |
| --- | --- | --- |
| 0 | Hop-by-hop options | `(len + 1) * 8` |
| 43 | Routing | `(len + 1) * 8` |
| 60 | Destination options | `(len + 1) * 8` |
| 44 | Fragment | fixed 8 |
| 51 | Authentication | `(len + 2) * 4` |
| 50 | Encapsulating security payload | terminal, no readable ports |
| 59 | No next header | terminal, legitimately no transport |

**Rationale**: The three option-style headers share an encoding whose length
byte counts eight-octet units excluding the first eight octets. The fragment
header is fixed at eight octets and carries the offset, flags, and a thirty two
bit identification. The authentication header is the odd one out: its length
byte counts four-octet units and excludes two of them, which is a different
formula and the single most likely place for an off-by-one.

The two terminal values are distinguished from each other in the output. An
encrypted payload is a packet whose ports exist but cannot be read, and is
counted as an unsupported transport. No-next-header is a well-formed packet
that legitimately has no transport at all, and is counted separately, because
conflating them would tell an operator they are missing traffic they are not.

**Alternatives considered**: Handling mobility, host identity protocol, and
shim6 headers as well. They share the option-style encoding and would be three
more match arms. Rejected as scope: specification section 12.5 does not name
them, and a packet carrying one falls through to a counted unsupported
transport rather than being misparsed, so the gap is visible if it ever
matters.

## R-5. Bounding the chain walk

**Decision**: Terminate after eight headers, and also terminate on any declared
length that would not advance the cursor. Both terminations advance the same
counter.

**Rationale**: The IPv6 standard places no limit on chain length, so a crafted
packet can carry an arbitrarily long chain, and a walk without a bound is a
denial of service against the capture thread rather than merely a parse bug.
Real traffic uses zero to two extension headers, so eight is generous by a wide
margin and no legitimate packet is lost to the bound.

The zero-advance guard is defence rather than a reachable path: given the
encodings in R-4, the smallest advance any of them can produce is eight octets
for the option headers and eight for the authentication header, so zero cannot
occur. It is kept because the safety property should hold whatever the
encodings turn out to be, and it shares the chain counter rather than getting
its own so that the statistics carry no counter that no frame can advance.

## R-6. The two reassembly keys are not the same

**Decision**: Fragment identity is the address pair, the protocol number, and
the sixteen bit identification for IPv4; and the address pair and the thirty
two bit identification for IPv6.

**Rationale**: The IPv4 standard defines the reassembly key as source,
destination, protocol, and identification, because identification is only
unique per protocol per address pair. The IPv6 standard defines it as source,
destination, and identification, and carries no protocol number in the fragment
header at all, because the identification is thirty two bits and per address
pair.

The first draft of the requirement used the IPv4 key for both, which would have
required inventing a protocol number for the IPv6 case. That is the same class
of fabrication as inventing a UDP remote endpoint: it would produce a key that
matches when it should and also, occasionally, when it should not. The
requirements checklist caught it and FR-022 now defines each separately.

**Alternatives considered**: Using the address pair and identification for both,
dropping the protocol number. Rejected because it collides across protocols in
IPv4, where the identifier is only sixteen bits and reuse is already the
limitation this slice accepts.

## R-7. Detecting a fragment

**Decision**: IPv4, a packet is a fragment when its more-fragments flag is set
or its fragment offset is non-zero. It is the initial fragment when its offset
is zero. IPv6, a packet is a fragment when the chain contains a fragment header,
and it is the initial fragment when that header's offset is zero.

**A non-initial fragment ends the chain walk.** Raised in review of pull
request 6. IPv6 splits a packet into an unfragmentable part, everything up to
and including the routing header, and a fragmentable part, everything after it.
Each fragment carries the unfragmentable part, then a fragment header, then a
chunk of the fragmentable part. The fragment header's next header field names
the first header of the fragmentable part *in the original packet*.

For the initial fragment that field does name what immediately follows, because
the chunk it carries is the start of the fragmentable part. For every other
fragment it does not: the chunk begins at an arbitrary offset and need not
contain that header at all. Walking on parses payload bytes as whatever the
field named, advances by a length read out of payload, and rejects a valid
fragment before its recorded identity is ever consulted. A destination options
or authentication header in that position is both legal and common, so this is
not a corner case.

The walk therefore returns as soon as it reads a fragment header with a
non-zero offset. Nothing is lost: a non-initial fragment carries no transport
header, and its ports come from the table.

**Rationale**: The two conditions differ because IPv4 signals fragmentation in
the fixed header of every fragment while IPv6 signals it in an optional
extension header, present only when the datagram was fragmented. An IPv4 packet
with offset zero and the more-fragments flag clear is not fragmented at all,
and must be parsed as an ordinary packet rather than recorded in the table,
otherwise the table fills with entries for datagrams that will never have a
second fragment.

The last fragment of a datagram is the one with a non-zero offset and the
more-fragments flag clear, and observing it is what removes the table entry.
This is what keeps entries from outliving their datagrams and is the main thing
narrowing the identifier reuse window.

## R-8. Measuring that parsing allocates nothing

**Decision**: A counting global allocator in a dedicated integration test
binary, backed by a thread-local counter with a const-initialized cell.

**Rationale**: A global allocator is installed per binary, so putting one in
the unit test build would count the test harness's allocations as well as the
parser's, and the measurement would be dominated by noise from other test
threads. A separate integration binary isolates it.

The counter is thread-local rather than a global atomic for the same reason:
the test harness runs tests concurrently by default, and a global counter would
be polluted by whatever else is running. A thread-local counter measures only
the allocations made on the thread doing the parsing, which is exactly the
claim being tested.

The cell must be const-initialized and must have no destructor. A thread-local
whose first access lazily initializes can itself allocate, which would make the
allocator reenter the counter it is trying to update.

Construction is outside the measured window on purpose. The address set
allocates once when it is built, and the requirement is that parsing allocates
nothing per packet, not that the parser can be built without ever touching the
heap.

**Alternatives considered**: Inspecting the generated code for calls into the
allocator, which is brittle across toolchain versions and unreadable as a test.
Asserting the types involved are all `Copy`, which is weaker: a `Copy` result
does not prove the code path that produced it allocated nothing.

## R-9. Representing the interface address set

**Decision**: A wrapper over a `Vec<IpAddr>` with a linear membership scan,
replaced wholesale rather than mutated.

**Rationale**: A capturing host has a handful of addresses, typically between
two and ten. A linear scan over that beats hashing, needs no hasher, and
allocates nothing on the lookup path. The allocation happens once when the set
is built.

Wholesale replacement rather than insert and remove is what makes FR-032's
requirement that no derivation of a previous set survives structural: there is
no incremental path for a stale entry to persist through. It also matches how
the platform reports the change, which is a notification that the set has
changed rather than a delta.

**Alternatives considered**: `HashSet<IpAddr>`, which is more code and slower at
this size. A sorted vector with binary search, which is faster in theory and
slower in practice below roughly sixteen elements, and adds an invariant to
maintain.
