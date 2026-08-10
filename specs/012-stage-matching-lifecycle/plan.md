# Implementation Plan: Stage Matching and Session Lifecycle

**Branch**: `feat/stage-matching-lifecycle` | **Date**: 2026-08-10 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/012-stage-matching-lifecycle/spec.md`

## Summary

S12 joins the profile's declared stages (S05) to the observed process tree (S11)
and drives the capture session through its five states. Two components: a pure
stage matcher in `fragcap-profile` that evaluates the five predicates against a
process node and selects the binding stage, and a capture-session state machine
in the `fragcap` facade that arms before the target exists, discards and counts
packets while watching, retains on the first match, and stops cleanly on any of
six conditions. The one core change is a binding method on `ProcessTree` that
writes the node stage field S11 reserved. Everything is tier-1 testable against
the scripted watcher.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.82

**Primary Dependencies**: none new. `regex` (already a `fragcap-profile`
dependency) evaluates `path_regex`; the glob matcher and process tree already
exist.

**Storage**: N/A (in-memory process tree and session state)

**Testing**: `cargo test` at tier 1, driven by `proc_script::ScriptedWatcher`
and a scripted packet/event sequence; no capture driver, elevation, or game.

**Target Platform**: platform-neutral. Matching lives in `fragcap-profile` and
the session in the `fragcap` facade; neither takes a platform dependency, so
both build for the backend-free target `cargo xtask neutral` checks.

**Project Type**: Rust library workspace (the library is the product).

**Performance Goals**: matching is per process start event, not per packet; the
per-packet path is a state check plus a counter increment. No hot-path
allocation is introduced on the packet path.

**Constraints**: dependency direction (section 8.3); no `fragcap-attr` to
`fragcap-profile` edge; `fragcap-core` stays platform-neutral.

**Scale/Scope**: a launcher chain is a handful of stages and a tree of tens of
nodes; a session is one per capture.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P-1 Passive Observation**: no denylisted technique. Matching reads image
  path, image name, and command line already carried on tree nodes (recorded by
  S11 without a process handle); the session opens no handle and reads no process
  memory. PASS.
- **P-2 Core Stays Platform-Neutral**: `fragcap-core` gains only
  `ProcessTree::bind_stage`, pure data mutation over existing types (`NodeId`,
  `StageId`), no platform dependency. Matching lands in `fragcap-profile`
  (depends on core, allowed); the session lands in the facade (depends on both,
  allowed). No sibling edge is created. PASS.
- **P-3 Capture And Attribution Stay Separate**: S12 adds neither a packet source
  nor an attributor and merges nothing. The session composes existing seams. PASS.
- **P-4 No Silent Loss**: packets discarded while Watching are counted in a named
  `watching_discarded` counter on the session's own accounting and surfaced; a
  session-level conservation identity (observed equals retained plus
  watching-discards) is asserted in tests. PASS.
- **P-6 Glossary First**: new terms (stage matching, stage binding, capture
  session and its states, acquisition timeout, stop condition) get glossary
  entries in this change. PASS.
- **P-9 The Instrument Does Not Lie**: an unavailable command line never
  satisfies `cmdline_contains` (no substitution of empty for unknown); the
  matcher alters no observation; the session discards only where the specification
  says (Watching) and counts what it discards. PASS.

No violations; Complexity Tracking is empty.

## Project Structure

### Documentation (this feature)

```text
specs/012-stage-matching-lifecycle/
├── plan.md
├── spec.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── matching-lifecycle-api.md
├── checklists/
│   ├── requirements.md
│   └── matching.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/fragcap-core/src/process/tree.rs      # + ProcessTree::bind_stage (writes the reserved node stage)
crates/fragcap-profile/src/matching.rs       # NEW: predicate evaluation + stage_for + bind_stages
crates/fragcap-profile/src/lib.rs            # + pub mod matching; re-exports
crates/fragcap/src/session.rs                # NEW: CaptureSession, SessionState, SessionConfig,
crates/fragcap/src/lib.rs                    #   SessionStats, StopReason, PacketDisposition
crates/fragcap/tests/session.rs              # NEW: tier-1 lifecycle + stop-condition tests
crates/fragcap-profile/tests/matching.rs     # NEW: tier-1 predicate + chain tests (or in matching.rs #[cfg(test)])
docs/glossary.md                             # + new terms (P-6)
changelog.d/S12-stage-matching.added.md
changelog.d/S12-stage-matching.decisions.md
```

**Structure Decision**: matching in `fragcap-profile` (the only crate that may
read both the profile schema and the core tree without a sibling edge); the
session in the `fragcap` facade (the only crate that sees watcher, pipeline,
profile, and sinks together); the single mutation point in `fragcap-core`.

## Phase 0: Research

See [research.md](research.md). Decisions D-1 through D-6 record the binding
storage, the descends_from evaluation model, the multi-stage precedence, the
watching-discard counter placement, the duration/timeout clock origin, and the
service-process treatment in the all-exited stop condition.

## Phase 1: Design

See [data-model.md](data-model.md) for the entities and [contracts/](contracts/)
for the public surface. [quickstart.md](quickstart.md) shows the tier-1 test
path.

## Complexity Tracking

No constitution violations; no entries.
