### Decisions

**2026-08-10: ring mode and triggers (slice S16), decisions worth recording for
promotion to specification section 29.**

- **The ring dump is the `Sink::finish` seam, not a new trigger path.** The
  pipeline already calls `finish(self, stats)` on every sink exactly once at
  drain, and drain is reached by all six session stop conditions. Ring mode
  therefore adds no code to the capture session, the pipeline, or the write gate;
  it swaps the sink built for `--out`. A dedicated ring-flush trigger observed by
  the orchestrator was rejected: it would re-implement drain and the stop
  conditions and risk the two disagreeing.
- **The retained window is a `VecDeque<CapturedPacket>` with evict-from-front,
  and no new dependency.** `CapturedPacket` owns its payload by reference-counted
  `Bytes`, so retaining a packet is a pointer clone, not a byte copy. The standard
  library deque is exactly the bounded-tail structure needed, the same reasoning
  S08 used to keep the pipeline buffer off a concurrency crate.
- **A size ring window is measured by captured length, matching `--max-bytes`,
  not by encoded pcapng block size.** An operator reasons about one notion of
  capture size across `--ring` and `--max-bytes`, and the retained set does not
  depend on the on-disk encoding. The dumped file is slightly larger than the
  window because it adds block framing and the mandatory header blocks, the same
  relationship a `--max-bytes` file has to its bound.
- **A duration window is measured back from the greatest capture instant
  observed, not from the last-arrived packet.** Using the last-arrived packet as
  the reference would let a late out-of-order packet carrying an old instant shrink
  the window and evict a genuinely recent packet, the dangerous (under-retention)
  direction. The running-max reference prevents that; a rare out-of-order old
  packet not at the front is over-retained (safe) rather than allowed to redefine
  "newest." A full `VecDeque::retain` scan per write that would evict every
  out-of-order old packet exactly was rejected as O(n-squared) over a capture, and
  the over-retention it avoids is harmless.
- **A ring eviction returns success and never advances `sink_dropped`.** Per the
  same argument S15 used for a streaming sink's per-consumer drops: the sink
  received every packet (conservation holds), and what it evicts from its window
  is the operator's declared retention scope, counted in the sink's own `evicted`
  accounting rather than the capture-wide loss counter (P-4, P-9).
- **Ring vocabulary is kept distinct from the section 12.4 bounded buffer.** The
  FR-8 capability is named ring mode, and the internal drop-oldest backpressure
  buffer of 12.4 stays the bounded buffer. Both are bounded, drop-oldest rings;
  conflating them would confuse a user-facing output mode with an internal
  mechanism. The glossary carries a ring-mode entry that names the distinction
  (constitution P-6).
- **The end-to-end ring run is proven through the CLI integration harness**
  (`crates/fragcap-cli/tests/cli_run.rs`) rather than a separate facade test: that
  harness already drives the whole offline pipeline through the real command
  entrypoint, including profile resolution and the write gate, so it subsumes what
  a facade-level test would assert. Both an interrupt trigger and a non-interrupt
  (terminal-stage-exit) trigger are exercised, and the whole-input window is shown
  equal in packet count to a plain file capture of the same input.

**2026-08-10: PR #30 review (Codex), three findings addressed.**

- **The eviction count is surfaced, not merely counted (P1).** The ring sink's
  `evicted` counter is now an `Arc<AtomicU64>` published through
  `RingSink::evicted_handle`; `build_sinks` keeps the handle and the orchestrator
  reads it after the run to emit a `ring.evicted` structured event and a summary
  progress line. Counting without surfacing was the P-4 gap: a run that rolled its
  window would otherwise report zero loss. This mirrors how a streaming sink's
  per-consumer drops reach the summary.
- **The dump file is opened at construction, not at finish (P1).** `RingSink::create`
  now opens the `--out` file eagerly (returning `Result`, like `RotatingFileSink::create`),
  so an unwritable destination fails before capture starts rather than discarding the
  whole captured window at drain.
- **The duration window compares in `i128` (P2).** `window.as_nanos() as i64` wrapped
  negative for a window beyond about 292 years, making a huge `--ring` retain only the
  newest packet; the comparison is now done in a non-wrapping representation.
