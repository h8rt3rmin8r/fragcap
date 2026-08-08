# Contract: JSON Lines Format

**Slice**: S07

**Created**: 2026-08-08

**Feature**: [spec.md](../spec.md)

Two audiences: S08 as a caller, and anyone with `jq` as a consumer. The second
is the one this format exists for.

## The Rust surface

```rust
pub enum PayloadMode {
    WithPayload,
    MetadataOnly,
}

pub struct JsonLinesWriter<W: Write> { /* ... */ }

impl<W: Write> JsonLinesWriter<W> {
    /// Begin a stream, writing the header line immediately.
    pub fn new(
        output: W,
        interfaces: &[&str],
        mode: PayloadMode,
    ) -> Result<Self, WriteError>;
}

impl<W: Write + Send> Sink for JsonLinesWriter<W> {
    fn write(&mut self, packet: &CapturedPacket) -> Result<(), SinkError>;
    fn flush(&mut self) -> Result<(), SinkError>;
    fn finish(self: Box<Self>, stats: &CaptureStats) -> Result<(), SinkError>;
}
```

### Contract obligations

| Obligation | Requirement |
| --- | --- |
| `new` writes the header before returning | FR-003 |
| Interfaces are fixed at construction, in order | FR-038b |
| `write` emits exactly one line | FR-002, FR-008 |
| `write` never drops or skips a packet | FR-032 |
| `finish` writes the trailer and consumes the writer | FR-004 |
| No method reads a clock | FR-038a |
| No value passes through a float | FR-012 |
| Attribution keys come from the S06 derivation | FR-022 |

### Error behavior

| Condition | Behavior |
| --- | --- |
| Timestamp predates the Unix epoch | Error, no line written |
| Interface identifier not declared | Error, no line written |
| Underlying writer fails | Error propagated; lines already written stay valid |

No condition is a discard.

## The on-the-wire contract

### Stream shape

- Newline-delimited JSON. One object per line. No enclosing array, no commas
  between records.
- Every line, including the last, ends with a single line feed.
- No value contains a literal newline, so line count equals record count.
- Line one is the header. The last line of a complete stream is the trailer.
  **A stream with no trailer was truncated**, and that is the only way to tell.

### Dispatch

Read the first key.

| First key | Record |
| --- | --- |
| `type` | Header or trailer; read its value |
| `ts` | Packet |

### Header

```json
{"type":"header","version":"fragcap/0.1.0","interfaces":["eth0"]}
```

The section 12.7 session anchor is specified for this record and is not yet
written. See the slice's known gaps.

### Packet

```json
{"ts":1754500000.123456,"iface":"eth0","pid":7412,"proc":"eso64.exe",
 "role":"client","dir":"out","attr":"live","proto":"udp",
 "src":"192.0.2.10:51834","dst":"198.51.100.7:24100","len":242,
 "orig_len":242,"data":"3f8a01"}
```

Shown wrapped; a real record is one line.

| Key | Presence | Notes |
| --- | --- | --- |
| `ts` | Always | Number. Seconds with exactly six fractional digits |
| `iface` | Always | Unlike the pcapng profile, which omits it when there is one interface |
| `pid`, `proc` | When attributed | Never one without the other |
| `role`, `stage` | Each when present | Independent of each other |
| `dir` | Always | `in`, `out`, `local`, or `unknown` |
| `attr` | Always | `live`, `retained`, or `none` |
| `proto` | When a flow key | `tcp` or `udp` |
| `src`, `dst` | Flow key and known direction | Wire order |
| `local`, `remote` | Flow key and unknown direction | Position, not wire order |
| `len`, `orig_len` | Always | Captured and original, exactly as recorded |
| `data` | Payload mode | Lowercase hex |

**Endpoints.** A record carries `src` and `dst` when the direction was
determined, and `local` and `remote` when it was not. It never carries both.
This is not a stylistic choice: the flow key normalizes endpoint position, so
wire order exists only in combination with direction, and loopback traffic has
a flow key and no direction. Emitting `src` and `dst` regardless would present
a guess as an observation.

A consumer filtering on destination should therefore expect `dst` to be absent
on records where direction was not determined, rather than treat its absence as
malformed.

**Timestamps** are exact. `ts` is produced from an integer nanosecond count by
integer arithmetic and never passes through a floating point value, so the six
digits are the recorded microseconds rather than the nearest representable
approximation of them. A consumer parsing into a double will reintroduce that
approximation; parse as a decimal if the sixth digit matters.

### Trailer

```json
{"type":"trailer","packets":6,"attributed":6,"unattributed":0,
 "kernel_dropped":0,"interface_dropped":0,"buffer_dropped":0,
 "sink_dropped":0,"filter_gaps":0}
```

Every counter is present even when zero, so "nothing was lost" is
distinguishable from "not reported".

## Stability

The record shapes are versioned with the annotation profile, declared in the
header `version`. Adding a key that consumers may ignore does not bump it;
changing what a key means, or which are present, does.

The Rust surface is not stable in v0.1.0.
