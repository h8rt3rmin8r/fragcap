# Implementation Plan: Explicit Guided Calibration Choice

**Branch**: `codex/s146-guided-calibration-choice` | **Date**: 2026-09-11 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/146-guided-calibration-choice/spec.md`

## Summary

Add a canonical content-derived choice identity for discovered target and Steam executable ambiguity, emit a stable pre-target choice contract, and extend S145 workflows with immutable launch, routing, and address-family intent. Reuse the existing proposal, registration, Steam authoring, authorization, recovery, and Deep Capture execution authorities without adding dependencies or topology-writing paths.

## Technical Context

**Language/Version**: Rust 1.88 workspace MSRV

**Primary Dependencies**: Existing clap, rusqlite, serde_json, and blake3 workspace dependencies

**Storage**: Existing user-owned SQLite `local.db`, additive schema version 12

**Testing**: Rust unit and integration tests plus `cargo xtask ci`

**Target Platform**: Windows 10/11 product paths; offline controlled tests remain cross-platform

**Project Type**: Multi-crate Rust CLI and libraries

**Performance Goals**: Candidate identity and matching remain linear in the bounded discovery result; no packet-path changes

**Constraints**: No process control, no system proxy, no new trust path, no persisted authorization or secret, no non-Steam topology rewrite, no new dependency or lockfile package

**Scale/Scope**: One candidate option, one event, three workflow intent fields, one additive migration, CLI and documentation integration

## Constitution Check

*GATE: Passed before Phase 0 research and rechecked after Phase 1 design.*

- **P-1 and authorized use**: Candidate selection and case validation add no instrumentation, process handle, memory access, or process control.
- **P-2 and P-3**: Selection lives in targets and guided facade/CLI ownership; capture and attribution boundaries do not change.
- **P-4 and P-9**: Ambiguity, duplicate authority, unsupported routing, stale selection, and mismatched topology are explicit no-effect outcomes.
- **P-5**: Artifact schemas are unchanged; CLI JSON adds a versioned event and additive guidance fields.
- **P-6**: Candidate choice, target identity, workflow intent, authorization, topology, and evidence remain distinct terms.
- **P-7**: The CLI orchestrates existing library authorities. Canonical candidate identity is a reusable targets-domain function.
- **P-8**: Tests lead implementation; Rust and repository gates remain mandatory.
- **P-10**: No alternate capture or Deep Capture execution path is introduced.
- **P-11**: Architecture and public CLI references will describe only the shipped S146 boundary.

## Project Structure

### Documentation

```text
specs/146-guided-calibration-choice/
├── checklists/
│   ├── requirements.md
│   └── security.md
├── contracts/
│   └── calibrate-choice-command.md
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
├── choice.rs
├── schema.rs
├── store.rs
└── workflow.rs

crates/fragcap-cli/src/
├── cli.rs
├── commands/calibrate.rs
└── events.rs

crates/fragcap-cli/tests/cli_calibrate.rs
docs/
site/content/docs/reference/cli.mdx
changelog.d/
```

**Structure Decision**: Canonical identity belongs to the target domain, durable intent extends the existing workflow authority, and command-specific choice emission and orchestration remain in the CLI. No new crate or abstraction layer is warranted.

## Complexity Tracking

No constitutional violation requires justification.
