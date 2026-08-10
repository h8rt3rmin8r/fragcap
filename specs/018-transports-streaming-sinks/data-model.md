# Phase 1 Data Model: Transports and streaming sinks

Entities are the transport-layer types added to `fragcap-sink`, plus the CLI
parse types they are built from. Field names are indicative; the invariants are
binding.

## SinkFactory

Constructs a fresh format encoder over a supplied writable connection.

- `format`: which format to produce (pcapng or JSON Lines).
- `interfaces`: the ordered interface declarations to replay into every new
  encoder (name, link type, snap length).
- `payload_mode`: with-payload or metadata-only (JSON Lines).

Behavior: `build(conn: Box<dyn Write + Send>) -> Result<Box<dyn Sink>,
WriteError>` creates the encoder and writes its header preamble (pcapng: SHB
then one IDB per interface; JSON Lines: none, records are self-contained).

Invariants:
- Every encoder it builds is independently valid from its first byte
  (P-5, FR-007).
- The interface set every encoder declares is identical (FR-007).

## RotatingFileSink (implements `Sink`)

A file transport with optional rotation.

- `base_path`: the operator-supplied path; the ordinal is inserted before the
  extension.
- `policy`: `None`, `Size(bytes)`, or `Duration(d)`.
- `factory`: the `SinkFactory` for the chosen format.
- `current`: the current segment's `Box<dyn Sink>` (encoder over the current
  `File`).
- `segment_index`: zero-based ordinal of the current segment.
- `bytes_in_segment`, `segment_opened_at`: rotation accounting for the current
  segment.

State transitions:
- On `write`, if `policy` is set and the current segment has reached its bound
  (`bytes_in_segment >= size`, or `now - segment_opened_at >= duration`), the
  sink finishes `current` (writing its trailer), opens the next numbered file,
  builds a fresh encoder, and increments `segment_index`; then it writes the
  packet.
- On `finish`, the current segment is finished with the run statistics.

Invariants:
- With `policy = None` the output is byte-identical to the pre-slice file sink
  (single segment, no ordinal churn) (Assumptions).
- Every produced segment begins with its own header preamble and ends cleanly;
  each is independently readable (FR-002, SC-002).
- The union of all segments' packets equals a single-file capture of the same
  input, none lost, duplicated, or reordered across joins (SC-002).

## StreamSink (implements `Sink`)

A transport-agnostic multi-consumer streaming sink.

- `factory`: the `SinkFactory` used to build each consumer's encoder.
- `consumers`: the live consumer registry (shared with the acceptor thread).
- `queue_depth`: per-consumer bounded-queue capacity (default 1024).
- `disconnect_timeout`: continuous-full duration after which a consumer is
  disconnected (default 5 s).
- `acceptor`: the transport's connection acceptor, spawned on its own thread,
  pushing new `Consumer`s into `consumers`.

State transitions:
- `write(packet)`: for each live consumer, `try_send` a clone. `Ok` enqueues;
  `Full` increments the consumer's `dropped` and updates its `full_since`;
  `Disconnected` marks the consumer removed. A consumer whose `full_since`
  exceeds `disconnect_timeout` has its connection shut down and is removed with
  reason `timeout`. Always returns `Ok(())`.
- `flush`: no-op at the sink level (each consumer thread flushes its own
  encoder).
- `finish(stats)`: signals every consumer to finalize with `stats`, joins the
  consumer and acceptor threads, and closes the transport.

Invariants:
- `write` never blocks and never returns `Err` (D3): the pipeline conservation
  invariant is preserved and the sink is never retired for downstream slowness.
- A per-consumer drop advances only that consumer's `dropped`, never
  `CaptureStats.sink_dropped` (FR-010).
- A consumer connecting mid-capture receives a valid stream from its connect
  point and no earlier packet (FR-008).

## Consumer

One connected reader of a streaming transport.

- `id`: a stable identity for logs and accounting (peer address for TCP, an
  ordinal for pipe/unix).
- `sender`: the bounded-queue `SyncSender<ToConsumer>`.
- `dropped`: packets refused for this consumer while its queue was full.
- `delivered`: packets accepted into its queue.
- `full_since`: the instant its queue first became continuously full, cleared
  when a send next succeeds.
- `thread`: the writer thread that owns the connection and the encoder.

`ToConsumer` is the channel message: `Packet(CapturedPacket)` or
`Finish(Arc<CaptureStats>)`.

Invariants:
- `delivered + dropped` equals the packets offered to this consumer while it was
  connected (per-consumer conservation, mirrors the pipeline identity).
- The writer thread owns the only handle to the connection and the encoder, so
  no lock guards a socket write.

## ConsumerReport

The per-consumer accounting surfaced at end of run (and on disconnect).

- `id`, `delivered`, `dropped`, `connected_at_reason`, `disconnect_reason`
  (one of `client-closed`, `timeout`, `capture-ended`, `write-error`).

Rendered through the CLI's existing `--json` NDJSON event stream and the
end-of-run summary. Kept separate from `CaptureStats`.

## SinkSpec (CLI, extended)

The parsed `--sink` value.

- Variants: `File(path)`, `JsonLines(path)`, `Pipe(name)`, `Unix(path)` (new),
  `Tcp(authority)`.
- Each carries resolved `SinkOptions`: `format` (inferred or explicit),
  `payload_mode`, and transport-appropriate options (`rotate_size`,
  `rotate_duration` for file; `queue`, `timeout` for streaming).

Invariants:
- A spec whose format cannot be resolved, whose transport is unavailable on the
  current platform, or whose destination cannot be bound is a configuration
  error reported before capture starts, naming the cause (FR-014).
- No accepted scheme reaches capture as an unimplemented stub (SC-005).
