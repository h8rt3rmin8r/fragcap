# Implementation Plan: Durable Guided Calibration Resume

**Branch**: `codex/s145-guided-calibration-resume` | **Date**: 2026-09-11 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/145-guided-calibration-resume/spec.md`

## Summary

Add a versioned, target-bound guided-calibration checkpoint to the existing local
SQLite store and let `fragcap calibrate --resume <ID>` rebuild the current S139
proposal from it. Persist only candidate intent, coverage projection, bounded exact
attempt history, attempt ordinal, and lifecycle state. Preserve existing recovery authority and require a fresh S134
plan and confirmation for every resumed effect. Add explicit no-effect pause reasons
for operator-owned login, EULA, gameplay, shutdown, and interruption work.

This deliberately changes S144's stateless cross-process continuation. The S144
safety rationale remains: current target, process, facts, proposal, bundle, recovery,
plan, and response are still fresh for every attempt. Only the unnecessary loss of
workflow intent, exact no-repeat history, and ordinal at process exit is removed.

## Technical Context

**Language/Version**: Rust 1.88 workspace MSRV

**Primary Dependencies**: Existing `rusqlite`, `serde_json`, `clap`, `fragcap`
facade, `fragcap-targets`, and native Deep Capture stack

**Storage**: Existing per-user SQLite target store, additive version 10 to 11
migration with one guided-calibration workflow table

**Testing**: Target-store unit and migration tests, CLI unit and controlled integration
tests, stable event serialization and human rendering, reference tests, `cargo xtask ci`

**Target Platform**: Windows production path, with persistence, orchestration, and
controlled session behavior testable on CI hosts

**Project Type**: Rust workspace libraries plus command-line binary

**Performance Goals**: One indexed workflow read and bounded conditional update per
checkpoint; no unbounded histories or polling; no more than fourteen attempt ordinals

**Constraints**: Feature branch and PR; UTF-8 without BOM; LF; 100-column Rust; no
new dependency or lockfile package; no persisted secrets or execution authority; no
process handle, target control, system proxy, hidden trust, pinning bypass, or parent
completion claim

**Scale/Scope**: One additive store entity and migration, resume and pause CLI surface,
guidance additions, focused controlled tests, public reference updates, architecture
record updates, and changelog fragments

## Constitution Check

*GATE: Passed before research and passed again after design.*

- **P-1**: PASS. Resume is explicit, target-bound, current-state checked, and every
  effect retains a fresh complete plan and confirmation.
- **P-2**: PASS. The target store owns durable target-associated progress. The facade
  retains proposal and lifecycle policy. The CLI maps arguments and orchestration.
- **P-3**: PASS. No capture, attribution, proxy, sink, or artifact dependency edge
  changes. Existing public crate direction is preserved.
- **P-4**: PASS. In-flight, pause, completion, and remaining work are explicit.
  Compatibility and effect losses remain under their existing counters and journals.
- **P-5**: PASS. Store version 11 is additive and tested from version 10. Event fields
  are nullable additions. Existing artifact schemas are unchanged.
- **P-6**: PASS. Existing calibration phase and protocol vocabularies are reused.
  Workflow state and pause reason are new closed domain types.
- **P-7**: PASS. Persistence and orchestration stay in Rust. No shell or wrapper path
  is added.
- **P-8**: PASS. TDD, migration tests, controlled resume tests, convergence, full gate,
  and review resolution are required.
- **P-9**: PASS. Intent, observed candidates, coverage projections, facts, pauses,
  plans, effects, and recovery obligations remain separate values.
- **P-10**: PASS. Resume uses the same local store, stable target resolution, S139
  proposal, and S134 executor.
- **P-11**: PASS. Architecture, outline, plan order, CLI reference, changelog, and
  agent record will be reconciled with the shipped boundary.
- **Third-party obligations**: PASS. No dependency, package, license, Npcap, or
  distribution change.

## Project Structure

### Documentation

```text
specs/145-guided-calibration-resume/
├── checklists/
├── contracts/calibrate-resume-command.md
├── data-model.md
├── plan.md
├── quickstart.md
├── research.md
├── spec.md
└── tasks.md
```

### Source Code

```text
crates/fragcap-targets/src/
├── lib.rs
├── schema.rs
├── store.rs
└── workflow.rs

crates/fragcap-cli/
├── src/
│   ├── cli.rs
│   ├── commands/calibrate.rs
│   └── events.rs
└── tests/
    ├── cli_calibrate.rs
    └── cli_reference.rs

docs/
├── fragcap-specification.md
├── fragcap-spec-outline.md
└── plans/README.md

site/content/docs/reference/cli.mdx
```

**Structure Decision**: Add closed workflow domain values beside compatibility values
in `fragcap-targets`, with SQLite mapping in its existing store. Keep fresh proposal,
authorization delegation, pause mapping, and rendering in the existing calibration
command. Extend the existing guidance event rather than creating a second event path.

## Phase 0: Research Decisions

The decisions in [research.md](research.md) select the existing local store, explicit
store-local identity, complete target snapshot, canonical protocol sets, revisioned
boundary checkpoints, bounded pause reasons, fresh authorization, and a narrow parent
boundary.

## Phase 1: Design

The durable entity and transitions are defined in [data-model.md](data-model.md), the
CLI and event contract in [contracts/calibrate-resume-command.md](contracts/calibrate-resume-command.md),
and executable validation in [quickstart.md](quickstart.md).

## Complexity Tracking

No constitution violation or exceptional complexity is required.
