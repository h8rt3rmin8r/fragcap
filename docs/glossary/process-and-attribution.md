# Process and Attribution

## Attribution

Associating a captured packet with the process that sent or received it.

Capture happens at the network driver layer, below the socket layer, by which
point the operating system has discarded the association. Recovering it means
joining packets against a separately maintained record of open sockets.

{: .matters }
> Attribution is fragcap's reason to exist. Packet capture is solved;
> attribution is not.

**See also:** [Socket table](process-and-attribution.md#socket-table), [5-tuple](capture-and-networking.md#5-tuple)

## Socket table

The operating system's record of open network endpoints and the process
identifier owning each.

{: .matters }
> The table is sampled periodically and joined against captured packets.
> Reconnaissance measured the gap this leaves: of 12,249 connections observed
> opening and closing across two sessions, none lived less than the 250
> millisecond sampling interval, so the race window is real but lands on
> traffic that does not matter, chiefly name resolution.

**See also:** [Attribution](process-and-attribution.md#attribution), [IP Helper](windows-internals.md#ip-helper),
[Socket table entry](process-and-attribution.md#socket-table-entry),
[Attribution index](process-and-attribution.md#attribution-index)

**References:**

- Microsoft, `GetExtendedTcpTable` and `GetExtendedUdpTable`. The interface
  fragcap reads. The table class selects the row shape; the owning-module
  classes carry a socket creation timestamp and the owning-process classes do
  not.

## Socket table entry

One row of a [socket table](process-and-attribution.md#socket-table): a protocol, a local endpoint, a
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

**See also:** [Socket table](process-and-attribution.md#socket-table), [5-tuple](capture-and-networking.md#5-tuple),
[Attribution fidelity](file-and-wire-formats.md#attribution-fidelity)

## Attribution index

The immutable value a lookup reads: a [socket table](process-and-attribution.md#socket-table) snapshot,
the image names resolved for the process identifiers in it, and the
[retention window](process-and-attribution.md#retention-window)'s map of endpoints that have left the
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

**See also:** [Socket table](process-and-attribution.md#socket-table), [Capture thread](capture-and-networking.md#capture-thread),
[Flow attributor](process-and-attribution.md#flow-attributor)

## Retention window

The grace period, defaulting to thirty seconds, during which an endpoint that
has left the [socket table](process-and-attribution.md#socket-table) remains resolvable.

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
> see [attribution fidelity](file-and-wire-formats.md#attribution-fidelity). It is also why the origin
> is exact. Measuring from the refresh that noticed the absence would make a
> thirty second window silently thirty-one, widening the exposure without
> saying so.

**See also:** [Socket table](process-and-attribution.md#socket-table),
[Attribution fidelity](file-and-wire-formats.md#attribution-fidelity), [Attribution](process-and-attribution.md#attribution)

## Refresh trigger

An event that causes the [socket table](process-and-attribution.md#socket-table) to be re-read before the
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

**See also:** [Attribution index](process-and-attribution.md#attribution-index),
[Capture thread](capture-and-networking.md#capture-thread), [Socket table](process-and-attribution.md#socket-table)

## Dual-stack socket

An IPv6 socket bound to the unspecified address that also accepts IPv4 traffic,
which the socket table reports under its IPv6 bind rather than under the
address a datagram arrived on.

{: .matters }
> Matching these is a judgement call fragcap makes deliberately. Reconnaissance
> found no focal title relying on one, so the rule is unexercised by them rather
> than wrong. Refusing to match would make a whole class of sockets silently
> unattributable, and a silent unattributable class is worse than an imprecise
> match that ranks below every exact one and still requires the port to agree.

**See also:** [Wildcard bind address](capture-and-networking.md#wildcard-bind-address),
[Socket table entry](process-and-attribution.md#socket-table-entry)

## Process tree

The ancestry relation among processes, recorded at creation time rather than
reconstructed from current state.

{: .matters }
> Reconnaissance found chains deeper than the specification assumed: five
> levels for one focal title, six for the other with an anti-cheat launcher in
> the middle. One title runs three processes sharing a single image name and
> only the last holds sockets, so identifying the right process requires
> ancestry rather than image name. Matching on name alone binds to a process
> that never transmits and reports an empty capture as success.

**See also:** [ETW](windows-internals.md#etw), [PID recycling](process-and-attribution.md#pid-recycling),
[Launcher chain](process-and-attribution.md#launcher-chain), [Stage](process-and-attribution.md#stage)

## PID recycling

The reuse of a process identifier by a new, unrelated process after the
original exits.

{: .matters }
> Recycling is why a process node is keyed by the pair of operating system
> identifier and start timestamp rather than by the identifier alone, and why
> ancestry must be captured live rather than walked afterward.

**See also:** [Process tree](process-and-attribution.md#process-tree),
[Synthetic process identifier](process-and-attribution.md#synthetic-process-identifier)

## Synthetic process identifier

The session-local identity fragcap assigns to each process it observes, never
reused within a session.

Distinct from the operating system process identifier, which is drawn from a
reusable pool and is unique only among live processes.

{: .matters }
> The distinction is what makes the [process tree](process-and-attribution.md#process-tree) correct across
> [PID recycling](process-and-attribution.md#pid-recycling). The synthetic identifier is a node's
> identity; the pair of operating system identifier and timestamp is the lookup
> key into the tree. An implementation that collapses the two merges two
> unrelated processes into one node, and every descendant of the second then
> claims ancestry it does not have.

**See also:** [Process node](process-and-attribution.md#process-node), [PID recycling](process-and-attribution.md#pid-recycling)

**References:**

- fragcap specification section 10.2. The tree's keying rule.

## Process node

One process in the [process tree](process-and-attribution.md#process-tree), carrying its operating system
identifier, its resolved parent, image path, command line, start and exit
timestamps, [ancestry provenance](process-and-attribution.md#ancestry-provenance), and the profile
[stage](process-and-attribution.md#stage) it is bound to where one matched.

{: .matters }
> Nodes are retained for the whole session after the process exits. Retention is
> what lets a packet arriving after its sender has terminated still be
> attributed, and specification section 5.4's observed chains are full of
> transient launchers that are already gone by the time the client matters.

**See also:** [Process tree](process-and-attribution.md#process-tree),
[Synthetic process identifier](process-and-attribution.md#synthetic-process-identifier),
[Ancestry provenance](process-and-attribution.md#ancestry-provenance)

**References:**

- fragcap specification section 10.2. The node's fields.

## Ancestry provenance

Whether a [process node](process-and-attribution.md#process-node) learned its parent from a creation event
or from the [startup snapshot](process-and-attribution.md#startup-snapshot).

{: .matters }
> The two differ in how much they can be trusted, and the difference is carried
> on the node rather than derived. A parent observed at creation is unambiguous;
> one read from a running process may name an unrelated process or nothing at
> all, because Windows records a parent identifier and then neither maintains it
> nor stops reusing the values. A consumer that cannot tell them apart treats a
> guess as a measurement.

**See also:** [Process node](process-and-attribution.md#process-node),
[Startup snapshot](process-and-attribution.md#startup-snapshot), [PID recycling](process-and-attribution.md#pid-recycling)

**References:**

- fragcap specification section 5.3. Why creation-time ancestry is the only
  reliable kind.

## Startup snapshot

The single enumeration of already-running processes fragcap takes when its
watcher starts, so that targets running before fragcap began are present in the
[process tree](process-and-attribution.md#process-tree).

{: .matters }
> Taken after the event subscription, never before. Subscribing first can report
> one process twice, which the tree reconciles into a single node; snapshotting
> first leaves a window in which a process created in between is reported by
> neither source, and nothing downstream can detect that it is missing. It is
> also the only source of processes whose command line fragcap cannot obtain,
> because reading one from a running process needs a memory-read right the
> [technique denylist](anti-cheat-and-security.md#technique-denylist) forbids.

**See also:** [Process tree](process-and-attribution.md#process-tree),
[Ancestry provenance](process-and-attribution.md#ancestry-provenance), [ETW](windows-internals.md#etw)

**References:**

- fragcap specification section 10.1. The snapshot establishes initial state;
  the event stream maintains it.

## Trace session

A named [ETW](windows-internals.md#etw) collection fragcap starts for itself, carrying the kernel
process provider, and stopped when fragcap finishes.

{: .matters }
> Never the machine-wide kernel logger, which exists once per machine.
> Contending for it would make fragcap fail whenever any other tool is tracing,
> and taking it by force would make fragcap the tool that silently breaks the
> operator's other instrumentation. Windows 8 and later permit several
> concurrent system loggers, subject to a small fixed limit, and exhausting that
> limit is reported with the platform's own reason rather than worked around.

**See also:** [ETW](windows-internals.md#etw), [Lost event](process-and-attribution.md#lost-event)

**References:**

- Microsoft Learn, Configuring and Starting a SystemTraceProvider Session.

## Lost event

An event the kernel reported dropping before fragcap could read it.

{: .matters }
> A lost event is not a lost packet. A packet's loss costs that packet; a lost
> process start event removes a node and silently orphans everything beneath it.
> That is why the channel between the trace consumer and its subscribers is
> unbounded rather than a bounded drop-oldest ring, and why a
> [process tree](process-and-attribution.md#process-tree) built while anything was lost reports itself
> incomplete rather than presenting as whole.

**See also:** [Trace session](process-and-attribution.md#trace-session), [Process tree](process-and-attribution.md#process-tree),
[Drop-oldest](capture-and-networking.md#drop-oldest)

**References:**

- fragcap specification section 10.1 and constitution principles P-4 and P-9.

## Launcher chain

The sequence of processes between a user starting a game and the game client
running, typically a platform client starting a publisher launcher which
starts the client.

{: .matters }
> The chain defeats detection that waits for the game executable to appear:
> by then the authentication exchange, frequently the most information-dense
> traffic of the session, has already happened. It also contains shims that
> hold no sockets at all.

**See also:** [Process tree](process-and-attribution.md#process-tree), [Stage](process-and-attribution.md#stage)

## Stage

A named position in a [launcher chain](process-and-attribution.md#launcher-chain) that a
[game profile](platform-and-distribution.md#game-profile) matches against, carrying a role and a lifecycle
class.

{: .matters }
> Stages are how fragcap stays game-agnostic while treating specific titles as
> first class. Adding support for a game means writing a TOML file, never
> modifying Rust.

**See also:** [Game profile](platform-and-distribution.md#game-profile),
[Launcher chain](process-and-attribution.md#launcher-chain), [Lifecycle class](process-and-attribution.md#lifecycle-class),
[Terminal stage](process-and-attribution.md#terminal-stage), [Match predicate](process-and-attribution.md#match-predicate)

## Lifecycle class

What a [stage](process-and-attribution.md#stage) declares about how long its process is expected to live,
and therefore how its exit is treated: `transient` exits during the session and
that exit is normal, `session` is expected to live for the session and its exit
is significant, `service` may have been running before the session began and is
never awaited during acquisition.

{: .matters }
> Waiting for a service to start deadlocks, because it has already started. The
> class is also what makes a [terminal stage](process-and-attribution.md#terminal-stage) meaningful: only
> a `session` process has an exit worth ending a capture on.

**See also:** [Stage](process-and-attribution.md#stage), [Terminal stage](process-and-attribution.md#terminal-stage),
[Launcher chain](process-and-attribution.md#launcher-chain)

## Terminal stage

The one [stage](process-and-attribution.md#stage) in a [game profile](platform-and-distribution.md#game-profile) whose exit ends the
capture. At most one per profile, and its [lifecycle class](process-and-attribution.md#lifecycle-class) is
always `session`.

{: .matters }
> A terminal `transient` stage would end the capture at the moment a launcher
> hands off, which is the point the whole [launcher chain](process-and-attribution.md#launcher-chain)
> exists to survive. Validation refuses it rather than leaving the mistake to be
> discovered in a short well-formed capture file.

**See also:** [Stage](process-and-attribution.md#stage), [Lifecycle class](process-and-attribution.md#lifecycle-class)

## Match predicate

One condition a [stage](process-and-attribution.md#stage) tests against a process start event: `exe`, an
image name glob compared case-insensitively; `path_contains`; `path_regex`;
`cmdline_contains`; and `descends_from`, an ancestor bound to a named role. All
predicates a stage declares must hold.

`descends_from` resolves against the synthetic [process tree](process-and-attribution.md#process-tree)
rather than the operating system parent chain, which is what makes it reliable
across a launcher that has already exited.

{: .matters }
> Where an image name is not unique within a chain, `descends_from` is required
> rather than advisory. See [ambiguous image match](process-and-attribution.md#ambiguous-image-match) for
> what happens when it is missing.

**See also:** [Stage](process-and-attribution.md#stage), [Ambiguous image match](process-and-attribution.md#ambiguous-image-match),
[Process tree](process-and-attribution.md#process-tree)

## Ambiguous image match

Two [stages](process-and-attribution.md#stage) in one [game profile](platform-and-distribution.md#game-profile) whose `exe` patterns
can match a common image name, where at least one of them declares no other
[match predicate](process-and-attribution.md#match-predicate). Validation refuses the profile and names
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

**See also:** [Match predicate](process-and-attribution.md#match-predicate), [Stage](process-and-attribution.md#stage),
[Launcher chain](process-and-attribution.md#launcher-chain)

## Stage matching

The decision that binds an observed process to a [stage](process-and-attribution.md#stage). Each process
start event is evaluated against every stage in the active
[game profile](platform-and-distribution.md#game-profile), and the process binds to the first stage, in
declaration order, all of whose [match predicates](process-and-attribution.md#match-predicate) hold.
Binding assigns the stage's role. Slice S12.

{: .matters }
> Matching is a decision over the [process tree](process-and-attribution.md#process-tree) and the profile.
> It opens nothing and touches no platform interface, so the whole of section
> 10.3 is tested against a scripted event stream with no capture driver, no
> elevation, and no game.

**See also:** [Match predicate](process-and-attribution.md#match-predicate),
[Stage binding](process-and-attribution.md#stage-binding), [Capture session](process-and-attribution.md#capture-session)

## Stage binding

The association of a [process node](process-and-attribution.md#process-node) with the [stage](process-and-attribution.md#stage) it
matched and the role that stage assigns, recorded on the node. A node binds to at
most one stage.

**See also:** [Stage matching](process-and-attribution.md#stage-matching), [Stage](process-and-attribution.md#stage),
[Process node](process-and-attribution.md#process-node)

## Capture session

The run of one capture, moving through five states: **Arming** (opening the
capture handle and attaching the [process watcher](process-and-attribution.md#process-watcher) before any
target exists), **Watching** (armed, no target matched, discarding packets),
**Capturing** (a stage has matched, packets retained), **Draining** (a
[stop condition](process-and-attribution.md#stop-condition) met, buffer draining and sinks finishing), and
**Complete**. Slice S12.

{: .matters }
> Arming before the target is what keeps the launcher authentication exchange,
> which precedes the client, from being missed. The Watching to Capturing
> transition costs no setup because the handle is already open, so no traffic is
> lost at the boundary.

**See also:** [Stop condition](process-and-attribution.md#stop-condition),
[Acquisition timeout](process-and-attribution.md#acquisition-timeout), [Stage matching](process-and-attribution.md#stage-matching)

## Acquisition timeout

The optional bound on how long a [capture session](process-and-attribution.md#capture-session) waits in
Watching for a target before completing without having captured. Measured from
the instant the session was armed. When unset, the session ends instead by the
duration bound or an operator interrupt.

**See also:** [Capture session](process-and-attribution.md#capture-session),
[Stop condition](process-and-attribution.md#stop-condition)

## Stop condition

Any of the six events that ends a [capture session](process-and-attribution.md#capture-session): the
elapsed duration bound, the byte or packet bound, the
[terminal stage](process-and-attribution.md#terminal-stage) exiting, all matched non-service processes
having exited with no stage still awaited, an operator interrupt, or an
unrecoverable sink error. The first to occur wins.

{: .matters }
> Every stop condition produces the same orderly shutdown and a valid capture
> file. Uniform shutdown is what lets an operator read any capture the same way,
> including one they interrupted; an interrupt is a normal stop, not an abort.

**See also:** [Capture session](process-and-attribution.md#capture-session),
[Terminal stage](process-and-attribution.md#terminal-stage), [Lifecycle class](process-and-attribution.md#lifecycle-class)

## Profile schema version

The `schema` key at the top of a [game profile](platform-and-distribution.md#game-profile), declaring which
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

**See also:** [Game profile](platform-and-distribution.md#game-profile),
[Profile resolution order](process-and-attribution.md#profile-resolution-order)

## Profile resolution order

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

**See also:** [Game profile](platform-and-distribution.md#game-profile),
[Profile schema version](process-and-attribution.md#profile-schema-version)

## Packet source

The seam that acquires packets. A live capture backend implements it in slice
S09; a replay source over recorded fixtures implements it in slice S04.

{: .matters }
> Keeping acquisition behind a trait is what makes the pipeline testable
> offline, with no capture driver, no elevation, and no game running.
> Constitution principle P-3 forbids merging it with the
> [flow attributor](process-and-attribution.md#flow-attributor).

**See also:** [Flow attributor](process-and-attribution.md#flow-attributor), [Sink](process-and-attribution.md#sink)

## Flow attributor

The seam that resolves a [flow key](capture-and-networking.md#flow-key) to the process owning it, by
matching against the [socket table](process-and-attribution.md#socket-table).

Returning nothing means attempted and unresolved. The packet is retained and
marked, per constitution principle P-4, never dropped.

**See also:** [Packet source](process-and-attribution.md#packet-source), [Attribution](process-and-attribution.md#attribution),
[Socket table](process-and-attribution.md#socket-table)

## Process watcher

The seam that reports process creation and exit, over
[ETW](windows-internals.md#etw) kernel providers.

Ancestry comes from creation-time events rather than from inspecting a running
process, which is what lets fragcap reconstruct a [launcher
chain](process-and-attribution.md#launcher-chain) without a process handle. Constitution principle P-1
forbids handles carrying memory-read rights against a target.

**See also:** [Process tree](process-and-attribution.md#process-tree), [ETW](windows-internals.md#etw),
[Launcher chain](process-and-attribution.md#launcher-chain)

## Sink

The seam that accepts captured packets and writes them somewhere: a file, a
stream, or a ring buffer.

Sinks are independent of one another and of the pipeline, and a session may
have any number attached. A sink that cannot accept a packet reports it, and
the pipeline counts it in a named counter rather than aborting the capture.

**See also:** [Packet source](process-and-attribution.md#packet-source), [.fcapng](file-and-wire-formats.md#fcapng),
[Backpressure](capture-and-networking.md#backpressure)

## Dissector

The seam for protocol dissection, declared in v0.2.0 with no implementations.

Fixing the shape before any protocol work begins prevents the eventual
dissector layer from being retrofitted against types that were not designed for
it.

**See also:** [Sink](process-and-attribution.md#sink)

## Replay source

A [packet source](process-and-attribution.md#packet-source) that reads a recorded capture file rather
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

**See also:** [Packet source](process-and-attribution.md#packet-source),
[Scripted attributor](process-and-attribution.md#scripted-attributor), [Fixture](capture-and-networking.md#fixture)

## Scripted attributor

A [flow attributor](process-and-attribution.md#flow-attributor) that answers from a declared
[attribution script](process-and-attribution.md#attribution-script) rather than a
[socket table](process-and-attribution.md#socket-table). The other half of the section 25.1 claim.

It matches through the same [attribution key](capture-and-networking.md#attribution-key) derivation and
[wildcard bind](capture-and-networking.md#wildcard-bind-address) allowance the real attributor will use,
so a test that passes against a script is one that implementation has to
satisfy. It cannot express an attribution the platform could never supply.

{: .matters }
> The attributor seam carries no timestamp, because a real attributor reads a
> table that is already current. A scripted one has to be told what "now" is,
> and that is a method on the double rather than a widening of the seam: a test
> double is a poor reason to hand every real implementation a parameter it does
> not want.

**See also:** [Flow attributor](process-and-attribution.md#flow-attributor),
[Attribution script](process-and-attribution.md#attribution-script), [Replay source](process-and-attribution.md#replay-source)

## Attribution script

A text file declaring what a [scripted attributor](process-and-attribution.md#scripted-attributor)
answers for each flow in each window of time.

The time dimension is the point. [PID recycling](process-and-attribution.md#pid-recycling) and port reuse
mean one local endpoint can belong to different processes at different
instants, and without windows there is no way to test that short of a live
machine and a stopwatch.

**See also:** [Scripted attributor](process-and-attribution.md#scripted-attributor),
[Fixture corpus](capture-and-networking.md#fixture-corpus), [PID recycling](process-and-attribution.md#pid-recycling)

## Parse outcome

What header parsing concluded about one frame: either a [flow key](capture-and-networking.md#flow-key)
with an optionally determined [direction](capture-and-networking.md#direction), or a named
[parse rejection cause](process-and-attribution.md#parse-rejection-cause). Never silence.

An undetermined direction accompanies a successful parse rather than being a
third outcome, because the frame was understood and one property of it was
not.

**See also:** [Parse rejection cause](process-and-attribution.md#parse-rejection-cause),
[Flow key](capture-and-networking.md#flow-key), [Direction](capture-and-networking.md#direction)

## Parse rejection cause

The specific reason a frame produced no [flow key](capture-and-networking.md#flow-key). Twelve of them,
a closed set, each with its own counter.

The set is separated exactly where the remedy differs. A short header means
raise the snapshot length; a malformed header means a broken sender or a
defect in fragcap; an unsupported [EtherType](capture-and-networking.md#ethertype) means unexpected
traffic; an unsupported [link type](capture-and-networking.md#link-type) means an unexpected capture
backend.

{: .matters }
> A packet that produced no flow key is retained and marked, never dropped, so
> a rejection is not loss. Constitution principle P-4 requires the cause be
> named and surfaced, and the set is closed so that adding a way to decline
> without adding a counter does not compile.

**See also:** [Parse outcome](process-and-attribution.md#parse-outcome),
[Parse statistics](process-and-attribution.md#parse-statistics)

## Parse statistics

One counter per [parse rejection cause](process-and-attribution.md#parse-rejection-cause), plus one for
an undetermined [loopback](capture-and-networking.md#loopback) direction and one for a
[fragment identity table](capture-and-networking.md#fragment-identity-table) eviction.

Carried beside the capture and source counters rather than folded into them,
and contributing to no drop total, because no parse outcome is a drop. There is
deliberately no counter for a successful parse: it is the captured count less
the rejections, and a stored total can drift from its parts.

**See also:** [Parse rejection cause](process-and-attribution.md#parse-rejection-cause),
[Backpressure](capture-and-networking.md#backpressure)

## Master target schema

The single versioned JSON Schema (Draft 2020-12) that governs every
machine-readable targeting and attribution artifact: a [game profile](platform-and-distribution.md#game-profile),
a [target hint record](#target-hint-record), a user-authored package, and a
hint-database export. Introduced by issue #75. Embedded in the binary as the
single source of truth, published under `docs/schema/`, and validated one-off
with `fragcap schema validate`.

{: .matters }
> The schema expresses structural conformance only: types, required keys, enum
> ranges, unknown-key refusal, and the `kind` and `schema` discriminators. The
> semantic invariants of profile validation (acyclic ancestry, at most one
> terminal stage, role reachability, no ambiguous image match) are not
> expressible in a schema and stay in the profile-load path. A document that
> passes `schema validate` asserts structural conformance, nothing more.

**See also:** [Target artifact kind](#target-artifact-kind), [Fidelity tier](#fidelity-tier),
[Provenance](#provenance), [Game profile](platform-and-distribution.md#game-profile)

## Target artifact kind

The closed discriminator on every artifact governed by the
[master target schema](#master-target-schema). One of `profile` (the strict,
authoritative description the pipeline runs against), `package` (a hand-authored
or community-submitted profile, highest precedence), `hint` (a loose, partial
heuristic guess), or `export` (the JSON projection of hint-database rows).
Profile and package share one strict shape; hint and export share one loose
shape.

**See also:** [Master target schema](#master-target-schema), [Target hint record](#target-hint-record)

## Target hint record

A loose, partial artifact emitted by a heuristic provider or the hint database.
It may omit fields a [game profile](platform-and-distribution.md#game-profile) requires, but it
MUST carry a [fidelity tier](#fidelity-tier) and [provenance](#provenance). A
hint that does not declare its trust level is refused: an undeclared guess is
exactly the guess-worn-as-fact the schema exists to prevent.

**See also:** [Fidelity tier](#fidelity-tier), [Provenance](#provenance),
[Target artifact kind](#target-artifact-kind)

## Fidelity tier

The structured, ordered trust level carried by every targeting artifact:
`authored` (a person wrote it), `verified` (confirmed correct),
`heuristic-unverified` (a machine guessed it), or `observed` (confirmed against
a live capture). The resolver reads it; the instrument never fabricates it.

{: .matters }
> Fidelity is data, not a comment, precisely so the tool can act on it: refuse
> to treat a heuristic as verified, surface it to the operator, and gate a
> submission. Constitution principle P-9 requires that a guess be presentable as
> a guess and never as a fact.

**See also:** [Master target schema](#master-target-schema), [Provenance](#provenance)

## Provenance

The structured record of where a targeting artifact came from: a `source` (for
example `steam-appinfo`, `engine-rule`, or `user`) and an optional seed time.
Required on a [target hint record](#target-hint-record) and on a hint-database
export, so an unverified artifact always names its origin.

**See also:** [Fidelity tier](#fidelity-tier), [Target hint record](#target-hint-record)
