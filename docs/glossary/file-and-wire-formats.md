# File and Wire Formats

## pcapng

**Also known as:** PCAP Next Generation

The block-structured capture file format that succeeded the original libpcap
format, supporting multiple interfaces, name resolution, capture statistics,
and per-block options.

{: .matters }
> Extensibility through options is what lets fragcap carry attribution in a
> file that unmodified tools still read as an ordinary capture.

**See also:** [.fcapng](file-and-wire-formats.md#fcapng),
[Enhanced Packet Block](file-and-wire-formats.md#enhanced-packet-block)

**References:**

- PCAP Next Generation Capture File Format specification. Block structure and
  option encoding.

## .fcapng

fragcap's extended [pcapng](file-and-wire-formats.md#pcapng) profile, carrying process attribution in
Enhanced Packet Block options.

{: .matters }
> The governing rule is constitution principle P-5: an unmodified analyzer must
> read the file as ordinary pcapng and ignore annotations it does not
> understand. Attribution data is worth having only if the file remains a
> capture file. The annotation profile carries its own version, independent of
> the fragcap version, because the grammar and the software change on different
> schedules.

**See also:** [pcapng](file-and-wire-formats.md#pcapng)

## Enhanced Packet Block

**Also known as:** EPB

The [pcapng](file-and-wire-formats.md#pcapng) block carrying one captured packet with its timestamp,
captured length, original length, and options.

{: .matters }
> Its option area is where fragcap writes attribution, which is what makes the
> annotation invisible to tools that do not look for it.

**See also:** [pcapng](file-and-wire-formats.md#pcapng), [.fcapng](file-and-wire-formats.md#fcapng),
[Attribution annotation](file-and-wire-formats.md#attribution-annotation)

## Section Header Block

**Also known as:** SHB

The [pcapng](file-and-wire-formats.md#pcapng) block that opens a file, declaring byte order, format
version, and the application that wrote it.

{: .matters }
> Its byte-order magic is what makes a capture readable on a machine with the
> opposite endianness. fragcap always declares little-endian rather than host
> order: both are valid, and only one produces the same bytes for the same
> input on every architecture, which is what a golden comparison needs.

**See also:** [pcapng](file-and-wire-formats.md#pcapng), [Golden file](file-and-wire-formats.md#golden-file)

## Interface Description Block

**Also known as:** IDB

The [pcapng](file-and-wire-formats.md#pcapng) block declaring one capture interface: its
[link type](capture-and-networking.md#link-type), [snapshot length](capture-and-networking.md#snapshot-length), name, and
timestamp resolution.

{: .matters }
> Interfaces are identified positionally, by declaration order, and every
> packet block references one by that index. An identifier with no preceding
> declaration leaves a reader with no link type, so the packet cannot be
> dissected at all.

**See also:** [pcapng](file-and-wire-formats.md#pcapng), [Link type](capture-and-networking.md#link-type),
[Snapshot length](capture-and-networking.md#snapshot-length)

## Interface Statistics Block

**Also known as:** ISB

The [pcapng](file-and-wire-formats.md#pcapng) block carrying per-interface capture counters, written at
capture end.

{: .matters }
> Its standard fields describe losses upstream of the capturing application,
> and fragcap has counters of its own that no standard field expresses.
> Constitution principle P-4 makes an unsurfaced discard a defect, and P-9
> forbids reporting a fragcap loss as an operating system loss, so those
> counters travel in a declared comment rather than being omitted or
> overloaded onto a field that means something else.

**See also:** [pcapng](file-and-wire-formats.md#pcapng), [Backpressure](capture-and-networking.md#backpressure)

## Attribution annotation

The structured string fragcap writes into an
[Enhanced Packet Block](file-and-wire-formats.md#enhanced-packet-block) comment, carrying the process
that produced a packet, its [direction](capture-and-networking.md#direction), and its
[attribution fidelity](file-and-wire-formats.md#attribution-fidelity).

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

**See also:** [.fcapng](file-and-wire-formats.md#fcapng),
[Attribution fidelity](file-and-wire-formats.md#attribution-fidelity),
[Enhanced Packet Block](file-and-wire-formats.md#enhanced-packet-block)

**References:**

- fragcap specification section 13.3. Grammar, key table, and the reasoning
  for choosing comments over custom options.

## Attribution fidelity

How an [attribution](process-and-attribution.md#attribution) was obtained: from the live
[socket table](process-and-attribution.md#socket-table), from the retention window after the socket left
it, or not at all. Written as the `attr` key of an
[attribution annotation](file-and-wire-formats.md#attribution-annotation).

{: .matters }
> Retained attribution is inferential rather than observed: an endpoint that
> closed and was reassigned to a different process inside the retention window
> attributes incorrectly. Recording which packets are exposed to that is what
> lets analysis discount them. A consumer never infers this value, because the
> distinction between an observation and an inference is precisely what a
> reader cannot reconstruct from the data.

**See also:** [Attribution](process-and-attribution.md#attribution),
[Attribution annotation](file-and-wire-formats.md#attribution-annotation),
[Socket table](process-and-attribution.md#socket-table), [PID recycling](process-and-attribution.md#pid-recycling)

**References:**

- fragcap specification section 13.4.

## Golden file

A committed file of expected output, reviewed once by a human and compared
mechanically on every run afterward.

fragcap keeps one per [fixture](capture-and-networking.md#fixture), holding the exact bytes the writer
produces for it.

{: .matters }
> Tests written alongside an implementation encode the author's assumptions,
> including the wrong ones. A golden encodes what the code actually produced on
> a day somebody looked, so a later change is visible to a reviewer who was not
> there. This only works if output is deterministic, which is why the writer
> reads no clock and fixes its byte order: a golden that legitimately varies is
> a golden that gets deleted the first time it fails.

**See also:** [Fixture corpus](capture-and-networking.md#fixture-corpus), [Fixture](capture-and-networking.md#fixture)

## JSON Lines

**Also known as:** JSONL, newline-delimited JSON, NDJSON

One JSON object per line, with no enclosing array and no separators between
records.

{: .matters }
> The property that matters is that a line is self-contained: a stream can be
> split, tailed, filtered, or truncated with ordinary line tools and every
> surviving line is still a complete record. fragcap writes it for consumers
> that do not read [pcapng](file-and-wire-formats.md#pcapng), and it drives the differences from the
> [.fcapng](file-and-wire-formats.md#fcapng) profile. The interface name appears on every record here
> and only in multi-interface captures there, because a pcapng file holds the
> interface in its container and a line has no container to hold it.

**See also:** [pcapng](file-and-wire-formats.md#pcapng), [Trailer record](file-and-wire-formats.md#trailer-record),
[Payload-free mode](file-and-wire-formats.md#payload-free-mode)

**References:**

- fragcap specification section 13.5.

## HAR

**Also known as:** HTTP Archive

A JSON-based archive format for HTTP request and response records, commonly
used by browser developer tools and HTTP inspection software.

In fragcap, HAR is a candidate Deep Capture artifact for supported HTTP traffic
and a candidate utility-wide export from ordinary Capture metadata where enough
information exists. It is not a packet format and does not replace pcapng.
fragcap emits a standard entry only when the native application stream contains
the mandatory request, response, status, URL, completion, and phase timing
evidence. Other transactions remain in the namespaced partial-entry extension;
missing facts are never replaced with zeroes or synthetic values.

{: .matters }
> HAR is useful because application developers already know how to inspect it.
> It also has a narrower scope than pcapng: traffic that is not HTTP, or that
> was not observed with enough application-layer detail, cannot be represented
> honestly as HAR.

**See also:** [Session bundle](file-and-wire-formats.md#session-bundle),
[pcapng](file-and-wire-formats.md#pcapng),
[Deep Capture](capture-and-networking.md#deep-capture)

**References:**

- Web Hypertext Application Technology Working Group, HAR 1.2 specification.

## Session bundle

A Deep Capture output set that groups the ordinary capture artifacts with the
inspection artifacts produced during the same run.

A session bundle may include pcapng, JSON Lines, HAR when supported, proxy logs,
proxy-owned TLS key-log export, process traces, compatibility facts, and a
manifest tying each artifact to the same run identity and time base.
Native bundles use manifest version 2, which is separate from the fragcap
product version and declares one authority owner plus sensitivity, finalization,
completeness, loss, and correlation for every artifact or omission. Version 1
bundles remain read-only inputs.

Native lifecycle bundles also carry `resource-journal.jsonl` as the synchronized
ownership authority, `proxy.jsonl` and `cleanup.jsonl` as crash-readable
chronologies, and `cleanup.json` as a derived compatibility summary.

{: .matters }
> Deep Capture should feel like an elevated Capture session, not a second
> product. The bundle is what keeps decrypted application-layer observations,
> encrypted packet evidence, process attribution, and cleanup facts correlated
> after the run.

**See also:** [Deep Capture](capture-and-networking.md#deep-capture),
[pcapng](file-and-wire-formats.md#pcapng),
[JSON Lines](file-and-wire-formats.md#json-lines),
[Proxy-owned TLS key-log export](file-and-wire-formats.md#proxy-owned-tls-key-log-export)

## Resource journal

A bounded append-only record of session-owned external resources and their
state transitions, synchronized before and after each effect.

Each transition names the resource kind, exact target, ownership evidence,
recovery action, state, and sequence. A complete journal has a reconciling
trailer. A crash prefix can be inspected or replayed only when its version,
sequence, state transitions, and ownership evidence validate.

{: .matters }
> Recovery must never remove a resource merely because it resembles one fragcap
> previously owned. Uncertain application and ambiguous ownership are recorded
> refusals, not cleanup guesses.

**See also:** [Session bundle](file-and-wire-formats.md#session-bundle),
[Trailer record](file-and-wire-formats.md#trailer-record),
[Deep Capture](capture-and-networking.md#deep-capture)

## Lifecycle stream

A versioned JSON Lines chronology with a header, ordered records, explicit gaps,
and one reconciling trailer.

Deep Capture uses lifecycle streams for native proxy operations and cleanup.
Every complete record remains usable after interruption, while a missing trailer
means the stream is incomplete. Bounded writer pressure is emitted as loss rather
than inferred from missing records.

{: .matters }
> Machine consumers can reconcile listeners, connections, cleanup obligations,
> attempts, and outcomes without parsing human log text.

**See also:** [JSON Lines](file-and-wire-formats.md#json-lines),
[Resource journal](file-and-wire-formats.md#resource-journal),
[Trailer record](file-and-wire-formats.md#trailer-record)

## Proxy-owned TLS key-log export

A key-log artifact written from the local inspection proxy side of a Deep
Capture session so an analyzer can correlate decrypted proxy traffic with the
captured packet stream where the protocol and analyzer support it.

The keys are proxy-owned because they are produced by fragcap's proxy endpoint,
not taken from the game client. The distinction is load-bearing: exporting
proxy-owned material is permitted for Deep Capture, while extracting target TLS
keys is denylisted.

{: .matters }
> This export exists for analyzer interoperability. It does not imply that
> Capture mode decrypts traffic, and it does not bypass certificate pinning or
> recover secrets from a target process.

**See also:** [Deep Capture](capture-and-networking.md#deep-capture),
[Local inspection proxy](capture-and-networking.md#local-inspection-proxy),
[Technique denylist](anti-cheat-and-security.md#technique-denylist)

## Trailer record

The final object of a [JSON Lines](file-and-wire-formats.md#json-lines) stream, carrying the capture's
statistics. Distinguished from a packet record by a `type` key that packet
records never carry.

{: .matters }
> Its absence is the only way a consumer can tell a truncated stream from a
> complete one, which makes it load-bearing rather than decorative. It carries
> every counter even when zero, so that "nothing was lost" is distinguishable
> from "this build does not report that": the same reasoning that puts the
> counters in an [Interface Statistics Block](file-and-wire-formats.md#interface-statistics-block) for
> the other format, and the reason constitution principle P-4 is satisfied for
> a consumer who never sees the pcapng file.

**See also:** [JSON Lines](file-and-wire-formats.md#json-lines),
[Interface Statistics Block](file-and-wire-formats.md#interface-statistics-block)

## Payload-free mode

A [JSON Lines](file-and-wire-formats.md#json-lines) stream that omits packet payloads, producing
metadata suitable for flow analysis at a fraction of the volume.

{: .matters }
> The key is omitted entirely rather than emitted empty, because an empty
> payload is a real observation that renders as an empty string. A consumer
> distinguishes the two by the length fields, which are present in both modes.

**See also:** [JSON Lines](file-and-wire-formats.md#json-lines)

## Percent-encoding

Representing a character as a percent sign followed by two hexadecimal digits
per byte of its UTF-8 encoding.

fragcap applies it inside an [attribution annotation](file-and-wire-formats.md#attribution-annotation)
to the characters that carry meaning in the grammar, and to control characters,
which would otherwise break the comment that contains them.

{: .matters }
> Lossless and reversible, which is why widening the escaped set beyond the
> grammar's own reserved characters does not conflict with constitution
> principle P-9. The alternative for a process name containing a newline would
> be stripping or replacing it, which alters the observation.

**See also:** [Attribution annotation](file-and-wire-formats.md#attribution-annotation)
