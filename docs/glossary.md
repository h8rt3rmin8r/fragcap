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

**See also:** [Attribution](#attribution), [IP Helper](#ip-helper)

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

**See also:** [Process tree](#process-tree)

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

**See also:** [Game profile](#game-profile), [Launcher chain](#launcher-chain)

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

The seam for protocol dissection, declared in v0.1.0 with no implementations.

Fixing the shape before any protocol work begins prevents the eventual
dissector layer from being retrofitted against types that were not designed for
it.

**See also:** [Sink](#sink)

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
capture defaults.

{: .matters }
> Profiles are data, not code. They carry the same license as the repository,
> and a contributor can add support for a title without writing Rust.

**See also:** [Stage](#stage)

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

**See also:** [pcapng](#pcapng), [.fcapng](#fcapng)
