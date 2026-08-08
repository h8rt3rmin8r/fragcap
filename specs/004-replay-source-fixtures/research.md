# Research: Replay Source and Fixture Corpus

**Slice**: S04

**Created**: 2026-08-08

Findings behind the decisions in [plan.md](plan.md). Wire format details are
recorded here rather than only in code comments, for the same reason S03
recorded the header layouts: a reader written from memory is how a subtly wrong
offset gets in, and a fixture generator written from the same memory produces
files that agree with the bug.

## R-1. The classic pcap file layout

**Decision**: A twenty-four byte file header, then a sequence of records, each
a sixteen byte record header followed by exactly `caplen` bytes of packet data.

File header fields, in order: a four byte magic number, two two-byte version
numbers, a four byte timezone offset, a four byte timestamp accuracy, a four
byte snapshot length, and a four byte link type.

Record header fields, in order: a four byte timestamp seconds, a four byte
timestamp fraction, a four byte captured length, and a four byte original
on-wire length.

**Rationale**: This is a far smaller format than the pcapng fragcap writes, and
that asymmetry is deliberate rather than accidental. Fixtures need to be
readable by ordinary tooling and writable by a generator with no dependencies;
the output format needs to carry attribution. Section 25.3 chose the simple one
for the corpus, and nothing here needs the other.

The timezone and accuracy fields are vestigial. Every writer sets them to zero
and every reader ignores them, and this reader does the same. Recorded because
a reader that validated them would reject files everything else accepts.

## R-2. Four magic numbers, two independent choices

**Decision**: Recognize all four, deriving byte order and timestamp resolution
independently.

| Magic as written | Read as native | Byte order | Fraction unit |
| --- | --- | --- | --- |
| `a1 b2 c3 d4` | `0xa1b2c3d4` | same | microseconds |
| `d4 c3 b2 a1` | `0xa1b2c3d4` swapped | swapped | microseconds |
| `a1 b2 3c 4d` | `0xa1b23c4d` | same | nanoseconds |
| `4d 3c b2 a1` | `0xa1b23c4d` swapped | swapped | nanoseconds |

**Rationale**: The magic is the only thing in the file that says how to read the
rest of it, which is why the reader must not consult the host's own byte order
at any point. A file written on a big-endian host and read on a little-endian
one must yield the same packets, and SC-003 asserts it by generating the same
fixture four ways.

The nanosecond variant is not exotic. It is what a modern capture writes when
asked for full resolution, and a reader that assumed microseconds would silently
report timestamps a thousand times too small. Silently, because the values stay
plausible.

**Alternatives considered**: recognizing only the two native-order magics, on
the grounds that the generator controls the corpus. Rejected because it would
make the corpus the only thing the reader can read, and the point of using a
standard format is that a contributor can drop in a capture from elsewhere.

## R-3. Timestamps and the resolution conversion

**Decision**: Convert to the core timestamp type at read time, using the unit
the magic declares, and never round.

The core timestamp is nanoseconds since the Unix epoch, which S02 chose as
finer than any backend supplies. Converting a microsecond fraction inward is a
multiplication by one thousand and is lossless. Converting a nanosecond
fraction is the identity.

**Rationale**: This is the direction S02's decision D-2 anticipated: the single
lossy conversion happens at the output boundary in S06, and everything inward
of it is exact. A reader that normalized both files to microseconds would throw
away three digits of a nanosecond capture at the point of reading it, which is
the alteration P-9 forbids and which no later stage could recover.

## R-4. What the snapshot length means, and does not

**Decision**: Read the file's declared snapshot length, compare each record's
captured length against it, count a record that exceeds it, and deliver the
record anyway.

**Rationale**: The field declares the limit the capture was taken under. A
record longer than it means the file contradicts itself, which is worth
counting, but the bytes are present and real. Discarding them would lose an
observation on the strength of a header field being wrong, which is the wrong
way round: the observation is the data, and the field is metadata about it.

The reader also does not use the snapshot length to bound reads. The record's
own captured length does that, and the file's length bounds the record. This is
the same lesson S03 learned about the IPv4 total length, arrived at before the
code rather than after it.

## R-5. Which addresses a fixture may contain

**Decision**: Four documentation ranges plus loopback.

| Range | Reserved by | Use here |
| --- | --- | --- |
| `192.0.2.0/24` | RFC 5737 | The capturing host in most fixtures |
| `198.51.100.0/24` | RFC 5737 | The peer |
| `203.0.113.0/24` | RFC 5737 | A third party, for the no-local-endpoint case |
| `2001:db8::/32` | RFC 3849 | Both endpoints of the IPv6 fixture |
| `127.0.0.0/8`, `::1` | RFC 1122, RFC 4291 | The loopback fixture |

Link layer addresses come from the locally administered range, which is any
address with the second-least-significant bit of the first octet set, so they
cannot collide with a real manufacturer's assignment.

**Rationale**: The documentation ranges exist for exactly this: addresses that
appear in published material and can never route to anyone. Using anything else
in a public repository risks naming a real host.

Loopback needed adding explicitly, and the first draft of the spec omitted it.
That draft forbade every address outside the documentation ranges while also
requiring a loopback fixture, so it prohibited a file it mandated. A loopback
address identifies no operator, which is why the rule can admit it without
weakening.

## R-6. Making the privacy rule mechanical

**Decision**: Require fixture payloads to be a documented repeating filler
pattern, and assert that, rather than trying to detect sensitive content.

**Rationale**: "Contains no account identifier or session token" cannot be
asserted. No test recognizes what a session token looks like, and one that
tried would be a source of false confidence: a fixture could pass the check and
still carry something real.

Inverting it is both checkable and stronger. If every payload byte must be the
filler pattern, then anything that is not the filler fails, including content
nobody thought to look for. The rule stops depending on anticipating what might
leak.

This also makes the fixtures more readable: a reviewer scanning a hex dump sees
an obvious pattern and can tell at a glance that no payload carries meaning.

## R-7. What each fixture must contain to exercise its condition

**Decision**: Each fixture's stated condition is turned into an assertion that
runs beside the generator.

| Fixture | Condition, from section 25.3 | Asserted by |
| --- | --- | --- |
| `tcp-session` | Ordinary TCP flow, both directions | Both directions resolve to one flow key |
| `udp-gameplay` | Sustained UDP at gameplay cadence | UDP flow keys, and inter-packet gaps within a stated band |
| `ipv6-mixed` | IPv6 with extension header chains | At least one packet's transport offset is past the fixed header |
| `fragmented` | IP fragmentation, first and subsequent | Both an initial and a non-initial fragment are present |
| `loopback` | Local traffic, direction ambiguity | At least one packet yields a key with no direction |
| `malformed` | Truncated and invalid headers | More than one distinct parse rejection cause is reached |
| `port-reuse` | Same port, different processes, over time | Its script resolves one local endpoint to two owners in two windows |
| `burst` | Sustained rate exceeding buffer capacity | A stated packet count within a stated span |

**No fixture is a damaged pcap.** `malformed.pcap` is named for what its
packets contain, not for what its file structure is: the records are
well-formed and the headers inside them are not, so it exercises the S03
parser's rejection causes rather than the reader's skip counters. The reader's
own causes are tested against byte arrays built in the test, because a
committed file that is a broken capture would confuse every other tool that
opens the corpus and would make the fixture directory itself untrustworthy.

**Rationale**: Without these, a corpus is a set of files that used to exercise
something. A generator refactor that flattens the IPv6 chain or drops the second
fragment would leave every downstream test passing over weaker input, and the
loss would surface as a mysterious gap in coverage several slices later.

The `burst` row is the one that changed. Section 25.3 says it exceeds buffer
capacity, and the buffer holds 65,536 packets, so a faithful fixture is several
megabytes and contradicts the same section's requirement that fixtures be small.
Backpressure is a relationship between a rate and a capacity rather than a
property of a file, so the fixture supplies the rate and S08's test supplies a
small capacity. Recorded for promotion to specification section 29.

## R-8. Detecting overlapping script windows

**Decision**: On load, group entries by the flow they identify and reject any
pair whose half-open windows intersect. `always` intersects everything.

**A loopback entry must be written in canonical order.** Found while generating
the corpus, not while planning it. When both endpoints are local there is no
local one in the usual sense, so slice S03's decision D-5 assigns the flow key's
positions by a canonical ordering. A script entry for such a flow has to be
written in that same order or it matches nothing, which presents as a fixture
that parses perfectly and attributes nothing.

`loopback.script` therefore names the lower endpoint first and says why in a
comment. The alternative, having the attributor try both orders, was rejected:
it would make the double resolve flows the real attributor could not, which is
the one thing the matching rules are designed to prevent. This is worth
knowing before S13, which owns loopback direction resolution and will write
more of these.

**Rationale**: The alternative is last-one-wins or first-one-wins, and both are
silent. A script is a test's statement of intent, and two statements that
contradict each other mean the author believed something untrue about the test
they were writing. Failing to load says so at the point the mistake was made.

Half-open windows make adjacency unambiguous: a window ending at an instant and
another starting at the same instant do not overlap, which is what a port reuse
script wants to express and would be fiddly with closed intervals.

## R-9. Where a corpus generator can live without cost

**Decision**: The integration test target, regenerated through an environment
variable.

**Rationale**: The constraint that decided it is that `xtask` has no
dependencies at all. That is deliberate: it parses manifests by hand
specifically so the task runner cannot drift from the thing it checks. Putting
the generator there means either duplicating frame construction a third time or
taking `xtask` into the product dependency graph and widening the expected edge
set that `cargo xtask deps` enforces.

Widening a constitution check's expectations to place a test helper is a poor
trade, and S02's decision D-8 is the precedent for treating such a change as
significant rather than incidental.

The test target costs nothing: it compiles only under `cargo test`, ships in no
crate, needs no manifest and no license file, and holds generation and checking
in one place so the two cannot disagree about the format they share.

**Alternatives considered**: a `fixturegen` workspace member, which needs a
manifest, a license file for `cargo xtask license`, and a position in the
publication order, all for one file. An `examples/` binary in
`fragcap-capture`, which is more discoverable than an environment variable but
lives in the published crate.
