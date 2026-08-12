# Implementation Plan: Watch / Attach Mode (Launch-Agnostic Capture)

**Branch**: `feat/watch-attach-mode` | **Date**: 2026-08-12 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/028-watch-attach-mode/spec.md`

## Summary

Add a `watch` subcommand that captures by a target identity (an executable glob
plus an optional path anchor), launch-agnostic, reusing the existing capture
engine. Close the one real runtime gap the code has: the process watcher takes a
P-1-safe startup snapshot of already-running processes but the capture path never
applies it, so a game already running at arm is never acquired. Add
`CaptureSession::apply_snapshot`, carry the snapshot on `CaptureComponents`, and
apply it at arm in both drivers, so an already-running match is acquired at arm
(attach-to-running) while a later start is acquired by the existing
wait-for-start. Wire the S027 `ObservationProvider` over the snapshot to produce
the honest `observed` answer that names the already-running process, keeping the
session the single acquisition authority. Reuse the acquisition timeout
(`--wait`) and its `StopReason::AcquisitionTimeout` give-up. Name watch mode as
the default launch-agnostic path in the spec and glossary.

## Technical Context

**Language/Version**: Rust, MSRV 1.82 (toolchain pinned 1.96)

**Primary Dependencies**: none new. Reuses `fragcap` (session, orchestrator),
`fragcap-profile` (S027 resolver + `Profile::parse`), `serde_json` (already
runtime, for identity synthesis as `tap` does).

**Testing**: `cargo test`, `cargo xtask ci`; offline tier-1 via the hidden
`OfflineArgs` substrate and `ProcessScript::with_snapshot`.

**Target Platform**: the `watch` surface and the offline path are
platform-neutral and CI-tested; live attach-to-running over ETW is tier-2.

**Project Type**: Rust workspace (CLI + library crates).

**Constraints**: no process handle (P-1); named give-up counter surfaced (P-4);
fidelity carried honestly, authored vs observed not conflated (P-9); output
byte-identical to an equivalent single-stage profile capture; UTF-8 no BOM, LF,
no em/en dashes.

**Scale/Scope**: one new subcommand + command module; a session method; a
components field and its two builders; a snapshot-application step in both
drivers; the resolver wiring in the command; spec sections 7.1/10.5 and a
glossary entry.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P-1 Passive Observation**: Attach-to-running reads only the toolhelp startup
  snapshot (`ProcessRecord`: image name, path, pid, parent) the watcher already
  takes; no process handle is opened. `cargo xtask lint` stays green. PASS.
- **P-2 Core Stays Platform-Neutral**: No change to fragcap-core. The session
  method and orchestrator changes are in the `fragcap` facade and the CLI. PASS.
- **P-3 Capture And Attribution Stay Separate**: The snapshot feeds process
  identity (the session's tree), not the attributor; the ObservationProvider is a
  resolver, not a `FlowAttributor`. PASS.
- **P-4 No Silent Loss**: The give-up reuses `StopReason::AcquisitionTimeout` and
  the watch-time discard accounting, surfaced in the summary. PASS.
- **P-5 Compatibility Outranks Richness**: No output format change; byte-identical
  to a single-stage profile capture. PASS.
- **P-6 Glossary First**: `watch mode` gains a glossary entry referencing
  `launcher_mediated`; spec 7.1/10.5 updated; docs linter enforces. PASS.
- **P-9 The Instrument Does Not Lie**: The synthesized identity is `authored`
  (never falsely `observed`, which S027 refuses on a profile); the observation
  provider's `observed` answer is a separate axis. A watch that captured nothing
  says so. PASS.

No violations. Complexity Tracking empty.

## Project Structure

### Source Code (repository root)

```text
crates/fragcap-cli/src/
├── cli.rs             # + WatchArgs; + Command::Watch(WatchArgs)
├── lib.rs             # + Command::Watch => commands::watch::run
├── commands/
│   └── watch.rs       # NEW: synthesize identity profile (exe + path anchors),
│                      #   build config, resolve attach-to-running via the S027
│                      #   ObservationProvider, capture
├── assemble.rs        # + effective_config_for_watch; carry the startup snapshot
│                      #   (records + instant) on CaptureComponents from both the
│                      #   offline ScriptedWatcher and the live EtwWatcher
└── orchestrator.rs    # apply the snapshot at arm in both drivers before the
                       #   acquisition loop; report the observed attach

crates/fragcap/src/
├── session.rs         # + CaptureSession::apply_snapshot(records, at): fold the
│                      #   snapshot into the tree and run matching, so an already
│                      #   running match acquires at arm
└── lib.rs             # re-export ProcessRecord if the CLI needs it by name

docs/
├── fragcap-specification.md          # 7.1 and 10.5: watch mode as the default
│                                     #   launch-agnostic path
└── glossary/process-and-attribution.md  # + watch mode (near Process watcher,
                                          #   Acquisition timeout)
```

**Structure Decision**: `watch` mirrors `tap`'s shape (a command module that
synthesizes a validated identity profile and hands it to
`orchestrator::capture`), extended with a path anchor, `--wait`, and the
attach-to-running wiring. The session gains the snapshot-application method the
capture path was missing; the orchestrator applies it at arm for both the offline
and live drivers so the two behave identically and the offline path is fully
CI-testable.

## Phased approach

- **Phase 0 (research.md)**: the seven decisions, chief among them the
  surface (a `watch` subcommand), the attach-to-running mechanism (session
  snapshot application as the single acquisition authority, with the
  ObservationProvider producing the honest observed answer), and the authored
  vs observed fidelity separation.
- **Phase 1 (data-model.md, contracts/, quickstart.md)**: `WatchArgs`, the
  `CaptureSession::apply_snapshot` contract, the components snapshot fields, and
  a walkthrough of the three scenarios offline.
- **Phase 2 (tasks.md)**: dependency-ordered tasks.

## Complexity Tracking

No constitution violations; no entries.
