# Implementation Plan: Capture scope and truthful narration

**Branch**: `064-capture-scope` | **Date**: 2026-08-20 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/064-capture-scope/spec.md`

## Summary

Add a fourth test to the one write gate, so a packet reaches the sinks only when
it belongs to the capture; count both ways it can fail to; report what was
written per image; and move the filter-narrowed line from a one-shot read at
acquisition to the transition it describes.

## Branch base

Branched from `main`. Unlike S063 this shares no file with S062 or S063 beyond
`crates/fragcap-cli/src/cli.rs`, where it adds one flag in the `capture` block
while those slices touched doc comments and the `catalog` block. The merge is
expected to be clean; if it is not, this slice rebases, since it is the later
work.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.82.

**Primary Dependencies**: none added.

**Storage**: N/A

**Testing**: `crates/fragcap/src/session.rs` gate unit tests (the natural home
for the predicate), `crates/fragcap-core/src/pipeline/mod.rs` conservation
assertions, `crates/fragcap-cli/tests/cli_capture.rs` and its goldens.

**Target Platform**: Windows for a live capture; the gate and its tests are
platform-neutral and run anywhere through the replay path.

**Project Type**: Library plus CLI.

**Performance Goals**: the predicate runs per packet on the output thread. It is
two `Option` checks and, under a narrowed `--roles`, one lookup in a small set.
Nothing here may touch the capture threads.

**Constraints**: `SessionGate::admit` is on the hot path between the bounded
buffer and the sinks. The scope test must be cheap and must not allocate.

**Scale/Scope**: one predicate, two counters, one flag, one summary block, two
emit sites replaced by one transition observer.

## Phase 0: Research (complete)

Five things were measured on this branch before the design was fixed. Three of
them bound the blast radius and are why this P0 is a tractable slice rather than
an open-ended one.

1. **The gate already sees everything it needs.** `CapturedPacket::attribution`
   carries `role: Option<Arc<str>>` and `stage: Option<StageId>`, stamped by
   `RoleStampingAttributor::resolve` from the session's binding snapshot. No new
   plumbing, no new field, no change to what the capture threads compute.
2. **`--process` captures stamp role and stage too.** The committed golden
   `crates/fragcap-cli/tests/goldens/capture.jsonl` comes from `capture
   --process game.exe`, and all 24 records carry
   `"role":"target","stage":"target"`. One predicate covers both target-selection
   paths, which was not obvious and would have been a serious defect if assumed
   the other way: a scope gate keyed on stamps that `--process` did not produce
   would silently empty every `--process` capture.
3. **The CLI capture goldens are entirely target traffic**, all 24 records
   `game.exe`, so the default change leaves them byte-identical (SC-005).
4. **The corpus tests attach no gate.** `Pipeline::run` takes `Option<Arc<dyn
   WriteGate>>` and `corpus_pipeline.rs` passes `None`, except one test that
   deliberately passes a reject-everything gate. Scope lives in `SessionGate`,
   which only the CLI capture path builds, so the fixture corpus is untouched.
5. **The CLI thread is free while the pipeline runs.** `spawn_pipeline` puts the
   pipeline on its own thread and the CLI then sits in `drive`
   (`orchestrator.rs:302`) holding the stamper `Arc`. #185's transition can be
   observed there, with no channel from `fragcap-core` to the CLI.

## Constitution Check

*GATE: passed before Phase 0. Re-checked after design; still passing.*

| Principle | Bearing | Verdict |
| --- | --- | --- |
| P-1 Passive Observation Only | No handle, no injection, no driver change. The gate reads a packet fragcap already holds. | Not engaged |
| P-2 Core Stays Platform-Neutral | The predicate lives in `fragcap` (the facade), not `fragcap-core`; `fragcap-core` keeps `gate_dropped` and the `WriteGate` trait it already had. | Satisfied |
| P-3 Capture And Attribution Stay Separate | The gate consumes attribution, it does not perform it. No new edge. | Satisfied |
| P-4 No Silent Loss | **The reason this slice is delicate.** Adding a discard path is the exact thing P-4 governs. Two counters, not one (FR-007, FR-008), the session sum reconciling to `gate_dropped` (FR-009), and the pipeline identity unchanged (FR-010). | **Primary constraint** |
| P-5 Compatibility Outranks Richness | Output format unchanged; the file just contains fewer packets. Unmodified analyzers read it the same way. | Satisfied |
| P-6 Glossary First | "Capture scope" is specification vocabulary already (sections 11.5, 12.3) but has no glossary entry, because nothing implemented it. It gets one. | **Gated** |
| P-7 Wrappers Stay Thin | `--scope` is one more pass-through flag; no wrapper parses output. | Satisfied |
| P-8 House Standards Apply | UTF-8 no BOM, LF, no dashes. | **Gated** |
| P-9 The Instrument Does Not Lie (NON-NEGOTIABLE) | The co-driver. `attributed 18184` means something other than it says (FR-012); `(enforced)` is false (FR-014); "filter narrowed to 0" means its opposite (FR-015 to FR-019). Each is the instrument misreporting itself. | **Primary driver** |
| P-10 One Path To A Target | Unchanged; this scopes output, not target selection. | Not engaged |
| P-11 The Specification Describes What Shipped | Sections 11.5 and 12.3 already specify this behavior and were never implemented. The code moves toward the specification, so no specification edit is needed; the changelog records `none`. | Satisfied |

No violations. Complexity Tracking omitted.

## Design

### 1. The predicate (FR-001 to FR-006)

A fourth test in `SessionGate::admit`, after the window tests and before the
bound, so an out-of-scope packet never counts against `--max-bytes`:

```
match scope {
    All     => admit
    Profile => admit when attribution carries a stage or a role
    Target  => admit when it carries a stage or role AND the role is in --roles
}
```

`--roles all` makes `Target` and `Profile` coincide, which is correct: the role
set is the scope. Making `Target` consult `--roles` is what turns the existing
`(enforced)` claim from false into true, so #184's item 5 and the `(enforced)`
half of the same issue close on one mechanism.

**Where the role set comes from.** It is held today as `allowed_roles` on
`CaptureSession` (`session.rs:171`), and the gate is built from `SessionConfig`,
which carries only the four bound fields. So `SessionConfig` gains the scope and
the role set, and `assemble.rs::session_config()` (`assemble.rs:83`) fills both
from the already-parsed `config.roles`. That keeps one structure describing what
the gate admits, rather than having the gate reach into the session.

The set is resolved once at construction into an owned collection, so the
per-packet cost is two `Option` checks plus, only under a narrowed role set, one
membership test on a short list. No allocation on the hot path.

Placement before the bound matters and is not incidental: a packet that is not
ours must not consume the operator's byte budget.

### 2. The counters (FR-007 to FR-011)

Two new atomics on `SessionGate`'s shared state, beside `watch_discarded` and
`out_of_window_discarded`:

| Counter | Meaning |
| --- | --- |
| `scope_discarded` | attribution resolved to a process no stage binds: confidently not ours |
| `scope_unresolved_discarded` | no attribution at all: might have been ours, dropped because attribution had not landed |

The split is FR-008 and is the requirement most likely to be "simplified" later.
It is what keeps the setup race visible: a non-zero unresolved count on a real
capture is a signal to investigate, and folding it into the confident counter
would bury a possible real loss inside an intended one.

`gate_dropped` in `fragcap-core` needs no change. It is the capture-wide
conservation term counted at the pipeline's gate call site, and the reasons have
always lived on `SessionGate`. That is why the existing conservation identity
keeps holding with no edit (FR-010), and why FR-009's new assertion is the
session-level sum reconciling to it.

### 3. The reporting (FR-012 to FR-014)

- `CompletionSummary` gains the two scope counters and a per-image breakdown
  from `stats.holder_tally`. Note that nothing in the CLI renders `holder_tally`
  today; `dominant_holder()` is read for the S059 promotion and the tally itself
  has never been shown. This is its first consumer, not a change to an existing
  render.
- The breakdown's meaning changes with this slice and the change must be stated
  where it renders: `holder_tally` counts only gate-admitted packets (asserted
  by an existing S059 test), so it is now a breakdown of the file rather than of
  the wire. Under the default scope it should be one image.
- `attributed` keeps its meaning (resolved to some process) and a target-scoped
  count sits beside it, rather than redefining a counter other output already
  uses.
- The scope line reports the effective `--scope` and drops `(enforced)` where it
  is not true of retention.

### 4. The narration (FR-015 to FR-020)

The two one-shot emits at `orchestrator.rs:277` and `:684` are deleted. In their
place:

- Before capture, a phase line saying capture is machine-wide while the target
  opens its first socket.
- `drive` polls `stamper.active_endpoints().len()` on its existing loop and
  emits on transition: the first narrowing gets a human line naming the target
  and the count; subsequent changes emit only the structured event (FR-020).
- `Event::FilterNarrowed` stays in the `--json` stream and is emitted per
  narrowing (FR-018).
- "endpoint(s)" leaves human output; the operator is told how many of their own
  sockets are being watched (FR-019).

## Project Structure

```text
specs/064-capture-scope/
├── spec.md
├── plan.md                    # this file, carrying Phase 0 inline
├── tasks.md
└── checklists/requirements.md
```

### Files changed

```text
crates/fragcap/src/session.rs                 # the predicate, two counters, gate tests
crates/fragcap-cli/src/cli.rs                 # --scope
crates/fragcap-cli/src/assemble.rs            # the flag into the session config
crates/fragcap-cli/src/orchestrator.rs        # the scope line, the narration
crates/fragcap-cli/src/output.rs              # the summary
crates/fragcap-cli/src/events.rs              # the scope counters in the stream
crates/fragcap-cli/tests/cli_capture.rs       # scope assertions
docs/glossary/capture-and-networking.md       # "capture scope" (P-6)
changelog.d/S064-capture-scope.*.md
```

**Structure Decision**: no new module. The predicate belongs to the gate that
already makes every other admission decision, and putting it anywhere else would
create a second place a packet can be withheld.

## Verification

`cargo xtask ci` in the foreground, watched to completion, with particular
attention to:

- the CLI capture goldens, which must not move (SC-005);
- the pipeline conservation tests, which must still pass unchanged (SC-003).

Then, beyond the gate set:

1. A replay capture with mixed attributed and non-attributed traffic, checking
   the counters reconcile (SC-002).
2. `--scope all` producing byte-identical output to `main` (SC-004).
3. **A real `--launch` capture on this machine**, parsing the written pcapng by
   `proc=` to confirm SC-001. This is the only check that exercises the defect as
   filed. The mechanism is proven without it by check 1, which drives the same
   gate over scripted attributions, so if the live run cannot be performed the
   claim degrades to "demonstrated by replay, not confirmed in the field" and
   that is what gets reported. It is not a silent omission either way.
