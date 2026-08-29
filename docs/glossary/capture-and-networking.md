# Capture and Networking

## 5-tuple

**Also known as:** connection tuple, flow tuple

The five values that identify one network conversation: protocol, source
address, source port, destination address, destination port.

A packet carries all five in its headers, so any observer can group packets
into conversations without cooperation from either endpoint. The tuple is the
natural key for anything that reasons about conversations rather than
individual packets.

{: .matters }
> The tuple is what fragcap joins against the [socket table](process-and-attribution.md#socket-table) to
> recover which process owns a packet. That join is the entire product.
> Critically, it works fully only for TCP: the UDP socket table carries no
> remote endpoint, so UDP attribution keys on the local endpoint alone. See
> specification section 8.4.

**See also:** [Flow](capture-and-networking.md#flow), [Socket table](process-and-attribution.md#socket-table),
[Attribution](process-and-attribution.md#attribution)

**References:**

- RFC 793, Transmission Control Protocol. Defines the endpoint pair that,
  with the protocol number, forms the tuple.

## Flow

One directional or bidirectional stream of packets sharing a
[5-tuple](capture-and-networking.md#5-tuple).

Tools differ on whether a flow is one direction or both. fragcap normalizes to
one key per conversation, with direction recorded per packet rather than baked
into the key, so a single conversation is one flow rather than two.

{: .matters }
> Normalizing direction out of the key is why `FlowKey` has a `local` and a
> `remote` field rather than a source and a destination. The local endpoint is
> the one on the capturing host, determined by matching against the interface
> address set.

**See also:** [5-tuple](capture-and-networking.md#5-tuple), [Attribution](process-and-attribution.md#attribution)

## Loopback

The virtual interface carrying traffic where both endpoints are on the same
host, conventionally `127.0.0.1` and `::1`.

Loopback traffic never reaches a physical adapter, so capturing it requires
either operating system support or a dedicated pseudo-adapter.

{: .matters }
> Reconnaissance found no launcher-to-client loopback conversation in either
> focal title: the largest loopback conversation had the same process on both
> ends. fragcap still captures loopback, but for intra-process communication
> rather than for observing a handoff. A conversation on the loopback adapter
> is **not** evidence that two processes communicated.

**See also:** [npcap](platform-and-distribution.md#npcap), [Named pipe](windows-internals.md#named-pipe)

## Backpressure

What happens when a producer generates data faster than a consumer accepts it.

Systems resolve it by blocking the producer, buffering without bound, or
discarding. Each choice trades a different failure: latency, memory, or data.

{: .matters }
> fragcap chooses a bounded ring with drop-oldest semantics, and constitution
> principle P-4 requires every discard to be counted in a named counter and
> surfaced. A capture tool that loses data without saying so produces
> conclusions the user cannot check.

**See also:** [Bounded buffer](capture-and-networking.md#bounded-buffer), [Drop-oldest](capture-and-networking.md#drop-oldest),
[Pipeline](capture-and-networking.md#pipeline)

## Flow key

The normalized identity of one conversation: a protocol plus a local and a
remote endpoint, where local is always the endpoint on the capturing host.

Normalizing the local position is what makes a single conversation one key
rather than two, and it is why [direction](capture-and-networking.md#direction) is recorded per packet
instead of being implied by which endpoint appears first.

{: .matters }
> A flow key is the lookup key into the attribution index on the capture
> thread, so equality and hashing are part of its contract rather than an
> implementation convenience. See specification section 8.4.

**See also:** [Flow](capture-and-networking.md#flow), [5-tuple](capture-and-networking.md#5-tuple),
[Attribution key](capture-and-networking.md#attribution-key), [Direction](capture-and-networking.md#direction)

## Attribution key

The part of a [flow key](capture-and-networking.md#flow-key) that a [socket table](process-and-attribution.md#socket-table) can
actually answer. It carries both endpoints for TCP and the local endpoint alone
for UDP.

The asymmetry is a property of the platform interface, not a fragcap choice.
The TCP socket table carries both endpoints, so a TCP flow resolves on the full
5-tuple. A UDP socket generally has no fixed peer, so the UDP table carries the
local endpoint and owning process only.

{: .matters }
> Specification section 8.4 requires that implementations never invent a remote
> endpoint for a UDP entry, because doing so produces confident wrong
> attributions rather than honest coarse ones. fragcap encodes this in the type:
> there is no variant that could carry a UDP remote.

**See also:** [Flow key](capture-and-networking.md#flow-key), [Socket table](process-and-attribution.md#socket-table),
[Wildcard bind address](capture-and-networking.md#wildcard-bind-address)

## Direction

Which way an individual packet travelled, inbound or outbound.

A property of the packet rather than of the [flow](capture-and-networking.md#flow). Because the
[flow key](capture-and-networking.md#flow-key) already normalized endpoint position, direction carries no
information the key duplicates and the two cannot disagree.

**See also:** [Flow key](capture-and-networking.md#flow-key), [Flow](capture-and-networking.md#flow)

## Capture scope

What a capture writes out, as distinct from what it observes.

Decided in userspace, on every packet, from the [attribution](process-and-attribution.md#attribution)
the packet already carries, and independent of whatever kernel filter happens to be
installed. That independence is the point: the narrowed filter cannot engage until the
target opens its first socket, and on a launcher-mediated title that is tens of seconds
during which everything on the wire is admitted.

Three scopes. `target` writes only traffic bound to a profile stage whose role is inside
the run's role set, and is the default because process attribution is what fragcap is for.
`profile` writes anything the profile binds, whatever the role set says. `all` writes
everything captured, which is what correlating a target against the rest of a machine, and
debugging attribution itself, both need.

{: .matters }
> Everything excluded is counted, and in two terms rather than one. A packet attributed to
> a process no stage binds is confidently not the capture's; a packet carrying no
> attribution at all might have been, and was excluded only because the socket table had
> not yet named it. Folding those together would hide a possible real loss inside an
> intended exclusion, which is the No Silent Loss principle (P-4) failing in the one place
> a scope decision can make it fail.

**See also:** [Attribution](process-and-attribution.md#attribution), [Stage binding](process-and-attribution.md#stage-binding)

## Capture mode

The shipped fragcap mode that passively records packets and attributes them to
processes without modifying traffic or routing it through a proxy.

Capture mode is the baseline experience behind `fragcap capture`: packet
capture, process attribution, scope accounting, output writing, streaming, and
ring behavior all operate without reaching inside the target process.

{: .matters }
> The word "passive" describes Capture mode specifically. The planned Deep
> Capture mode is active local proxy inspection, so future documents must name
> the mode they mean instead of applying the passive claim to the whole product.

**See also:** [Deep Capture](capture-and-networking.md#deep-capture),
[Capture scope](capture-and-networking.md#capture-scope),
[Capture session](process-and-attribution.md#capture-session)

## Deep Capture

The shipped fragcap mode that augments a Capture session with explicit, scoped
local proxy inspection for a selected target.

Deep Capture serves authorized game-development and research workflows
where application-layer traffic needs to be inspectable. It may route the target
through a [local inspection proxy](capture-and-networking.md#local-inspection-proxy) and may produce proxy-owned analyzer artifacts,
but it does not inject code, install hooks, read target memory, modify
executables, install packet interception drivers, or extract TLS keys from the
target process.

{: .matters }
> Deep Capture is a product mode, not a permission slip. Its implementation is
> valid only when the operator selected it explicitly, the scope is visible, the
> residue has a cleanup path, and the output can be correlated with the ordinary
> Capture session.

**See also:** [Capture mode](capture-and-networking.md#capture-mode),
[Local inspection proxy](capture-and-networking.md#local-inspection-proxy),
[Session bundle](file-and-wire-formats.md#session-bundle),
[Technique denylist](anti-cheat-and-security.md#technique-denylist)

## Compatibility calibration

An explicitly confirmed Deep Capture measurement that collects local compatibility evidence for one stored target, one declared launch case, and either proxy reachability or TLS behavior.

Reachability calibration runs without a certificate trust change. TLS calibration is a separate phase and is available only after current evidence shows that the same target and launch case reached the final client through the scoped proxy. Each phase writes only directly observed facts to the existing local target store and records its run outcome separately.

{: .matters }
> Calibration is an evidence-producing workflow, not a bypass around ordinary Deep Capture eligibility. Its plan is displayed before confirmation, its effects have finite deadlines, and its local evidence is never published automatically.

**See also:** [Deep Capture](capture-and-networking.md#deep-capture), [Target-scoped proxy configuration](capture-and-networking.md#target-scoped-proxy-configuration), [Lifecycle event](command-line-and-diagnostics.md#lifecycle-event)

## Local inspection proxy

A proxy process started by fragcap on the local machine to receive selected
target traffic during a Deep Capture session, establish the corresponding
upstream connection, and expose supported application-layer observations.

It is local because it runs on the operator's machine. It is an inspection proxy
because the proxy endpoint, not the game process, owns the decrypted side of
traffic that accepts the configured trust relationship.

{: .matters }
> The local inspection proxy is distinct from a packet interception driver or a
> Winsock catalog modification. It is an ordinary process that a selected target
> session is configured to use, and its configuration must be explicit,
> reversible, and logged.

**See also:** [Deep Capture](capture-and-networking.md#deep-capture),
[Target-scoped proxy configuration](capture-and-networking.md#target-scoped-proxy-configuration),
[Local development certificate authority](anti-cheat-and-security.md#local-development-certificate-authority)

## Target-scoped proxy configuration

The Deep Capture routing choice that applies proxy settings to the selected
target session rather than silently changing proxy behavior for the whole
machine.

The current mechanism applies proxy environment variables only to a managed launch owned by fragcap. If a game is launched under Deep Capture, proxy routing should follow that launched target where the platform permits it. fragcap does not silently widen the configuration to the machine.

{: .matters }
> Steam and publisher launchers can re-launch child processes in ways that may
> escape the original environment. Deep Capture therefore treats proxy
> inheritance as a measured compatibility fact, not an assumption.

**See also:** [Deep Capture](capture-and-networking.md#deep-capture),
[Managed launch](platform-and-distribution.md#managed-launch),
[Launcher-mediated](process-and-attribution.md#launcher-mediated)

## Wildcard bind address

An address of `0.0.0.0` or `::` recorded for a socket bound to every local
interface rather than to one.

The socket table reports the address a socket was bound to, not the address a
given datagram arrived on. A UDP socket bound to the wildcard therefore has to
be matched against both the wildcard and the specific interface address, or
attribution misses traffic it should have resolved.

**See also:** [Attribution key](capture-and-networking.md#attribution-key),
[Socket table](process-and-attribution.md#socket-table)

## Snapshot length

A limit on how many bytes of each frame are retained, set by the operator.

Choosing one is scope: the operator decides what to record, and the choice is
visible in their own invocation. Constitution principle P-9 permits that and
forbids something different, namely a record that fails to say truncation
happened.

{: .matters }
> fragcap keeps the original on-wire length beside the possibly shorter
> payload, so a truncated capture is self-describing. A single length field
> would have made truncation invisible after the fact.

**See also:** [Backpressure](capture-and-networking.md#backpressure)

## Link type

**Also known as:** DLT, data link type

The link layer encapsulation a capture source produces, identified by the
standard numeric code (the DLT number) shared by libpcap and pcapng. Ethernet is
DLT 1.

fragcap carries the code rather than a closed enumeration, so a backend
reporting an encapsulation fragcap has never seen is representable and is
written through unchanged rather than becoming a parse failure. The
[extcap](windows-internals.md#extcap) interface declares a DLT per interface; fragcap declares
Ethernet as the default, and the pcapng stream's own interface blocks carry the
true per-packet link type.

**See also:** [pcapng](file-and-wire-formats.md#pcapng), [EtherType](capture-and-networking.md#ethertype),
[BSD loopback encapsulation](capture-and-networking.md#bsd-loopback-encapsulation), [extcap](windows-internals.md#extcap)

## EtherType

The two byte field at the end of an Ethernet header naming the protocol its
payload carries. `0x0800` is IPv4 and `0x86DD` is IPv6.

fragcap dispatches on it and parses those two. Anything else, VLAN tagged
frames included, produces no flow key and advances a named counter, because
specification section 12.5 enumerates what fragcap parses and does not name
them.

{: .matters }
> The counter is what makes the gap visible. A VLAN tagged capture would
> otherwise present as traffic fragcap simply failed to attribute, with no
> indication that the encapsulation was the reason.

**See also:** [Link type](capture-and-networking.md#link-type),
[Parse rejection cause](process-and-attribution.md#parse-rejection-cause)

## BSD loopback encapsulation

The link layer encapsulation identified by link type code 0, in which a four
byte address family value in the capturing host's byte order precedes the
network header.

Distinct from code 101, raw IP, which has no link layer header at all. The two
are easily confused because both deliver a network header near the start of the
frame.

{: .matters }
> The value is host ordered and a capture file records no host byte order, so
> fragcap reads it both ways and accepts whichever matches a known family. That
> resolves rather than guesses: no known family value is also a known value
> byte-swapped.

**See also:** [Link type](capture-and-networking.md#link-type), [EtherType](capture-and-networking.md#ethertype)

## Extension header chain

The sequence of optional headers an IPv6 packet may carry between its fixed
header and its transport header, each naming the next.

fragcap walks the chain to reach the transport ports. The walk is bounded at
eight headers, because the IPv6 standard sets no limit and an unbounded walk
over attacker-controlled bytes on the capture thread would be a denial of
service against the capture rather than merely a parse defect. Real traffic
uses zero to two.

**See also:** [IP fragment](capture-and-networking.md#ip-fragment), [Flow key](capture-and-networking.md#flow-key)

## IP fragment

One piece of an IP datagram that was split to fit a smaller path maximum
transmission unit. Only the first piece carries the transport header, and
therefore the ports.

fragcap does not reassemble. Reassembly is an analysis concern, and performing
it during capture would destroy the on-wire fidelity that makes the capture
worth taking. Non-initial fragments, also called subsequent fragments, are
attributed instead from a [fragment identity table](capture-and-networking.md#fragment-identity-table).

**See also:** [Fragment identity](capture-and-networking.md#fragment-identity),
[Fragment identity table](capture-and-networking.md#fragment-identity-table)

## Fragment identity

What associates the fragments of one datagram with each other, so a fragment
carrying no transport header can be attributed from what the first one said.

The two address families define it differently, and fragcap follows each rather
than imposing one shape. IPv4 keys on the address pair, the protocol number,
and a sixteen bit identification, because that identification is only unique
per protocol. IPv6 keys on the address pair and a thirty two bit
identification, and carries no protocol number.

{: .matters }
> A shared definition would have meant inventing a protocol number for the IPv6
> case, which is the same fabrication specification section 8.4 prohibits for
> UDP remote endpoints. Honest coarse attribution beats confident wrong
> attribution.

**See also:** [IP fragment](capture-and-networking.md#ip-fragment),
[Fragment identity table](capture-and-networking.md#fragment-identity-table)

## Fragment identity table

The bounded memory from a [fragment identity](capture-and-networking.md#fragment-identity) to the
protocol and ports its first fragment carried.

Two hundred and fifty six entries, evicting oldest first, with the eviction
counted. It stores ports rather than an assembled [flow key](capture-and-networking.md#flow-key), so
that direction and the local position are recomputed for every fragment
against the current [interface address set](capture-and-networking.md#interface-address-set).

{: .matters }
> Bounded by entry count rather than by age, because an age bound needs a
> clock, and a clock in `fragcap-core` is a platform surface constitution
> principle P-2 excludes. A sixteen bit IPv4 identifier can therefore be reused
> before its entry is evicted; that residual case is stated in slice S03 rather
> than claimed away, because it is not detectable from the capture and so
> cannot be counted.

**See also:** [IP fragment](capture-and-networking.md#ip-fragment),
[Fragment identity](capture-and-networking.md#fragment-identity), [Backpressure](capture-and-networking.md#backpressure)

## pcap

The original libpcap capture file format: a twenty-four byte file header, then
records of a sixteen byte header and their packet bytes. Distinct from
[pcapng](file-and-wire-formats.md#pcapng), which is a much larger format and the one fragcap writes.

fragcap reads pcap and writes pcapng, and the asymmetry is deliberate. The
[fixture corpus](capture-and-networking.md#fixture-corpus) is written in the small format because a
reader for it needs no dependency and because ordinary tooling opens it. The
output format carries attribution, which pcap cannot.

{: .matters }
> Byte order and timestamp resolution are both declared by the file's magic
> number, in four combinations. A reader that assumed either would silently
> report a nanosecond capture's timestamps a thousand times too small, or read
> a foreign-endian file as garbage that still looked plausible.

**See also:** [pcapng](file-and-wire-formats.md#pcapng), [Replay source](process-and-attribution.md#replay-source),
[Fixture](capture-and-networking.md#fixture)

## Fixture

One small, committed, synthetic capture file that exists to exercise one stated
condition, paired with an [attribution script](process-and-attribution.md#attribution-script).

Synthetic is not incidental. A capture from a real game session carries account
identifiers, session tokens, and addresses, none of which belong in a public
repository, so every fixture is generated from constants and every payload byte
is filler.

**See also:** [Fixture corpus](capture-and-networking.md#fixture-corpus), [pcap](capture-and-networking.md#pcap)

## Fixture corpus

The eight [fixtures](capture-and-networking.md#fixture) of specification section 25.3 together, with
their scripts, the generator that produces them, and the check that proves the
committed bytes still match it.

{: .matters }
> The generator is the readable record of what each fixture contains; the
> binary is its output. A committed capture nobody can read is a test input
> nobody can review, and the drift check is what stops a hand-edited fixture
> passing quietly.

**See also:** [Fixture](capture-and-networking.md#fixture),
[Attribution script](process-and-attribution.md#attribution-script), [Test tier](rust-and-tooling.md#test-tier)

## Interface address set

The addresses belonging to the capturing host, against which a packet's
endpoints are tested to decide [direction](capture-and-networking.md#direction).

Supplied to the parser by its caller and replaced wholesale on an address
change notification, never polled and never queried from inside
`fragcap-core`. A local source is outbound and a local destination is inbound.
Both local is [loopback](capture-and-networking.md#loopback), which leaves direction undetermined.
Neither local yields no flow key at all, because a flow key's local field is
defined as the endpoint on the capturing host and there is not one.

{: .matters }
> A stale set silently inverts direction on every subsequent packet, which is
> why it is refreshed on notification rather than on a timer. An empty or stale
> set now announces itself: every packet lands in the no-local-endpoint
> counter, rather than yielding keys that no socket table lookup could resolve.

**See also:** [Direction](capture-and-networking.md#direction), [Flow key](capture-and-networking.md#flow-key),
[Loopback](capture-and-networking.md#loopback)

## Pipeline

The composition that reads packets from a [packet source](process-and-attribution.md#packet-source),
derives a [flow key](capture-and-networking.md#flow-key), resolves attribution through a
[flow attributor](process-and-attribution.md#flow-attributor), and writes the result to a set of
[sinks](process-and-attribution.md#sink).

Specification section 8.6 places it in `fragcap-core` and puts it on three
threads: a [capture thread](capture-and-networking.md#capture-thread), a control thread owning the
process watcher and the filter manager, and a [sink thread](capture-and-networking.md#sink-thread). A
[bounded buffer](capture-and-networking.md#bounded-buffer) sits between the first and the last.

{: .matters }
> The pipeline is the only thing that produces fragcap's own loss counters.
> Until slice S08 built it, `CaptureStats` had named fields and no producer,
> and every value written into a capture file was composed by hand in a test.

**See also:** [Bounded buffer](capture-and-networking.md#bounded-buffer),
[Capture thread](capture-and-networking.md#capture-thread), [Sink thread](capture-and-networking.md#sink-thread),
[Fan-out](capture-and-networking.md#fan-out)

## Bounded buffer

The fixed-capacity queue between the [capture thread](capture-and-networking.md#capture-thread) and the
[sink thread](capture-and-networking.md#sink-thread), holding 65,536 packets by default per
specification section 12.4.

It applies no [backpressure](capture-and-networking.md#backpressure). When full it evicts rather than
blocking, per [drop-oldest](capture-and-networking.md#drop-oldest), and each eviction advances the
`buffer_dropped` counter.

{: .matters }
> The capacity is the whole budget for a slow [sink](process-and-attribution.md#sink). A buffer drop
> means the sink could not keep up, which is a different remedy from a kernel
> drop, and section 12.4 keeps the two counters apart for exactly that reason.

**See also:** [Backpressure](capture-and-networking.md#backpressure), [Drop-oldest](capture-and-networking.md#drop-oldest),
[Pipeline](capture-and-networking.md#pipeline)

## Drop-oldest

The eviction policy of the [bounded buffer](capture-and-networking.md#bounded-buffer): when full, the
oldest buffered packet is discarded to admit the newest, and the discard is
counted.

The alternatives are blocking the producer, which specification section 12.4
rejects because it stalls the kernel buffer behind the capture and converts a
fragcap drop into a less visible kernel drop, and drop-newest, which section
12.4 rejects because a stalled sink is usually transient and preserving recent
traffic keeps the capture aligned with whatever caused the stall.

{: .matters }
> Drop-oldest is a declared omission, not an exception to constitution
> principle P-9. The instrument does not lie about it: the packets are counted
> and the count is written into the output.

**See also:** [Bounded buffer](capture-and-networking.md#bounded-buffer),
[Backpressure](capture-and-networking.md#backpressure)

## Capture thread

The thread that acquires packets from a [packet source](process-and-attribution.md#packet-source),
parses their headers, and looks up attribution, then pushes into the
[bounded buffer](capture-and-networking.md#bounded-buffer).

It never waits for a [sink](process-and-attribution.md#sink) to make progress. Specification section 12.1
gives each captured interface its own handle and its own capture thread.

{: .matters }
> Anything that blocks this thread stalls the capture driver's buffer behind
> it. That is why the [bounded buffer](capture-and-networking.md#bounded-buffer) evicts rather than
> applying [backpressure](capture-and-networking.md#backpressure).

**See also:** [Sink thread](capture-and-networking.md#sink-thread), [Pipeline](capture-and-networking.md#pipeline),
[Bounded buffer](capture-and-networking.md#bounded-buffer)

## Sink thread

The thread that drains the [bounded buffer](capture-and-networking.md#bounded-buffer) and performs the
[fan-out](capture-and-networking.md#fan-out) to every attached [sink](process-and-attribution.md#sink), then flushes and finishes
each one with the run's final statistics.

{: .matters }
> Trailing statistics are written by this thread after the buffer is drained,
> so the last packet is in the file before the counters that describe it.

**See also:** [Capture thread](capture-and-networking.md#capture-thread), [Fan-out](capture-and-networking.md#fan-out),
[Pipeline](capture-and-networking.md#pipeline)

## Fan-out

Offering each captured packet to every attached [sink](process-and-attribution.md#sink), so that one pass
over a [packet source](process-and-attribution.md#packet-source) produces every configured output.

A sink that cannot accept a packet advances the `sink_dropped` counter, once
per sink rather than once per packet, because each refusal is one output left
short.

{: .matters }
> Counting per packet would report one loss where three files are short, and
> the number would shrink as more outputs were attached, which is backwards.

**See also:** [Sink](process-and-attribution.md#sink), [Sink thread](capture-and-networking.md#sink-thread),
[Pipeline](capture-and-networking.md#pipeline)

## Transport

Where a [sink](process-and-attribution.md#sink) writes its bytes, as opposed to the format those bytes
take. fragcap has four: a file (with optional rotation), a [named
pipe](windows-internals.md#named-pipe), a Unix domain socket, and TCP. Format and transport are
orthogonal, so any format writes to any transport.

{: .matters }
> The orthogonality is a design choice, not an accident: a single factory builds
> a fresh format encoder over any transport connection, so a Wireshark-ready
> pcapng stream and a line-oriented JSON stream reach a pipe, a socket, or a file
> through the same seam. See specification section 14.1.

**See also:** [Streaming sink](capture-and-networking.md#streaming-sink), [Named pipe](windows-internals.md#named-pipe),
[Rotation segment](capture-and-networking.md#rotation-segment)

## Streaming sink

A [sink](process-and-attribution.md#sink) that serves any number of live [consumers](capture-and-networking.md#stream-consumer)
over a [transport](capture-and-networking.md#transport) that accepts connections (a named pipe or TCP).
Each connected consumer receives its own complete, independently valid stream,
including its own header preamble replayed on connect, so a consumer that joins
mid-capture still opens cleanly in an unmodified analyzer.

{: .matters }
> A streaming sink never blocks the capture and never returns a refusal to the
> [pipeline](capture-and-networking.md#pipeline): it accepts every packet and drops per consumer, so the
> pipeline conservation identity is preserved and the sink is never retired for a
> slow downstream reader. Its per-consumer drops are its own accounting, distinct
> from the capture-wide `sink_dropped`. See specification sections 14.3 and 14.4.

**See also:** [Transport](capture-and-networking.md#transport), [Stream consumer](capture-and-networking.md#stream-consumer),
[Per-consumer queue](capture-and-networking.md#per-consumer-queue)

## Stream consumer

One connected reader of a [streaming sink](capture-and-networking.md#streaming-sink), with its own
[per-consumer queue](capture-and-networking.md#per-consumer-queue), its own encoder writing to its
connection, and its own drop and disconnect accounting.

{: .matters }
> A consumer is isolated: its slowness degrades only itself. A consumer whose
> queue stays full past a timeout is disconnected and the disconnection reported,
> so a dead reader never holds capture buffer indefinitely.

**See also:** [Streaming sink](capture-and-networking.md#streaming-sink),
[Per-consumer queue](capture-and-networking.md#per-consumer-queue), [Backpressure](capture-and-networking.md#backpressure)

## Per-consumer queue

The bounded, drop-when-full buffer standing between the capture path and one
[stream consumer](capture-and-networking.md#stream-consumer)'s connection. It isolates that consumer's
speed from the capture and from every other consumer: a full queue drops packets
on that connection only, counted and reported for that consumer.

{: .matters }
> This is the [backpressure](capture-and-networking.md#backpressure) of section 14.4 made per consumer:
> unlike the capture-wide [bounded buffer](capture-and-networking.md#bounded-buffer), which is drop-oldest,
> a per-consumer queue refuses the newest packet when full, because a live reader
> that has fallen behind gains nothing from the sink spending work evicting its
> backlog, and every dropped packet is counted regardless.

**See also:** [Backpressure](capture-and-networking.md#backpressure), [Stream consumer](capture-and-networking.md#stream-consumer),
[Bounded buffer](capture-and-networking.md#bounded-buffer)

## Rotation segment

One numbered output file produced when a file [transport](capture-and-networking.md#transport) rotates by
size or by duration. Each segment is closed at a clean section boundary and
opens on its own in an unmodified analyzer; the union of a run's segments is the
capture, with no packet lost, duplicated, or reordered across the joins.

{: .matters }
> Rotation happens only at a section boundary, never mid-block, which is what
> makes every segment independently readable. A capture with no rotation policy
> is a single segment, byte identical to a non-rotating file. See specification
> section 14.2.

**See also:** [Transport](capture-and-networking.md#transport), [Sink](process-and-attribution.md#sink), [Pipeline](capture-and-networking.md#pipeline)

## Ring mode

The capture mode in which fragcap retains a rolling in-memory window of the most
recently captured packets, bounded by a [ring window](capture-and-networking.md#ring-window), discarding
the oldest as new ones arrive, and writes the retained window to a capture file
when the capture ends. The dump fires on any [stop condition](process-and-attribution.md#stop-condition),
the operator interrupt being the headline one; ring mode adds no stop condition of
its own. See specification section 7.2 (FR-8).

{: .matters }
> Ring mode is not the [bounded buffer](capture-and-networking.md#bounded-buffer) of section 12.4, which is
> also a bounded, drop-oldest ring. That buffer is the internal backpressure stage
> between the [capture thread](capture-and-networking.md#capture-thread) and the [sink thread](capture-and-networking.md#sink-thread);
> ring mode is an output mode an operator selects, and its evictions are the
> operator's declared retention scope, counted rather than lost (constitution P-4,
> P-9). Confusing the two conflates an internal mechanism with a user-facing mode.

**See also:** [Ring window](capture-and-networking.md#ring-window), [Bounded buffer](capture-and-networking.md#bounded-buffer),
[Sink](process-and-attribution.md#sink), [Stop condition](process-and-attribution.md#stop-condition)

## Ring window

The bound on a [ring mode](capture-and-networking.md#ring-mode) capture's retained set: either a duration or
a byte size, from the `--ring` option. A duration window keeps the packets whose
capture instant is within the window measured back from the newest instant
observed; a size window keeps the newest packets whose total captured length is
within the window, the same per-packet quantity the `--max-bytes` bound sums.

{: .matters }
> Measuring the size window by captured length, not encoded file size, lets an
> operator reason about one notion of capture size across `--ring` and
> `--max-bytes`. A window smaller than one packet still retains that one packet, so
> a capture that observed traffic never dumps an empty file.

**See also:** [Ring mode](capture-and-networking.md#ring-mode), [Duration literal](capture-and-networking.md#duration-literal),
[Write gate](capture-and-networking.md#write-gate)

## Write gate

A decision the [sink thread](capture-and-networking.md#sink-thread) consults, synchronously, before the
[fan-out](capture-and-networking.md#fan-out): whether a captured packet is admitted to the sinks at all. A
generic `WriteGate` seam in `fragcap-core` answers admit-or-discard for a packet; a
session-driven implementation in the facade admits only while the session is
[capturing](capture-and-networking.md#capture-window) and the configured volume bound has not been reached.
A packet the gate withholds is written to no sink and counted in the `gate_dropped`
counter, a term of the pipeline conservation identity distinct from a buffer drop
and a sink drop.

{: .matters }
> Making the decision on the write path, rather than observing the count after the
> fact, is what makes a `--max-packets` or `--max-bytes` bound produce an exactly
> bounded file: a packet the gate discards is never written, so the file and the
> accounting are the same set by construction. A `gate_dropped` counter keeps that
> synchronous discard inside the P-4 accounting rather than letting it escape.

**See also:** [Fan-out](capture-and-networking.md#fan-out), [Capture window](capture-and-networking.md#capture-window),
[Completion summary](command-line-and-diagnostics.md#completion-summary), [Stop condition](process-and-attribution.md#stop-condition)

## Capture window

The state a [write gate](capture-and-networking.md#write-gate) reads to decide whether a packet is
admitted: open while the [capture session](process-and-attribution.md#capture-session) is capturing, closed
while it is watching for a target or draining after a stop. A live capture holds
its handle open from arm, so frames arrive while the window is still closed for
watching; the gate discards and counts those rather than letting them go
unobserved. Offline the window is opened before the pipeline starts, so no
watch-time frame is seen and an unbounded run is a pass-through.

{: .matters }
> The window is published lock-free and written only by the driver, the same
> discipline the attribution snapshot uses (section 11.6), so the sink thread
> reads it without ever blocking the thread that advances the session.

**See also:** [Write gate](capture-and-networking.md#write-gate), [Capture session](process-and-attribution.md#capture-session),
[Completion summary](command-line-and-diagnostics.md#completion-summary)

## Duration literal

A capture duration as an operator writes it: one unsigned decimal integer
followed by one unit from `ms`, `s`, `m`, or `h`. `30m` is thirty minutes.

A bare integer is refused rather than given a default unit, zero is refused, and
compound forms such as `1h30m` are not accepted in this schema version. The
grammar lives in `fragcap-core` because a [game profile](platform-and-distribution.md#game-profile), the
command line, and ring mode all need the same one.

{: .matters }
> A guessed unit is a guess about how much of a session the operator loses, and
> two implementations of `30m` that disagree produce a capture of the wrong
> length. Widening the accepted syntax later keeps every profile written today
> valid; narrowing it does not, so the narrow form ships first.

**See also:** [Game profile](platform-and-distribution.md#game-profile),
[Profile schema version](process-and-attribution.md#profile-schema-version)

## Bootstrap filter

**Also known as:** phase one filter

The deliberately permissive kernel filter fragcap installs on every capture
handle before any packet is delivered: IPv4 and IPv6 traffic, and nothing else.

Specification section 12.2 divides the filter lifecycle into three phases, and
this is the first. Game endpoints are not known until a session begins, so no
narrow filter can be installed in advance. Packets admitted during this phase
are discarded in userspace, because no attribution exists yet to decide with.

{: .matters }
> The temptation is to narrow it early to reduce volume, and reconnaissance
> showed the volume is real: one unrelated background process accounted for up
> to 94 percent of captured bytes. Narrowing before attribution exists would
> discard traffic in the kernel with no way to know what was lost, which is a
> discard with no counter and therefore a constitution P-4 violation. The cost
> is paid in bytes rather than in fidelity, deliberately.

**See also:** [Filter gap](capture-and-networking.md#filter-gap), [Interface inventory](capture-and-networking.md#interface-inventory)

## Narrowing

**Also known as:** phase two filter

The second phase of the specification section 12.2 filter lifecycle. Once the
attribution map holds endpoints belonging to profiled processes, fragcap compiles
a filter admitting only those endpoints and installs it on each live handle,
dropping the volume crossing the kernel boundary to the target's traffic plus
whatever shares its ports.

The endpoint set comes from the attribution map, never from observed name
resolution, because gameplay endpoints are reached by address with no preceding
lookup in both focal titles. Over-admission of shared-port traffic is accepted and
resolved by userspace attribution, not tightened in the kernel.

**See also:** [Bootstrap filter](capture-and-networking.md#bootstrap-filter), [Maintenance](capture-and-networking.md#maintenance),
[Filter gap](capture-and-networking.md#filter-gap)

## Maintenance

**Also known as:** phase three filter

The third phase of the specification section 12.2 filter lifecycle. As the
endpoint set changes, fragcap recompiles and reinstalls, debounced by two seconds
and rate limited to one reinstallation per five seconds per handle, because
installing a filter briefly interrupts capture on that handle and endpoint sets
churn during connection establishment.

**See also:** [Narrowing](capture-and-networking.md#narrowing), [Filter gap](capture-and-networking.md#filter-gap),
[Filter manager](capture-and-networking.md#filter-manager)

## Filter program

The compiled kernel filter fragcap hands to a capture handle, in the backend's own
syntax (libpcap, for the npcap backend). The bootstrap program admits `ip or ip6`;
a narrowed program admits only the profiled endpoints.

Modeled as `FilterProgram` in `fragcap-core`, which treats it as opaque text. Only
`fragcap-capture` compiles it onto a handle, which is what keeps core
platform-neutral (constitution P-2): a filter expression is just a string until a
backend compiles it.

**See also:** [Bootstrap filter](capture-and-networking.md#bootstrap-filter),
[Filter manager](capture-and-networking.md#filter-manager), [Narrowing](capture-and-networking.md#narrowing)

## Filter manager

The control-thread component that reads the attribution map's active endpoints,
runs the narrowing and maintenance policy, and hands each capture thread its
current [filter program](capture-and-networking.md#filter-program) over a private channel.

It bridges the packet source and the flow attributor without merging them
(constitution P-3): it names neither trait in a signature, adds no `Sync` bound to
the source, and lives on the control thread, which is the one place that already
holds both. Compilation and the debounce-and-rate-limit policy are pure over core
types, so the whole strategy is tested with synthetic instants and no capture
driver.

**See also:** [Narrowing](capture-and-networking.md#narrowing), [Maintenance](capture-and-networking.md#maintenance),
[Filter program](capture-and-networking.md#filter-program)

## Filter gap

An endpoint active in the attribution map while a narrowed kernel filter that does
not admit it is installed on a handle, for the interval from the endpoint appearing
until the reinstall that admits it.

Counted in the `filter_gaps` statistic and surfaced (specification section 12.3).
Because correctness never depends on filter freshness, a stale filter that briefly
excludes wanted traffic is not a silent loss: userspace attribution still runs on
every packet, and the gap is reported.

{: .matters }
> The count is of gap occurrences, not of packets. A packet the kernel filter
> excludes is never delivered to fragcap, so a packet count would be fabricated,
> and constitution P-9 forbids reporting a number the instrument did not observe.
> A bootstrap-to-first-narrowing transition opens no gap, because bootstrap
> admitted everything and the narrowing excludes only unwanted traffic; gaps arise
> only when an endpoint appears while a strictly narrowed filter is installed.

**See also:** [Bootstrap filter](capture-and-networking.md#bootstrap-filter), [Narrowing](capture-and-networking.md#narrowing),
[Maintenance](capture-and-networking.md#maintenance)

## Install acknowledgement

The report a capture thread sends back to the control thread after calling
`set_filter`, saying whether the backend accepted the maintenance filter program.
The filter manager commits a handle's installed program, and clears its gap set,
only on a success acknowledgement; a rejection leaves the prior program in place
and the install is retried.

Carried as a `(handle, installed_ok)` message over the reverse of the per-source
filter channel. It exists because the manager otherwise marked a program installed
the moment it decided to install it, before the capture thread had applied it, so a
rejected install left the manager's model diverged from the real handle with the
divergence recorded nowhere. A program the manager has issued and not yet seen
acknowledged is a **pending install**, and while one is pending the manager issues
no new install for that handle, so a bare acknowledgement is unambiguous.

**See also:** [Filter manager](capture-and-networking.md#filter-manager), [Filter gap](capture-and-networking.md#filter-gap),
[Maintenance](capture-and-networking.md#maintenance)

## OwnedEndpoint

An active endpoint paired with the process identifier that owns it, when the
source can supply one. Modeled as `OwnedEndpoint` in `fragcap-core::flow` and
returned by `FlowAttributor::active_endpoints_owned`.

The plain endpoint list drops the owner the socket table carried; the owned form
keeps it so the section 12.2 narrowing can restrict the kernel filter to endpoints
belonging to profiled processes. The owner is optional: the live socket-table
backend always supplies one, while the scripted attributor supplies none, in which
case the endpoint is treated as not known to belong to any particular process.

**See also:** [Profiled endpoint set](capture-and-networking.md#profiled-endpoint-set),
[Narrowing](capture-and-networking.md#narrowing)

## Profiled endpoint set

The endpoints owned by a profiled process: the input the section 12.2 narrowing
actually compiles into the kernel filter. Derived by joining the active endpoints
(each carrying its owner) against the session's stage bindings, whose process
identifiers are the profiled ones.

The join runs in the role-stamping attributor, the one seam above both the socket
table and the session, so neither the pipeline nor the attribution crate learns
about profiles (constitution P-3). An endpoint whose owner is not known is kept
rather than excluded, so on the live backend the set is exactly the profiled
endpoints while on the offline scripted substrate it is a pass-through.

**See also:** [OwnedEndpoint](capture-and-networking.md#ownedendpoint), [Narrowing](capture-and-networking.md#narrowing)

## Interface identifier

The identity fragcap assigns to a capture interface for the duration of one
run, carried on every packet acquired from it and preserved into output.

Assigned by [selection](capture-and-networking.md#selection-outcome) from position, not taken from the
platform, because platform interface names are not guaranteed unique and
specification section 12.1 requires every packet to name where it arrived.

{: .matters }
> It is not optional on a captured packet. Every packet arrived somewhere, so
> an absent identifier would be a claim that one came from nowhere, and a
> default value would be right for a single-interface capture and silently
> wrong for every other. The wrongness would appear in the output as a packet
> attributed to an adapter it never touched.

**See also:** [Interface inventory](capture-and-networking.md#interface-inventory),
[Selection outcome](capture-and-networking.md#selection-outcome)

## Interface inventory

What a machine reports about its capture-capable interfaces at a moment in
time, as a value: for each one a name, a description, addresses, whether it is
up, whether it is a loopback adapter, plus the source address the routing table
would choose for an off-link destination.

{: .matters }
> Being a value rather than a query is the whole design. A capture backend
> produces one by enumerating a real machine; a test writes one by hand.
> Selection cannot tell the difference, so the entire specification section
> 12.1 precedence is testable with no capture driver, no privilege, and no
> network.

**See also:** [Interface identifier](capture-and-networking.md#interface-identifier),
[Selection outcome](capture-and-networking.md#selection-outcome),
[Virtual interface](capture-and-networking.md#virtual-interface)

## Interface retirement

The end of one capture thread, recorded with the interface it belonged to and
the reason it stopped.

A run with several interfaces does not end when one of them fails. That
interface retires, the others keep delivering, and the run ends when the last
has retired or a stop was requested. This mirrors the treatment of a failed
sink established in slice S08.

{: .matters }
> A retirement advances no drop counter, and that is deliberate rather than an
> omission. A retired interface stops producing observations; it does not
> discard observations fragcap already had. Counting it as loss would report
> packets that were never observed as packets that were thrown away, which is a
> constitution P-9 problem rather than an arithmetic one.

**See also:** [Interface identifier](capture-and-networking.md#interface-identifier)

## Selection outcome

The complete result of applying specification section 12.1's precedence to an
[interface inventory](capture-and-networking.md#interface-inventory): the interfaces chosen, in order,
plus every interface not chosen and the named reason it was passed over.

{: .matters }
> The second half is what the type exists for. Capturing on the wrong interface
> produces a run that exits zero, writes a well-formed capture file, and
> contains nothing, which is invisible unless the decision is reported. The
> chosen and the passed-over together must account for the whole inventory, and
> a test asserts that rather than trusting it, so a future precedence rule
> cannot drop an interface on the floor.

**See also:** [Interface inventory](capture-and-networking.md#interface-inventory),
[Virtual interface](capture-and-networking.md#virtual-interface)

## Virtual interface

An adapter created by a hypervisor, container runtime, virtual private network,
or subsystem networking layer, which fragcap excludes from automatic interface
selection while leaving it explicitly selectable.

fragcap decides this by matching the adapter description against a documented
list of patterns.

{: .matters }
> This is a heuristic and fragcap says so rather than presenting it as fact:
> no platform reports a "this is a hypervisor adapter" bit. Two things keep it
> honest. The verdict only ever excludes from **automatic** selection, so an
> operator who names an interface gets it whatever the rule concluded. And the
> verdict is recorded with the pattern that matched, so a misclassified adapter
> is visible in the run's report rather than discovered as an empty capture.

**See also:** [Interface inventory](capture-and-networking.md#interface-inventory),
[Selection outcome](capture-and-networking.md#selection-outcome)
