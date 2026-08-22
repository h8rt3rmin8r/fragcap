# Contract: the optional `capture.progress` JSON event (FR-009)

A new `Event::CaptureProgress` variant in `crates/fragcap-cli/src/events.rs`,
following the existing `Event` pattern exactly (a `kind()` string, a `render`
match arm appending fields to the shared `{"ts":...,"event":...}` envelope
every event already uses).

## Fields

The scalar counters from `LiveStatusSnapshot`, matching `SessionComplete`'s
existing naming where the same quantity already has a name there:

```json
{"ts":"...", "event":"capture.progress", "elapsed_secs": 135, "packets": 4102,
 "bytes": 812004, "active_endpoints": 3, "watching_discarded": 0,
 "discarded_out_of_window": 0, "buffer_dropped": 0, "sink_dropped": 0,
 "scope_discarded": 0, "scope_unresolved_discarded": 0}
```

`holder_tally` is deliberately not included: the issue frames the live
per-process breakdown as a human-display aid, and a `--json` consumer that
wants per-process detail already has it, per packet, in the captured file's
own attribution comments (pcapng) or record fields (JSON Lines); this event
is a coarse liveness/progress signal, not a second copy of that data.

## Emission rule

Emitted from the same `drive_live` tick that would have redrawn the human
block, but only when `Format::Json` (never alongside the human block, and
never when `Format::Human`, matching FR-009's "never appears when `--json`
is not set"). Sent through the plain `Emitter::event` call, the same one
`SessionArmed`, `StageMatched`, and `FilterNarrowed` already use, which
carries no verbosity gate of its own today (confirmed against `emit.rs`: only
`progress`, `warn`, and the human branch of `summary` check `Verbosity` at
all). `--quiet`/`--silent` therefore affect `capture.progress` exactly as
they already affect every other JSON event: not at all. FR-006's
quiet/silent suppression rule is about the human live display and heartbeat,
which this event has no bearing on since the two are format-exclusive
(FR-005).
