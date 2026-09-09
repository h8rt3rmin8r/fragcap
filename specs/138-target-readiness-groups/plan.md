# Implementation Plan: Target Readiness Groups

**Branch**: `codex/s138-target-readiness-groups` | **Date**: 2026-09-09 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/138-target-readiness-groups/spec.md`

## Summary

Close issue #376 by deriving one final ready-then-setup target order before presentation, rendering each non-empty readiness partition under a fixed plain-language heading with independent table widths, and using that same order for continuous row numbers, the persisted listing snapshot, and footer selection. Existing target entries, readiness derivation, discovery, evidence cells, and machine-readable export remain unchanged.

## Technical Context

**Language/Version**: Rust 2021, workspace MSRV 1.88

**Primary Dependencies**: Existing `fragcap-cli` and facade target APIs; no new dependency or lockfile package

**Storage**: Existing `local.db` listing snapshot only; no schema migration or target-entry rewrite

**Testing**: Rust unit and CLI integration tests, focused target-list tests, full workspace and repository gates

**Target Platform**: Human CLI output on Windows 10 and 11 with portable deterministic tests

**Project Type**: Rust workspace CLI

**Performance Goals**: Linear partitioning and width calculation after the existing target load, with no additional store query

**Constraints**: Ready group first, deterministic handle order within groups, continuous global numbering, per-group widths, no hidden or altered evidence, snapshot parity, ready-first footer, stable export and identity

**Scale/Scope**: One hero listing render path, focused empty and readiness-shape coverage, architecture and user documentation synchronization

## Constitution Check

*GATE: Passed before Phase 0 research and re-checked after Phase 1 design.*

- **P-1 No covert target instrumentation**: Pass. S138 changes only local presentation and snapshot ordering.
- **P-2 Core stays platform-neutral**: Pass. The change remains inside `fragcap-cli` and documentation.
- **P-3 Capture and attribution stay separate**: Pass. Neither boundary changes.
- **P-4 No silent loss**: Pass. Every target and evidence cell remains visible, and independent width calculation never truncates or wraps a value.
- **P-5 Compatibility outranks richness**: Pass. Machine-readable export and target identity remain byte-compatible; only human presentation changes.
- **P-6 Glossary first**: Pass. Existing hero listing, listing snapshot, and capture readiness entries are updated before the new grouped behavior is described elsewhere.
- **P-7 Wrappers stay thin**: Pass. No wrapper changes or parsing are introduced.
- **P-8 House standards apply**: Pass. UTF-8 without BOM, LF, format, lint, tests, docs, and repository checks remain blocking.
- **P-9 The instrument does not lie**: Pass. Grouping uses the existing readiness authority and changes neither classification nor evidence.
- **P-10 One path to a target**: Pass. Target creation, storage, and resolution remain unchanged; one final presentation order feeds both listing and snapshot.
- **P-11 The specification describes what shipped**: Pass. The master specification, outline, roadmap, glossary, public examples, and changelog move with the behavior.

Post-design re-check: passed. The design partitions the already filtered target vector once, retains all entries, and makes rendering consume slices of that final vector. Numeric selection and the footer therefore cannot acquire a second competing order. No constitution exception is introduced.

## Project Structure

### Documentation (this feature)

```text
specs/138-target-readiness-groups/
|-- spec.md
|-- plan.md
|-- research.md
|-- data-model.md
|-- quickstart.md
|-- contracts/
|   `-- hero-listing.md
|-- checklists/
|   |-- requirements.md
|   `-- ux.md
`-- tasks.md
```

### Source Code (repository root)

```text
crates/fragcap-cli/src/commands/
`-- targets.rs                    # final ordering, grouped rendering, snapshot, footer

crates/fragcap-cli/tests/
`-- cli_targets.rs                # empty, all-ready, all-setup, mixed, widths, selectors

docs/
|-- fragcap-specification.md
|-- fragcap-spec-outline.md
|-- glossary/command-line-and-diagnostics.md
`-- plans/README.md

site/content/docs/
`-- getting-started.mdx           # current hero-listing specimen and row rule

README.md                         # synthetic CLI specimen

changelog.d/
`-- 376-target-readiness-groups.changed.md
```

**Structure Decision**: Keep the final presentation order and its split point in the existing hero-listing function, then pass each contiguous group slice and its global starting row into one table renderer. This reuses current evidence-cell authorities, prevents snapshot drift, and lets every group measure only its own rows.

## Implementation Sequence

1. Add failing focused tests for exact headings, empty-group omission, ready-first grouping, handle ordering, continuous numbering, independent widths, snapshot resolution, and footer selection.
2. Derive one final ready-then-setup order from the existing filtered entries and retain the ready-group boundary.
3. Render each non-empty group with a fixed heading, one shared table renderer, independent widths, and global row offsets.
4. Write the listing snapshot from the final order and restrict next-command candidate selection to the ready partition whenever it is non-empty.
5. Reconcile the master specification, outline, roadmap, glossary, README, getting-started example, and changelog fragment.
6. Run spec analysis, focused tests, full CI parity, dependency stability, encoding and mojibake checks, and final diff review.

## Complexity Tracking

No constitution violations require justification.
