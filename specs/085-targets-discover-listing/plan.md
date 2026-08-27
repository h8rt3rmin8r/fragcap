# Implementation Plan: Targets Discover Listing

**Branch**: `codex/085-targets-discover-listing` | **Date**: 2026-08-27 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/085-targets-discover-listing/spec.md`

## Summary

Fix issue #207 by replacing the raw `targets discover` tab dump with the same human-listing discipline used by the hero `targets` command: labelled store paths, a headed aligned candidate table, indented evidence, and a readable discovery account block. The existing discovery data model and warning emitter stay unchanged, and no machine-readable discovery contract is added in this slice.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.82

**Primary Dependencies**: Existing workspace crates only (`fragcap-cli`, `fragcap-targets`, `fragcap-profile`)

**Storage**: No storage changes

**Testing**: `cargo test`, targeted CLI tests, command unit tests, `cargo xtask ci`

**Target Platform**: Windows CLI first, with rendering logic platform-neutral

**Project Type**: Rust workspace CLI and libraries

**Performance Goals**: No additional discovery, catalog, filesystem, process, proxy, or network work; rendering remains linear over already collected candidates and evidence

**Constraints**: Preserve read-only discovery semantics, warnings on the diagnostics stream, no output truncation, no new format flag, no new dependency

**Scale/Scope**: One CLI renderer, fixture-backed command tests, one master-spec revision

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P-1 No Covert Target Instrumentation**: Pass. This slice changes only CLI rendering and adds no capture, proxy, process, ETW, socket table, trust, or network behavior.
- **P-2 Core Stays Platform-Neutral**: Pass. No `fragcap-core` changes.
- **P-3 Capture And Attribution Stay Separate**: Pass. No capture or attribution changes.
- **P-4 No Silent Loss**: Pass. The discovery account becomes more readable and warnings stay visible.
- **P-5 Compatibility Outranks Richness**: Pass. No capture output format changes.
- **P-6 Glossary First**: Pass. No new domain term is introduced; target, discovery, evidence, fidelity, and account vocabulary already exist.
- **P-7 Wrappers Stay Thin**: Pass. No wrapper changes.
- **P-8 House Standards Apply**: Pass. All generated and edited text must satisfy repository lint, including no em or en dashes.
- **P-9 The Instrument Does Not Lie**: Pass. The renderer preserves observed values and removes a misleading low-value classification column from the human table without altering the data.
- **P-10 One Path To A Target**: Pass. Discovery data and registration paths stay unchanged.
- **P-11 The Specification Describes What Shipped**: Pass. The master specification will be updated with the shipped human rendering contract.

## Project Structure

### Documentation (this feature)

```text
specs/085-targets-discover-listing/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── discover-listing.md
├── checklists/
│   ├── requirements.md
│   └── rendering.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/fragcap-cli/src/commands/targets.rs
crates/fragcap-cli/tests/cli_targets.rs
docs/fragcap-specification.md
changelog.d/
```

**Structure Decision**: Keep the change in the CLI target command renderer because the discovery model already carries every value required by the listing and this is a human presentation defect. Reuse the existing `width_of` helper for computed columns.

## Complexity Tracking

No constitution violations or complexity exceptions are needed.

## Phase 0: Research

See [research.md](research.md).

## Phase 1: Design

See [data-model.md](data-model.md), [contracts/discover-listing.md](contracts/discover-listing.md), and [quickstart.md](quickstart.md).

## Post-Design Constitution Check

The design still passes all constitution checks. It changes only human rendering, preserves all observed discovery fields in the internal model, and improves P-4 readability of the account without changing discovery semantics.
