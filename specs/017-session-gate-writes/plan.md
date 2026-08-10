# Implementation Plan: The Session Gates Sink Writes (Watch From Arm, Hard Bounds)

**Branch**: `feat/session-gate-writes` (spec dir `017-`) | **Date**: 2026-08-10 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/017-session-gate-writes/spec.md`

## Summary

Close the deferred half of the S14 review (issue #22, findings C2 and C3). A generic
`WriteGate` seam is added to `fragcap-core` and consulted by `output_loop` before the
per-sink fan-out; a packet the gate does not admit is counted in a new capture-wide
`gate_dropped` counter folded into the conservation identity and is written to no
sink. The facade adds a `SessionGate` implementing `WriteGate`: it admits a packet
only while its published window is open (the session is capturing) and the configured
bound has not been reached, discarding and counting every other packet by cause. This
makes `--max-packets` and `--max-bytes` produce exactly-bounded files, makes the
completion summary match what is on disk, and lets the live driver run the packet path
from arm so watch-time frames are read and counted. The offline driver stays two-phase
and its committed goldens stay byte-identical. Decisions in [research.md](research.md);
types in [data-model.md](data-model.md).

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.82.

**Primary Dependencies**: None added. A `WriteGate` trait object behind an `Arc`, a
`std::sync::atomic` window and tallies, and the existing `std::sync::mpsc` channel
S14 built (now driven by the gate rather than a tee sink).

**Testing**: `cargo test --workspace --locked`; pipeline unit tests (a scripted gate,
the extended conservation identity, a no-gate run); a `SessionGate` unit test (a
closed watching window admits nothing and counts); CLI `cli_run` tests (a
`--max-packets N` and a `--max-bytes B` run asserting the produced-packet count and
the summary); the unbounded goldens and the corpus goldens byte-identical.

**Target Platform**: Platform-neutral tier 1; `cargo xtask neutral` still builds core
with no capture backend. The run-from-arm live wiring is tier 2 (compiled and linked,
not executed in CI).

**Project Type**: Rust workspace (library + CLI).

**Performance Goals**: One `Arc<dyn WriteGate>` deref and one atomic load per packet
on the output thread (a branch the acquisition path never sees), plus the channel send
S14 already had for retained packets. No new per-packet cross-thread hop.

**Constraints**: P-2 (no new dependency; core takes no platform dependency), P-3
(`fragcap-core` gains no session or profile knowledge; the gate is a generic trait,
the session-aware impl is in the facade), P-4 (`gate_dropped` is a named counter in
the conservation identity; no discard is uncounted), P-9 (no fabricated packet count;
the file and the accounting are the same set by construction). UTF-8 no BOM, LF, no
em/en dashes.

**Scale/Scope**: One trait and one counter in core, an `output_loop` branch and a
`Pipeline` setter; one `SessionGate` type in the facade; the orchestrator's tee
replaced by the gate and the two drivers adjusted (offline swap, live run-from-arm);
the completion summary sourced from the gate; test additions.

## Constitution Check

*GATE: passed at plan time; re-checked after design.*

- **P-1**: PASS. No process handle or denylisted technique; the gate reads a packet
  length and a published state.
- **P-2**: PASS. No new dependency; the gate uses `std::sync::atomic` and the existing
  `mpsc` channel. `fragcap-core` takes no platform dependency; `cargo xtask neutral`
  still builds.
- **P-3**: PASS. `fragcap-core` gains a generic `WriteGate` trait only; it learns
  nothing of sessions or profiles. The session-aware `SessionGate` lives in the facade
  beside `CaptureSession` and `RoleStampingAttributor`, the crate already above both
  sibling crates. `PacketSource`, `FlowAttributor`, and `Sink` are unchanged.
- **P-4**: PASS. Every gate discard is counted in `gate_dropped` and folded into the
  conservation identity; the summary's watch-time and out-of-window lines break the
  discards down by cause. The reconciliation invariant `gate_dropped ==
  watch_discarded + out_of_window_discarded` guards against a double count.
- **P-9**: PASS. The gate withholds a packet from the sinks; it never alters, reorders,
  or fabricates one. The bound makes the produced file the retained set exactly, which
  removes the S14 disagreement rather than papering over it.
- No violations; Complexity Tracking empty.

## Project Structure

### Documentation (this feature)

```text
specs/017-session-gate-writes/
├── plan.md              # This file
├── spec.md
├── research.md          # Decisions D-1..D-7
├── data-model.md        # WriteGate, gate_dropped, SessionGate
├── quickstart.md        # Tier-1 validation guide
└── checklists/requirements.md
```

### Source Code (repository root)

```text
crates/fragcap-core/src/
├── traits.rs            # WriteGate trait (new)
├── stats.rs             # CaptureStats.gate_dropped; docs; a stats unit test
├── pipeline/mod.rs      # Pipeline.gate + set_write_gate; output_loop consults the gate and counts gate_dropped; conservation-identity docs and the test helper extended; a scripted-gate test and a no-gate assertion
└── lib.rs               # re-export WriteGate

crates/fragcap/src/
├── session.rs           # SessionGate (new): window state, bounds, tallies, admit(); handles for the driver
└── lib.rs               # re-export SessionGate and WriteGate through the facade

crates/fragcap-cli/src/
├── orchestrator.rs      # remove TeeCountingSink; attach the gate via set_write_gate; offline sets Capturing before spawn; live spawns from arm at Watching then Capturing on acquire; build_summary reads gate tallies
└── output.rs            # CompletionSummary sourced so watch/out-of-window come from the gate; no double count

crates/fragcap-cli/tests/
└── cli_run.rs           # --max-packets and --max-bytes produce exactly-bounded files; summary matches disk

changelog.d/             # S017 added fragment + a decisions fragment (reverses D-c/D-e for two cases)
```

**Structure Decision**: the seam is in core (generic), the policy is in the facade
(session-aware), and the wiring is in the CLI. This is the same three-layer split S14
used for role stamping (a core trait, a facade decorator, a CLI orchestrator), applied
to the write decision.

## Implementation order (TDD, tier 1)

1. **Core `WriteGate` + `gate_dropped`** (`traits.rs`, `stats.rs`, `lib.rs`): add the
   trait, the counter (with `absorb` leaving it capture-wide), and the re-export. Add a
   stats unit test that `gate_dropped` is a drop term and `absorb` does not sum it.
2. **Pipeline wiring** (`pipeline/mod.rs`): add the `gate` field and `set_write_gate`;
   thread the gate into `output_loop`; count `gate_dropped` and skip the sinks for a
   rejected packet; extend the module's conservation-identity documentation and the
   test helper to include the new term.
3. **Pipeline tests** (`pipeline/mod.rs`): a scripted `WriteGate` stub that admits a
   chosen subset; assert the four-term conservation identity per sink (SC-004); assert
   a no-gate run has `gate_dropped == 0` and the prior three-term identity (FR-004).
4. **Facade `SessionGate`** (`session.rs`, `lib.rs`): the window state, the bounds, the
   tallies, and `admit`; the driver handles; the re-exports. Unit tests: a closed
   watching window admits nothing and counts watch discards (SC-003); a `Capturing`
   window admits within a packet bound and rejects beyond it, counting out-of-window
   (FR-006 at the unit level); the reconciliation invariant holds.
5. **Orchestrator** (`orchestrator.rs`): remove `TeeCountingSink`; build the
   `SessionGate`, attach it with `set_write_gate`, and hand it the tee sender; offline
   set the window `Capturing` before `spawn_pipeline` and `Other` on drain; keep
   `drive` feeding `on_packet`/`on_tick` from admitted receipts; `build_summary` reads
   the gate tallies. Then the live path: `capture_live` spawns the pipeline at arm with
   the window `Watching`, sets `Capturing` on acquire, `Other` on drain.
6. **Summary** (`output.rs`): source `watching_discarded` and `discarded_out_of_window`
   from the gate; keep the fragcap-drops line as `buffer_dropped + sink_dropped`; no
   double count. Update the summary unit tests if a field's provenance assertion needs
   it.
7. **CLI tests** (`cli_run.rs`): replace the stop-reason-only bound test with one that
   asserts the produced pcapng and JSON Lines each contain exactly N packet records for
   `--max-packets N`, the summary reports N retained and zero out of window, and the
   stop reason is `volume-reached` (SC-001, SC-002); a `--max-bytes B` case (FR-006).
   Keep the unbounded golden test unchanged (FR-011).
8. **Docs / changelog**: S017 added fragment; a decisions fragment recording that this
   reverses D-c/D-e for the watch-time and bound cases and keeps them for the offline
   unbounded case; a glossary entry for the write gate / capture window if a new term
   is introduced (P-6).
9. **Verify**: `cargo xtask ci`, `neutral`, `msrv`; the unbounded CLI goldens and the
   corpus goldens byte-identical; `cargo check -p fragcap-cli --features
   live,socket-table,etw` so the run-from-arm live path type-checks.

## Risks

- **The offline goldens must not move.** The gate is a pass-through only if, for an
  unbounded run, the window is `Capturing` and no bound is set, so `admit` always
  returns `true` and forwards every receipt exactly as the tee did. Guarded by the
  unbounded `cli_run` goldens and the corpus pipeline goldens; any drift fails there.
- **The channel order must be preserved.** The gate forwards a receipt from `admit` on
  the output thread, the same thread and the same per-packet point the tee sink wrote
  from, so the driver's `drive` loop sees the same sequence. The `run-events.ndjson`
  golden (an unbounded `--json` run) guards the sequence.
- **The live run-from-arm is tier 2.** It is compiled and linked but not executed in
  CI. Guarded by `cargo check --features live,socket-table,etw` and by the tier-1
  `SessionGate` unit tests that cover the watch-time discard counting the live path
  relies on; the executed live run remains an operator step, unchanged from S09.
- **Double counting.** `gate_dropped` (core) and the session's watch/out-of-window
  counts view the same physical discards from two accounting layers; the summary must
  not add them. Guarded by the reconciliation invariant test (`gate_dropped ==
  watch_discarded + out_of_window_discarded`) and by the summary keeping its
  fragcap-drops line at `buffer_dropped + sink_dropped`.
