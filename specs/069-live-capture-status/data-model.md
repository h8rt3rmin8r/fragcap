# Phase 1 Data Model: Live capture status display

This slice adds no persisted entity and no store schema change. Everything
below is in-memory state for the lifetime of one `fragcap capture` invocation.

## `LiveStats` (new, `fragcap-core::pipeline`)

The live-readable counterpart to the pipeline output loop's local
bookkeeping. Owned by `Pipeline`, constructed in `Pipeline::new`, obtained by
a caller via `pipeline.live_stats() -> LiveStats` (a cheap `Arc`-clone,
callable any time after construction, in particular before `run(self)`
consumes the pipeline by value). Only `drive_live` reads it; `drive` ignores
it, per research R-1.

| Field | Type | Written by | Read by |
| --- | --- | --- | --- |
| `sink_dropped` | `Arc<AtomicU64>` | the output loop, at the same three sites that update the existing local `sink_dropped: u64` | the CLI's status-snapshot builder, each redraw tick |
| `holder_tally` | `Arc<Mutex<BTreeMap<Arc<str>, u64>>>` | the output loop, at the same site that updates the existing local `holder_tally` | the CLI's status-snapshot builder, each redraw tick |
| `buffer_dropped` | `Arc<AtomicU64>` | the output loop, once per `Consumer::next_and_evicted()` return (a new method beside the existing, untouched `next()`, reusing its lock hold to also return the evicted count at no extra lock acquisition; research R-2) | the CLI's status-snapshot builder, each redraw tick, via a plain atomic load |

No field here duplicates a counter already reachable through `GateHandle`;
this type exists only for the two counters (`sink_dropped`, `holder_tally`)
that have no live-readable path today, plus a zero-extra-contention live
mirror of the one (`buffer_dropped`) that already exists but only inside the
pipeline thread's local scope.

## `LiveStatusSnapshot` (new, `fragcap-cli`)

A plain, `Clone`-able, platform-independent struct: the single input to the
pure renderer (research R-5). Assembled once per redraw tick inside
`drive_live` from `gate_handle`, the new `LiveStats` handle, the stamper's
`active_endpoints()`, `started.elapsed()`, and the existing `bound: HashMap<u32,
String>` plus the process image name already known at the `stage matched`
site.

| Field | Type | Source |
| --- | --- | --- |
| `elapsed` | `std::time::Duration` | `started.elapsed()` |
| `process` | `Option<BoundProcess>` (name, pid, role, stage) | the driver's existing `bound` map / `session.role_bindings()` |
| `written_packets` | `u64` | `gate_handle.admitted()` (existing) |
| `written_bytes` | `u64` | `gate_handle.admitted_bytes()` (existing) |
| `byte_bound` / `packet_bound` | `Option<u64>` each | `gate_handle`'s existing bound fields |
| `active_endpoints` | `usize` | `stamper.active_endpoints().len()` (existing, same call `FilterNarration` makes) |
| `narrowed` | `bool` | `active_endpoints > 0`, mirroring `FilterNarration`'s own definition |
| `watch_discarded`, `out_of_window_discarded`, `scope_discarded`, `scope_unresolved_discarded` | `u64` each | `gate_handle`'s existing atomics |
| `buffer_dropped` | `u64` | the new `LiveStats::buffer_dropped()` accessor (Copilot review of PR #196: corrected from an earlier-planned `buffer_reader` type that the implementation replaced with a plain atomic, per research R-2's `Consumer::next_and_evicted`) |
| `sink_dropped` | `u64` | the new `LiveStats::sink_dropped` atomic |
| `holder_tally` | `Vec<(Arc<str>, u64)>`, sorted by count descending then name ascending (a total order, matching `CaptureStats::dominant_holder`'s own tiebreak discipline) | a snapshot copy of the new `LiveStats::holder_tally` |

`LiveStatusSnapshot` carries no reference and no lock guard; it is a value
built once per tick and handed to the pure renderer, so the renderer itself
never touches a live handle, a socket, or a platform type (this is what makes
it testable everywhere, per research R-5).

## `RedrawState` (new, `fragcap-cli`)

Terminal-only bookkeeping, not constructed at all on the non-terminal path.

| Field | Type | Purpose |
| --- | --- | --- |
| `previous_lines` | `usize` | how many lines the last frame occupied, so the next redraw erases exactly that many before writing the new frame (FR-002) |

## `Heartbeat` (new, `fragcap-cli`)

Non-terminal-only bookkeeping.

| Field | Type | Purpose |
| --- | --- | --- |
| `last_progress_at` | `std::time::Instant` | reset by every call to `Emitter::progress` (FR-004's "resets on every progress line," per the Clarifications session); compared against the fixed 30-second interval each tick to decide whether to emit a heartbeat line |

## No changes to existing types

`CompletionSummary`, `GateShared`/`GateHandle`, `CaptureStats`, `Event`, and
every profile/target/detection type are unchanged by this slice. The optional
`capture.progress` JSON event (FR-009) is a new variant on the existing
`Event` enum in `crates/fragcap-cli/src/events.rs`, carrying the same fields
as `LiveStatusSnapshot`'s scalar counters (not the holder-tally breakdown,
which stays human-display-only per the issue's scope; see
`contracts/capture-progress-event.md`).
