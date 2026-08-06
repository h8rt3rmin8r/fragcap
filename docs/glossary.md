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

**See also:** [.fcapng](#fcapng), [Enhanced Packet Block](#enhanced-packet-block)

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
