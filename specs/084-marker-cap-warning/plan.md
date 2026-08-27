# Implementation Plan: Marker Cap Warning Subject

**Branch**: `codex/084-marker-cap-warning` | **Date**: 2026-08-26 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/084-marker-cap-warning/spec.md`

## Summary

Fix issue #206 by making the shared detection coverage warning for capped binary-marker scans name the scanned root, preserve the skipped candidate count, and state that technology detection for that root may be incomplete. The warning remains produced by `ScanOutcome::coverage_warnings()` so `fragcap technologies`, `fragcap targets`, and `fragcap targets discover` inherit one contract instead of growing caller-specific wording.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.82

**Primary Dependencies**: Existing workspace crates only (`fragcap-profile`, `fragcap-targets`, `fragcap-cli`)

**Storage**: No storage changes

**Testing**: `cargo test`, targeted crate tests, CLI integration tests, `cargo xtask ci`

**Target Platform**: Windows CLI first, with affected detection logic platform-neutral

**Project Type**: Rust workspace CLI and libraries

**Performance Goals**: No additional filesystem walk or binary read work; the scanned root is stored from the existing detection call

**Constraints**: Preserve bounded scan behavior, exact skipped count, shared warning helper, no new dependencies, no process instrumentation

**Scale/Scope**: One detection outcome type, warning text tests at library and command boundaries, one master-spec revision

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P-1 No Covert Target Instrumentation**: Pass. This slice changes only local detection diagnostics and adds no capture, proxy, process, ETW, socket table, trust, or network behavior.
- **P-2 Core Stays Platform-Neutral**: Pass. No `fragcap-core` changes.
- **P-3 Capture And Attribution Stay Separate**: Pass. No capture or attribution changes.
- **P-4 No Silent Loss**: Pass. The slice makes an existing reduced-coverage warning recoverable to its scan root.
- **P-5 Compatibility Outranks Richness**: Pass. No capture output format changes.
- **P-6 Glossary First**: Pass. No new domain term is introduced; marker, scan, and detection vocabulary already exist in the specification and glossary.
- **P-7 Wrappers Stay Thin**: Pass. No wrapper changes.
- **P-8 House Standards Apply**: Pass. All generated and edited text must satisfy repository lint, including no em or en dashes.
- **P-9 The Instrument Does Not Lie**: Pass. The warning reports the observed skipped count and reduced coverage without implying clean absence.
- **P-10 One Path To A Target**: Pass. The same scan outcome continues to feed all target sources.
- **P-11 The Specification Describes What Shipped**: Pass. The master specification will be updated with the shipped warning contract.

## Project Structure

### Documentation (this feature)

```text
specs/084-marker-cap-warning/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── coverage-warning.md
├── checklists/
│   ├── requirements.md
│   └── warnings.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/fragcap-profile/src/signature.rs
crates/fragcap-targets/tests/user_pointed.rs
crates/fragcap-targets/src/classifier.rs
crates/fragcap-cli/tests/cli_targets.rs
crates/fragcap-cli/src/commands/technologies.rs
docs/fragcap-specification.md
changelog.d/
```

**Structure Decision**: Store the scanned root on `ScanOutcome` because `coverage_warnings()` is the single warning surface every caller is supposed to use. Caller-level formatting keeps prefixes and indentation, but not warning content.

## Complexity Tracking

No constitution violations or complexity exceptions are needed.

## Phase 0: Research

See [research.md](research.md).

## Phase 1: Design

See [data-model.md](data-model.md), [contracts/coverage-warning.md](contracts/coverage-warning.md), and [quickstart.md](quickstart.md).

## Post-Design Constitution Check

The design still passes all constitution checks. It carries one additional value already known at scan time, adds no new authority or scan behavior, and improves the P-4/P-9 reporting contract.
