# Research and Decisions: The Session Gates Sink Writes

**Feature**: `017-session-gate-writes` | **Date**: 2026-08-10

This slice reverses two S14 decisions (D-c's "the session never sees packets" and
D-e's "a tee observes but does not decide") for the two cases they were wrong for,
and keeps them for the case they were right for. The decisions below record the
reversal precisely so a later reader does not relitigate it without the argument.

## D-1. A generic `WriteGate` seam in core, consulted by the output loop

**Decision**: add a `WriteGate` trait to `fragcap-core`
(`traits.rs`): `Send + Sync`, one method `admit(&self, packet: &CapturedPacket)
-> bool`, interior mutability. `Pipeline` gains an
`Option<Arc<dyn WriteGate>>` set by a `set_write_gate` setter (mirroring
`set_filter_config`, so no `Pipeline::new` caller changes). `output_loop` consults it
once per packet, before the per-sink fan-out: a packet the gate does not admit is
counted and skipped for every sink.

**Rationale**: the gate decision must be synchronous with the write to make the bound
hard (a packet the gate rejects is never written). The output loop is the one place
the write happens, so the gate belongs there. It is generic because constitution P-3
keeps `fragcap-core` free of session and profile knowledge: the trait answers a
question about a packet, and the session-aware answer lives in the facade, exactly as
`FlowAttributor` and `Sink` do. A setter rather than a `new` parameter keeps every
existing `Pipeline::new` call site (the corpus tests, the CLI, the pipeline unit
tests) unchanged.

**Alternatives considered**: a composite gating `Sink` in the CLI that wraps the user
sinks (would duplicate the output loop's per-sink retire, count, flush, and finish
machinery inside the composite, and hide per-sink `SinkFailure` indices behind one
outer sink); enforcing the bound in the driver's tick loop over the tee (the S14
shape, which is exactly the soft bound this slice removes); passing the session into
core (a P-3 violation).

## D-2. A new `gate_dropped` counter, folded into the conservation identity

**Decision**: add `gate_dropped: u64` to `CaptureStats`, set by `output_loop` from a
local counter the same way `sink_dropped` is. It is capture-wide (the output thread
owns it), so `CaptureStats::absorb` does not touch it, matching `buffer_dropped` and
`sink_dropped`. The pipeline's conservation identity becomes, for every sink:

```text
received + buffer_dropped + gate_dropped + refusals == packets_captured
```

**Rationale**: the gate is a new discard path, and constitution P-4 makes an
uncounted discard a defect. Because the gate sits before the per-sink fan-out, a gate
discard is withheld from every sink uniformly, so it is a single capture-wide term
rather than a per-sink one, and the identity stays exact under every interleaving.
The counter is distinct from the two loss counters because the cause and the remedy
differ: `buffer_dropped` is a slow sink, `sink_dropped` is a downstream that could not
accept, and `gate_dropped` is a packet the operator's own configuration placed
outside the capture window or beyond the bound, which is intended and not remedied.

**Alternatives considered**: reusing `sink_dropped` (conflates an intended
configuration discard with a sink that could not keep up, defeating the by-cause
separation P-4 exists for); leaving the gate discards only in the session's
`SessionStats` (would leave the pipeline's own identity unable to see them, so a
core-level discard would escape the core-level accounting).

## D-3. The gate is the packet-accounting authority; the session owns the stops

**Decision**: the `SessionGate` (facade) counts what it admits and what it discards by
cause (admitted packets and bytes, watch-time discards, out-of-window discards). The
completion summary reads the retained, watching-discarded, and out-of-window counts
from the gate. The `CaptureSession` remains the single owner of the six stop
conditions (section 10.6): the gate forwards each admitted packet's `(len, ts)` to the
driver over the channel S14's tee used, and the driver calls `session.on_packet(len)`
and `on_tick(ts)` for it, so `VolumeReached` fires in the session on the admitted
packet that reaches the bound, in the same timeline as the folded process events, and
"the first stop condition wins" stays in one place.

**Rationale**: the file-versus-accounting disagreement the slice fixes comes from two
authorities (the sinks write, the session counts, asynchronously). Making the gate the
single authority for what is written and what is counted-as-discarded removes the
disagreement by construction: the retained count is the admitted count is the file.
The session keeps the stop conditions because they are a lifecycle decision over
events and ticks, not a per-packet write decision, and splitting them across the gate
and the session would put "which stop won" in two places. The gate forwards only
admitted packets, so the session's own `watching_discarded` and
`discarded_out_of_window` stay zero on this path and the summary reads the real counts
from the gate; that is deliberate and is why the summary sources those two fields from
the gate.

**Alternatives considered**: driving `session.on_packet` for every captured packet and
recomputing the disposition from the session's state (reintroduces the disagreement,
because the session's state can advance between the gate admitting a packet and the
driver processing its receipt, so a written packet could be counted as discarded);
the gate holding the `StopHandle` and firing the stop itself (splits the stop
ordering across two components).

## D-4. The gate's bound matches the session's pre-existing bound semantics

**Decision**: the gate admits exactly `packet_bound` packets (it admits while its
admitted count is below the bound and closes on the packet that reaches it), and for
`byte_bound` it admits the packet that first reaches or crosses the bound and then
closes, matching `CaptureSession::check_volume_bounds`, which fires `VolumeReached`
when `retained >= packet_bound` or `retained_bytes >= byte_bound`. The captured
length the gate counts is `packet.data.as_ref().len()`, the same length S14's tee
forwarded.

**Rationale**: the gate and the session must reach the bound on the same packet or the
file and the stop reason would disagree. Reusing the session's existing comparison
(`>=`, retained-inclusive for bytes) makes them agree without changing what a bound
means; the slice makes the pre-existing semantics hard, it does not redefine them.

**Alternatives considered**: a byte bound that excludes the crossing packet (would
disagree with the session's `retained_bytes >= byte_bound` and produce a file one
packet short of what the session reports); counting on-wire length rather than
captured length (would disagree with the tee's precedent and with what the sinks
actually write).

## D-5. The window is a lock-free published capture interval, keyed on the packet's own instant

**Decision**: the `SessionGate` holds the capture window as the half-open interval of
capture instants `[admit_from, admit_until)`, two single-writer `AtomicI64` values the
driver publishes, read lock-free on the output thread. A packet is classified by its
own capture timestamp: `ts < admit_from` is a watch-time discard, `ts >= admit_until`
is out of window, otherwise it is subject to the bound. Offline the driver opens the
interval at `i64::MIN` before the pipeline starts, so every replayed frame is in
window; live the driver opens it at the acquiring event's instant and closes it at a
terminal-stage exit's instant. A bound reached is a separate `bound_hit` flag; the
window has exactly one writer.

**Rationale**: the bounded buffer sits between capture and the gate, so a coarse
open/close window state read at write time misclassifies buffered frames that cross a
transition. A pre-acquisition frame still buffered when the window opens would be
written and omitted from `watching_discarded`, and a post-stop frame still draining
would be written and miscounted as retained (both found in review of PR #26). Keying
on the packet's own instant fixes both, because the instant records when the frame was
captured regardless of how the drain races the transition. On the live path the packet
instant (the pcap header) and the event instant that opens or closes the window (the
ETW event header) are both Unix wall-clock, so they are directly comparable. A
lock-free single-writer atomic is the same discipline section 11.6 already requires of
the attribution snapshot: the reader never blocks the writer.

**Alternatives considered**: an `AtomicU8` window state read at write time (the first
implementation; misclassifies buffered frames at a transition, review of PR #26); an
`RwLock<WindowState>` (a lock on the per-packet read path, which section 11.6 forbids);
closing the window by state on a stop rather than by instant (drops valid pre-stop
frames still draining, or keeps invalid post-stop ones, depending on the drain race).
An interrupt or duration stop deliberately does not set `admit_until`, so what was
captured before the stop is kept (specification FR-005).

## D-8. A zero volume bound stops for `VolumeReached` explicitly

**Decision**: a zero bound (`--max-packets 0` or `--max-bytes 0`) is met before any
packet is retained, so the per-packet `check_volume_bounds` never runs to fire it and
the gate forwards no receipt. `CaptureSession::on_volume_reached` is added, and the
driver calls it once immediately after acquisition when a zero bound is configured, so
the stop reason is the promised `VolumeReached` rather than a later source-exhausted
one. It is not folded into the acquisition transition, because the offline driver
detects acquisition by the session resting in `Capturing`, which an immediate stop
would skip past.

**Rationale**: found in review of PR #26. The gate correctly writes zero packets for a
zero bound, but without this the run reported the wrong stop reason. Firing it from the
driver after acquisition keeps the session's "acquisition rests in Capturing" contract
the offline two-phase driver depends on.

**Alternatives considered**: firing `VolumeReached` inside `match_and_bind` on the
Capturing transition (breaks the offline acquisition detection, which checks for the
`Capturing` state); special-casing the gate to set `bound_hit` for a zero bound (the
gate is not the owner of the stop conditions, and the driver would still need to fire
the session stop).

## D-6. Only the live driver runs from arm; the offline driver stays two-phase

**Decision**: `capture_live` spawns the pipeline at arm, before the acquisition loop,
with the gate's window `Watching`, so pre-acquisition frames are read and the gate
discards and counts them; the acquisition loop sets the window to `Capturing` when a
stage matches. `capture_prerecorded` keeps its two-phase shape (fold events until
`Capturing`, then spawn the pipeline with the window already `Capturing`), so it never
sees a watch-time packet and, for an unbounded run, the gate is a pure pass-through.

**Rationale**: offline the whole timeline is pre-collected and every packet is
available at once, so running from arm would flow packets while `Watching` and discard
them, changing the offline behavior and moving the committed goldens away from
correctness for no gain. Only a live capture has the handle open before acquisition,
which is the condition that makes watch-time frames real. Confining the run-from-arm
change to the live path keeps the offline goldens byte-identical and keeps the
behavioral change on the path issue #22 is about.

**Alternatives considered**: running both drivers from arm (moves the offline goldens,
violating the slice's own guardrail); a run-from-arm flag threaded through both
(needless: the offline path has a natural two-phase point and no watch-time frames to
count).

## D-7. Testing without a capture driver

**Decision**: three tier-1 test surfaces. The bound behavior is tested through the
offline substrate by counting packet records in the produced pcapng and JSON Lines for
a `--max-packets N` and a `--max-bytes B` run. The watch-time discard counting is
tested by driving the `SessionGate` directly with its window set to `Watching` and
asserting nothing is admitted and the watch count advances. The conservation identity
is tested in the pipeline unit tests with a scripted gate that admits a subset,
asserting `received + buffer_dropped + gate_dropped + refusals == packets_captured`
for every sink and that a no-gate run leaves `gate_dropped` zero.

**Rationale**: the live run-from-arm wiring is tier 2 (compiled and linked in CI, not
executed), but the two properties that matter, the hard bound and the discard
counting, are reachable at tier 1 through the offline substrate and by driving the
gate directly, so the slice is verified without a capture driver, no elevation, and no
game, which is the section 25.1 discipline every slice since S08 has kept.

**Alternatives considered**: asserting only the stop reason for a bound (the S14 test,
which is exactly what let the soft bound pass); a mock capture driver (out of scope and
against the tier discipline).
