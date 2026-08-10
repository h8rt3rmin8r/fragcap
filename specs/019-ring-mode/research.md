# Phase 0 Research: Ring mode and triggers

Slice S16, specification section 7.2 (FR-8). Decisions taken under the autopilot
policy from the constitution, the architecture of record, and the existing sink,
session, and pipeline code. No item was left as NEEDS CLARIFICATION.

## R1. What is the dump trigger, and does ring mode add stop machinery?

**Decision**: No new stop machinery. The dump is the existing
`Sink::finish(self: Box<Self>, stats)` seam, which the pipeline calls exactly
once per sink at drain. Drain is reached by all six stop conditions already
implemented in the capture session (`StopReason`: `Interrupt`,
`DurationReached`, `TerminalStageExited`, `AllProcessesExited`, a
source-exhausted end, `SinkError`).

**Rationale**: "Writes the retained window to a capture file on trigger" (FR-8)
maps precisely onto "materialize at finish." The session lifecycle (S12) and the
write gate (S17) already decide when a capture ends and which packets are in
window; adding a parallel ring-specific trigger would duplicate that and risk
the two disagreeing. The worked invocation's "dumped on interrupt" is just the
`Interrupt` stop condition reaching `finish` like any other.

**Alternatives considered**: a dedicated ring-flush trigger observed by the
orchestrator (rejected: it re-implements drain and the stop conditions); flushing
the ring on a timer during capture (rejected: FR-8 is a rolling window dumped at
the end, not periodic snapshots, and periodic dumps are a different feature the
spec does not ask for).

## R2. Where does the retained window live, and in what structure?

**Decision**: A `RingSink` in `fragcap-sink` holding a
`std::collections::VecDeque<CapturedPacket>`. `write` pushes to the back and pops
from the front while the retained set exceeds the window; `finish` drains the
deque into a fresh pcapng encoder built by the existing `SinkFactory`.

**Rationale**: `RingSink` is a near-twin of `RotatingFileSink`: both are a `Sink`
wrapping a `SinkFactory`, but where the rotating sink writes through to an
encoder per packet, the ring sink buffers and writes through only at finish.
`CapturedPacket` owns its payload by reference-counted `Bytes` (a `Payload`), so
retaining a packet is a cheap pointer clone, not a byte copy. `VecDeque` is the
exact double-ended structure eviction-from-front needs. No third-party crate is
warranted, the same reasoning S08 used to keep the pipeline buffer on
`VecDeque` rather than reach for `crossbeam`.

**Alternatives considered**: a fixed-capacity array ring (rejected: the window is
by duration or by variable-size packets, not a fixed packet count, so a growable
deque bounded by policy is the natural fit); writing packets to a temp file and
trimming (rejected: needless IO, and the window is small and memory-bounded by
the operator's own `--ring` value).

## R3. How is the size window measured?

**Decision**: By each packet's captured length (`CapturedPacket::captured_len()`,
which is `data.len()`), summed as a running retained-bytes total. This is the
identical quantity the `--max-bytes` volume bound uses (the write gate sums
`packet.data.as_ref().len()`; the session's `retained_bytes` is the same).

**Rationale**: an operator who writes `--ring 64mb` and an operator who writes
`--max-bytes 64mb` should be reasoning about the same notion of "capture size."
Measuring the ring by encoded pcapng block size instead would make the retained
set depend on the on-disk framing and diverge from `--max-bytes` for no operator
benefit. The dumped file is then slightly larger than the window (block framing
plus the mandatory header blocks), exactly as a `--max-bytes` file already
exceeds its bound by its framing.

**Alternatives considered**: encoded-block-size accounting (rejected as above);
`orig_len` / on-wire length (rejected: a metadata-only or snap-limited capture
retains fewer bytes than the wire length, and the file the operator gets is sized
by captured bytes, so captured length is the honest measure of what is retained).

## R4. Duration window origin.

**Decision**: Measured back from the greatest capture instant observed so far (a
running max), not from the last-arrived packet. After each push, evict every
front packet whose instant is more than the window before that greatest instant.

**Rationale**: this makes the retained set the recent tail by capture instant and
keeps the sink independent of when drain runs, matching how the write gate
classifies a packet by its own capture instant rather than the wall clock at
processing time. Using the last-arrived packet as the reference would let a late
out-of-order packet carrying an old instant shrink the window and evict a
genuinely recent packet, which is the dangerous (under-retention) direction; the
running max prevents that. A rare out-of-order old packet that is not at the
front is over-retained (kept until it reaches the front), which is the safe
direction: the retained set may briefly exceed the window but never drops a
recent packet. In the common monotonic case front eviction is exact and
O(evicted).

**Alternatives considered**: last-arrived instant as the reference (rejected: a
stale late packet would corrupt the window); a full `VecDeque::retain` scan per
write to evict every out-of-order old packet exactly (rejected: O(n) per packet,
O(n-squared) over a capture, unacceptable for a large ring, and the over-retention
it avoids is harmless); `Instant::now()` at each write (rejected: couples
retention to processing time, not capture time, and breaks deterministic offline
replay).

## R5. The window-smaller-than-one-packet degenerate case.

**Decision**: Eviction never reduces the deque below one packet. The newest
packet is always retained even if it alone exceeds the size window.

**Rationale**: a capture that observed traffic must not dump an empty file; that
would report "nothing captured" when packets were seen, a P-9/P-4 falsehood. This
is the retained-inclusive rule the write gate already applies to `--max-bytes`,
which admits the crossing packet.

## R6. Conservation accounting for evictions.

**Decision**: `RingSink::write` returns `Ok` for every delivered packet. An
eviction is the sink's own retention decision, surfaced as an evicted (and
retained) count on the sink, not a capture-loss counter. It never advances the
pipeline's `sink_dropped` or the session discard tallies.

**Rationale**: the pipeline conservation invariant is "the sink received every
packet" (received + buffer_dropped + refusals = captured). The ring sink does
receive every packet; what it later evicts from its retained window is a scope
decision the operator made by choosing a window, which P-9 explicitly permits as
declared omission provided it is counted (P-4). This is the exact shape S15 used
for per-consumer streaming drops: the sink's own accounting, never the
capture-wide counter, never a retirement.

## R7. CLI resolution and refusals.

**Decision**: Keep the refusal logic in `reject_unsupported` / `effective_config`
(where the effective mode is already computed as command line over profile
default) and in `build_sinks`. Replace the two "not yet supported (slice S16)"
errors with: (a) in ring mode, require `--out` and `--ring`, else usage error
naming the missing flag; (b) in ring mode, refuse `--max-bytes`/`--max-packets`
with a message explaining a rolling window does not stop on volume; (c) refuse
`--ring` given outside ring mode. When the effective mode is ring, `build_sinks`
constructs a `RingSink` over the `--out` file instead of a `RotatingFileSink`.

**Rationale**: this reuses the exact seam S15 used for its transport and launch
refusals, so ring mode's configuration errors are reported before capture starts
with a message naming the cause, and the effective-mode resolution (so a profile
`mode = "ring"` is honored) is already in place.

**Alternatives considered**: making `--out`/`--ring` clap-required only in ring
mode via argument groups (rejected: the mode is resolved from the profile too, so
the requirement cannot be expressed purely in the clap grammar and must be a
post-parse check anyway; a single post-parse check is clearer than a split one).

## R8. Multiple interfaces in the retained window.

**Decision**: The deque holds packets from all declared interfaces interleaved by
arrival; eviction is global (oldest first regardless of interface). The dump
declares every interface (one Interface Description Block each), and each retained
packet references its own interface, exactly as the file sink does.

**Rationale**: the pipeline already feeds one bounded buffer from all interfaces
(specification 12.1, 12.4), and `CapturedPacket` carries its `InterfaceId`. The
ring is one more downstream buffer with the same global-ordering property; a
per-interface ring would be a different, unrequested feature and would complicate
the "recent tail" definition.
