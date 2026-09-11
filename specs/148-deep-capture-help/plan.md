# Implementation Plan: Deep Capture Embedded Workflow Help

**Branch**: `codex/s148-deep-capture-help` | **Date**: 2026-09-11 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/148-deep-capture-help/spec.md`

## Summary

Make the complete Deep Capture first-run journey discoverable offline through concise short help, structured long help, and actionable pre-session refusals. Add one typed workflow and example registry, use the production Clap command tree for progressive disclosure and parser-backed validation, organize Deep Capture and calibration options by operator consequence, and preserve every structured, authorization, compatibility, cleanup, and effect contract.

## Technical Context

**Language/Version**: Rust 1.88 workspace MSRV

**Primary Dependencies**: Existing Clap 4.5 command model and repository CLI utilities; no dependency change

**Storage**: No persistent schema change; one in-process workflow/help registry

**Testing**: Rust unit and integration tests, production Clap rendering and parsing, controlled temporary-store refusal tests, and `cargo xtask ci`

**Target Platform**: Cross-platform help and controlled tests, with Windows-specific product wording preserved; no live game or trust effect

**Project Type**: Multi-crate Rust CLI and libraries with repository task runner

**Performance Goals**: Help rendering and registry validation remain instantaneous relative to command startup and add no runtime work outside help or human error paths

**Constraints**: UTF-8 without BOM, one parser authority, concise `-h`, complete `--help`, readable 40-through-80-column semantics, no structured-schema change, no real game, no trust mutation, no sensitive live capture, and no compatibility or completion claim

**Scale/Scope**: Ten audited help path pairs, nine refusal categories, one first-run journey, two launch-family examples, existing Doctor and bundle recovery wording, and focused semantic tests

## Constitution Check

*GATE: Passed before Phase 0 research and rechecked after Phase 1 design.*

- **P-1 and authorized use**: S148 changes help, human guidance, and controlled tests only. It acquires no process, proxy, network, capture, certificate, trust, or cleanup authority.
- **P-2 and P-3**: The registry belongs to the CLI presentation layer. Core, packet capture, attribution, and native proxy crate boundaries remain unchanged.
- **P-4 and P-9**: Refusal reasons preserve exact observed state, and unknown, stale, partial, negative, conflicting, or interrupted evidence never becomes a success claim.
- **P-5**: Capture files, bundle schemas, structured records, and analyzer compatibility remain unchanged.
- **P-6**: Workflow stage, command example, first-run refusal, calibration outcome, sensitive artifact, and recovery authority retain distinct definitions.
- **P-7**: Clap remains the only parser and renderer. Tests walk the command tree and validate the shared registry rather than adding a wrapper parser.
- **P-8**: Semantic tests lead the user-visible changes, Markdown follows the one-logical-line rule, and the complete repository gate remains mandatory.
- **P-10**: Target discovery, registration, selection, and storage continue through the existing single target path.
- **P-11**: The master specification, outline, plans, changelog, and agent summary will record the embedded-help boundary without claiming general Deep Capture completion.

## Project Structure

### Documentation

```text
specs/148-deep-capture-help/
├── checklists/
│   ├── requirements.md
│   ├── security.md
│   └── ux.md
├── contracts/
│   └── embedded-workflow-help.md
├── data-model.md
├── plan.md
├── quickstart.md
├── research.md
├── spec.md
└── tasks.md

docs/fragcap-specification.md
docs/fragcap-spec-outline.md
docs/plans/README.md
```

### Source Code

```text
crates/fragcap-cli/src/workflow_help.rs
crates/fragcap-cli/src/cli.rs
crates/fragcap-cli/src/lib.rs
crates/fragcap-cli/src/commands/deep_capture.rs
crates/fragcap-cli/src/commands/calibrate.rs
crates/fragcap-cli/src/commands/doctor.rs
crates/fragcap-cli/src/commands/bundle.rs
crates/fragcap-cli/tests/cli_help.rs
crates/fragcap-cli/tests/cli_deep_capture.rs
crates/fragcap-cli/tests/cli_calibrate.rs
crates/fragcap-cli/tests/cli_doctor.rs
crates/fragcap-cli/tests/cli_bundle.rs
AGENTS.md
changelog.d/
```

**Structure Decision**: One CLI-owned registry supplies workflow vocabulary, display examples, parse arguments, and refusal actions. Clap owns rendering and grammar; command modules use the registry only to append human next-step guidance, and integration tests validate the production surface.

## Complexity Tracking

No constitutional violation requires justification.
