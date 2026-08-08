# Data Model: Core Types and Traits

**Slice**: S02

**Created**: 2026-08-08

The shapes this slice writes into `fragcap-core`. Signatures from specification
sections 8.4 and 8.5 are reproduced as the architecture of record gives them;
anything the specification leaves undefined is marked **filled here** and is
recorded as a deviation in `plan.md` D-7.

## Entities

### Proto

Which transport protocol a flow uses. Exactly two variants, because the socket
table join is defined for these two only.

Derives equality, ordering, hashing, copy, and debug. Hashing because it is part
of a map key (V-1 below).

### Direction

Inbound or outbound, per packet. Not a property of the flow: the flow key
already normalized endpoint position, so direction is independent per packet.

### Endpoint (filled here)

An address and port on some host, with the protocol it was seen under. Returned
in bulk by `FlowAttributor::active_endpoints`. Section 11.4 describes endpoint
retention after a socket disappears from the table, which S10 implements; this
slice defines only the shape.

### FlowKey

| Field | Meaning |
| --- | --- |
| `proto` | Transport protocol |
| `local` | The endpoint on the capturing host, always |
| `remote` | The other endpoint |

The local position is load-bearing and is a validation rule rather than a
convention (V-2).

### AttributionKey

The subset of a flow key that a socket table can answer. Two variants:

| Variant | Carries | Because |
| --- | --- | --- |
| `Pair` | local and remote | The TCP socket table carries both endpoints |
| `Local` | local only | A UDP socket generally has no fixed peer, so the table carries no remote |

There is deliberately no variant carrying a remote for UDP. That absence is the
enforcement of the specification's requirement that implementations must not
invent one (V-3).

### StageId (filled here)

Identifies a stage in a launcher chain. A profile in S05 names stages; this is
the identity a resolved stage carries into an attribution. Modelled as a shared
immutable string rather than an index, because an index would only be meaningful
relative to a profile the attribution does not carry, and a name survives being
written into an output file and read back.

### Attribution

| Field | Meaning |
| --- | --- |
| `pid` | Operating system process identifier |
| `process` | Process name, shared rather than copied |
| `role` | Optional role from the matching profile |
| `stage` | Optional launcher chain stage |

Process and role are reference-counted shared strings because both are drawn
from a small set and repeat on every packet of a flow (V-4).

### Timestamp (filled here)

A count of nanoseconds since the Unix epoch, signed. One canonical resolution,
no per-packet format resolution. See `research.md` R-2.

### RawPacket

| Field | Meaning |
| --- | --- |
| `ts` | When the capture driver observed the frame |
| `data` | The bytes retained, possibly fewer than were on the wire |
| `orig_len` | How long the frame was on the wire, before any snapshot limit |

`orig_len` is separate from the payload length so truncation is self-describing
(V-5).

### CapturedPacket

Everything `RawPacket` carries, plus what the pipeline resolved:

| Field | Meaning | Populated by |
| --- | --- | --- |
| `flow` | Optional flow key | S03 header parsing |
| `direction` | Optional direction | S03 |
| `attribution` | Optional attribution | S10 socket table attributor |

### AttributionState (filled here, derived not stored)

Not a field. A view over the pair (`flow`, `attribution`):

| `flow` | `attribution` | State |
| --- | --- | --- |
| absent | absent | Never attempted; there was no key to attempt with |
| present | absent | Attempted and unresolved; packet is retained and marked |
| present | present | Resolved |
| absent | present | Unrepresentable in practice; treated as resolved and documented |

See `plan.md` D-5 for why this is derived rather than stored.

### LinkType (filled here)

The link layer encapsulation a source produces, which the output layer writes
into a pcapng interface description block. Ethernet and raw IP are named; the
type is extensible because S09 discovers what npcap actually reports and S18 may
add more.

### FilterProgram (filled here, minimal)

A capture filter to be installed on a source. Carries the filter expression as
text at this slice. S13 owns filter management and will extend this; the
documentation says so, so a thin type is not read as a finished one.

### ProcessEvent and ProcessRecord (filled here, minimal)

A process lifecycle event and a point-in-time process record respectively. Both
are the minimal shape their signatures in section 8.5 require. S11 owns the ETW
process watcher and the process tree, and will extend both.

### SourceStats

What the capture backend reports about itself:

| Field | Meaning |
| --- | --- |
| `received` | Frames the backend saw |
| `kernel_dropped` | Frames the driver dropped before fragcap, per section 12.4 |
| `interface_dropped` | Frames the interface dropped before the driver |

### CaptureStats

What fragcap's own pipeline counted, holding the source's report by value rather
than merging it:

| Field | Meaning |
| --- | --- |
| `packets_captured` | Packets fragcap accepted |
| `packets_attributed` | Packets that resolved to a process |
| `packets_unattributed` | Packets retained and marked, per P-4 |
| `buffer_dropped` | Dropped by the bounded buffer, per section 12.4 |
| `sink_dropped` | Dropped by a sink that could not accept, per section 12.4 |
| `filter_gaps` | Counted gaps during filter narrowing, per section 13 |
| `source` | The backend's own report, unaltered |

No stored total. Totals are methods (V-6).

## Error types

Three enums, each with named variants, each extensible without a breaking
change.

| Type | Raised by | Variants distinguish |
| --- | --- | --- |
| `SourceError` | `PacketSource` | A timeout, which is normal and continues the loop, from a terminal backend or device failure |
| `AttrError` | `FlowAttributor` | A transient refresh failure from an unavailable platform facility |
| `SinkError` | `Sink` | A write failure from a backpressure condition the pipeline should count rather than abort on |

## Traits

Reproduced from specification section 8.5 unchanged. All four behavioral traits
are constrained to remain usable as trait objects (V-7).

| Trait | Bounds | Methods |
| --- | --- | --- |
| `PacketSource` | none | `next_packet`, `set_filter`, `stats`, `link_type` |
| `FlowAttributor` | `Send` | `resolve`, `refresh`, `active_endpoints` |
| `ProcessWatcher` | `Send` | `subscribe`, `snapshot` |
| `Sink` | `Send` | `write`, `flush`, `finish` |
| `Dissector` | none | Declared with no implementations, on purpose |

## Validation Rules

Each rule is asserted by a test named for it.

- **V-1**: `FlowKey` and `AttributionKey` support equality and hashing, and
  every field of either participates in the hash. Satisfies FR-013.
- **V-2**: `FlowKey::local` is documented as always being the capturing host's
  endpoint, and `attribution_key` derives from that position rather than
  inspecting addresses. Satisfies FR-002.
- **V-3**: No `AttributionKey` variant carries a remote endpoint for UDP. A UDP
  flow key always derives `Local`. Satisfies FR-003 and FR-004.
- **V-4**: `Attribution` clones without allocating a new string. Satisfies
  FR-007.
- **V-5**: `RawPacket` retains `orig_len` independently of `data.len()`, and a
  truncated packet reports the original length. Satisfies FR-008.
- **V-6**: No statistics type stores a total. Every aggregate is computed from
  named counters. Satisfies FR-025.
- **V-7**: `PacketSource`, `FlowAttributor`, `ProcessWatcher`, and `Sink` are
  all usable as trait objects, asserted by a test that constructs each behind a
  pointer. Satisfies FR-019.
- **V-8**: Neither `PacketSource` nor `FlowAttributor` names the other in any
  signature. Satisfies FR-020 and P-3.
- **V-9**: The three attribution states are readable from a `CapturedPacket`
  alone. Satisfies FR-010.
- **V-10**: A UDP local endpoint matches both a wildcard bind address and a
  specific interface address. Satisfies FR-005.
