# File and Wire Formats

## pcapng

**Also known as:** PCAP Next Generation

The block-structured capture file format that succeeded the original libpcap
format, supporting multiple interfaces, name resolution, capture statistics,
and per-block options.

{: .matters }
> Extensibility through options is what lets fragcap carry attribution in a
> file that unmodified tools still read as an ordinary capture.

**See also:** [.fcapng](/docs/glossary/file-and-wire-formats#fcapng),
[Enhanced Packet Block](/docs/glossary/file-and-wire-formats#enhanced-packet-block)

**References:**

- PCAP Next Generation Capture File Format specification. Block structure and
  option encoding.

## .fcapng

fragcap's extended [pcapng](/docs/glossary/file-and-wire-formats#pcapng) profile, carrying process attribution in
Enhanced Packet Block options.

{: .matters }
> The governing rule is constitution principle P-5: an unmodified analyzer must
> read the file as ordinary pcapng and ignore annotations it does not
> understand. Attribution data is worth having only if the file remains a
> capture file. The annotation profile carries its own version, independent of
> the fragcap version, because the grammar and the software change on different
> schedules.

**See also:** [pcapng](/docs/glossary/file-and-wire-formats#pcapng)

## Enhanced Packet Block

**Also known as:** EPB

The [pcapng](/docs/glossary/file-and-wire-formats#pcapng) block carrying one captured packet with its timestamp,
captured length, original length, and options.

{: .matters }
> Its option area is where fragcap writes attribution, which is what makes the
> annotation invisible to tools that do not look for it.

**See also:** [pcapng](/docs/glossary/file-and-wire-formats#pcapng), [.fcapng](/docs/glossary/file-and-wire-formats#fcapng),
[Attribution annotation](/docs/glossary/file-and-wire-formats#attribution-annotation)

## Section Header Block

**Also known as:** SHB

The [pcapng](/docs/glossary/file-and-wire-formats#pcapng) block that opens a file, declaring byte order, format
version, and the application that wrote it.

{: .matters }
> Its byte-order magic is what makes a capture readable on a machine with the
> opposite endianness. fragcap always declares little-endian rather than host
> order: both are valid, and only one produces the same bytes for the same
> input on every architecture, which is what a golden comparison needs.

**See also:** [pcapng](/docs/glossary/file-and-wire-formats#pcapng), [Golden file](/docs/glossary/file-and-wire-formats#golden-file)

## Interface Description Block

**Also known as:** IDB

The [pcapng](/docs/glossary/file-and-wire-formats#pcapng) block declaring one capture interface: its
[link type](/docs/glossary/capture-and-networking#link-type), [snapshot length](/docs/glossary/capture-and-networking#snapshot-length), name, and
timestamp resolution.

{: .matters }
> Interfaces are identified positionally, by declaration order, and every
> packet block references one by that index. An identifier with no preceding
> declaration leaves a reader with no link type, so the packet cannot be
> dissected at all.

**See also:** [pcapng](/docs/glossary/file-and-wire-formats#pcapng), [Link type](/docs/glossary/capture-and-networking#link-type),
[Snapshot length](/docs/glossary/capture-and-networking#snapshot-length)

## Interface Statistics Block

**Also known as:** ISB

The [pcapng](/docs/glossary/file-and-wire-formats#pcapng) block carrying per-interface capture counters, written at
capture end.

{: .matters }
> Its standard fields describe losses upstream of the capturing application,
> and fragcap has counters of its own that no standard field expresses.
> Constitution principle P-4 makes an unsurfaced discard a defect, and P-9
> forbids reporting a fragcap loss as an operating system loss, so those
> counters travel in a declared comment rather than being omitted or
> overloaded onto a field that means something else.

**See also:** [pcapng](/docs/glossary/file-and-wire-formats#pcapng), [Backpressure](/docs/glossary/capture-and-networking#backpressure)

## Attribution annotation

The structured string fragcap writes into an
[Enhanced Packet Block](/docs/glossary/file-and-wire-formats#enhanced-packet-block) comment, carrying the process
that produced a packet, its [direction](/docs/glossary/capture-and-networking#direction), and its
[attribution fidelity](/docs/glossary/file-and-wire-formats#attribution-fidelity).

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

**See also:** [.fcapng](/docs/glossary/file-and-wire-formats#fcapng),
[Attribution fidelity](/docs/glossary/file-and-wire-formats#attribution-fidelity),
[Enhanced Packet Block](/docs/glossary/file-and-wire-formats#enhanced-packet-block)

**References:**

- fragcap specification section 13.3. Grammar, key table, and the reasoning
  for choosing comments over custom options.

## Attribution fidelity

How an [attribution](/docs/glossary/process-and-attribution#attribution) was obtained: from the live
[socket table](/docs/glossary/process-and-attribution#socket-table), from the retention window after the socket left
it, or not at all. Written as the `attr` key of an
[attribution annotation](/docs/glossary/file-and-wire-formats#attribution-annotation).

{: .matters }
> Retained attribution is inferential rather than observed: an endpoint that
> closed and was reassigned to a different process inside the retention window
> attributes incorrectly. Recording which packets are exposed to that is what
> lets analysis discount them. A consumer never infers this value, because the
> distinction between an observation and an inference is precisely what a
> reader cannot reconstruct from the data.

**See also:** [Attribution](/docs/glossary/process-and-attribution#attribution),
[Attribution annotation](/docs/glossary/file-and-wire-formats#attribution-annotation),
[Socket table](/docs/glossary/process-and-attribution#socket-table), [PID recycling](/docs/glossary/process-and-attribution#pid-recycling)

**References:**

- fragcap specification section 13.4.

## Golden file

A committed file of expected output, reviewed once by a human and compared
mechanically on every run afterward.

fragcap keeps one per [fixture](/docs/glossary/capture-and-networking#fixture), holding the exact bytes the writer
produces for it.

{: .matters }
> Tests written alongside an implementation encode the author's assumptions,
> including the wrong ones. A golden encodes what the code actually produced on
> a day somebody looked, so a later change is visible to a reviewer who was not
> there. This only works if output is deterministic, which is why the writer
> reads no clock and fixes its byte order: a golden that legitimately varies is
> a golden that gets deleted the first time it fails.

**See also:** [Fixture corpus](/docs/glossary/capture-and-networking#fixture-corpus), [Fixture](/docs/glossary/capture-and-networking#fixture)

## JSON Lines

**Also known as:** JSONL, newline-delimited JSON, NDJSON

One JSON object per line, with no enclosing array and no separators between
records.

{: .matters }
> The property that matters is that a line is self-contained: a stream can be
> split, tailed, filtered, or truncated with ordinary line tools and every
> surviving line is still a complete record. fragcap writes it for consumers
> that do not read [pcapng](/docs/glossary/file-and-wire-formats#pcapng), and it drives the differences from the
> [.fcapng](/docs/glossary/file-and-wire-formats#fcapng) profile. The interface name appears on every record here
> and only in multi-interface captures there, because a pcapng file holds the
> interface in its container and a line has no container to hold it.

**See also:** [pcapng](/docs/glossary/file-and-wire-formats#pcapng), [Trailer record](/docs/glossary/file-and-wire-formats#trailer-record),
[Payload-free mode](/docs/glossary/file-and-wire-formats#payload-free-mode)

**References:**

- fragcap specification section 13.5.

## Trailer record

The final object of a [JSON Lines](/docs/glossary/file-and-wire-formats#json-lines) stream, carrying the capture's
statistics. Distinguished from a packet record by a `type` key that packet
records never carry.

{: .matters }
> Its absence is the only way a consumer can tell a truncated stream from a
> complete one, which makes it load-bearing rather than decorative. It carries
> every counter even when zero, so that "nothing was lost" is distinguishable
> from "this build does not report that": the same reasoning that puts the
> counters in an [Interface Statistics Block](/docs/glossary/file-and-wire-formats#interface-statistics-block) for
> the other format, and the reason constitution principle P-4 is satisfied for
> a consumer who never sees the pcapng file.

**See also:** [JSON Lines](/docs/glossary/file-and-wire-formats#json-lines),
[Interface Statistics Block](/docs/glossary/file-and-wire-formats#interface-statistics-block)

## Payload-free mode

A [JSON Lines](/docs/glossary/file-and-wire-formats#json-lines) stream that omits packet payloads, producing
metadata suitable for flow analysis at a fraction of the volume.

{: .matters }
> The key is omitted entirely rather than emitted empty, because an empty
> payload is a real observation that renders as an empty string. A consumer
> distinguishes the two by the length fields, which are present in both modes.

**See also:** [JSON Lines](/docs/glossary/file-and-wire-formats#json-lines)

## Percent-encoding

Representing a character as a percent sign followed by two hexadecimal digits
per byte of its UTF-8 encoding.

fragcap applies it inside an [attribution annotation](/docs/glossary/file-and-wire-formats#attribution-annotation)
to the characters that carry meaning in the grammar, and to control characters,
which would otherwise break the comment that contains them.

{: .matters }
> Lossless and reversible, which is why widening the escaped set beyond the
> grammar's own reserved characters does not conflict with constitution
> principle P-9. The alternative for a process name containing a newline would
> be stripping or replacing it, which alters the observation.

**See also:** [Attribution annotation](/docs/glossary/file-and-wire-formats#attribution-annotation)
