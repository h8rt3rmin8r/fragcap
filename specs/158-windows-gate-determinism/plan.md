# Implementation Plan: S158 Windows gate determinism

**Branch**: `codex/s158-windows-gate-determinism` | **Date**: 2026-09-19 UTC | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/158-windows-gate-determinism/spec.md`

## Summary

Close the remaining Windows scheduling races behind #413 and #429. Publish the application sink only after its writer thread proves readiness, replace the stale shared capture-clock check with explicit bounded observation-drain completion, and make controlled calibration environment ownership recoverable and unwind-safe. Preserve every queue, conservation, deadline, cleanup and security authority while requiring first-attempt hosted Windows evidence.

## Technical Context

**Language/Version**: Rust 2021 with MSRV 1.88

**Primary Dependencies**: Rust standard library synchronization and existing workspace crates only

**Storage**: Existing application JSON Lines version 2 and Deep Capture bundle contracts, unchanged

**Testing**: Rust unit and integration tests, scripted clocks, controlled loopback fixtures, full workspace gates and GitHub-hosted Windows performance and platform workflows

**Target Platform**: Windows-first product behavior with portable deterministic unit coverage; hosted Ubuntu performance remains a comparison gate

**Project Type**: Rust workspace, CLI and isolated native performance harness

**Performance Goals**: Preserve the 4,096-event application queue and S128 thresholds; fourteen Windows cases complete with zero queue or storage loss

**Constraints**: No producer blocking, event suppression, queue expansion, deadline grace, retry waiver, installed local product, real game, real trust mutation or sensitive live capture

**Scale/Scope**: One writer publication boundary, one internal observation-drain result, one session coordinator, one calibration test binary and two affected hosted workflows

## Constitution Check

*GATE: Passed before research and re-checked after design.*

- **P-1**: Pass. Work uses controlled loopback fixtures and existing explicit Deep Capture behavior. No target instrumentation or real-game execution is introduced.
- **P-2 and P-3**: Pass. Changes remain in the facade, native adapter, tests and repository evidence. Core, capture-source and attribution separation is unchanged.
- **P-4**: Pass. Queue and storage loss remain named, conserved and failing in the performance authority. No event class is suppressed.
- **P-5**: Pass. Capture and application artifact formats remain unchanged.
- **P-6**: Pass. No new product-domain term enters user documentation. Slice-local terms are defined in the data model and contract.
- **P-7**: Pass. No wrapper logic changes.
- **P-8**: Pass. Rust and Markdown follow repository standards and all mechanical gates remain required.
- **P-9**: Pass. Complete structured drain evidence is retained as observed, and genuine incomplete work remains a failure. Scheduler timing cannot fabricate loss or invalidate completed truth.
- **P-10**: Pass. Target creation, storage and resolution remain unchanged.
- **P-11**: Pass. The master specification will record the S158 correction without changing the published v0.10.1 baseline.

## Project Structure

### Documentation (this feature)

```text
specs/158-windows-gate-determinism/
├── checklists/
│   ├── reliability.md
│   └── requirements.md
├── contracts/
│   └── windows-gate-determinism.md
├── data-model.md
├── plan.md
├── quickstart.md
├── research.md
├── spec.md
└── tasks.md
```

### Source Code (repository root)

```text
changelog.d/
├── S158-windows-gate-determinism.decisions.md
└── S158-windows-gate-determinism.fixed.md

crates/fragcap/src/deep_capture/
├── adapters.rs
├── application.rs
├── native.rs
└── session.rs

crates/fragcap/tests/
└── deep_capture_session.rs

crates/fragcap-cli/tests/
└── cli_calibrate.rs

docs/
├── fragcap-specification.md
└── plans/README.md
```

**Structure Decision**: Correct existing ownership boundaries in place. No new crate, dependency, artifact format, queue or workflow is warranted.

## Complexity Tracking

No constitution violation or architecture exception is required.
