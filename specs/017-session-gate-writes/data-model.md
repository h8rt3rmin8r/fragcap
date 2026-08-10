# Data Model: The Session Gates Sink Writes

**Feature**: `017-session-gate-writes` | **Date**: 2026-08-10

The slice adds one core trait, one core counter, and one facade type. Nothing else
gains a field.

## Core (`fragcap-core`)

### `WriteGate` (new trait, `traits.rs`)

```rust
/// A decision the output loop consults before writing a captured packet to the
/// sinks. Consulted once per packet, on the output thread, so a rejected packet
/// is never written. `Send + Sync` because the pipeline holds it as
/// `Arc<dyn WriteGate>` shared with the output thread; interior mutability
/// because the implementor counts what it admits and discards.
pub trait WriteGate: Send + Sync {
    /// Whether this packet is admitted to the sinks. `false` withholds it from
    /// every sink; the output loop counts it in `gate_dropped`.
    fn admit(&self, packet: &CapturedPacket) -> bool;
}
```

- No default implementation. A pipeline with no gate attached behaves as before.
- The offline unbounded run attaches a gate that admits everything, so it is a
  pass-through.

### `CaptureStats.gate_dropped` (new field, `stats.rs`)

- `pub gate_dropped: u64`.
- Set by `output_loop` from a local counter, exactly as `sink_dropped` is.
- `absorb` does NOT touch it: it is capture-wide (the single output thread owns it),
  like `buffer_dropped` and `sink_dropped`.
- Documented as a term of the conservation identity, distinct from the two loss
  counters: an intended discard (outside the window or beyond the bound), not loss to
  be remedied.

### `Pipeline` (extended, `pipeline/mod.rs`)

- New field `gate: Option<Arc<dyn WriteGate>>`, default `None`.
- New setter `set_write_gate(&mut self, gate: Arc<dyn WriteGate>)`, mirroring
  `set_filter_config`, so no `Pipeline::new` caller changes.
- `run` moves the gate into `output_loop`.

### `output_loop` (extended, `pipeline/mod.rs`)

- Signature gains `gate: Option<Arc<dyn WriteGate>>`.
- Per `Item::Packet`, before the per-sink loop: if a gate is present and does not
  admit the packet, increment a local `gate_dropped` and `continue` (skip every
  sink).
- Sets `stats.gate_dropped = gate_dropped` alongside `stats.sink_dropped`.

## Facade (`fragcap`, `session.rs`)

### Capture window (published to the gate)

The window is the half-open interval of capture instants `[admit_from, admit_until)`,
two single-writer `AtomicI64` values keyed on the packet's own capture timestamp
rather than a coarse write-time state (review of PR #26; see research D-5).

- `admit_from` (init `i64::MAX`): the acquisition instant. A packet captured before it
  is a watch-time discard, even one buffered before acquisition and processed after.
- `admit_until` (init `i64::MAX`): the stop instant. A packet captured at or after it
  is out of window, even one still draining. Left at the sentinel for an interrupt or
  duration stop, keeping what was captured before it (FR-005).

### `SessionGate` (new type implementing `WriteGate`) and `GateHandle` (driver side)

`GateShared` (behind an `Arc`, all single-writer atomics or immutable):

- `admit_from: AtomicI64`, `admit_until: AtomicI64` - the capture window; written only
  by the driver through the handle.
- `packet_bound: Option<u64>`, `byte_bound: Option<u64>` - immutable, from
  `SessionConfig`.
- `admitted: AtomicU64`, `admitted_bytes: AtomicU64` - what reached the sinks.
- `watch_discarded: AtomicU64` - packets captured before `admit_from`.
- `out_of_window_discarded: AtomicU64` - packets captured at or after `admit_until`, or
  beyond the bound.
- `bound_hit: AtomicBool` - set the moment a bound is reached; informational.

`SessionGate` additionally owns `tee: Sender<(u32, Timestamp)>` (NOT in `GateShared`,
so it drops when the pipeline finishes and the driver's read loop ends). `GateHandle`
holds only the `Arc<GateShared>`.

`admit(&self, packet)` (called only on the output thread), classifying by `ts =
packet.ts`:

1. `ts < admit_from`: `watch_discarded += 1`; return `false`.
2. `ts >= admit_until`: `out_of_window_discarded += 1`; return `false`.
3. At the bound (`admitted >= packet_bound` or `admitted_bytes >= byte_bound`):
   `out_of_window_discarded += 1`; return `false`.
4. Else admit: `admitted += 1`; `admitted_bytes += len`; set `bound_hit` if the bound
   is now reached; `tee.send((len as u32, packet.ts))`; return `true`.

`GateHandle` methods (driver-side):

- `open_from(from: Timestamp)` and `close_at(until: Timestamp)` publish the interval.
- Accessors for the tallies (`admitted`, `admitted_bytes`, `watch_discarded`,
  `out_of_window_discarded`, `bound_hit`) the driver reads to build the summary.

## Orchestrator (`fragcap-cli`, `orchestrator.rs`)

- `TeeCountingSink` is removed; the `SessionGate` forwards admitted receipts instead.
- `spawn_pipeline` attaches the gate with `set_write_gate` and no longer prepends a
  tee sink; the sink list is the user sinks only.
- The offline driver opens the window from `i64::MIN` before spawning the pipeline (it
  is already capturing; every replayed frame is in window). A zero volume bound is
  stopped explicitly via `CaptureSession::on_volume_reached` (research D-8).
- The live driver spawns the pipeline at arm (window empty, so pre-acquisition frames
  are watch discards), opens the window from the acquiring event's instant on
  acquisition, and closes it at a terminal-stage exit's instant in `drive_live`. Its
  acquisition loop observes tee-channel disconnect so a pipeline that ends before
  acquisition does not hang the run (review of PR #26).
- `drive` and `drive_live` still feed `session.on_packet(len)` and `on_tick(ts)` from
  the channel for admitted receipts, so `VolumeReached` and the duration bound fire in
  the session as before.
- `build_summary` sources `retained`, `retained_bytes` view, `watching_discarded`, and
  `discarded_out_of_window` from the gate's tallies, and `packets_captured`,
  `packets_attributed`, `buffer_dropped`, `sink_dropped`, and `gate_dropped` from the
  pipeline report.

## Completion summary (`fragcap-cli`, `output.rs`)

- `CompletionSummary` gains no operator-facing double count: `watching_discarded` and
  `discarded_out_of_window` are now sourced from the gate (the real discard counts),
  and `gate_dropped == watching_discarded + discarded_out_of_window` holds as the
  reconciliation invariant. The summary's fragcap-drops line stays `buffer_dropped +
  sink_dropped`; the gate discards are the watch-time and out-of-window lines, so no
  packet is reported twice.

## Invariants

- **Conservation (core, per sink)**: `received + buffer_dropped + gate_dropped +
  refusals == packets_captured`.
- **File equals retained**: the number of packet records in the produced capture
  equals `admitted` equals the summary's retained count.
- **Reconciliation (no double count)**: `gate_dropped == watch_discarded +
  out_of_window_discarded`.
- **Bound**: with `packet_bound = N`, `admitted == min(N, packets_offered_while_open)`;
  with `byte_bound = B`, `admitted_bytes` is the first prefix sum that reaches or
  exceeds `B`.
