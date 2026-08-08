# Data Model: JSON Lines Writer

**Slice**: S07

**Created**: 2026-08-08

**Feature**: [spec.md](spec.md)

Types S02 and S06 already fixed (`CapturedPacket`, `Attribution`, `Fidelity`,
`FlowKey`, `Timestamp`, `CaptureStats`, `Annotation`) are inputs, not new
entities.

## Record

The three record shapes the stream contains. Not a Rust enum in the
implementation, since each is written by its own function; described together
because a consumer dispatches between them.

| Record | Distinguished by | Written |
| --- | --- | --- |
| Header | `"type":"header"` first | Once, before any packet |
| Packet | No `type` key | Once per packet |
| Trailer | `"type":"trailer"` first | Once, at finish |

A consumer reads the first key. If it is `type`, the record is metadata; if it
is `ts`, the record is a packet. That is why `type` is required to come first
and why packet records must never carry one.

## Header record

| Key | Type | Source |
| --- | --- | --- |
| `type` | string, `"header"` | Constant |
| `version` | string | The crate version, as `fragcap/0.1.0` |
| `interfaces` | array of string | The declared interface names, in order |

The section 12.7 session anchor belongs here per section 13.5 and is absent.
There is no session in this slice; S08 owns capture start. Recorded as a known
gap rather than given a placeholder, because a placeholder would have to be
either wrong or null, and a consumer cannot tell an absent anchor from a null
one that meant something.

`interfaces` is emitted in declaration order from an ordered collection, per
FR-038b. An unordered collection here would make the header vary between runs
and every golden unusable.

## Packet record

| Key | Presence | Type | Source |
| --- | --- | --- | --- |
| `ts` | Always | number | `Timestamp`, exact microseconds |
| `iface` | Always | string | The record's interface name |
| `pid` | When attributed | number | `Annotation::pid` |
| `proc` | When attributed | string | `Annotation::process` |
| `role` | When present | string | `Annotation::role` |
| `stage` | When present | string | `Annotation::stage` |
| `dir` | Always | string | `Annotation::direction` |
| `attr` | Always | string | `Annotation::fidelity` |
| `proto` | When a flow key | string | `FlowKey::proto` |
| `src` | Flow key and known direction | string | Wire order, derived |
| `dst` | Flow key and known direction | string | Wire order, derived |
| `local` | Flow key and unknown direction | string | `FlowKey::local` |
| `remote` | Flow key and unknown direction | string | `FlowKey::remote` |
| `len` | Always | number | Captured length |
| `orig_len` | Always | number | Original length |
| `data` | Payload mode only | string | Lowercase hex |

**Every attribution key above comes from `Annotation`, not from the packet.**
That is the rule this slice exists to test: the presence logic for `pid` and
`proc` as a pair, for `role` and `stage` independently, and for fidelity, lives
in one place and is read here.

**Endpoint naming.** With a flow key and a known direction, `src` and `dst`
carry wire order: outbound means source is the local endpoint, inbound means
source is the remote one. With a flow key and no known direction, `local` and
`remote` are emitted under their own names instead, and wire order is not
claimed. A record never carries both pairs. See research R-4.

**Divergences from the pcapng profile**, all deliberate, all in rendering
rather than derivation:

| Difference | Here | pcapng | Why |
| --- | --- | --- | --- |
| `iface` | Always | Multi-interface only | A line is self-contained |
| Hex case | Lowercase | Uppercase percent escapes | Each format's convention |
| Endpoints | `src`/`dst` or `local`/`remote` | Not carried | pcapng has the packet bytes |

## Trailer record

| Key | Type | Source |
| --- | --- | --- |
| `type` | string, `"trailer"` | Constant |
| `packets` | number | `CaptureStats::packets_captured` |
| `attributed` | number | `CaptureStats::packets_attributed` |
| `unattributed` | number | `CaptureStats::packets_unattributed` |
| `kernel_dropped` | number | `SourceStats::kernel_dropped` |
| `interface_dropped` | number | `SourceStats::interface_dropped` |
| `buffer_dropped` | number | `CaptureStats::buffer_dropped` |
| `sink_dropped` | number | `CaptureStats::sink_dropped` |
| `filter_gaps` | number | `CaptureStats::filter_gaps` |

Every counter is present even when zero, per FR-031. Omitting a zero would make
"nothing was lost" indistinguishable from "this build does not report that",
which is the ambiguity P-4 exists to remove.

Unlike the pcapng Interface Statistics Block, this record genuinely has no
per-interface problem: it describes the capture, is written once, and says so.
The pcapng equivalent had to be per-interface because the format defines it
that way. That was one of the two reasons S06 restricted itself to a single
interface; this format escapes that one and not the other, which is what made
the earlier claim above look right.

## PayloadMode

| Variant | Effect |
| --- | --- |
| `WithPayload` | `data` is written |
| `MetadataOnly` | `data` is omitted entirely |

Fixed at construction, per plan D-5. Not a per-record decision: a stream that
mixed modes would be uninterpretable, since a missing `data` would mean either
suppression or a defect.

`MetadataOnly` omits the key rather than emitting an empty string, because a
zero-length payload is a real observation that renders as an empty string in
`WithPayload` mode. `len` disambiguates in both directions.

## JsonLinesWriter

| Field | Purpose |
| --- | --- |
| Output target | Any `std::io::Write` |
| Interface names | In declaration order, indexed by identifier |
| Payload mode | Fixed at construction |
| Header written flag | The header is emitted once, before anything else |

**State transitions**:

```text
new  ->  write*  ->  finish
         (any number)  (consumes)
```

Interfaces are supplied at construction rather than declared incrementally,
which differs from the pcapng writer. The header must list them all and is
written first, so there is no point at which a later declaration could be
accommodated.

**At most one, and a second is refused.** An earlier version of this document
claimed the JSON format escaped the single-interface restriction the pcapng
writer needed, on the reasoning that a JSON record names its interface
explicitly where a pcapng packet block cannot. Review of pull request 9 refuted
it. Naming the interface is not the difficulty; choosing it is.
`CapturedPacket` carries no interface identifier and `Sink::write` has nowhere
to pass one, so every packet routes through index 0 regardless of how many were
declared. A stream declaring two interfaces would name both in its header and
then label every record with the first, which is a false statement on every
line rather than a missing field, and worse than the pcapng case because each
record asserts it individually.

Both writers therefore wait for the same thing: an interface identifier on the
packet, which arrives with live capture in S09.

**Invariants**:

- The header is the first line, always.
- Every packet record's `iface` names one of the declared interfaces.
- No method reads a clock, environment, locale, or host property. FR-038a.
- No value passes through a floating point number. FR-012.

## Golden

One `.jsonl` per fixture in the S04 corpus, beside the `.fcapng` goldens.

| Property | Value |
| --- | --- |
| Location | `fixtures/goldens/<fixture>.jsonl` |
| Produced by | The committed generator |
| Checked by | A drift check in `cargo xtask ci` |
| Reviewed | Once by a human, at the commit that introduces it |
