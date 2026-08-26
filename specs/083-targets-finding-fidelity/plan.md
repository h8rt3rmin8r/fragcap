# Implementation Plan: Targets Finding Fidelity

**Branch**: `codex/083-targets-finding-fidelity` | **Date**: 2026-08-26 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/083-targets-finding-fidelity/spec.md`

## Summary

Fix issue #211 by carrying technology finding fidelity into the derived target listing summaries. The human table will suffix below-verified technology products with `?`; verified-or-stronger findings stay unmarked; duplicate findings for one product collapse to the strongest fidelity. The raw target-entry export/import evidence payload already carries fidelity, so this slice guards that contract with tests and updates the master CLI specification to name the marker.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.82

**Primary Dependencies**: Existing workspace crates only (`fragcap-targets`, `fragcap-cli`, `fragcap-profile`, `serde_json` already in graph)

**Storage**: Existing targets SQLite schema and target-entry JSON export, no migration

**Testing**: `cargo test`, targeted crate tests, CLI integration tests, `cargo xtask ci`

**Target Platform**: Windows CLI first, with affected derivation logic platform-neutral inside `fragcap-targets`

**Project Type**: Rust workspace CLI and libraries

**Performance Goals**: No new filesystem, database, network, capture, or process access during listing projection

**Constraints**: Preserve no-truncation table rule, preserve category partition, preserve coverage markers, no new dependencies, no process instrumentation

**Scale/Scope**: One presentation derivation module, one CLI table surface, one JSON export/import guard, one master-spec revision

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P-1 No Covert Target Instrumentation**: Pass. This slice only derives target listing text from stored evidence and touches no capture, proxy, process, ETW, socket table, or trust behavior.
- **P-2 Core Stays Platform-Neutral**: Pass. No `fragcap-core` changes.
- **P-3 Capture And Attribution Stay Separate**: Pass. No capture or attribution changes.
- **P-4 No Silent Loss**: Pass. The slice removes a silent loss of fidelity from the listing summary.
- **P-5 Compatibility Outranks Richness**: Pass. No capture output format changes.
- **P-6 Glossary First**: Pass. No new domain term is introduced; "fidelity" and target concepts already exist.
- **P-7 Wrappers Stay Thin**: Pass. No wrapper changes.
- **P-8 House Standards Apply**: Pass. All generated and edited text must satisfy repository lint, including no em or en dashes.
- **P-9 The Instrument Does Not Lie**: Pass. The change prevents heuristic findings from being presented as verified facts.
- **P-10 One Path To A Target**: Pass. The same stored evidence continues to feed all target sources and exports.
- **P-11 The Specification Describes What Shipped**: Pass. The master specification will be updated with the shipped marker contract.

## Project Structure

### Documentation (this feature)

```text
specs/083-targets-finding-fidelity/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── contracts/
│   └── targets-finding-fidelity.md
├── quickstart.md
├── checklists/
│   ├── requirements.md
│   └── fidelity.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/fragcap-targets/src/readiness.rs
crates/fragcap-targets/src/targets_export.rs
crates/fragcap-cli/tests/cli_targets.rs
docs/fragcap-specification.md
changelog.d/
```

**Structure Decision**: Keep the fidelity projection in `fragcap-targets::readiness`, where the listing already derives ENGINE and SENSITIVITIES cells, and keep CLI tests at the command boundary to prove the actual table and export behavior.

## Complexity Tracking

No constitution violations or complexity exceptions are needed.

## Phase 0: Research

See [research.md](research.md).

## Phase 1: Design

See [data-model.md](data-model.md), [contracts/targets-finding-fidelity.md](contracts/targets-finding-fidelity.md), and [quickstart.md](quickstart.md).

## Post-Design Constitution Check

The design still passes all constitution checks. The only product decision is a presentation marker. It is intentionally low-richness, plain text, and derived from already stored evidence.
