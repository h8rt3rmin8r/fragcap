# Implementation Plan: Deep Capture compatibility facts

**Branch**: `072-deep-capture-compatibility-facts` | **Date**: 2026-08-25 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/072-deep-capture-compatibility-facts/spec.md`

## Summary

Issue #217 needs durable local storage for Deep Capture compatibility facts without implementing Deep Capture itself. The implementation adds a `fragcap-targets` compatibility model, a v9 targets-store table keyed to `targets(id)`, an additive v8-to-v9 migration, insert/read APIs, and tests for round-trip fidelity, invalid values, migration behavior, and cascade cleanup. The store records proxy backend provenance and final-owner details as structured fields, because review of the initial PR correctly identified those as observation-defining context rather than optional prose.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.82.

**Primary Dependencies**: none added. Reuses `rusqlite`, already present in `fragcap-targets` behind the existing `targets` feature.

**Storage**: SQLite targets database. Schema version advances from 8 to 9. The new table is additive and keyed to `targets(id)`.

**Testing**: Unit tests in `fragcap-targets` plus full workspace and repository gates.

**Target Platform**: Platform-neutral store/model code. No Windows API, process handle, proxy, or capture driver interaction.

**Project Type**: Rust Cargo workspace, existing crate layout unchanged.

**Performance Goals**: Compatibility facts are sparse local metadata. Insert/read operations are ordinary indexed target-store operations; no capture-path code is touched.

**Constraints**: No new dependency; no CLI/export surface in this slice; no committed PII or real local title names from fact-finding.

## Constitution Check

- **P-1 (Passive Observation Only)**: PASS. This slice stores local observations and opens no process handle, injects nothing, reads no memory, and changes no traffic.
- **P-2 (Core Stays Platform-Neutral)**: PASS. All code lands in `fragcap-targets`; `fragcap-core` is untouched.
- **P-3 (Capture And Attribution Stay Separate)**: PASS. No capture source or attribution backend changes.
- **P-4 (No Silent Loss)**: PASS. The feature preserves observed compatibility context and explicitly records stale/unknown states rather than discarding uncertainty.
- **P-5 (Compatibility Outranks Richness)**: PASS. The public pcapng/JSON Lines capture formats are untouched.
- **P-6 (Glossary First)**: PASS. No user-facing term requiring a glossary entry is introduced by this storage-only slice.
- **P-8 (House Standards Apply)**: PASS, gated by `cargo xtask lint`.
- **P-9 (The Instrument Does Not Lie)**: PASS. The migration invents no facts, unknown is explicit, and invalid tokens are rejected.
- **P-10 / P-11**: PASS. Target identity remains in the target store and schema impact is recorded in the spec/changelog.
- **Licensing**: PASS. No new dependency.

No violation requires Complexity Tracking.

## Project Structure

```text
specs/072-deep-capture-compatibility-facts/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── analysis.md
├── quickstart.md
├── contracts/
│   └── compatibility-facts-schema.md
├── checklists/
│   └── requirements.md
└── tasks.md
```

```text
crates/fragcap-targets/src/
├── compatibility.rs   # new value types and key/value validation
├── lib.rs             # re-export compatibility types
├── schema.rs          # schema v9 and v8-to-v9 migration
└── store.rs           # insert/read APIs and migration tests
```

## Complexity Tracking

No constitution violation requires justification; this section is empty by design.
