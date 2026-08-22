# Implementation Plan: Live capture status display

**Branch**: `069-live-capture-status` | **Date**: 2026-08-22 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/069-live-capture-status/spec.md`

## Summary

A `fragcap capture` run over the live (`--launch`/attach) driver goes silent
from acquisition until the run stops, sometimes for many minutes, hiding
exactly the fact an operator needs (issue #186's measured case: 91 percent of
a run's volume went to an unrelated process, invisible until the file was
opened in Wireshark afterward). The technical approach: reuse `drive_live`'s
existing 200ms tick to redraw a hand-rolled ANSI status block on stderr when
stderr is a real terminal, built from a pure, platform-independent renderer
over a plain snapshot struct (so every rendering rule is unit-tested on any
CI runner, not only the Windows/ETW Tier 2 path); fall back to today's plain
progress lines plus an occasional heartbeat line when stderr is not a
terminal; and add one small new live-readable handle to the pipeline's output
loop (mirroring the existing `SessionGate`/`GateHandle` split) for the two
counters (`sink_dropped`, `holder_tally`) that are not already exposed live
through `GateHandle`. `capture_prerecorded`/`drive` (offline replay, every
committed golden, and `extcap`) is not touched. No new dependency.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.82 (`rust-toolchain.toml` pins
the build channel at 1.96.0; unaffected by this slice)

**Primary Dependencies**: none added. Reuses `std::io::IsTerminal`
(stabilized stdlib), the existing hand-rolled ANSI constants in
`crates/fragcap-cli/src/color.rs`, and `std::sync::{Arc, Mutex, atomic}`
already used throughout `fragcap-core::pipeline` and `fragcap::session`.

**Storage**: N/A, no persisted state; all new types are in-memory for one
capture invocation's lifetime.

**Testing**: `cargo test --workspace --locked` (existing gate); the design
(research R-5) keeps the new rendering and timer logic as pure functions over
plain structs so their tests run on every CI platform, not only Windows.

**Target Platform**: the redraw's wiring lives behind
`#[cfg(all(feature = "etw", windows))]` (the existing gate on `capture_live`);
the pure renderer, the heartbeat timer, and the new `LiveStats` pipeline
handle are platform-independent and compile and test everywhere the
workspace already does.

**Project Type**: CLI (single Cargo workspace, existing 8-crate layout;
unchanged)

**Performance Goals**: redraw at least once per second (FR-001); reading the
new live counters must not block or slow the capture or output threads
(spec Key Entities), satisfied by construction, since every read is either
an atomic load or a `Mutex` held only for a `BTreeMap` clone/read, on the
same coarse cadence the existing `FilterNarration`/`active_endpoints()` read
already uses.

**Constraints**: zero new runtime dependency (`Cargo.lock` package count
unchanged, FR-011, SC-004); stderr-only, no stdout byte under any flag
combination (FR-007); byte-identical `--json`/`--mode stream --out
-`/`--quiet`/`--silent`/`extcap` output versus a pre-feature baseline
(FR-005 through FR-008).

**Scale/Scope**: one CLI feature slice; touches `fragcap-core::pipeline`
(one new handle type, three call sites), `fragcap-cli::color` (a
stream-parameterized `use_color`), and adds a small number of new modules
under `fragcap-cli` for the renderer, the redraw state, and the heartbeat
timer, plus the `drive_live` call sites that wire them in.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

- **P-1 (Passive Observation Only)**: Not implicated. This slice reads
  counters the pipeline already computes and renders them; it opens no
  process handle, injects nothing, and adds no capture technique. PASS.
- **P-2 (Core Stays Platform-Neutral)**: The new `LiveStats` handle lives in
  `fragcap-core::pipeline` and is pure `std::sync` types with no
  platform-specific dependency and no I/O crate; `fragcap-core` continues to
  build for a target with no capture backend. PASS.
- **P-3 (Capture And Attribution Stay Separate)**: Not implicated; no
  `PacketSource` or `FlowAttributor` is touched. PASS.
- **P-4 (No Silent Loss)**: Directly reinforced, not weakened: every discard
  counter this slice surfaces live is a counter that already exists and is
  already surfaced at the end (`CompletionSummary`); this slice adds no new
  discard path and every counter it reads has a name already established by
  an earlier slice. PASS.
- **P-5 (Compatibility Outranks Richness)**: Not implicated; no output file
  format changes. PASS.
- **P-6 (Glossary First)**: New terms this slice introduces ("live status
  block", "redraw", "heartbeat line" in the capture-output sense) get
  glossary entries in the same change, per the task list. GATE, tracked as a
  task.
- **P-7 (Wrappers Stay Thin)**: Not implicated; no shell wrapper exists for
  `capture`. PASS.
- **P-8 (House Standards Apply)**: All new/edited files follow
  `CONVENTIONS.md` (UTF-8 no BOM, LF, no em/en dashes); enforced by
  `cargo xtask lint` in the verification gate. PASS (gate, not a claim,
  until `cargo xtask ci` actually runs green).
- **P-9 (The Instrument Does Not Lie)**: Directly on-topic and the primary
  justification for the whole slice: this is a truthfulness fix (the tool
  already knows what is happening and was not saying so). The design
  introduces no new discretion to withhold, redact, or normalize a value;
  the status block's "not yet narrowed" line (rather than omitting the
  filter line at zero) and the "N more" overflow line (rather than silently
  dropping holder-tally entries) are both explicit anti-silent-omission
  choices, consistent with this principle. PASS.
- **P-10 / P-11**: Not implicated (no target-source or specification-version
  concern here). PASS.
- **Licensing**: No new dependency, so no new license to vet. PASS.
- **Verification discipline**: `cargo xtask ci` runs in the foreground,
  watched to completion, per house rule; the Tier 2 manual run (quickstart.md
  step set 2) is recorded with its own evidence, not claimed from a green CI
  run, per `AGENTS.md`'s standing rule on the live/ETW path.

No violation requires an entry in Complexity Tracking.

## Project Structure

### Documentation (this feature)

```text
specs/069-live-capture-status/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md         # Phase 1 output
├── quickstart.md         # Phase 1 output
├── contracts/            # Phase 1 output
│   ├── status-block.md
│   ├── heartbeat-line.md
│   └── capture-progress-event.md
└── tasks.md              # Phase 2 output (/speckit-tasks, not this command)
```

### Source Code (repository root)

```text
crates/fragcap-core/src/
├── pipeline/
│   ├── mod.rs            # Pipeline gains a `live: LiveStats` field and
│   │                      # `live_stats(&self)`; output_loop gains a `live:
│   │                      # LiveStats` param and updates it at the same
│   │                      # sink_dropped / holder_tally / evicted sites
│   └── buffer.rs          # add Consumer::next_and_evicted() beside the
│                            # existing next(), returning the evicted count
│                            # from the same already-held lock
└── (new) live_stats.rs     # LiveStats: the Arc-shared sink_dropped /
                             # holder_tally / buffer_dropped bundle

crates/fragcap-cli/src/
├── color.rs               # use_color(stream) parameterized; existing two
│                            # doctor call sites updated to pass Stdout
├── orchestrator.rs         # drive_live gains: a LiveStatusSnapshot built
│                            # each tick, the terminal/non-terminal branch,
│                            # the redraw or heartbeat call
├── events.rs               # new Event::CaptureProgress variant (FR-009)
├── output.rs                # unchanged (CompletionSummary stays as is)
└── (new) live_status/
    ├── mod.rs               # LiveStatusSnapshot, render_status (pure)
    ├── redraw.rs             # RedrawState, the cursor-up/erase sequence
    └── heartbeat.rs           # Heartbeat timer

docs/glossary/
└── command-line-and-diagnostics.md   # new entries: live status block,
                                        # redraw, heartbeat line (P-6);
                                        # index.md regenerates via lint-docs.sh
```

**Structure Decision**: Everything new lives inside the two crates the
feature already touches (`fragcap-core` for the one pipeline-internal
counter handle, `fragcap-cli` for every presentation concern), matching this
project's existing placement of `CompletionSummary` and `Emitter` in the CLI
crate and the pipeline's own counters in `fragcap-core`. No new crate; no
change to the 8-crate workspace layout.

## Complexity Tracking

No constitution violation requires justification; this section is empty by
design.
