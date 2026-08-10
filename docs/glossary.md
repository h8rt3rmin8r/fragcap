# Glossary

Vocabulary used across fragcap's documentation and code. Constitution principle
P-6 requires a term to gain an entry here **in the same change that introduces
it**, which is why this file exists before the documentation site does: a
principle with nowhere to land is a principle that quietly accumulates a
backlog.

Entries follow the structure in specification section 4.3, are filed under one
of the categories in section 4.4, and prefer primary sources per section 4.5.

**This is the interim home.** Section 22.4 places the glossary in the
documentation site, one page per category with a generated alphabetical index,
and slice S18 builds it. Until then everything lives here, grouped by the same
categories, so nothing has to be reconstructed later. The documentation linter
in section 4.6 arrives with S18; until it does, these rules are kept by hand
and that is a weaker guarantee, stated rather than glossed.

## Capture and Networking

### 5-tuple

**Also known as:** connection tuple, flow tuple

The five values that identify one network conversation: protocol, source
address, source port, destination address, destination port.

A packet carries all five in its headers, so any observer can group packets
into conversations without cooperation from either endpoint. The tuple is the
natural key for anything that reasons about conversations rather than
individual packets.

{: .matters }
> The tuple is what fragcap joins against the [socket table](#socket-table) to
> recover which process owns a packet. That join is the entire product.
> Critically, it works fully only for TCP: the UDP socket table carries no
> remote endpoint, so UDP attribution keys on the local endpoint alone. See
> specification section 8.4.

**See also:** [Flow](#flow), [Socket table](#socket-table),
[Attribution](#attribution)

**References:**

- RFC 793, Transmission Control Protocol. Defines the endpoint pair that,
  with the protocol number, forms the tuple.

### Flow

One directional or bidirectional stream of packets sharing a
[5-tuple](#5-tuple).

Tools differ on whether a flow is one direction or both. fragcap normalizes to
one key per conversation, with direction recorded per packet rather than baked
into the key, so a single conversation is one flow rather than two.

{: .matters }
> Normalizing direction out of the key is why `FlowKey` has a `local` and a
> `remote` field rather than a source and a destination. The local endpoint is
> the one on the capturing host, determined by matching against the interface
> address set.

**See also:** [5-tuple](#5-tuple), [Attribution](#attribution)

### Loopback

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

**See also:** [npcap](#npcap), [Named pipe](#named-pipe)

### Backpressure

What happens when a producer generates data faster than a consumer accepts it.

Systems resolve it by blocking the producer, buffering without bound, or
discarding. Each choice trades a different failure: latency, memory, or data.

{: .matters }
> fragcap chooses a bounded ring with drop-oldest semantics, and constitution
> principle P-4 requires every discard to be counted in a named counter and
> surfaced. A capture tool that loses data without saying so produces
> conclusions the user cannot check.

**See also:** [Bounded buffer](#bounded-buffer), [Drop-oldest](#drop-oldest),
[Pipeline](#pipeline)

### Flow key

The normalized identity of one conversation: a protocol plus a local and a
remote endpoint, where local is always the endpoint on the capturing host.

Normalizing the local position is what makes a single conversation one key
rather than two, and it is why [direction](#direction) is recorded per packet
instead of being implied by which endpoint appears first.

{: .matters }
> A flow key is the lookup key into the attribution index on the capture
> thread, so equality and hashing are part of its contract rather than an
> implementation convenience. See specification section 8.4.

**See also:** [Flow](#flow), [5-tuple](#5-tuple),
[Attribution key](#attribution-key), [Direction](#direction)

### Attribution key

The part of a [flow key](#flow-key) that a [socket table](#socket-table) can
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

**See also:** [Flow key](#flow-key), [Socket table](#socket-table),
[Wildcard bind address](#wildcard-bind-address)

### Direction

Which way an individual packet travelled, inbound or outbound.

A property of the packet rather than of the [flow](#flow). Because the
[flow key](#flow-key) already normalized endpoint position, direction carries no
information the key duplicates and the two cannot disagree.

**See also:** [Flow key](#flow-key), [Flow](#flow)

### Wildcard bind address

An address of `0.0.0.0` or `::` recorded for a socket bound to every local
interface rather than to one.

The socket table reports the address a socket was bound to, not the address a
given datagram arrived on. A UDP socket bound to the wildcard therefore has to
be matched against both the wildcard and the specific interface address, or
attribution misses traffic it should have resolved.

**See also:** [Attribution key](#attribution-key),
[Socket table](#socket-table)

### Snapshot length

A limit on how many bytes of each frame are retained, set by the operator.

Choosing one is scope: the operator decides what to record, and the choice is
visible in their own invocation. Constitution principle P-9 permits that and
forbids something different, namely a record that fails to say truncation
happened.

{: .matters }
> fragcap keeps the original on-wire length beside the possibly shorter
> payload, so a truncated capture is self-describing. A single length field
> would have made truncation invisible after the fact.

**See also:** [Backpressure](#backpressure)

### Link type

The link layer encapsulation a capture source produces, identified by the
standard numeric code shared by libpcap and pcapng.

fragcap carries the code rather than a closed enumeration, so a backend
reporting an encapsulation fragcap has never seen is representable and is
written through unchanged rather than becoming a parse failure.

**See also:** [pcapng](#pcapng), [EtherType](#ethertype),
[BSD loopback encapsulation](#bsd-loopback-encapsulation)

### EtherType

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

**See also:** [Link type](#link-type),
[Parse rejection cause](#parse-rejection-cause)

### BSD loopback encapsulation

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

**See also:** [Link type](#link-type), [EtherType](#ethertype)

### Extension header chain

The sequence of optional headers an IPv6 packet may carry between its fixed
header and its transport header, each naming the next.

fragcap walks the chain to reach the transport ports. The walk is bounded at
eight headers, because the IPv6 standard sets no limit and an unbounded walk
over attacker-controlled bytes on the capture thread would be a denial of
service against the capture rather than merely a parse defect. Real traffic
uses zero to two.

**See also:** [IP fragment](#ip-fragment), [Flow key](#flow-key)

### IP fragment

One piece of an IP datagram that was split to fit a smaller path maximum
transmission unit. Only the first piece carries the transport header, and
therefore the ports.

fragcap does not reassemble. Reassembly is an analysis concern, and performing
it during capture would destroy the on-wire fidelity that makes the capture
worth taking. Non-initial fragments, also called subsequent fragments, are
attributed instead from a [fragment identity table](#fragment-identity-table).

**See also:** [Fragment identity](#fragment-identity),
[Fragment identity table](#fragment-identity-table)

### Fragment identity

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

**See also:** [IP fragment](#ip-fragment),
[Fragment identity table](#fragment-identity-table)

### Fragment identity table

The bounded memory from a [fragment identity](#fragment-identity) to the
protocol and ports its first fragment carried.

Two hundred and fifty six entries, evicting oldest first, with the eviction
counted. It stores ports rather than an assembled [flow key](#flow-key), so
that direction and the local position are recomputed for every fragment
against the current [interface address set](#interface-address-set).

{: .matters }
> Bounded by entry count rather than by age, because an age bound needs a
> clock, and a clock in `fragcap-core` is a platform surface constitution
> principle P-2 excludes. A sixteen bit IPv4 identifier can therefore be reused
> before its entry is evicted; that residual case is stated in slice S03 rather
> than claimed away, because it is not detectable from the capture and so
> cannot be counted.

**See also:** [IP fragment](#ip-fragment),
[Fragment identity](#fragment-identity), [Backpressure](#backpressure)

### pcap

The original libpcap capture file format: a twenty-four byte file header, then
records of a sixteen byte header and their packet bytes. Distinct from
[pcapng](#pcapng), which is a much larger format and the one fragcap writes.

fragcap reads pcap and writes pcapng, and the asymmetry is deliberate. The
[fixture corpus](#fixture-corpus) is written in the small format because a
reader for it needs no dependency and because ordinary tooling opens it. The
output format carries attribution, which pcap cannot.

{: .matters }
> Byte order and timestamp resolution are both declared by the file's magic
> number, in four combinations. A reader that assumed either would silently
> report a nanosecond capture's timestamps a thousand times too small, or read
> a foreign-endian file as garbage that still looked plausible.

**See also:** [pcapng](#pcapng), [Replay source](#replay-source),
[Fixture](#fixture)

### Fixture

One small, committed, synthetic capture file that exists to exercise one stated
condition, paired with an [attribution script](#attribution-script).

Synthetic is not incidental. A capture from a real game session carries account
identifiers, session tokens, and addresses, none of which belong in a public
repository, so every fixture is generated from constants and every payload byte
is filler.

**See also:** [Fixture corpus](#fixture-corpus), [pcap](#pcap)

### Fixture corpus

The eight [fixtures](#fixture) of specification section 25.3 together, with
their scripts, the generator that produces them, and the check that proves the
committed bytes still match it.

{: .matters }
> The generator is the readable record of what each fixture contains; the
> binary is its output. A committed capture nobody can read is a test input
> nobody can review, and the drift check is what stops a hand-edited fixture
> passing quietly.

**See also:** [Fixture](#fixture),
[Attribution script](#attribution-script), [Test tier](#test-tier)

### Interface address set

The addresses belonging to the capturing host, against which a packet's
endpoints are tested to decide [direction](#direction).

Supplied to the parser by its caller and replaced wholesale on an address
change notification, never polled and never queried from inside
`fragcap-core`. A local source is outbound and a local destination is inbound.
Both local is [loopback](#loopback), which leaves direction undetermined.
Neither local yields no flow key at all, because a flow key's local field is
defined as the endpoint on the capturing host and there is not one.

{: .matters }
> A stale set silently inverts direction on every subsequent packet, which is
> why it is refreshed on notification rather than on a timer. An empty or stale
> set now announces itself: every packet lands in the no-local-endpoint
> counter, rather than yielding keys that no socket table lookup could resolve.

**See also:** [Direction](#direction), [Flow key](#flow-key),
[Loopback](#loopback)

### Pipeline

The composition that reads packets from a [packet source](#packet-source),
derives a [flow key](#flow-key), resolves attribution through a
[flow attributor](#flow-attributor), and writes the result to a set of
[sinks](#sink).

Specification section 8.6 places it in `fragcap-core` and puts it on three
threads: a [capture thread](#capture-thread), a control thread owning the
process watcher and the filter manager, and a [sink thread](#sink-thread). A
[bounded buffer](#bounded-buffer) sits between the first and the last.

{: .matters }
> The pipeline is the only thing that produces fragcap's own loss counters.
> Until slice S08 built it, `CaptureStats` had named fields and no producer,
> and every value written into a capture file was composed by hand in a test.

**See also:** [Bounded buffer](#bounded-buffer),
[Capture thread](#capture-thread), [Sink thread](#sink-thread),
[Fan-out](#fan-out)

### Bounded buffer

The fixed-capacity queue between the [capture thread](#capture-thread) and the
[sink thread](#sink-thread), holding 65,536 packets by default per
specification section 12.4.

It applies no [backpressure](#backpressure). When full it evicts rather than
blocking, per [drop-oldest](#drop-oldest), and each eviction advances the
`buffer_dropped` counter.

{: .matters }
> The capacity is the whole budget for a slow [sink](#sink). A buffer drop
> means the sink could not keep up, which is a different remedy from a kernel
> drop, and section 12.4 keeps the two counters apart for exactly that reason.

**See also:** [Backpressure](#backpressure), [Drop-oldest](#drop-oldest),
[Pipeline](#pipeline)

### Drop-oldest

The eviction policy of the [bounded buffer](#bounded-buffer): when full, the
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

**See also:** [Bounded buffer](#bounded-buffer),
[Backpressure](#backpressure)

### Capture thread

The thread that acquires packets from a [packet source](#packet-source),
parses their headers, and looks up attribution, then pushes into the
[bounded buffer](#bounded-buffer).

It never waits for a [sink](#sink) to make progress. Specification section 12.1
gives each captured interface its own handle and its own capture thread.

{: .matters }
> Anything that blocks this thread stalls the capture driver's buffer behind
> it. That is why the [bounded buffer](#bounded-buffer) evicts rather than
> applying [backpressure](#backpressure).

**See also:** [Sink thread](#sink-thread), [Pipeline](#pipeline),
[Bounded buffer](#bounded-buffer)

### Sink thread

The thread that drains the [bounded buffer](#bounded-buffer) and performs the
[fan-out](#fan-out) to every attached [sink](#sink), then flushes and finishes
each one with the run's final statistics.

{: .matters }
> Trailing statistics are written by this thread after the buffer is drained,
> so the last packet is in the file before the counters that describe it.

**See also:** [Capture thread](#capture-thread), [Fan-out](#fan-out),
[Pipeline](#pipeline)

### Fan-out

Offering each captured packet to every attached [sink](#sink), so that one pass
over a [packet source](#packet-source) produces every configured output.

A sink that cannot accept a packet advances the `sink_dropped` counter, once
per sink rather than once per packet, because each refusal is one output left
short.

{: .matters }
> Counting per packet would report one loss where three files are short, and
> the number would shrink as more outputs were attached, which is backwards.

**See also:** [Sink](#sink), [Sink thread](#sink-thread),
[Pipeline](#pipeline)

### Duration literal

A capture duration as an operator writes it: one unsigned decimal integer
followed by one unit from `ms`, `s`, `m`, or `h`. `30m` is thirty minutes.

A bare integer is refused rather than given a default unit, zero is refused, and
compound forms such as `1h30m` are not accepted in this schema version. The
grammar lives in `fragcap-core` because a [game profile](#game-profile), the
command line, and ring mode all need the same one.

{: .matters }
> A guessed unit is a guess about how much of a session the operator loses, and
> two implementations of `30m` that disagree produce a capture of the wrong
> length. Widening the accepted syntax later keeps every profile written today
> valid; narrowing it does not, so the narrow form ships first.

**See also:** [Game profile](#game-profile),
[Profile schema version](#profile-schema-version)

### Bootstrap filter

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

**See also:** [Filter gap](#filter-gap), [Interface inventory](#interface-inventory)

### Narrowing

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

**See also:** [Bootstrap filter](#bootstrap-filter), [Maintenance](#maintenance),
[Filter gap](#filter-gap)

### Maintenance

**Also known as:** phase three filter

The third phase of the specification section 12.2 filter lifecycle. As the
endpoint set changes, fragcap recompiles and reinstalls, debounced by two seconds
and rate limited to one reinstallation per five seconds per handle, because
installing a filter briefly interrupts capture on that handle and endpoint sets
churn during connection establishment.

**See also:** [Narrowing](#narrowing), [Filter gap](#filter-gap),
[Filter manager](#filter-manager)

### Filter program

The compiled kernel filter fragcap hands to a capture handle, in the backend's own
syntax (libpcap, for the npcap backend). The bootstrap program admits `ip or ip6`;
a narrowed program admits only the profiled endpoints.

Modeled as `FilterProgram` in `fragcap-core`, which treats it as opaque text. Only
`fragcap-capture` compiles it onto a handle, which is what keeps core
platform-neutral (constitution P-2): a filter expression is just a string until a
backend compiles it.

**See also:** [Bootstrap filter](#bootstrap-filter),
[Filter manager](#filter-manager), [Narrowing](#narrowing)

### Filter manager

The control-thread component that reads the attribution map's active endpoints,
runs the narrowing and maintenance policy, and hands each capture thread its
current [filter program](#filter-program) over a private channel.

It bridges the packet source and the flow attributor without merging them
(constitution P-3): it names neither trait in a signature, adds no `Sync` bound to
the source, and lives on the control thread, which is the one place that already
holds both. Compilation and the debounce-and-rate-limit policy are pure over core
types, so the whole strategy is tested with synthetic instants and no capture
driver.

**See also:** [Narrowing](#narrowing), [Maintenance](#maintenance),
[Filter program](#filter-program)

### Filter gap

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

**See also:** [Bootstrap filter](#bootstrap-filter), [Narrowing](#narrowing),
[Maintenance](#maintenance)

### OwnedEndpoint

An active endpoint paired with the process identifier that owns it, when the
source can supply one. Modeled as `OwnedEndpoint` in `fragcap-core::flow` and
returned by `FlowAttributor::active_endpoints_owned`.

The plain endpoint list drops the owner the socket table carried; the owned form
keeps it so the section 12.2 narrowing can restrict the kernel filter to endpoints
belonging to profiled processes. The owner is optional: the live socket-table
backend always supplies one, while the scripted attributor supplies none, in which
case the endpoint is treated as not known to belong to any particular process.

**See also:** [Profiled endpoint set](#profiled-endpoint-set),
[Narrowing](#narrowing)

### Profiled endpoint set

The endpoints owned by a profiled process: the input the section 12.2 narrowing
actually compiles into the kernel filter. Derived by joining the active endpoints
(each carrying its owner) against the session's stage bindings, whose process
identifiers are the profiled ones.

The join runs in the role-stamping attributor, the one seam above both the socket
table and the session, so neither the pipeline nor the attribution crate learns
about profiles (constitution P-3). An endpoint whose owner is not known is kept
rather than excluded, so on the live backend the set is exactly the profiled
endpoints while on the offline scripted substrate it is a pass-through.

**See also:** [OwnedEndpoint](#ownedendpoint), [Narrowing](#narrowing)

### Interface identifier

The identity fragcap assigns to a capture interface for the duration of one
run, carried on every packet acquired from it and preserved into output.

Assigned by [selection](#selection-outcome) from position, not taken from the
platform, because platform interface names are not guaranteed unique and
specification section 12.1 requires every packet to name where it arrived.

{: .matters }
> It is not optional on a captured packet. Every packet arrived somewhere, so
> an absent identifier would be a claim that one came from nowhere, and a
> default value would be right for a single-interface capture and silently
> wrong for every other. The wrongness would appear in the output as a packet
> attributed to an adapter it never touched.

**See also:** [Interface inventory](#interface-inventory),
[Selection outcome](#selection-outcome)

### Interface inventory

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

**See also:** [Interface identifier](#interface-identifier),
[Selection outcome](#selection-outcome),
[Virtual interface](#virtual-interface)

### Interface retirement

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

**See also:** [Interface identifier](#interface-identifier)

### Selection outcome

The complete result of applying specification section 12.1's precedence to an
[interface inventory](#interface-inventory): the interfaces chosen, in order,
plus every interface not chosen and the named reason it was passed over.

{: .matters }
> The second half is what the type exists for. Capturing on the wrong interface
> produces a run that exits zero, writes a well-formed capture file, and
> contains nothing, which is invisible unless the decision is reported. The
> chosen and the passed-over together must account for the whole inventory, and
> a test asserts that rather than trusting it, so a future precedence rule
> cannot drop an interface on the floor.

**See also:** [Interface inventory](#interface-inventory),
[Virtual interface](#virtual-interface)

### Virtual interface

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

**See also:** [Interface inventory](#interface-inventory),
[Selection outcome](#selection-outcome)

## Windows Internals

### ETW

**Also known as:** Event Tracing for Windows

A Windows kernel facility that emits structured events from instrumented
subsystems to registered consumers.

Providers publish events, consumers subscribe. The kernel process provider
emits an event at the moment a process is created, carrying the creating
process's identifier.

{: .matters }
> ETW supplies fragcap's [process tree](#process-tree) at creation time, which
> is the only way to get it right. Reconstructing ancestry afterward does not
> work, because Windows records a parent identifier but does not maintain it
> and recycles the values. Consuming an ETW session requires elevation.

**See also:** [Process tree](#process-tree), [PID recycling](#pid-recycling)

**References:**

- Microsoft Learn, Event Tracing for Windows. The provider and consumer model.

### IP Helper

The Windows API family exposing network configuration and connection state,
including the tables of open TCP and UDP endpoints and their owning processes.

{: .matters }
> `GetExtendedTcpTable` and `GetExtendedUdpTable` are fragcap's
> [socket table](#socket-table) source. Measurement matters here: the direct
> call costs 1 to 3 milliseconds against roughly 1800 sockets, while the
> object-model projection of the same data costs 1400 to 2000. An
> implementation reaching for the convenient interface would wrongly conclude
> that polling is unworkable.

**See also:** [Socket table](#socket-table)

### Named pipe

A Windows inter-process communication channel identified by a path under
`\\.\pipe\`, carrying a byte or message stream between processes on one host
or across a network.

{: .matters }
> Named pipes are invisible to packet capture. Reconnaissance observed one
> focal title's platform service receiving a pipe path on its command line,
> which is direct evidence for the fallback in specification section 6.2: a
> handoff over a pipe is out of scope for a network capture tool, and the
> documentation says so rather than leaving users to discover it.

**See also:** [Loopback](#loopback)

## Process and Attribution

### Attribution

Associating a captured packet with the process that sent or received it.

Capture happens at the network driver layer, below the socket layer, by which
point the operating system has discarded the association. Recovering it means
joining packets against a separately maintained record of open sockets.

{: .matters }
> Attribution is fragcap's reason to exist. Packet capture is solved;
> attribution is not.

**See also:** [Socket table](#socket-table), [5-tuple](#5-tuple)

### Socket table

The operating system's record of open network endpoints and the process
identifier owning each.

{: .matters }
> The table is sampled periodically and joined against captured packets.
> Reconnaissance measured the gap this leaves: of 12,249 connections observed
> opening and closing across two sessions, none lived less than the 250
> millisecond sampling interval, so the race window is real but lands on
> traffic that does not matter, chiefly name resolution.

**See also:** [Attribution](#attribution), [IP Helper](#ip-helper),
[Socket table entry](#socket-table-entry),
[Attribution index](#attribution-index)

**References:**

- Microsoft, `GetExtendedTcpTable` and `GetExtendedUdpTable`. The interface
  fragcap reads. The table class selects the row shape; the owning-module
  classes carry a socket creation timestamp and the owning-process classes do
  not.

### Socket table entry

One row of a [socket table](#socket-table): a protocol, a local endpoint, a
remote endpoint for TCP only, an owning process identifier, and the instant the
socket was created when the platform reports one.

The absent remote for UDP is a property of the platform interface rather than a
fragcap simplification, and specification section 8.4 forbids inventing one.

{: .matters }
> The creation instant is what tells the previous owner of a reused port from
> the current one. A socket created after a packet cannot have owned that
> packet, so an entry that postdates the packet is not a candidate at all.
> Without it, a port reassigned between two snapshots attributes the new
> owner's identity to the old owner's traffic, confidently and silently.
>
> Reconnaissance recorded the timestamp as a property of the TCP table.
> Slice S10 found it on both, which matters more for UDP: a UDP entry has no
> remote, so its key is the weakest of the two and a reused port is least
> distinguishable there.

**See also:** [Socket table](#socket-table), [5-tuple](#5-tuple),
[Attribution fidelity](#attribution-fidelity)

### Attribution index

The immutable value a lookup reads: a [socket table](#socket-table) snapshot,
the image names resolved for the process identifiers in it, and the
[retention window](#retention-window)'s map of endpoints that have left the
table.

The control thread builds a new one on each refresh and publishes it
atomically. Capture threads read the current one without locking.

{: .matters }
> Everything an answer can contain is in the index before the lookup begins.
> That is what makes attribution lookup unable to block packet acquisition:
> there is nothing on the lookup path to block on. An implementation that
> resolved an image name lazily would put an operating system call on the
> capture thread at the start of a session, which is exactly when the most
> sockets are opening at once.

**See also:** [Socket table](#socket-table), [Capture thread](#capture-thread),
[Flow attributor](#flow-attributor)

### Retention window

The grace period, defaulting to thirty seconds, during which an endpoint that
has left the [socket table](#socket-table) remains resolvable.

Measured from the instant the endpoint was last observed *present* in a table,
not from the refresh that first noticed it gone. Those differ by up to one
poll interval.

{: .matters }
> Capture and socket table observation are not synchronized. A connection
> closing produces final packets processed after the socket has gone, so
> discarding attribution the moment an endpoint disappears would leave the tail
> of every connection unattributed.
>
> The cost is that a retained answer can be wrong, in the one case where the
> port was reassigned inside the window. That is why such answers are marked:
> see [attribution fidelity](#attribution-fidelity). It is also why the origin
> is exact. Measuring from the refresh that noticed the absence would make a
> thirty second window silently thirty-one, widening the exposure without
> saying so.

**See also:** [Socket table](#socket-table),
[Attribution fidelity](#attribution-fidelity), [Attribution](#attribution)

### Refresh trigger

An event that causes the [socket table](#socket-table) to be re-read before the
poll interval elapses.

Two exist. A process start matching a profile stage triggers one immediately,
because a newly matched process is about to open sockets. An unattributed
packet on a previously unseen endpoint triggers one too, rate limited to one
per two hundred milliseconds.

{: .matters }
> The rate limit is the load-bearing half. Without it, traffic fragcap will
> never attribute, which arrives at line rate and is unattributable no matter
> how often the table is read, would drive the table read rate. The limit is
> measured in wall-clock time rather than capture time for the same reason:
> replaying an hour of traffic in one second must not request thousands of
> reads.
>
> The trigger is recorded rather than acted on, because it arrives on the
> capture thread, where reading a table is precisely what the publication
> contract forbids.

**See also:** [Attribution index](#attribution-index),
[Capture thread](#capture-thread), [Socket table](#socket-table)

### Dual-stack socket

An IPv6 socket bound to the unspecified address that also accepts IPv4 traffic,
which the socket table reports under its IPv6 bind rather than under the
address a datagram arrived on.

{: .matters }
> Matching these is a judgement call fragcap makes deliberately. Reconnaissance
> found no focal title relying on one, so the rule is unexercised by them rather
> than wrong. Refusing to match would make a whole class of sockets silently
> unattributable, and a silent unattributable class is worse than an imprecise
> match that ranks below every exact one and still requires the port to agree.

**See also:** [Wildcard bind address](#wildcard-bind-address),
[Socket table entry](#socket-table-entry)

### Process tree

The ancestry relation among processes, recorded at creation time rather than
reconstructed from current state.

{: .matters }
> Reconnaissance found chains deeper than the specification assumed: five
> levels for one focal title, six for the other with an anti-cheat launcher in
> the middle. One title runs three processes sharing a single image name and
> only the last holds sockets, so identifying the right process requires
> ancestry rather than image name. Matching on name alone binds to a process
> that never transmits and reports an empty capture as success.

**See also:** [ETW](#etw), [PID recycling](#pid-recycling),
[Launcher chain](#launcher-chain), [Stage](#stage)

### PID recycling

The reuse of a process identifier by a new, unrelated process after the
original exits.

{: .matters }
> Recycling is why a process node is keyed by the pair of operating system
> identifier and start timestamp rather than by the identifier alone, and why
> ancestry must be captured live rather than walked afterward.

**See also:** [Process tree](#process-tree),
[Synthetic process identifier](#synthetic-process-identifier)

### Synthetic process identifier

The session-local identity fragcap assigns to each process it observes, never
reused within a session.

Distinct from the operating system process identifier, which is drawn from a
reusable pool and is unique only among live processes.

{: .matters }
> The distinction is what makes the [process tree](#process-tree) correct across
> [PID recycling](#pid-recycling). The synthetic identifier is a node's
> identity; the pair of operating system identifier and timestamp is the lookup
> key into the tree. An implementation that collapses the two merges two
> unrelated processes into one node, and every descendant of the second then
> claims ancestry it does not have.

**See also:** [Process node](#process-node), [PID recycling](#pid-recycling)

**References:**

- fragcap specification section 10.2. The tree's keying rule.

### Process node

One process in the [process tree](#process-tree), carrying its operating system
identifier, its resolved parent, image path, command line, start and exit
timestamps, [ancestry provenance](#ancestry-provenance), and the profile
[stage](#stage) it is bound to where one matched.

{: .matters }
> Nodes are retained for the whole session after the process exits. Retention is
> what lets a packet arriving after its sender has terminated still be
> attributed, and specification section 5.4's observed chains are full of
> transient launchers that are already gone by the time the client matters.

**See also:** [Process tree](#process-tree),
[Synthetic process identifier](#synthetic-process-identifier),
[Ancestry provenance](#ancestry-provenance)

**References:**

- fragcap specification section 10.2. The node's fields.

### Ancestry provenance

Whether a [process node](#process-node) learned its parent from a creation event
or from the [startup snapshot](#startup-snapshot).

{: .matters }
> The two differ in how much they can be trusted, and the difference is carried
> on the node rather than derived. A parent observed at creation is unambiguous;
> one read from a running process may name an unrelated process or nothing at
> all, because Windows records a parent identifier and then neither maintains it
> nor stops reusing the values. A consumer that cannot tell them apart treats a
> guess as a measurement.

**See also:** [Process node](#process-node),
[Startup snapshot](#startup-snapshot), [PID recycling](#pid-recycling)

**References:**

- fragcap specification section 5.3. Why creation-time ancestry is the only
  reliable kind.

### Startup snapshot

The single enumeration of already-running processes fragcap takes when its
watcher starts, so that targets running before fragcap began are present in the
[process tree](#process-tree).

{: .matters }
> Taken after the event subscription, never before. Subscribing first can report
> one process twice, which the tree reconciles into a single node; snapshotting
> first leaves a window in which a process created in between is reported by
> neither source, and nothing downstream can detect that it is missing. It is
> also the only source of processes whose command line fragcap cannot obtain,
> because reading one from a running process needs a memory-read right the
> [technique denylist](#technique-denylist) forbids.

**See also:** [Process tree](#process-tree),
[Ancestry provenance](#ancestry-provenance), [ETW](#etw)

**References:**

- fragcap specification section 10.1. The snapshot establishes initial state;
  the event stream maintains it.

### Trace session

A named [ETW](#etw) collection fragcap starts for itself, carrying the kernel
process provider, and stopped when fragcap finishes.

{: .matters }
> Never the machine-wide kernel logger, which exists once per machine.
> Contending for it would make fragcap fail whenever any other tool is tracing,
> and taking it by force would make fragcap the tool that silently breaks the
> operator's other instrumentation. Windows 8 and later permit several
> concurrent system loggers, subject to a small fixed limit, and exhausting that
> limit is reported with the platform's own reason rather than worked around.

**See also:** [ETW](#etw), [Lost event](#lost-event)

**References:**

- Microsoft Learn, Configuring and Starting a SystemTraceProvider Session.

### Lost event

An event the kernel reported dropping before fragcap could read it.

{: .matters }
> A lost event is not a lost packet. A packet's loss costs that packet; a lost
> process start event removes a node and silently orphans everything beneath it.
> That is why the channel between the trace consumer and its subscribers is
> unbounded rather than a bounded drop-oldest ring, and why a
> [process tree](#process-tree) built while anything was lost reports itself
> incomplete rather than presenting as whole.

**See also:** [Trace session](#trace-session), [Process tree](#process-tree),
[Drop-oldest](#drop-oldest)

**References:**

- fragcap specification section 10.1 and constitution principles P-4 and P-9.

### Launcher chain

The sequence of processes between a user starting a game and the game client
running, typically a platform client starting a publisher launcher which
starts the client.

{: .matters }
> The chain defeats detection that waits for the game executable to appear:
> by then the authentication exchange, frequently the most information-dense
> traffic of the session, has already happened. It also contains shims that
> hold no sockets at all.

**See also:** [Process tree](#process-tree), [Stage](#stage)

### Stage

A named position in a [launcher chain](#launcher-chain) that a
[game profile](#game-profile) matches against, carrying a role and a lifecycle
class.

{: .matters }
> Stages are how fragcap stays game-agnostic while treating specific titles as
> first class. Adding support for a game means writing a TOML file, never
> modifying Rust.

**See also:** [Game profile](#game-profile),
[Launcher chain](#launcher-chain), [Lifecycle class](#lifecycle-class),
[Terminal stage](#terminal-stage), [Match predicate](#match-predicate)

### Lifecycle class

What a [stage](#stage) declares about how long its process is expected to live,
and therefore how its exit is treated: `transient` exits during the session and
that exit is normal, `session` is expected to live for the session and its exit
is significant, `service` may have been running before the session began and is
never awaited during acquisition.

{: .matters }
> Waiting for a service to start deadlocks, because it has already started. The
> class is also what makes a [terminal stage](#terminal-stage) meaningful: only
> a `session` process has an exit worth ending a capture on.

**See also:** [Stage](#stage), [Terminal stage](#terminal-stage),
[Launcher chain](#launcher-chain)

### Terminal stage

The one [stage](#stage) in a [game profile](#game-profile) whose exit ends the
capture. At most one per profile, and its [lifecycle class](#lifecycle-class) is
always `session`.

{: .matters }
> A terminal `transient` stage would end the capture at the moment a launcher
> hands off, which is the point the whole [launcher chain](#launcher-chain)
> exists to survive. Validation refuses it rather than leaving the mistake to be
> discovered in a short well-formed capture file.

**See also:** [Stage](#stage), [Lifecycle class](#lifecycle-class)

### Match predicate

One condition a [stage](#stage) tests against a process start event: `exe`, an
image name glob compared case-insensitively; `path_contains`; `path_regex`;
`cmdline_contains`; and `descends_from`, an ancestor bound to a named role. All
predicates a stage declares must hold.

`descends_from` resolves against the synthetic [process tree](#process-tree)
rather than the operating system parent chain, which is what makes it reliable
across a launcher that has already exited.

{: .matters }
> Where an image name is not unique within a chain, `descends_from` is required
> rather than advisory. See [ambiguous image match](#ambiguous-image-match) for
> what happens when it is missing.

**See also:** [Stage](#stage), [Ambiguous image match](#ambiguous-image-match),
[Process tree](#process-tree)

### Ambiguous image match

Two [stages](#stage) in one [game profile](#game-profile) whose `exe` patterns
can match a common image name, where at least one of them declares no other
[match predicate](#match-predicate). Validation refuses the profile and names
both stages.

The decision is exact rather than approximate: two patterns over `*`, `?`, and
literals either can match a common name or cannot.

{: .matters }
> A stage bound to the wrong process among several sharing an image name
> produces a capture that exits zero, is well formed, and contains no gameplay.
> One focal title runs three processes under one image name and only the last
> holds sockets, so this is a recorded case rather than a hypothetical. It is
> the configuration-side form of the loss constitution principle P-4 forbids:
> every packet is lost and none is counted.

**See also:** [Match predicate](#match-predicate), [Stage](#stage),
[Launcher chain](#launcher-chain)

### Stage matching

The decision that binds an observed process to a [stage](#stage). Each process
start event is evaluated against every stage in the active
[game profile](#game-profile), and the process binds to the first stage, in
declaration order, all of whose [match predicates](#match-predicate) hold.
Binding assigns the stage's role. Slice S12.

{: .matters }
> Matching is a decision over the [process tree](#process-tree) and the profile.
> It opens nothing and touches no platform interface, so the whole of section
> 10.3 is tested against a scripted event stream with no capture driver, no
> elevation, and no game.

**See also:** [Match predicate](#match-predicate),
[Stage binding](#stage-binding), [Capture session](#capture-session)

### Stage binding

The association of a [process node](#process-node) with the [stage](#stage) it
matched and the role that stage assigns, recorded on the node. A node binds to at
most one stage.

**See also:** [Stage matching](#stage-matching), [Stage](#stage),
[Process node](#process-node)

### Capture session

The run of one capture, moving through five states: **Arming** (opening the
capture handle and attaching the [process watcher](#process-watcher) before any
target exists), **Watching** (armed, no target matched, discarding packets),
**Capturing** (a stage has matched, packets retained), **Draining** (a
[stop condition](#stop-condition) met, buffer draining and sinks finishing), and
**Complete**. Slice S12.

{: .matters }
> Arming before the target is what keeps the launcher authentication exchange,
> which precedes the client, from being missed. The Watching to Capturing
> transition costs no setup because the handle is already open, so no traffic is
> lost at the boundary.

**See also:** [Stop condition](#stop-condition),
[Acquisition timeout](#acquisition-timeout), [Stage matching](#stage-matching)

### Acquisition timeout

The optional bound on how long a [capture session](#capture-session) waits in
Watching for a target before completing without having captured. Measured from
the instant the session was armed. When unset, the session ends instead by the
duration bound or an operator interrupt.

**See also:** [Capture session](#capture-session),
[Stop condition](#stop-condition)

### Stop condition

Any of the six events that ends a [capture session](#capture-session): the
elapsed duration bound, the byte or packet bound, the
[terminal stage](#terminal-stage) exiting, all matched non-service processes
having exited with no stage still awaited, an operator interrupt, or an
unrecoverable sink error. The first to occur wins.

{: .matters }
> Every stop condition produces the same orderly shutdown and a valid capture
> file. Uniform shutdown is what lets an operator read any capture the same way,
> including one they interrupted; an interrupt is a normal stop, not an abort.

**See also:** [Capture session](#capture-session),
[Terminal stage](#terminal-stage), [Lifecycle class](#lifecycle-class)

### Profile schema version

The `schema` key at the top of a [game profile](#game-profile), declaring which
version of the file format it is written against. Currently `1`.

A profile declaring an unsupported version is refused with one diagnostic naming
the supported version, and the rest of the file is not reported on.

{: .matters }
> The version is what makes strict key checking safe. Unknown keys are refused
> rather than ignored, because ignoring `payloads = false` written for
> `payload = false` hands the operator a capture containing contents they meant
> to exclude and says nothing. Refusing needs a way for the format to grow, and
> this is it. Reporting forty unknown-key faults when the real answer is that
> the profile is newer than the build would be misleading rather than merely
> unhelpful.

**See also:** [Game profile](#game-profile),
[Profile resolution order](#profile-resolution-order)

### Profile resolution order

The four steps by which a profile reference becomes a profile, first match
winning: an existing file at that path, then `<ref>.toml` in a profile directory
given on the command line, then `<ref>.toml` in the user profile directory, then
a bundled profile whose `game.id` matches.

A reference used in the last three steps must be a valid identifier and is
refused before any path is joined to it. An explicit path is exempt, because an
operator who types a path has named a file.

{: .matters }
> User profiles shadow bundled ones by design, so a bundled profile that has
> drifted from a game update is corrected locally without waiting for a release.
> The identifier check happens before the join rather than relying on the open
> failing, because a check that depends on what is at the target is not a check.

**See also:** [Game profile](#game-profile),
[Profile schema version](#profile-schema-version)

### Packet source

The seam that acquires packets. A live capture backend implements it in slice
S09; a replay source over recorded fixtures implements it in slice S04.

{: .matters }
> Keeping acquisition behind a trait is what makes the pipeline testable
> offline, with no capture driver, no elevation, and no game running.
> Constitution principle P-3 forbids merging it with the
> [flow attributor](#flow-attributor).

**See also:** [Flow attributor](#flow-attributor), [Sink](#sink)

### Flow attributor

The seam that resolves a [flow key](#flow-key) to the process owning it, by
matching against the [socket table](#socket-table).

Returning nothing means attempted and unresolved. The packet is retained and
marked, per constitution principle P-4, never dropped.

**See also:** [Packet source](#packet-source), [Attribution](#attribution),
[Socket table](#socket-table)

### Process watcher

The seam that reports process creation and exit, over
[ETW](#etw) kernel providers.

Ancestry comes from creation-time events rather than from inspecting a running
process, which is what lets fragcap reconstruct a [launcher
chain](#launcher-chain) without a process handle. Constitution principle P-1
forbids handles carrying memory-read rights against a target.

**See also:** [Process tree](#process-tree), [ETW](#etw),
[Launcher chain](#launcher-chain)

### Sink

The seam that accepts captured packets and writes them somewhere: a file, a
stream, or a ring buffer.

Sinks are independent of one another and of the pipeline, and a session may
have any number attached. A sink that cannot accept a packet reports it, and
the pipeline counts it in a named counter rather than aborting the capture.

**See also:** [Packet source](#packet-source), [.fcapng](#fcapng),
[Backpressure](#backpressure)

### Dissector

The seam for protocol dissection, declared in v0.2.0 with no implementations.

Fixing the shape before any protocol work begins prevents the eventual
dissector layer from being retrofitted against types that were not designed for
it.

**See also:** [Sink](#sink)

### Replay source

A [packet source](#packet-source) that reads a recorded capture file rather
than an interface. Half of what makes specification section 25.1's claim true.

Deterministic by construction: the same bytes yield the same packets, on every
run and platform. That is the property golden comparison depends on, and a test
whose input varies is a failure nobody can reproduce.

{: .matters }
> It accepts a capture filter and applies nothing, and says so. Failing would
> break a pipeline that filters unconditionally; accepting silently would let a
> test believe filtering happened. Exhaustion is reported as the terminal
> closed condition rather than as a timeout, because a timeout means keep going
> and would spin forever on a finished file.

**See also:** [Packet source](#packet-source),
[Scripted attributor](#scripted-attributor), [Fixture](#fixture)

### Scripted attributor

A [flow attributor](#flow-attributor) that answers from a declared
[attribution script](#attribution-script) rather than a
[socket table](#socket-table). The other half of the section 25.1 claim.

It matches through the same [attribution key](#attribution-key) derivation and
[wildcard bind](#wildcard-bind-address) allowance the real attributor will use,
so a test that passes against a script is one that implementation has to
satisfy. It cannot express an attribution the platform could never supply.

{: .matters }
> The attributor seam carries no timestamp, because a real attributor reads a
> table that is already current. A scripted one has to be told what "now" is,
> and that is a method on the double rather than a widening of the seam: a test
> double is a poor reason to hand every real implementation a parameter it does
> not want.

**See also:** [Flow attributor](#flow-attributor),
[Attribution script](#attribution-script), [Replay source](#replay-source)

### Attribution script

A text file declaring what a [scripted attributor](#scripted-attributor)
answers for each flow in each window of time.

The time dimension is the point. [PID recycling](#pid-recycling) and port reuse
mean one local endpoint can belong to different processes at different
instants, and without windows there is no way to test that short of a live
machine and a stopwatch.

**See also:** [Scripted attributor](#scripted-attributor),
[Fixture corpus](#fixture-corpus), [PID recycling](#pid-recycling)

### Parse outcome

What header parsing concluded about one frame: either a [flow key](#flow-key)
with an optionally determined [direction](#direction), or a named
[parse rejection cause](#parse-rejection-cause). Never silence.

An undetermined direction accompanies a successful parse rather than being a
third outcome, because the frame was understood and one property of it was
not.

**See also:** [Parse rejection cause](#parse-rejection-cause),
[Flow key](#flow-key), [Direction](#direction)

### Parse rejection cause

The specific reason a frame produced no [flow key](#flow-key). Twelve of them,
a closed set, each with its own counter.

The set is separated exactly where the remedy differs. A short header means
raise the snapshot length; a malformed header means a broken sender or a
defect in fragcap; an unsupported [EtherType](#ethertype) means unexpected
traffic; an unsupported [link type](#link-type) means an unexpected capture
backend.

{: .matters }
> A packet that produced no flow key is retained and marked, never dropped, so
> a rejection is not loss. Constitution principle P-4 requires the cause be
> named and surfaced, and the set is closed so that adding a way to decline
> without adding a counter does not compile.

**See also:** [Parse outcome](#parse-outcome),
[Parse statistics](#parse-statistics)

### Parse statistics

One counter per [parse rejection cause](#parse-rejection-cause), plus one for
an undetermined [loopback](#loopback) direction and one for a
[fragment identity table](#fragment-identity-table) eviction.

Carried beside the capture and source counters rather than folded into them,
and contributing to no drop total, because no parse outcome is a drop. There is
deliberately no counter for a successful parse: it is the captured count less
the rejections, and a stored total can drift from its parts.

**See also:** [Parse rejection cause](#parse-rejection-cause),
[Backpressure](#backpressure)

## Anti-Cheat and Security

### Anti-cheat

Software that detects or prevents unauthorized modification of a game client
or its environment.

Modern implementations observe process handles, loaded modules, memory
integrity, and inline code modification.

{: .matters }
> fragcap is designed so that nothing it does resembles what anti-cheat
> watches for. Specification section 19.3's denylist exists because those
> techniques are the primitives detection systems monitor, and because none of
> them are needed for anything fragcap claims. Reconnaissance observed an
> anti-cheat launcher in one chain; fragcap records the relationship from
> creation-time telemetry and does not interact with the process.

**See also:** [Technique denylist](#technique-denylist)

### Technique denylist

The enumerated set of techniques fragcap will not use, regardless of
convenience: packet interception drivers, code injection, function hooking,
process handles carrying memory-read rights, layered service providers, and
executable image modification.

{: .matters }
> Constitution principle P-1 makes the list absolute, and it is enforced at
> three points: as a constitution principle inherited by every agent session,
> as a dependency policy checked in continuous integration, and as a code
> review gate on process handle access rights.

**See also:** [Anti-cheat](#anti-cheat)

## Rust and Tooling

### MSRV

**Also known as:** minimum supported Rust version

The oldest toolchain release a crate compiles with, declared as
`rust-version` in its manifest.

Distinct from the toolchain a project builds with. The first is a compatibility
promise to consumers; the second is a reproducibility control for the project.

{: .matters }
> fragcap declares 1.82 while pinning its build toolchain at 1.96.0. A
> declared minimum that is never exercised is an unverified claim, so a
> dedicated check builds at the minimum. That check is currently vacuous, since
> the workspace has no external dependencies, and it says so in its own output.

**See also:** [xtask](#xtask)

### Test tier

One of the four levels specification section 25.2 defines, distinguished by
what each needs in order to run.

| Tier | Needs | In continuous integration |
| --- | --- | --- |
| 0, unit | nothing | yes |
| 1, pipeline | nothing | yes |
| 2, platform | privilege and a capture driver | yes, on a Windows runner |
| 3, live | privilege, a driver, and a game | no, manual |

{: .matters }
> Tier 1 is the one the architecture was shaped to make possible. Because a
> [replay source](#replay-source) and a [scripted
> attributor](#scripted-attributor) substitute for the two platform-dependent
> seams, the whole pipeline is testable on any machine with no privilege. That
> is the return on keeping capture and attribution apart.

**See also:** [Fixture corpus](#fixture-corpus),
[Replay source](#replay-source)

### xtask

A Cargo convention in which repository-wide tasks are implemented as a
workspace member invoked through a cargo alias, requiring nothing installed
beyond the language toolchain.

{: .matters }
> fragcap's conventions and dependency-direction checks live here rather than
> in shell scripts, because a check written in the project's own language can
> be unit tested against known-bad input. A linter whose matcher never fires is
> indistinguishable from a clean repository.

## Platform and Distribution

### npcap

A Windows packet capture driver and library, the current successor to WinPcap.

{: .matters }
> **npcap is not redistributable.** fragcap detects it rather than shipping it,
> and no distribution artifact contains it. Two non-default installation
> options are required: loopback traffic capture support and WinPcap API
> compatible mode. Both are verifiable from the registry, which is how
> `fragcap doctor` names the specific missing option.

**See also:** [Loopback](#loopback)

**References:**

- npcap project documentation, https://npcap.com. Installation options and
  license terms.

### Game profile

A TOML file describing a game's process topology, stage match rules, and
capture defaults. Versioned: every profile declares a
[profile schema version](#profile-schema-version), and a reference to one
resolves through the [profile resolution order](#profile-resolution-order).

{: .matters }
> Profiles are data, not code. They carry the same license as the repository,
> and a contributor can add support for a title without writing Rust. Validation
> reports every problem in a profile rather than stopping at the first, because
> the population writing these files is not the population that can debug a
> parser.

**See also:** [Stage](#stage), [Lifecycle class](#lifecycle-class),
[Terminal stage](#terminal-stage), [Match predicate](#match-predicate),
[Ambiguous image match](#ambiguous-image-match),
[Profile schema version](#profile-schema-version),
[Profile resolution order](#profile-resolution-order),
[Duration literal](#duration-literal)

## File and Wire Formats

### pcapng

**Also known as:** PCAP Next Generation

The block-structured capture file format that succeeded the original libpcap
format, supporting multiple interfaces, name resolution, capture statistics,
and per-block options.

{: .matters }
> Extensibility through options is what lets fragcap carry attribution in a
> file that unmodified tools still read as an ordinary capture.

**See also:** [.fcapng](#fcapng),
[Enhanced Packet Block](#enhanced-packet-block)

**References:**

- PCAP Next Generation Capture File Format specification. Block structure and
  option encoding.

### .fcapng

fragcap's extended [pcapng](#pcapng) profile, carrying process attribution in
Enhanced Packet Block options.

{: .matters }
> The governing rule is constitution principle P-5: an unmodified analyzer must
> read the file as ordinary pcapng and ignore annotations it does not
> understand. Attribution data is worth having only if the file remains a
> capture file. The annotation profile carries its own version, independent of
> the fragcap version, because the grammar and the software change on different
> schedules.

**See also:** [pcapng](#pcapng)

### Enhanced Packet Block

**Also known as:** EPB

The [pcapng](#pcapng) block carrying one captured packet with its timestamp,
captured length, original length, and options.

{: .matters }
> Its option area is where fragcap writes attribution, which is what makes the
> annotation invisible to tools that do not look for it.

**See also:** [pcapng](#pcapng), [.fcapng](#fcapng),
[Attribution annotation](#attribution-annotation)

### Section Header Block

**Also known as:** SHB

The [pcapng](#pcapng) block that opens a file, declaring byte order, format
version, and the application that wrote it.

{: .matters }
> Its byte-order magic is what makes a capture readable on a machine with the
> opposite endianness. fragcap always declares little-endian rather than host
> order: both are valid, and only one produces the same bytes for the same
> input on every architecture, which is what a golden comparison needs.

**See also:** [pcapng](#pcapng), [Golden file](#golden-file)

### Interface Description Block

**Also known as:** IDB

The [pcapng](#pcapng) block declaring one capture interface: its
[link type](#link-type), [snapshot length](#snapshot-length), name, and
timestamp resolution.

{: .matters }
> Interfaces are identified positionally, by declaration order, and every
> packet block references one by that index. An identifier with no preceding
> declaration leaves a reader with no link type, so the packet cannot be
> dissected at all.

**See also:** [pcapng](#pcapng), [Link type](#link-type),
[Snapshot length](#snapshot-length)

### Interface Statistics Block

**Also known as:** ISB

The [pcapng](#pcapng) block carrying per-interface capture counters, written at
capture end.

{: .matters }
> Its standard fields describe losses upstream of the capturing application,
> and fragcap has counters of its own that no standard field expresses.
> Constitution principle P-4 makes an unsurfaced discard a defect, and P-9
> forbids reporting a fragcap loss as an operating system loss, so those
> counters travel in a declared comment rather than being omitted or
> overloaded onto a field that means something else.

**See also:** [pcapng](#pcapng), [Backpressure](#backpressure)

### Attribution annotation

The structured string fragcap writes into an
[Enhanced Packet Block](#enhanced-packet-block) comment, carrying the process
that produced a packet, its [direction](#direction), and its
[attribution fidelity](#attribution-fidelity).

The grammar is a `fragcap:` sentinel followed by semicolon-separated key and
value pairs, with values percent-encoded where they would otherwise break the
grammar or the containing format.

{: .matters }
> A comment rather than a custom option, deliberately. Every pcapng reader
> displays comments, so attribution is visible in an unmodified analyzer with
> no configuration, which is constitution principle P-5 in its practical form.
> Custom options would also require a Private Enterprise Number this project
> does not hold. The cost is parsing overhead in consumers and a modest size
> increase, and both are accepted.

**See also:** [.fcapng](#fcapng),
[Attribution fidelity](#attribution-fidelity),
[Enhanced Packet Block](#enhanced-packet-block)

**References:**

- fragcap specification section 13.3. Grammar, key table, and the reasoning
  for choosing comments over custom options.

### Attribution fidelity

How an [attribution](#attribution) was obtained: from the live
[socket table](#socket-table), from the retention window after the socket left
it, or not at all. Written as the `attr` key of an
[attribution annotation](#attribution-annotation).

{: .matters }
> Retained attribution is inferential rather than observed: an endpoint that
> closed and was reassigned to a different process inside the retention window
> attributes incorrectly. Recording which packets are exposed to that is what
> lets analysis discount them. A consumer never infers this value, because the
> distinction between an observation and an inference is precisely what a
> reader cannot reconstruct from the data.

**See also:** [Attribution](#attribution),
[Attribution annotation](#attribution-annotation),
[Socket table](#socket-table), [PID recycling](#pid-recycling)

**References:**

- fragcap specification section 13.4.

### Golden file

A committed file of expected output, reviewed once by a human and compared
mechanically on every run afterward.

fragcap keeps one per [fixture](#fixture), holding the exact bytes the writer
produces for it.

{: .matters }
> Tests written alongside an implementation encode the author's assumptions,
> including the wrong ones. A golden encodes what the code actually produced on
> a day somebody looked, so a later change is visible to a reviewer who was not
> there. This only works if output is deterministic, which is why the writer
> reads no clock and fixes its byte order: a golden that legitimately varies is
> a golden that gets deleted the first time it fails.

**See also:** [Fixture corpus](#fixture-corpus), [Fixture](#fixture)

### JSON Lines

**Also known as:** JSONL, newline-delimited JSON, NDJSON

One JSON object per line, with no enclosing array and no separators between
records.

{: .matters }
> The property that matters is that a line is self-contained: a stream can be
> split, tailed, filtered, or truncated with ordinary line tools and every
> surviving line is still a complete record. fragcap writes it for consumers
> that do not read [pcapng](#pcapng), and it drives the differences from the
> [.fcapng](#fcapng) profile. The interface name appears on every record here
> and only in multi-interface captures there, because a pcapng file holds the
> interface in its container and a line has no container to hold it.

**See also:** [pcapng](#pcapng), [Trailer record](#trailer-record),
[Payload-free mode](#payload-free-mode)

**References:**

- fragcap specification section 13.5.

### Trailer record

The final object of a [JSON Lines](#json-lines) stream, carrying the capture's
statistics. Distinguished from a packet record by a `type` key that packet
records never carry.

{: .matters }
> Its absence is the only way a consumer can tell a truncated stream from a
> complete one, which makes it load-bearing rather than decorative. It carries
> every counter even when zero, so that "nothing was lost" is distinguishable
> from "this build does not report that": the same reasoning that puts the
> counters in an [Interface Statistics Block](#interface-statistics-block) for
> the other format, and the reason constitution principle P-4 is satisfied for
> a consumer who never sees the pcapng file.

**See also:** [JSON Lines](#json-lines),
[Interface Statistics Block](#interface-statistics-block)

### Payload-free mode

A [JSON Lines](#json-lines) stream that omits packet payloads, producing
metadata suitable for flow analysis at a fraction of the volume.

{: .matters }
> The key is omitted entirely rather than emitted empty, because an empty
> payload is a real observation that renders as an empty string. A consumer
> distinguishes the two by the length fields, which are present in both modes.

**See also:** [JSON Lines](#json-lines)

### Percent-encoding

Representing a character as a percent sign followed by two hexadecimal digits
per byte of its UTF-8 encoding.

fragcap applies it inside an [attribution annotation](#attribution-annotation)
to the characters that carry meaning in the grammar, and to control characters,
which would otherwise break the comment that contains them.

{: .matters }
> Lossless and reversible, which is why widening the escaped set beyond the
> grammar's own reserved characters does not conflict with constitution
> principle P-9. The alternative for a process name containing a newline would
> be stripping or replacing it, which alters the observation.

**See also:** [Attribution annotation](#attribution-annotation)

## Command Line and Diagnostics

### Readiness check

One line of the `fragcap doctor` report: a section, a name, a detail, a status,
and, when it fails, a remediation. The status vocabulary is exactly four words:
**ok** (ready), **warn** (a non-blocking concern), **skip** (not applicable or
not built into this binary), and **fail** (a blocking problem that must be fixed
before capture is possible). The report exits 1 if any check is `fail` and 0
otherwise.

{: .matters }
> `skip` and `fail` are deliberately distinct. A process-tracing session that is
> not built into the binary is a `skip`, because attribution still works from
> the socket table; a session that could not open while elevated is a `fail`,
> because attribution is then degraded. Collapsing the two would either block a
> capture that would have worked or pass one that will not.

**See also:** [npcap](#npcap), [Attribution fidelity](#attribution-fidelity)

### Lifecycle event

One record in the machine-readable event stream `fragcap` emits on standard
error under `--json`, over a capture's life. There are five: **session.armed**
(the handle is open and the watcher attached), **stage.matched** (a stage bound
a process), **stage.exited** (a bound process exited), **filter.narrowed** (the
capture filter narrowed to a set of active endpoints), and **session.complete**
(the run ended, carrying the headline counters). Each carries an RFC3339 `Z`
timestamp.

{: .matters }
> The event stream is what lets a wrapper react to a capture without parsing
> human-readable progress, which is what keeps a wrapper thin under constitution
> principle P-7. It is newline-delimited JSON on standard error, so capture data
> written to a sink, even one on standard output, is never contaminated by it.

**See also:** [Completion summary](#completion-summary)

### Completion summary

The end-of-run accounting an operator reads: the captured and attributed counts,
the stop reason, and every discard counter, the packets discarded while watching
before a target was acquired, those discarded out of the capture window, buffer
drops, and per-sink drops.

{: .matters }
> The summary surfaces the counters the pipeline and session already maintain
> and invents none, which is what constitution principle P-4 requires: a bare
> success that hid a watch-time discard or a buffer drop is exactly the silent
> loss the principle forbids.

**See also:** [Lifecycle event](#lifecycle-event)

### Effective configuration

The capture options actually used, formed by overlaying the command-line options
onto a profile's `[capture]` defaults. The command line wins, and an option
absent from both stays absent, so a profile that chose a value and a profile that
said nothing remain distinguishable.

{: .matters }
> The overlay preserves the declared-versus-absent distinction the profile schema
> depends on, rather than substituting a default the moment a value is missing.
> Substituting one would destroy the information an operator supplied and make a
> later override behave differently than they wrote.

**See also:** [Game profile](#game-profile), [Completion summary](#completion-summary)
