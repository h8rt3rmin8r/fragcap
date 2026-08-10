# Feature Specification: Transports and streaming sinks

**Feature Branch**: `018-transports-streaming-sinks`

**Created**: 2026-08-10

**Status**: Draft

**Input**: Roadmap slice S15 (spec sections 14.1 to 14.4). Fill in the transport
half of the sink model: file rotation, named pipe, Unix domain socket, and TCP
transports, each serving one or more consumers with per-consumer valid streams
and independent bounded backpressure. Format (pcapng, JSON Lines) already
exists and stays orthogonal to transport.

## Clarifications

### Session 2026-08-10

Resolved under autopilot from the constitution, the architecture of record, and
the existing pipeline contract (no unresolved item required an operator call).

- Q: Which crate owns the transport layer, given P-2 requires `fragcap-core` to
  stay platform-neutral and `fragcap-sink` currently carries no platform or
  networking dependency? → A: `fragcap-sink` owns every transport. Cross-platform
  transports (file, TCP) are unconditional; platform-specific ones (the Windows
  named pipe, the Unix domain socket) are gated by `cfg(target_os)` and/or a
  crate feature so the transport set on a target is determined at compile time.
  `fragcap-core` and the `Sink` trait it defines take no transport dependency,
  and the core build for a backendless target stays green. `fragcap-sink` is not
  core and may take platform dependencies.
- Q: When a streaming sink drops a packet for one consumer whose bounded queue
  is full, does that advance the pipeline's capture-wide `sink_dropped` counter?
  → A: No. A streaming sink's `write` accepts the packet (enqueues to each
  connected consumer and records a per-consumer drop where a queue is full) and
  returns success to the pipeline. Per-consumer drops are a separate, named,
  per-consumer accounting owned and surfaced by the streaming sink; they never
  advance the pipeline's capture-wide `sink_dropped` and never retire the sink.
  This preserves the pipeline's conservation invariant (the sink received every
  packet) while satisfying P-4 through the sink's own reported counters.
- Q: What does a streaming sink's `write` return when no consumer is connected?
  → A: Success. A streaming transport with zero consumers is idle, not failing:
  nothing was promised to a consumer, so no drop is counted, no `sink_dropped`
  advances, and the sink is not retired. This differs from a file sink, whose
  write failure is a real destination failure that retires the sink through the
  existing pipeline path.
- Q: In stream mode (`--mode stream`, currently deferred to this slice), is a
  run whose only sinks are streaming transports and which writes no capture file
  valid? → A: Yes. Stream mode permits a capture whose sinks are all streaming
  transports; a file sink is not required. The command surface enabling
  `--mode stream` is part of this slice's CLI wiring.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Live analysis over a named pipe (Priority: P1)

An analyst wants to watch a game client's traffic live, with attribution, in
an unmodified analyzer. They start fragcap with a named-pipe sink and point
Wireshark at `\\.\pipe\fragcap`. Wireshark opens the pipe as a capture
interface and shows packets as they are captured, each carrying its process
attribution in an ordinary comment. No fragcap plugin is installed in the
analyzer.

**Why this priority**: This is the transport the specification singles out as
"the transport that makes live analysis work with no additional software." It
is the headline capability of the slice and the reason streaming sinks exist.

**Independent Test**: Start a capture with a named-pipe sink, connect a reader
to the pipe, and confirm the reader receives a well-formed pcapng stream (a
Section Header Block, one Interface Description Block per declared interface,
then packet blocks) that an unmodified pcapng parser accepts.

**Acceptance Scenarios**:

1. **Given** a capture writing to a named-pipe sink, **When** a consumer
   connects before the first packet, **Then** the consumer receives a Section
   Header Block and every declared Interface Description Block, followed by the
   packet blocks in capture order.
2. **Given** a capture already in progress writing to a named-pipe sink,
   **When** a consumer connects mid-capture, **Then** the consumer receives its
   own Section Header Block and Interface Description Blocks first, then only
   packets captured after it connected, and never a prior packet.
3. **Given** two consumers connected to the same named-pipe sink, **When**
   packets are captured, **Then** each consumer receives a complete,
   independently valid stream declaring the same interfaces.

---

### User Story 2 - Long capture with file rotation (Priority: P1)

An operator captures for an extended session and does not want a single
unbounded file. They start fragcap with a file sink and a rotation policy by
size or by duration. fragcap writes numbered segments, closing each at a clean
section boundary so every segment opens on its own in an unmodified analyzer.

**Why this priority**: Unbounded capture files are unusable for long sessions,
and clean-boundary rotation is what lets an operator inspect a segment while
capture continues. It is the file-side deliverable of the slice and is
testable entirely offline.

**Independent Test**: Run the corpus through a rotating file sink with a small
size threshold, then confirm the produced segments are numbered, each opens
independently as valid pcapng, and their concatenated packet sequence equals
the un-rotated output with no packet lost or duplicated.

**Acceptance Scenarios**:

1. **Given** a file sink with a size-based rotation policy, **When** the
   written bytes for the current segment would cross the threshold at a section
   boundary, **Then** the current segment is closed and a new numbered segment
   is started.
2. **Given** a file sink with a duration-based rotation policy, **When** the
   elapsed time since the current segment opened crosses the threshold at a
   section boundary, **Then** the current segment is closed and a new numbered
   segment is started.
3. **Given** any produced segment, **When** it is opened in an unmodified
   analyzer, **Then** it parses cleanly on its own, beginning with a Section
   Header Block and its Interface Description Blocks.
4. **Given** a completed rotated capture, **When** the segments' packets are
   read in order, **Then** the total set of packets equals a single-file
   capture of the same input, with none lost, reordered across the join, or
   duplicated.

---

### User Story 3 - Remote or containerized consumer over TCP (Priority: P2)

A consumer that cannot reach a local pipe, for example an analyzer in a
container or on another host, connects to fragcap over TCP. fragcap listens on
a configured address and port and writes the chosen format to each connected
client.

**Why this priority**: TCP is required by the specification for consumers that
cannot reach a local pipe. It reuses the same multi-consumer streaming
machinery as the named pipe, so it is lower cost once that machinery exists,
but it is a distinct, separately valuable delivery path.

**Independent Test**: Start a capture with a TCP sink bound to a loopback port,
connect a client, and confirm the client receives an independently valid stream
in the configured format.

**Acceptance Scenarios**:

1. **Given** a TCP sink bound to an address and port, **When** a client
   connects, **Then** the client receives a complete, independently valid
   stream in the configured format from the connection point onward.
2. **Given** a TCP sink with two connected clients, **When** packets are
   captured, **Then** each client receives its own complete, independently
   valid stream.

---

### User Story 4 - A slow or dead consumer never stalls capture (Priority: P2)

One consumer stops reading (its analyzer is paused, its host is slow, or its
connection is dead). Capture continues at full rate. The file sink and every
other consumer are unaffected. The slow consumer has packets dropped on its own
connection only, those drops are counted and reported, and a consumer that
stays stalled beyond a timeout is disconnected and the disconnection is logged.

**Why this priority**: Without per-consumer isolation, a single stalled network
reader would corrupt or stall the whole capture, which violates the project's
no-silent-loss posture and makes concurrent file-and-stream capture unsafe. It
is essential to the streaming transports but depends on them existing first.

**Independent Test**: Attach a streaming sink with two consumers, stop one from
reading, keep feeding packets, and confirm the reading consumer receives the
full stream, the stalled consumer's drops are counted per consumer, and the
stalled consumer is disconnected after the timeout while capture never blocks.

**Acceptance Scenarios**:

1. **Given** a streaming sink with a fast and a slow consumer, **When** the
   slow consumer stops reading and its queue fills, **Then** further packets
   are dropped on the slow consumer's connection only, the fast consumer
   receives every packet, and capture does not block.
2. **Given** a consumer whose queue has filled, **When** packets are dropped
   for it, **Then** each drop is counted in a named per-consumer counter and
   surfaced in the run's statistics.
3. **Given** a consumer whose queue remains full beyond the disconnect
   timeout, **When** the timeout elapses, **Then** the consumer is disconnected
   and the disconnection is logged with the consumer's identity and the reason.
4. **Given** a streaming sink and a file sink attached to the same capture,
   **When** a streaming consumer stalls, **Then** the file sink continues
   receiving and writing every packet.

---

### User Story 5 - Unix domain socket parity (Priority: P3)

An operator (or a future non-Windows platform) uses a Unix domain socket sink,
which creates a socket at a filesystem path and serves connected consumers with
the same streaming semantics as the named pipe.

**Why this priority**: The specification lists the Unix domain socket as
present "for parity and for future platform support." It carries no unique
capability today on Windows and so is the lowest priority, but including it
keeps the transport set complete and the abstraction honest.

**Independent Test**: On a platform with Unix domain socket support, start a
capture with a Unix socket sink, connect a consumer, and confirm it receives an
independently valid stream with the same per-consumer semantics as the pipe.

**Acceptance Scenarios**:

1. **Given** a Unix domain socket sink, **When** a consumer connects, **Then**
   it receives a complete, independently valid stream from the connection point
   onward with per-consumer backpressure identical to the named pipe.

---

### Edge Cases

- A named-pipe or TCP sink with no consumer connected: capture proceeds, the
  sink accepts and discards (or holds no more than its own bounds), nothing
  stalls, and packets not delivered to any consumer are not miscounted as
  capture loss.
- A consumer disconnects abruptly mid-stream: the sink detects the closed
  connection, stops writing to it, logs the disconnection, and continues
  serving the remaining consumers without dropping their packets.
- A file rotation policy with a threshold smaller than a single section's
  mandatory header blocks: rotation still produces valid segments and never
  emits a segment that cannot hold its own header.
- A destination whose format cannot be inferred from an extension (a pipe name,
  a TCP authority) and no explicit `format=` qualifier: the sink specification
  is rejected as a configuration error naming the missing qualifier, before
  capture starts.
- A named-pipe sink requested on a non-Windows target: the sink scheme is
  refused at configuration time with a message naming the platform limitation,
  rather than failing opaquely at capture start.
- Two sinks configured for the same pipe name or TCP address: the second bind
  fails, and the failure is reported at startup as a configuration error, not
  silently.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST provide a file transport that writes a chosen
  format to a path, and, when a rotation policy is configured, MUST close the
  current segment at a clean pcapng section boundary and open a new numbered
  segment.
- **FR-002**: File rotation MUST support a size threshold and a duration
  threshold, and each produced segment MUST be independently readable by an
  unmodified analyzer.
- **FR-003**: The system MUST provide a named-pipe transport that creates a
  pipe under `\\.\pipe\` on Windows and writes the chosen format to connected
  clients.
- **FR-004**: The system MUST provide a TCP transport that listens on a
  configured address and port and writes the chosen format to connected
  clients.
- **FR-005**: The system MUST provide a Unix domain socket transport that
  creates a socket at a filesystem path and writes the chosen format to
  connected clients on platforms that support it.
- **FR-006**: Format MUST be orthogonal to transport: any implemented format
  writes to any implemented transport. Format MUST be inferred from the
  destination extension where one exists and otherwise taken from an explicit
  format qualifier on the sink specification.
- **FR-007**: The named-pipe and TCP transports MUST accept multiple
  simultaneous consumers, and each connected consumer MUST receive a complete,
  independently valid stream: for pcapng, its own Section Header Block and
  Interface Description Blocks before any packet data, declaring the same
  interfaces as every other consumer.
- **FR-008**: A consumer connecting mid-capture MUST receive a valid stream
  beginning at the connection point and MUST NOT receive packets captured
  before it connected.
- **FR-009**: Each consumer MUST have an independent bounded queue. A consumer
  whose queue fills MUST have subsequent packets dropped on its own connection
  only, with no effect on other consumers, on the file sink, or on capture
  throughput.
- **FR-010**: Per-consumer drops MUST be counted in a named, per-consumer
  counter owned and surfaced by the streaming sink. They MUST NOT advance the
  pipeline's capture-wide `sink_dropped` counter, and a streaming sink MUST
  return success to the pipeline for every packet it accepts (including when no
  consumer is connected), so the pipeline's conservation invariant is preserved
  and the sink is never retired for per-consumer backpressure.
- **FR-011**: A consumer whose queue remains full beyond a configured timeout
  MUST be disconnected, and the disconnection MUST be logged with the
  consumer's identity and the reason.
- **FR-012**: A slow, stalled, or dead network consumer MUST NOT stall the
  capture pipeline or any other sink. Concurrent file-and-stream capture MUST
  remain safe.
- **FR-013**: The command surface MUST wire every sink scheme (`file:`,
  `pcapng:`, `jsonl:`, `pipe:`, `unix:`, and `tcp://`) to its real transport,
  replacing the current deferral stubs for the pipe and TCP schemes and adding
  the Unix socket scheme.
- **FR-014**: A sink specification whose format cannot be resolved (no
  inferable extension and no explicit qualifier), whose transport is
  unavailable on the current platform, or whose destination cannot be bound
  MUST be reported as a configuration error before capture starts, naming the
  specific cause.
- **FR-015**: No transport may alter, mask, truncate, reorder, or withhold an
  observation it forwards, other than the counted, reported per-consumer
  backpressure drops of FR-009 and FR-010.
- **FR-016**: Platform-specific transport code MUST NOT introduce a
  platform-specific dependency or assumption into the platform-neutral core;
  the transport set available on a given target MUST be determined without
  breaking a core build for a target that has no capture backend.
- **FR-017**: The command surface MUST enable `--mode stream`, and a run whose
  only sinks are streaming transports (no capture file) MUST be valid.

### Key Entities *(include if feature involves data)*

- **Transport**: A destination a sink writes to (file, named pipe, Unix domain
  socket, TCP). Orthogonal to format. Determines how bytes reach consumers and
  whether multiple consumers are supported.
- **Rotation policy**: A rule (by size or by duration) that governs when a file
  transport closes the current segment and starts the next numbered one at a
  clean section boundary.
- **Consumer**: A single connected reader of a multi-consumer transport, with
  its own bounded queue, its own independently valid stream, and its own drop
  and disconnect accounting.
- **Consumer queue**: The bounded, drop-when-full buffer standing between the
  capture path and one consumer's connection, isolating that consumer's speed
  from the rest of the capture.
- **Sink specification**: The parsed `--sink` value combining a transport
  destination with a resolved format and its options.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A consumer connecting to a named-pipe or TCP sink at any time,
  before or during capture, receives a stream that an unmodified pcapng parser
  accepts in full, with a correct header preamble and only packets from the
  connection point onward.
- **SC-002**: 100% of file-rotation segments open independently as valid
  pcapng, and the union of their packets equals the un-rotated capture of the
  same input, with zero packets lost, duplicated, or reordered across segment
  joins.
- **SC-003**: With one consumer stalled and another reading, the reading
  consumer receives 100% of packets and the file sink writes 100% of packets,
  while the stalled consumer's dropped-packet count is reported exactly and is
  non-zero.
- **SC-004**: A consumer stalled beyond the disconnect timeout is disconnected
  and the event appears in the run's log with the consumer's identity; capture
  throughput shows no stall attributable to that consumer.
- **SC-005**: Every sink scheme accepted by the command surface resolves to a
  working transport or is rejected before capture with a message naming the
  cause; no accepted scheme reaches capture as an unimplemented stub.
- **SC-006**: The full repository gate (`cargo xtask ci`) passes, and the
  platform-neutral core build (`cargo xtask neutral`, which `ci` does not run)
  is confirmed to still build, with the new transports covered by tests that run
  without a capture driver, elevation, or a game.

## Assumptions

- The pcapng and JSON Lines format writers from S06 and S07 are reused
  unchanged; this slice adds transports around them and does not modify the
  byte-level format output. A single-consumer, single-segment capture remains
  byte-identical to the current file output.
- The pipeline's existing per-sink model (a sink is retired on a non-countable
  failure and each withheld packet advances `sink_dropped`) is the integration
  seam. Per-consumer accounting is additional detail a streaming sink reports;
  it does not change the pipeline's per-sink contract.
- Default per-consumer queue depth, disconnect timeout, and (when rotation is
  requested) default thresholds are chosen as documented defaults in the plan;
  the operator can override them. Their exact values are an implementation
  decision recorded in the plan, not a scope question.
- The named-pipe transport is Windows-only and the Unix domain socket transport
  is available where the platform supports it; the file and TCP transports are
  cross-platform. The platform gating mirrors how S09 and S10 gate their
  platform surfaces behind features, keeping the core build platform-neutral.
- Tests for the streaming transports use in-process readers (a connected pipe,
  socket, or loopback TCP client) rather than a running analyzer, so the whole
  slice is verifiable at tier 1 with no external software, consistent with the
  project's offline-testability discipline. Interoperability with a real
  analyzer is asserted by producing bytes a standard parser accepts, not by
  driving the analyzer.
- Ordering, retention, and per-consumer isolation are verified with the same
  conservation invariant the pipeline already checks (received plus dropped
  plus refused equals captured), extended per consumer where the streaming sink
  reports it.
