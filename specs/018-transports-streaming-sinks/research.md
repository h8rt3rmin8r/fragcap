# Phase 0 Research: Transports and streaming sinks

Every decision below was resolvable from the constitution, the architecture of
record, and the existing pipeline contract. No item required an operator call,
so none is a NEEDS CLARIFICATION.

## R1. Where the transport layer lives

**Decision**: A new `transport` module inside `fragcap-sink`. `fragcap-core`
and its `Sink` trait are not modified.

**Rationale**: `fragcap-sink` is the crate the module doc already names as the
home of "transports and streaming sinks in S15." It is a leaf crate that may
take platform dependencies. P-2 constrains only `fragcap-core`, which stays
untouched, so the neutral core build is unaffected. The `Sink` trait already
models exactly what the pipeline needs (`write`/`flush`/`finish`); a streaming
sink is just another `Sink`.

**Alternatives considered**: A separate `fragcap-transport` crate. Rejected:
it adds a crate and a dependency edge for no separation the module boundary
inside `fragcap-sink` does not already provide, and it would still depend on
`fragcap-sink` for the format writers.

## R2. Reusing the format writers unchanged (format/transport orthogonality)

**Decision**: A `SinkFactory` produces a fresh `Box<dyn Sink>` over a supplied
`Box<dyn Write + Send>`, replaying the format's header. For pcapng that is
`PcapngWriter::new(conn)` followed by `declare_interface` per interface; for
JSON Lines it is `JsonLinesWriter::new(conn, names, mode)`. Each connected
consumer gets its own factory-built encoder.

**Rationale**: The writers are already generic over `W: Write` and already
write their header eagerly at construction (the pcapng SHB is written in
`new`). Constructing one per connection is exactly what specification 14.3
requires: every consumer receives its own SHB + IDBs regardless of connect
time. Nothing in the writers changes, and "any format writes to any transport"
falls out for free. This is decision D1 in the plan.

**Alternatives considered**: A single shared encoder writing to a broadcast
`Write` that fans bytes to all connections. Rejected: pcapng framing is
stateful and per-stream (each consumer needs its own SHB and its own EPB
sequence from its connect point); a shared byte stream cannot give a
mid-capture joiner a valid header, which violates 14.3 and P-5.

## R3. Per-consumer isolation and non-blocking capture

**Decision**: Each consumer is a bounded `std::sync::mpsc::sync_channel(depth)`
plus a dedicated writer thread that owns the connection and the factory-built
encoder. The streaming sink's `write` does `try_send` to each consumer:
`Ok` enqueues; `Err(Full)` increments that consumer's drop counter; `Err(
Disconnected)` marks the consumer for removal. `write` never blocks and always
returns `Ok` to the pipeline.

**Rationale**: This delivers SC-003/SC-004 (a stalled consumer degrades only
itself) and D2/D3. `sync_channel` supplies bounded capacity with a
non-blocking `try_send`, which is precisely the "producer never waits, evict
rather than stall" shape S08 needed and hand-built for the pipeline buffer.
Here, unlike the pipeline buffer, the natural per-consumer semantics is
drop-newest (refuse the incoming packet when the queue is full): a live
analyzer that has fallen behind should not have the sink spend work evicting
its backlog, and every dropped packet is counted, so P-4 is satisfied either
way. Cloning a `CapturedPacket` into the channel is cheap: its payload is
`bytes::Bytes` (reference-counted), so a clone is an `Arc` bump.

**Alternatives considered**:
- A concurrency crate (`crossbeam`, an async runtime). Rejected on the same
  grounds as S08: the standard library supplies the bounded, non-blocking
  producer this needs; a dependency would supply nothing missing.
- Drop-oldest per consumer (evict the front). Rejected as the default: it costs
  a dequeue on the hot path for a consumer that is already behind, with no
  fidelity gain since the drop is counted regardless. Recorded as a possible
  future option, not built.
- One writer thread multiplexing all consumers. Rejected: one slow socket write
  would head-of-line block the others, defeating isolation.

## R4. Disconnect-on-timeout

**Decision**: Track, per consumer, the instant its queue first became
continuously full. When the streaming sink observes a consumer full for longer
than the disconnect timeout (default 5 s), it shuts down that consumer's
connection, which unblocks its writer thread's blocking socket write and lets
it exit; the sink reaps it and logs the disconnection with the consumer's
identity and the reason.

**Rationale**: FR-011. A permanently stalled consumer must not hold its queue
forever. Shutting the connection is the portable way to unblock a thread parked
in a blocking write. Reading `Instant::now()` is legitimate here: this is
runtime control-plane timing, not the deterministic golden output path the
writers deliberately keep clock-free.

**Alternatives considered**: Per-socket write timeouts (`set_write_timeout`).
Viable for TCP/Unix but not uniformly available for the Windows named pipe in
the blocking model; the connection-shutdown approach is uniform across all
three transports, so it is preferred for one code path.

## R5. File rotation

**Decision**: `RotatingFileSink` holds the base path, an optional rotation
policy (size in bytes, or duration), the current segment's `PcapngWriter<File>`
(or JSON writer), a running byte count and the segment-open instant, the
segment ordinal, and the interface declarations to replay. On `write`, if a
policy is set and the current segment has reached its bound, the sink finishes
the current segment (writing its trailer) and opens the next numbered file,
then writes the packet. With no policy the sink is a single segment, byte
identical to today's file sink.

**Rationale**: FR-001/FR-002, SC-002, D5. Finishing before opening the next
segment guarantees each file begins with its own SHB + IDBs and ends cleanly,
so every segment is independently readable. Numbering is a zero-padded ordinal
inserted before the extension (`name-00000.fcapng`), which sorts lexically in
capture order. The no-policy path preserves the committed goldens exactly.

**Alternatives considered**:
- Rotating mid-section and stitching a shared header. Rejected: it produces
  segments that are not independently valid, violating the requirement and P-5.
- Rotation living in the pipeline. Rejected: rotation is a property of the file
  transport, and the pipeline is format- and transport-agnostic by design.

## R6. Windows named pipe

**Decision**: A listener thread creates a named-pipe instance with
`CreateNamedPipeW` (message-agnostic byte pipe, outbound, `PIPE_UNLIMITED_
INSTANCES`), blocks in `ConnectNamedPipe`, and on connection hands the instance
handle (wrapped in a `Write`) to the streaming sink as a new consumer, then
loops to create the next instance. Implemented over `windows-sys` under
`cfg(windows)`.

**Rationale**: FR-003, D6. This is the textbook multi-client named-pipe server
and needs no overlapped IO. It reuses `windows-sys` already pinned at 0.36 by
S10, adding no `Cargo.lock` package and, per AGENTS.md, avoiding a second
`windows-sys` tree. Wireshark opens `\\.\pipe\<name>` directly as a capture
interface, which is the capability specification 14.2 singles out.

**Alternatives considered**:
- The `windows` (high-level) crate or a named-pipe crate. Rejected: adds a
  package (and likely a second `windows` binding tree) for FFI the project
  already does by hand elsewhere; inconsistent with the S09/S10 precedent of
  transcribing the small C ABI surface directly.
- Overlapped/async IO. Rejected: unnecessary for a blocking one-thread-per-
  instance server at fragcap's consumer counts.

**Verification note**: `windows-sys` 0.36 feature flags for
`CreateNamedPipeW`/`ConnectNamedPipe`/`CreateFileW` are confirmed at
implementation time against the vendored version; the exact feature module
names are pinned by a compiling test, not from memory.

## R7. TCP and Unix domain socket

**Decision**: TCP uses `std::net::TcpListener` in a listener thread, one
consumer per accepted `TcpStream`, cross-platform. The Unix domain socket uses
`std::os::unix::net::UnixListener` under `cfg(unix)`, same shape. Both yield a
`Box<dyn Write + Send>` connection to the shared streaming sink.

**Rationale**: FR-004/FR-005. Both are in the standard library, need no
dependency, and share the streaming machinery with the named pipe. The Unix
socket is `cfg(unix)` because `std::os::unix::net` is not available on Windows;
on Windows the `unix:` scheme is refused at configuration time.

**Alternatives considered**: `socket2` for finer socket control. Rejected: the
standard listeners are sufficient; no option they lack is needed.

## R8. Surfacing per-consumer accounting (P-4)

**Decision**: The streaming sink owns a per-consumer accounting record
(consumer identity, packets delivered, packets dropped, connect/disconnect
reason) exposed through the sink and rendered by the CLI's existing NDJSON event
stream (`--json`) and end-of-run summary. It is kept separate from
`CaptureStats.sink_dropped`, which stays capture-wide.

**Rationale**: FR-010, P-4. Per-consumer figures are the streaming sink's own,
not a pipeline counter; folding them into `sink_dropped` would break the
pipeline conservation identity and misreport downstream slowness as capture
loss. The CLI already emits structured events, so surfacing there needs no new
output channel.

**Alternatives considered**: Extending `CaptureStats` with per-consumer fields.
Rejected: `CaptureStats` is a `fragcap-core` type describing the capture, and
per-consumer streaming detail is transport state that belongs to the sink, not
the core capture record.

## R9. CLI sink grammar and `--mode stream`

**Decision**: `SinkSpec` gains a `Unix(PathBuf)` variant and the `unix:`
scheme. Per-sink options after the target, comma-separated `key=value`
(`format=`, `payload=`, and for the file sink `rotate-size=`,
`rotate-duration=`; for streaming sinks `queue=`, `timeout=`), are parsed per
specification 14.1. `build_sinks` constructs the real transports;
`reject_unsupported` drops the pipe/TCP "deferred to S15" stubs and keeps only
the platform-availability and configuration refusals. `--mode stream` is
enabled, and a run whose only sinks are streaming transports is valid.

**Rationale**: FR-006/FR-013/FR-014/FR-017. The grammar in 14.1 explicitly
shows comma-separated options on the sink, so parsing them is the honest
reading. Format resolution stays: infer from a file extension, otherwise the
explicit `format=` qualifier, and reject a destination with neither.

**Alternatives considered**: Global flags for rotation/queue instead of
per-sink options. Rejected: a run can carry several sinks with different
policies, and 14.1 attaches options to the sink; global flags could not express
two files with different rotation sizes.
