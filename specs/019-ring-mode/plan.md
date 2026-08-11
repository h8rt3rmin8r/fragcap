# Implementation Plan: Ring mode and triggers

**Branch**: `019-ring-mode` | **Date**: 2026-08-10 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/019-ring-mode/spec.md` (roadmap
slice S16, specification section 7.2, FR-8).

## Summary

Deliver ring mode: a rolling in-memory window of the most recently captured
packets, bounded by a duration or a byte size, dumped to a single capture file
when the capture ends. The six session stop conditions that end a capture
already exist (S12, S17: interrupt, duration, terminal-stage exit,
all-non-service-exited, source exhaustion, sink error) and are reused unchanged;
ring mode adds no stop condition. This slice adds:

1. A `RingSink` in `fragcap-sink` implementing the existing `Sink` trait. Its
   `write` enqueues each accepted packet into a bounded in-memory deque and
   evicts the oldest to keep the retained set within the window; its `finish`
   materializes the retained window as one independently valid pcapng file
   through the existing `SinkFactory` and pcapng writer.
2. CLI wiring in `fragcap-cli` that replaces the current `reject_unsupported`
   stubs for `--mode ring` and `--ring` with real construction, and adds the
   ring-specific configuration refusals (missing `--out`, missing `--ring`, a
   volume bound in ring mode, a ring window without ring mode).
3. A `docs/glossary.md` entry for ring mode distinguishing it from the internal
   ring buffer of specification 12.4 (P-6).

The load-bearing structural insight: the dump is the `Sink::finish(self, stats)`
seam, which the pipeline already calls exactly once at drain for every stop
condition. Ring mode is therefore "attach a `RingSink` instead of a
`RotatingFileSink` for `--out`," with the retention policy living entirely inside
the sink. No pipeline, session, or write-gate change is required.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.82 (pinned toolchain per
`rust-toolchain.toml`).

**Primary Dependencies**: standard library only (`std::collections::VecDeque`,
`std::fs`). The `RingSink` reuses the existing `SinkFactory` and pcapng writer in
`fragcap-sink`. No new third-party crate.

**Storage**: one capture file on disk, written at drain; the retained window is
held in memory during capture.

**Testing**: `cargo test`, tier 1 (offline, no capture driver, no elevation, no
game). The `RingSink` retention logic is unit-tested directly; the end-to-end
dump is tested by replaying the committed fixture corpus through ring mode with a
small window and reading the dumped file back with the same `serde_json`/pcapng
parsers the sink tests already use. The whole-input case is checked against a
plain `--out` file capture of the same input.

**Target Platform**: cross-platform. The `RingSink` is ordinary in-memory
buffering plus a file write; it is not platform gated. It runs in the neutral
core build environment and on Windows alike.

**Project Type**: Rust workspace (library crates plus a CLI), single repository.

**Performance Goals**: `RingSink::write` is O(1) amortized (push back, pop front
while over the window). Retention is bounded by the configured window, so memory
is bounded by the operator's `--ring` value plus one crossing packet.

**Constraints**: the dumped file MUST parse cleanly in an unmodified pcapng
analyzer (P-5). The ring alters nothing it retains and dumps (P-9); an eviction
is a counted, reported retention decision, never a silent capture loss (P-4). The
retained set is exactly the recent tail (FR-001, FR-002). Core stays
platform-neutral (P-2): all new code is in `fragcap-sink` and `fragcap-cli`, not
`fragcap-core`.

**Scale/Scope**: one ring window per run; one `--out` dump target. Window sizes
are whatever `--ring` accepts (a duration or a byte size); the size window is
measured by captured length, matching `--max-bytes`.

## Constitution Check

*GATE: evaluated before Phase 0 and re-evaluated after Phase 1 design.*

| Principle | Assessment |
| --- | --- |
| P-1 Passive Observation | PASS. The `RingSink` is in-memory buffering plus a file write. No denylisted technique, no process handle, no capture or transmit call. `cargo xtask lint` is unaffected. |
| P-2 Core Stays Platform-Neutral | PASS. All new code lands in `fragcap-sink` (a leaf already permitted platform deps, though none is needed here) and `fragcap-cli`. `fragcap-core` and its `Sink` trait gain nothing. The neutral core build is unchanged. |
| P-3 Capture And Attribution Separate | PASS. A sink is neither a `PacketSource` nor a `FlowAttributor`; no merge occurs. |
| P-4 No Silent Loss | PASS, and central. The `RingSink`'s `write` returns `Ok` for every delivered packet, so the pipeline conservation invariant (received + buffer_dropped + refusals = captured) is preserved. An eviction is the sink's own reported retention accounting (an evicted count), not a capture-wide loss; the terminology mirrors S15's per-consumer drops. |
| P-5 Compatibility Outranks Richness | PASS, and central. The dump is a single pcapng beginning with its Section Header Block and one Interface Description Block per declared interface, then the retained packets in capture order. It opens in an unmodified analyzer. Verified by producing bytes a standard pcapng parser accepts. |
| P-6 Glossary First | ACTION. The term "ring mode" (and "ring window") gets a `docs/glossary.md` entry in this slice's change, explicitly distinguished from the internal ring buffer of 12.4. |
| P-7 Wrappers Stay Thin | N/A. No wrapper logic added; `doctor` and the shell wrappers are untouched. |
| P-8 House Standards Apply | PASS by gate. `cargo fmt`/`clippy`, UTF-8/LF, no em/en dashes. |
| P-9 The Instrument Does Not Lie | PASS. The ring retains and dumps observed bytes unaltered; the only omission is the eviction of old packets, which is the operator's declared retention scope (a chosen window) and is counted, exactly the "declared omission" P-9 permits. |
| Licensing | PASS. No new crate. |
| Pinned artifacts | No change required. Ring mode is exercised under the existing `cargo test` step; no workflow, toolchain, or release-config edit is needed. |

No principle is violated; the Complexity Tracking table is empty.

## Project Structure

### Documentation (this feature)

```text
specs/019-ring-mode/
├── plan.md              # This file
├── research.md          # Phase 0: decisions, rationale, alternatives
├── data-model.md        # Phase 1: entities and their invariants
├── quickstart.md        # Phase 1: runnable validation scenarios
├── contracts/
│   ├── ring-cli-grammar.md   # --mode ring / --ring / --out contract + refusals
│   └── ring-sink.md          # RingSink retention + dump contract
├── checklists/
│   ├── requirements.md  # spec quality (from /speckit-specify)
│   └── ring-mode.md     # requirements-quality checklist (from /speckit-checklist)
└── tasks.md             # Phase 2 (/speckit-tasks)
```

### Source Code (repository root)

```text
crates/fragcap-sink/src/
├── lib.rs                     # re-export RingSink and RingWindow
├── transport/
│   ├── mod.rs                 # unchanged (SinkFactory, InterfaceSpec, Format)
│   ├── ring.rs                # RingSink: bounded deque, evict-by-window,
│   │                          # dump-on-finish; RingWindow (Duration | Size)
│   ├── file.rs                # unchanged
│   └── stream.rs, tcp.rs, ... # unchanged
├── pcapng/                    # unchanged (reused through SinkFactory)
└── json/, annotation.rs, error.rs  # unchanged

crates/fragcap-sink/tests/
└── ring.rs                    # corpus through RingSink at several windows;
                               # read back and assert the retained tail + validity

crates/fragcap-cli/src/
└── assemble.rs                # reject_unsupported drops the ring stub and adds
                               # the ring config refusals; build a RingSink for
                               # --out when the effective mode is ring

crates/fragcap/tests/
└── (extend the existing pipeline corpus test)  # a ring run end to end offline

docs/glossary.md               # "ring mode" / "ring window" entries (P-6)
changelog.d/S16-ring-mode.*.md # added + decisions fragments
```

**Structure Decision**: The retention and dump live in one new `RingSink` inside
`fragcap-sink`'s `transport` module, reusing the pcapng writer through the
existing `SinkFactory`. The CLI is the only other crate that changes, to resolve
ring mode and enforce its refusals. `fragcap-core`, the pipeline, the capture
session, and the write gate are all unmodified: the existing `Sink::finish` seam
is the dump trigger, and the six stop conditions already drive drain.

## Key design decisions (recorded per autopilot decision policy)

Decided from the constitution, the architecture of record, and the existing sink
and session contracts; reasoning and alternatives are in [research.md](research.md).
The architecture-affecting ones are promoted to a changelog decisions fragment.

- **D1. The dump is the `Sink::finish` seam, not a new trigger path.** The
  pipeline calls `finish(self, stats)` on every sink exactly once at drain, and
  drain is reached by all six stop conditions. `RingSink::finish` writes the
  retained window. Ring mode adds no code to the session, the pipeline, or the
  write gate; it swaps the sink built for `--out`.
- **D2. Retention is a `VecDeque<CapturedPacket>` with evict-from-front.**
  `write` pushes the packet to the back, then pops from the front while the
  retained set exceeds the window. `CapturedPacket` already owns its payload by
  reference-counted `Bytes`, so retaining it is a cheap clone, not a copy of the
  bytes. No new dependency: the standard library `VecDeque` is exactly the
  bounded-tail structure needed, the same reasoning S08 used to reject a
  concurrency crate for the pipeline buffer.
- **D3. A size window is measured by captured length, matching `--max-bytes`.**
  The retained-bytes running total sums each packet's captured length (the same
  quantity `SessionStats::retained_bytes` uses), not the encoded pcapng block
  size. An operator reasons about one notion of capture size across `--ring` and
  `--max-bytes`, and the retained set does not depend on the on-disk encoding.
- **D4. A window smaller than one packet keeps that one packet.** Eviction never
  empties the deque below one element: the newest packet is always retained even
  if it alone exceeds the size window, so a capture that saw traffic never dumps
  an empty file. This is the retained-inclusive rule the write gate already uses
  for `--max-bytes` (it admits the crossing packet).
- **D5. A duration window is measured back from the newest retained instant.**
  After pushing a packet, evict every front packet whose capture instant is more
  than the window before the newest retained packet's instant. Measuring from
  the newest packet (not a wall clock or the stop instant) makes the retained set
  the recent tail by capture instant and keeps the sink independent of when drain
  happens to run, matching how the write gate classifies by the packet's own
  instant.
- **D6. `RingSink::finish` materializes through a `SinkFactory`.** It builds a
  fresh pcapng encoder over the `--out` file (the same factory the file sink
  uses), writes the header preamble and every retained packet in order, and
  finishes with the run's `CaptureStats`. The bytes are produced by the
  unchanged pcapng writer, so a whole-input dump is byte-comparable to a plain
  `--out` capture (FR-012).
- **D7. The ring's own accounting is an evicted count surfaced at finish.** The
  number of packets evicted (and retained) is the sink's reported accounting,
  the way S15's streaming sink reports per-consumer drops. It is not a
  capture-loss counter and does not touch the pipeline's `sink_dropped` or the
  session's discard tallies. P-4 is satisfied by naming and surfacing it.
- **D8. Ring config refusals live in `reject_unsupported`/`effective_config`.**
  The CLI already resolves the effective mode there and already refused ring
  naming this slice. That refusal is replaced by: require `--out` and `--ring`
  in ring mode; refuse `--max-bytes`/`--max-packets` in ring mode; refuse
  `--ring` outside ring mode. Each is a `CliError::usage` (exit 2) naming the
  cause, reported before any capture starts, the same pattern the transport and
  launch refusals already follow.
- **D9. `--out` in ring mode builds a `RingSink`, not a `RotatingFileSink`.** In
  `build_sinks`, when the effective mode is ring, the `--out` file becomes the
  ring dump target (a `RingSink` over a pcapng `SinkFactory`) instead of the
  continuous rotating file sink. Rotation options are not offered on the ring
  dump in this slice; the ring dump is a single file.

## Open honesty note (surfaced at the pre-push halt)

Ring mode's "dumped on interrupt" headline is exercised in tests through the
existing `--fire-interrupt` offline hook (the hidden flag that fires an operator
interrupt at the end of an offline capture), not through a real console signal.
This is the same substrate every prior slice's interrupt path is tested on, and
it drives the identical `StopReason::Interrupt` path; a real Ctrl+C on a live
capture remains covered only by the untested live path, exactly as it was before
this slice. What this slice proves offline is that every stop condition reaches
`finish` and dumps the retained window; that the interrupt is delivered by a
signal rather than the test hook is unchanged from the rest of the tool.
