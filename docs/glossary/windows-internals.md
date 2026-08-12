# Windows Internals

## ETW

**Also known as:** Event Tracing for Windows

A Windows kernel facility that emits structured events from instrumented
subsystems to registered consumers.

Providers publish events, consumers subscribe. The kernel process provider
emits an event at the moment a process is created, carrying the creating
process's identifier.

{: .matters }
> ETW supplies fragcap's [process tree](/docs/glossary/process-and-attribution#process-tree) at creation time, which
> is the only way to get it right. Reconstructing ancestry afterward does not
> work, because Windows records a parent identifier but does not maintain it
> and recycles the values. Consuming an ETW session requires elevation.

**See also:** [Process tree](/docs/glossary/process-and-attribution#process-tree), [PID recycling](/docs/glossary/process-and-attribution#pid-recycling)

**References:**

- Microsoft Learn, Event Tracing for Windows. The provider and consumer model.

## IP Helper

The Windows API family exposing network configuration and connection state,
including the tables of open TCP and UDP endpoints and their owning processes.

{: .matters }
> `GetExtendedTcpTable` and `GetExtendedUdpTable` are fragcap's
> [socket table](/docs/glossary/process-and-attribution#socket-table) source. Measurement matters here: the direct
> call costs 1 to 3 milliseconds against roughly 1800 sockets, while the
> object-model projection of the same data costs 1400 to 2000. An
> implementation reaching for the convenient interface would wrongly conclude
> that polling is unworkable.

**See also:** [Socket table](/docs/glossary/process-and-attribution#socket-table)

## Named pipe

**Also known as:** FIFO

A Windows inter-process communication channel identified by a path under
`\\.\pipe\`, carrying a byte or message stream between processes on one host
or across a network. The Unix equivalent, a named FIFO, plays the same role for
the [extcap](/docs/glossary/windows-internals#extcap) stream on non-Windows hosts.

{: .matters }
> Named pipes are invisible to packet capture. Reconnaissance observed one
> focal title's platform service receiving a pipe path on its command line,
> which is direct evidence for the fallback in specification section 6.2: a
> handoff over a pipe is out of scope for a network capture tool, and the
> documentation says so rather than leaving users to discover it.
>
> A named pipe is also the transport the extcap integration streams to: the
> analyzer creates the pipe and hands fragcap the path, and fragcap connects as
> a client and writes pcapng to it.

**See also:** [Loopback](/docs/glossary/capture-and-networking#loopback), [extcap](/docs/glossary/windows-internals#extcap),
[Streaming sink](/docs/glossary/capture-and-networking#streaming-sink)

## extcap

The interface an analyzer (Wireshark and compatible tools) uses to enumerate,
configure, and start an external program as a capture source, defined by four
command-line invocations the analyzer makes: list interfaces, list link types,
declare configurable options, and capture to a named pipe.

fragcap implements extcap so it appears in an analyzer's interface list and is
configured through a native dialog the analyzer renders from fragcap's option
declaration, with no graphical code in fragcap. The capture streams pcapng to
the analyzer's [FIFO](/docs/glossary/windows-internals#named-pipe), the same bytes a file capture produces, so
an unmodified analyzer reads a process-attributed live capture.

{: .matters }
> extcap is how fragcap reaches an analyst's existing tool without a plugin. The
> configurable option names are the `run` command's own flag names, so the
> analyzer's dialog and the command line select capture identically. See
> specification section 14.5.

**See also:** [Link type](/docs/glossary/capture-and-networking#link-type), [Named pipe](/docs/glossary/windows-internals#named-pipe),
[pcapng](/docs/glossary/file-and-wire-formats#pcapng), [Streaming sink](/docs/glossary/capture-and-networking#streaming-sink)
