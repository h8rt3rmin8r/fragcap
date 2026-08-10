# Feature Specification: Ring mode and triggers

**Feature Branch**: `019-ring-mode`

**Created**: 2026-08-10

**Status**: Draft

**Input**: Roadmap slice S16 (spec section 7.2, FR-8 ring capture, and the
ring-dump trigger). Deliver ring mode: a rolling in-memory window of captured
packets bounded by a duration or a byte size, discarding the oldest retained
content as new packets arrive, written to a capture file on a terminating
trigger. The terminating triggers are the six session stop conditions already
implemented in S12 and S17; ring mode reuses them rather than adding a parallel
mechanism. Wire `--mode ring` and `--ring` in the command surface, replacing the
current refusals that name this slice.

## Clarifications

### Session 2026-08-10

Resolved under autopilot from the constitution, the architecture of record
(specification sections 7.2, 8.6, 12.4, and 17.2), and the existing sink and
session contracts. No item required an operator call.

- Q: Where does the retained window live, and what materializes it? → A: A new
  ring sink in `fragcap-sink` implementing the existing `Sink` trait
  (`write`/`flush`/`finish`) over the existing `SinkFactory` and pcapng format
  writer. `write` enqueues each accepted packet into a bounded in-memory deque
  and evicts the oldest to keep the retained set within the window; `finish`
  materializes the retained window as one independently valid pcapng file at the
  `--out` path. The dump is the `finish` seam, which the pipeline already calls
  once at drain for every stop condition, so no new trigger path is introduced.
- Q: What is the terminating trigger that dumps the ring? → A: Every one of the
  six session stop conditions of specification section 10.6 (operator interrupt,
  duration bound, terminal-stage exit, all-non-service-processes-exited, source
  exhaustion, unrecoverable sink error). They are already implemented in the
  capture session; ring mode adds no stop condition. The headline case is the
  interrupt, per the worked invocation "rolling ten-minute window, dumped on
  interrupt."
- Q: Does the ring window interact with the write gate's volume bounds? → A: No.
  The ring window is a retention policy applied inside the ring sink to packets
  the write gate has already admitted. It is not a stop condition and does not
  touch `SessionGate`. The volume stop bounds `--max-bytes` and `--max-packets`
  are meaningless in ring mode (a ring never stops on accumulated volume; it
  rolls) and are refused as configuration errors when combined with ring mode.
- Q: Is a ring eviction a dropped packet the pipeline must count as loss? → A:
  No. The pipeline delivers each admitted packet to the ring sink, whose `write`
  returns success; conservation (received plus buffer-dropped plus refusals
  equals captured) is preserved exactly as for any other sink. An eviction is a
  retention decision the ring sink makes over packets it already accepted, and
  the count of evicted packets is the ring sink's own reported accounting, not a
  capture-loss counter. This mirrors how a streaming sink's per-consumer drops
  (S15) are the sink's own accounting and never advance `sink_dropped`.
- Q: What are the ring dump targets in this slice? → A: The single `--out`
  capture file, written as pcapng. Ring mode requires `--out`. Concurrent
  streaming or additional file `--sink` destinations alongside a ring `--out`
  are out of scope for this slice; only the `--out` file is the ring dump
  target.
- Q: The term "ring buffer" already denotes the pipeline's internal bounded
  backpressure buffer (specification 12.4). How is the collision avoided? → A:
  The FR-8 capability is named **ring mode** (equivalently ring capture, and the
  retained set is the ring window). The glossary carries an entry for ring mode
  that explicitly distinguishes it from the internal ring buffer, added in this
  same change (constitution P-6).
- Q: A size ring window bounds "total retained size" -- is that measured by each
  packet's captured length or by its encoded pcapng block size (with block
  headers and padding)? → A: By captured length, the same quantity the
  `--max-bytes` volume bound already sums (`retained_bytes` in the session). An
  operator then reasons about one notion of capture size across `--ring 64mb`
  and `--max-bytes 64mb`, and the retained set does not depend on the on-disk
  encoding. The dumped file is correspondingly slightly larger than the window
  because it adds the block framing and the mandatory header blocks, which is
  the same relationship a `--max-bytes` file already has to its bound.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Keep only the tail, dumped on interrupt (Priority: P1)

An operator wants a capture of what just happened, not the whole session. They
run fragcap in ring mode with a bounded window and let it run. fragcap holds a
rolling window of the most recent traffic in memory, continuously discarding
older content, and when the operator interrupts the capture it writes the
retained window to the output file. The file opens in an unmodified analyzer and
contains the recent tail, with attribution carried in ordinary packet comments.

**Why this priority**: This is the whole capability of the slice and the exact
worked invocation the specification gives ("rolling ten-minute window, dumped on
interrupt"). Without it ring mode does not exist.

**Independent Test**: Replay the fixture corpus through ring mode with a window
smaller than the corpus, fire the interrupt, and confirm the output file is
valid pcapng containing exactly the most recent packets that fit the window, in
capture order, with everything older evicted.

**Acceptance Scenarios**:

1. **Given** a ring capture with a size window smaller than the input, **When**
   the capture stops, **Then** the output file is a single valid pcapng
   beginning with a Section Header Block and one Interface Description Block per
   declared interface, followed by only the most recent packets whose total
   retained size is within the window, in capture order.
2. **Given** a ring capture with a duration window smaller than the input's
   timespan, **When** the capture stops, **Then** the output retains exactly the
   packets whose capture instant is within the window measured back from the
   newest retained packet, and no older packet.
3. **Given** a ring capture whose window is larger than the whole input, **When**
   the capture stops, **Then** the output retains every packet, and the packet
   records equal those of a plain `--out` file capture of the same input with
   none lost, reordered, or duplicated.
4. **Given** a ring capture, **When** any of the six session stop conditions
   ends the capture (not only an interrupt), **Then** the retained window is
   dumped once, to the same output file, by the same path.

---

### User Story 2 - Ring mode is configured unambiguously or refused (Priority: P1)

An operator invokes ring mode. If they omit the output file the window has
nowhere to be dumped; if they omit the window there is nothing to bound the
retention; if they combine ring mode with a volume stop bound the two disagree
about what stops the capture. In each case fragcap refuses the invocation before
capture starts, with a message naming the specific missing or conflicting flag,
rather than starting a capture that cannot produce what was asked for.

**Why this priority**: A ring capture that starts and then cannot dump, or that
silently ignores a window, is the configuration-side form of the loss the
project forbids: it runs, exits zero, and produces nothing the operator wanted.
Refusing before capture is as essential as the capability itself.

**Independent Test**: Invoke ring mode missing `--out`, missing `--ring`, and
with `--max-packets`/`--max-bytes`, and confirm each is a configuration error
(exit 2) whose message names the cause, with no capture started.

**Acceptance Scenarios**:

1. **Given** `--mode ring` with no `--out`, **When** the command is invoked,
   **Then** it is rejected as a configuration error naming the missing output
   file, before capture starts.
2. **Given** `--mode ring` with no `--ring`, **When** the command is invoked,
   **Then** it is rejected as a configuration error naming the missing ring
   window.
3. **Given** `--mode ring` combined with `--max-bytes` or `--max-packets`,
   **When** the command is invoked, **Then** it is rejected as a configuration
   error explaining that a volume stop bound does not apply to a rolling window.
4. **Given** `--ring` given without `--mode ring` (a ring window in a non-ring
   mode), **When** the command is invoked, **Then** it is rejected as a
   configuration error, because the window would otherwise be silently ignored.

---

### Edge Cases

- A ring window smaller than a single packet's retained size: the window admits
  the newest packet even though it alone exceeds the size window, so the dump is
  never empty when at least one packet was captured; retaining zero packets when
  traffic was seen would misreport the capture as containing nothing.
- A ring capture over an input that produced no in-window packets at all (the
  session never acquired a target, or every packet was discarded upstream by the
  write gate): the dump is a valid pcapng with its header blocks and no packet
  records, the same well-formed empty capture a plain file sink produces.
- A duration window when packets are not strictly ordered by capture instant:
  the window is measured back from the greatest capture instant observed, not
  from the last-arrived packet, so a late out-of-order packet carrying an old
  instant never redefines "newest" and never evicts a genuinely recent packet.
  Such a stale late packet is over-retained (kept rather than dropped), which is
  the safe direction: the retained set may briefly hold slightly more than the
  window, but never less, so a recent packet is never lost to reordering.
- The `--out` path is unwritable (a bad directory, a permission error): the
  failure surfaces the same way a plain file capture's does, as a run failure
  naming the path, rather than being discovered only at dump time with the
  window already gone.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST provide a ring capture mode that retains a rolling
  in-memory window of the most recently captured packets, discarding the oldest
  retained packets as new ones arrive to keep the retained set within a
  configured window.
- **FR-002**: The ring window MUST be expressible as either a duration or a byte
  size. A duration window retains packets whose capture instant is within the
  window measured back from the newest retained packet; a size window retains
  the newest packets whose total retained captured length is within the window,
  where captured length is the same per-packet quantity the `--max-bytes` volume
  bound sums.
- **FR-003**: The system MUST write the retained window to the output capture
  file when the capture ends, for every one of the six session stop conditions
  (operator interrupt, duration bound, terminal-stage exit,
  all-non-service-processes-exited, source exhaustion, unrecoverable sink
  error), by a single dump path rather than a per-condition one.
- **FR-004**: The dumped file MUST be a single, independently valid pcapng that
  an unmodified analyzer opens: a Section Header Block, one Interface Description
  Block per declared interface, then the retained packet blocks in capture
  order, carrying the same attribution comments the file sink already writes.
- **FR-005**: Ring mode MUST require an output file (`--out`) and a ring window
  (`--ring`). An invocation missing either MUST be reported as a configuration
  error before capture starts, naming the missing flag.
- **FR-006**: The volume stop bounds (`--max-bytes`, `--max-packets`) MUST be
  refused in ring mode as a configuration error, because a rolling window does
  not stop on accumulated volume.
- **FR-007**: A ring window supplied without ring mode MUST be refused as a
  configuration error, so the window is never silently ignored.
- **FR-008**: The command surface MUST resolve ring mode as the command line
  over the profile's `[capture]` default (an explicit `--mode ring` wins, and a
  profile declaring `mode = "ring"` with no override selects ring mode),
  replacing the current stub refusals that name this slice.
- **FR-009**: A ring eviction MUST NOT be counted as a captured-packet loss. The
  ring sink accepts every packet the pipeline delivers and returns success, so
  the pipeline's conservation accounting is preserved; the count of evicted
  packets is the ring sink's own reported accounting.
- **FR-010**: `--duration` MUST remain valid in ring mode, ending the capture
  (and so dumping the window) after the elapsed bound, exactly as it does in
  file mode.
- **FR-011**: The ring capability MUST introduce a glossary entry for ring mode
  in the same change, distinguishing it from the pipeline's internal bounded
  ring buffer of specification section 12.4 (constitution P-6).
- **FR-012**: A single-interface, whole-input ring capture (a window larger than
  the input) MUST produce a packet record sequence equal to a plain `--out` file
  capture of the same input, with no packet lost, reordered, or duplicated.

### Key Entities *(include if data involved)*

- **Ring mode**: The capture mode (FR-8) in which fragcap retains a rolling
  window of recent packets and dumps it to a file on a terminating trigger.
  Distinct from the internal ring buffer of specification 12.4.
- **Ring window**: The bound on the retained set, a duration or a byte size,
  from `--ring`. Governs which packets the ring sink keeps and which it evicts. A
  size window is measured by captured length, matching `--max-bytes`.
- **Ring sink**: The output sink that holds the retained window in memory during
  capture and materializes it to the output file at drain.
- **Terminating trigger**: Any of the six session stop conditions that ends the
  capture and so causes the ring window to be dumped. Not a new mechanism; the
  existing stop conditions.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A ring capture with a size or duration window smaller than the
  input produces an output file that an unmodified pcapng parser accepts in full
  and that contains exactly the most recent packets fitting the window, with
  every older packet evicted.
- **SC-002**: A ring capture with a window larger than the whole input produces
  a packet record sequence identical to a plain `--out` file capture of the same
  input, with zero packets lost, reordered, or duplicated.
- **SC-003**: 100% of the six session stop conditions dump the retained window
  to the output file; an interrupt, a duration bound, and a source-exhaustion
  stop each produce the same well-formed dump.
- **SC-004**: Every misconfigured ring invocation (no `--out`, no `--ring`, a
  volume bound, or a ring window without ring mode) is rejected with exit code 2
  and a message naming the cause, and no capture is started.
- **SC-005**: The pipeline conservation invariant (received plus buffer-dropped
  plus refusals equals captured) holds for a ring capture exactly as for a file
  capture; a ring eviction advances no capture-loss counter.
- **SC-006**: The full repository gate (`cargo xtask ci`) passes, and the
  platform-neutral core build (`cargo xtask neutral`, which `ci` does not run)
  still builds, with ring mode covered by tests that run with no capture driver,
  no elevation, and no game.

## Assumptions

- The pcapng format writer and `SinkFactory` from S06 and S15 are reused
  unchanged; ring mode adds a sink around them and does not modify the
  byte-level format output. A whole-input ring dump of a single-interface
  capture is byte-comparable to the current file output.
- The six session stop conditions and the write gate from S12 and S17 are the
  integration seam and are not modified. Ring mode attaches a different sink; it
  does not change the session lifecycle, the stop conditions, or the write
  gate's admit decision.
- The default retained-window is always explicit: ring mode has no implicit
  window, and the absence of `--ring` is an error rather than a default, so an
  operator never gets a differently sized window than they asked for.
- Tests replay the committed fixture corpus through ring mode with a small
  window and read the dumped file back with the same parser the JSON and pcapng
  writer tests already use, so the whole slice is verifiable at tier 1 with no
  external software, consistent with the project's offline-testability
  discipline.
- Concurrent sinks alongside a ring `--out` (a ring dump plus a live stream, or
  multiple ring files) are out of scope for this slice; the ring dump target is
  the single `--out` file.
