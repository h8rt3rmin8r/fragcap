# Implementation Plan: Doctor Progress And Timing

**Branch**: `codex/079-doctor-progress-timing` | **Date**: 2026-08-26 | **Spec**: `specs/079-doctor-progress-timing/spec.md`

**Input**: Feature specification from `specs/079-doctor-progress-timing/spec.md`

## Summary

Make `fragcap doctor` visibly alive while it gathers readiness facts, and make
probe cost measurable without changing the stable report surfaces. The
implementation adds a progress/timing observer around doctor probe gathering,
routes interactive progress to stderr only for human terminal runs, preserves the
existing report renderers as the only stdout authority, and records timing
evidence for the slow local probes that motivated issue #202.

## Technical Context

**Language/Version**: Rust 1.82 minimum, current pinned toolchain for feature-on checks

**Primary Dependencies**: Existing workspace crates only; no new dependency planned

**Storage**: N/A for this slice

**Testing**: `cargo test` package tests, CLI integration tests, doctor golden checks, `cargo xtask ci`

**Target Platform**: Windows CLI behavior, with pure cross-platform tests for rendering and suppression where possible

**Project Type**: Rust CLI inside the existing Cargo workspace

**Performance Goals**: Interactive human `fragcap doctor` must emit visible progress within roughly 200 ms under an injected slow-probe path; normal probe work must not be skipped or fabricated

**Constraints**: Preserve `doctor --json`, redirected human stdout, doctor goldens, report classification, `--fix` behavior, extcap/script output, and defensive P-1/P-9 boundaries

**Scale/Scope**: One CLI command path, one probe-gathering seam, one progress/timing presentation path, one master-spec reconciliation

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- P-1 Authorized, scoped, defensive: PASS. Doctor remains a local readiness
  check and adds no capture, proxying, process access, or traffic inspection.
- P-2 Capture boundaries: PASS. No packet acquisition or Deep Capture routing
  behavior changes.
- P-3 Crate boundaries: PASS. Work stays in `fragcap-cli` doctor code and
  public facade calls already used by doctor.
- P-4 Loss and failure accounting: PASS. Readiness failures remain represented
  by existing report items; progress does not reinterpret them.
- P-5 Profile validation before capture: PASS. No capture/profile path changes.
- P-6 Stable artifacts: PASS. JSON and redirected human outputs remain byte
  stable; final renderers remain authoritative.
- P-7 Minimal dependency graph: PASS. No new crates.
- P-8 Spec/test traceability: PASS. Tests target progress, suppression, stable
  output, and timing evidence.
- P-9 Deterministic truthfulness: PASS. Progress says which probe is running and
  reports measured elapsed durations, not guessed status.
- P-10 Accessibility and operator clarity: PASS. Interactive doctor no longer
  appears hung during slow readiness checks.
- P-11 Contributor workflow: PASS. Slice artifacts, tasks, changelog fragments,
  and full gate remain required before handoff.

## Project Structure

### Documentation (this feature)

```text
specs/079-doctor-progress-timing/
|-- checklists/
|   |-- progress.md
|   `-- requirements.md
|-- contracts/
|   |-- doctor-progress.md
|   `-- doctor-timings.md
|-- data-model.md
|-- plan.md
|-- quickstart.md
|-- research.md
|-- spec.md
`-- tasks.md
```

### Source Code (repository root)

```text
crates/fragcap-cli/src/cli.rs
crates/fragcap-cli/src/commands/doctor.rs
crates/fragcap-cli/src/doctor/mod.rs
crates/fragcap-cli/src/doctor/probe.rs
crates/fragcap-cli/src/doctor/progress.rs
crates/fragcap-cli/tests/cli_doctor.rs
docs/fragcap-specification.md
changelog.d/202-doctor-progress.fixed.md
changelog.d/202-doctor-progress.decisions.md
```

**Structure Decision**: Keep doctor classification and final rendering in the
existing doctor command/report path. Add only a narrow progress observer module
and instrumentation around existing probe phases, so the stable report format
does not depend on progress output.

## Complexity Tracking

No constitution violations.
