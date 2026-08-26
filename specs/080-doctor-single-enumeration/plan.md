# Implementation Plan: Doctor Single Enumeration

**Branch**: `codex/080-doctor-single-enumeration` | **Date**: 2026-08-26 | **Spec**: `specs/080-doctor-single-enumeration/spec.md`

**Input**: Feature specification from `specs/080-doctor-single-enumeration/spec.md`

## Summary

Fix issue #203 by making the doctor capture driver and interfaces probe reuse the interface inventory it already gathers to classify loopback support. The implementation will keep `detect_driver()` intact for callers without an inventory, introduce a small shared loopback predicate or probe seam as needed, and cover the three-valued loopback contract with tests so enumeration failure remains unknown rather than false.

## Technical Context

**Language/Version**: Rust 1.82 minimum, current pinned workspace toolchain for feature-on checks

**Primary Dependencies**: Existing workspace crates only; no new dependency planned

**Storage**: N/A for this slice

**Testing**: `cargo test` focused CLI tests, doctor golden coverage, `cargo xtask ci`

**Target Platform**: Windows live-capture readiness path, with pure unit coverage for classifier behavior where possible

**Project Type**: Rust CLI inside the existing Cargo workspace

**Performance Goals**: One successful doctor live probe must perform one live device-list enumeration instead of two

**Constraints**: Preserve final doctor human and JSON report contracts, preserve S079 progress and `--timings` behavior, preserve loopback `Some(true)`, `Some(false)`, and `None` semantics, and avoid touching capture behavior

**Scale/Scope**: One doctor probe path plus any small shared predicate needed to avoid duplicating loopback logic

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- P-1 No covert target instrumentation: PASS. Doctor remains a local readiness check and adds no capture, proxying, target process access, traffic modification, or certificate behavior.
- P-2 Core stays platform-neutral: PASS. Planned work stays in CLI and live-capture facade code already responsible for platform-backed readiness.
- P-3 Capture and attribution stay separate: PASS. No packet source or attributor behavior changes.
- P-4 No silent loss: PASS. The slice does not touch packet loss paths or capture statistics.
- P-5 Compatibility outranks richness: PASS. No output-format or capture-file changes.
- P-6 Glossary first: PASS. No new domain term is introduced; existing npcap, loopback, interface, and diagnostics vocabulary applies.
- P-7 Wrappers stay thin: PASS. No shell wrapper logic changes.
- P-8 House standards apply: PASS. Slice artifacts and source changes must pass repository lint, formatting, and text hygiene checks.
- P-9 The instrument does not lie: PASS. Enumeration failure remains unknown, not false, and any speedup claim requires evidence.
- P-10 One path to a target: PASS. No target storage or resolution behavior changes.
- P-11 Specification describes what shipped: PASS. The master spec's doctor report contract is unchanged; no master-spec edit is planned unless implementation changes that contract.

## Project Structure

### Documentation (this feature)

```text
specs/080-doctor-single-enumeration/
|-- checklists/
|   `-- requirements.md
|-- contracts/
|   `-- doctor-live-probe.md
|-- data-model.md
|-- plan.md
|-- quickstart.md
|-- research.md
|-- spec.md
`-- tasks.md
```

### Source Code (repository root)

```text
crates/fragcap-cli/src/doctor/probe.rs
crates/fragcap-cli/src/doctor/progress.rs
crates/fragcap-cli/tests/cli_doctor.rs
crates/fragcap-capture/src/live/driver.rs
crates/fragcap-capture/src/live/enumerate.rs
crates/fragcap/src/lib.rs
changelog.d/203-doctor-single-enumeration.fixed.md
```

**Structure Decision**: Keep the behavior in the existing doctor probe path if `InterfaceRecord` already exposes the needed fields. If avoiding duplicated loopback logic requires a shared helper, place it in the live-capture layer where both `detect_driver()` and doctor can use it without adding a new crate dependency.

## Phase 0 Research

### Decision: Reuse the first successful interface inventory

Doctor currently needs both the interface list and the loopback support verdict in one report. The issue shows both facts are available from the first enumeration because `InterfaceRecord` carries loopback evidence and description text.

**Rationale**: A second `pcap::Device::list()` call is pure duplicate work for doctor and can be expensive on Windows. Reusing the first inventory preserves the observed data and removes the redundant cost.

**Rejected Alternative**: Keep calling `detect_driver()` from doctor and rely on S079 progress to make the duplicate cost visible. This leaves a known duplicate cost in the first troubleshooting command.

### Decision: Preserve unknown on failed enumeration

When enumeration fails, doctor did not observe loopback absence. The loopback verdict therefore remains `None`.

**Rationale**: `Some(false)` means observed absent. Reporting false after an error would violate P-9 and make remediation harder.

**Rejected Alternative**: Treat an empty failed inventory as `Some(false)`. That produces a comfortable but false answer.

### Decision: Avoid a stable output change

The final doctor report and JSON record contracts stay unchanged. Tests should focus on probe inputs and existing golden stability.

**Rationale**: Issue #203 is an internal duplicate-work defect. Changing report text would create unnecessary consumer churn.

**Rejected Alternative**: Add new JSON timing or enumeration-count fields. S079 explicitly made timings an interactive maintainer surface, not a stable machine contract.

## Phase 1 Design

The probe will keep its existing readiness inputs and change only their source:

- successful live enumeration returns interfaces and enough loopback evidence to set `Some(true)` or `Some(false)`;
- failed live enumeration returns no interfaces, carries the existing error, and leaves loopback `None`;
- unavailable live backend or unloadable `wpcap.dll` leaves loopback `None` without trying enumeration;
- `detect_driver()` remains a standalone call for callers that do not already hold an inventory.

The final implementation choice is bounded by code inspection:

- if `InterfaceRecord` and `detect_driver()` can share a predicate over public fields, use one helper;
- if the live backend has a cleaner single-enumeration API, use that path instead;
- do not add a dependency or move platform-specific logic into `fragcap-core`.

## Complexity Tracking

No constitution violations.
