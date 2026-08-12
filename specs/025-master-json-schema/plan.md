# Implementation Plan: Master JSON Schema for Targeting and Attribution

**Branch**: `feat/master-json-schema` | **Date**: 2026-08-12 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/025-master-json-schema/spec.md`

## Summary

Deliver one versioned master JSON Schema (Draft 2020-12) that governs every
machine-readable targeting/attribution artifact, discriminated by an explicit
top-level `kind` field into four variants (profile, hint, package, export) over a
shared core. The schema is authored as a standard JSON Schema document, embedded
in the binary as the single source of truth, published in the repository, and
rendered on the docs site. A new `fragcap schema` command surface validates any
JSON file against it (`validate`, reporting every structural violation at once)
and emits it (`print`).

The decisive design choice, resolved in Phase 0 research: fragcap **publishes**
the standard schema for the ecosystem (editors, agents, the future submission
pipeline validate against it natively) but validates **internally by hand** over
`serde_json::Value`, rather than embedding a JSON Schema validator crate. A
validator crate (`boon`) was evaluated and rejected because it adds 42 transitive
crates (the entire ICU4X stack via `url`/`idna`) for `format`/`$ref` machinery
this schema does not need, which is irreconcilable with the project's dependency
discipline. The only new runtime dependency is `serde_json`, promoted from its
existing dev-only status. A conformance test binds the published schema and the
hand-rolled validator so they cannot drift.

This slice delivers the schema, the validation surface, and the shared-core
vocabulary. It does not migrate the profile parser (that is #76), does not build
the resolver that reads `fidelity` (#77), and does not build the hint database
(#78). Profile semantic validation (acyclic ancestry, single terminal stage, role
reachability, ambiguous image match) stays where section 15.4 puts it and is
rewired onto JSON by #76; this slice defines the structural layer beneath it and
declares the seam.

## Technical Context

**Language/Version**: Rust, workspace edition, MSRV 1.82 (toolchain pinned 1.96)

**Primary Dependencies**: `serde` + `serde_json` (serde_json promoted dev ->
runtime in `fragcap-profile`); existing `regex` (already runtime) for pattern
predicates. No JSON Schema validator crate (see research.md).

**Storage**: JSON files on disk (profiles, packages, hint exports); the embedded
schema is a compile-time-included static asset.

**Testing**: `cargo test` (unit + a fixture-corpus conformance test binding the
published schema to the hand-rolled validator), `cargo xtask ci`,
`cargo xtask msrv` at 1.82, `cargo xtask deps`, `cargo xtask license`.

**Target Platform**: Windows primary; the schema and validator are
platform-neutral and build on any target (they live in `fragcap-profile`, not a
platform crate).

**Project Type**: Rust workspace (library crates + CLI).

**Performance Goals**: Validation of a single target file is interactive
(sub-second); not a hot path. All-errors-at-once accumulation over one parsed
`Value` is linear in document size.

**Constraints**: MSRV 1.82 must stay green; UTF-8 without BOM, LF, no em/en
dashes anywhere including schema `description` strings; unknown keys refused;
every structural error reported in one pass.

**Scale/Scope**: Four artifact variants over one shared core; a handful of
top-level entities; one new CLI subcommand group with two subcommands.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P-1 Passive Observation**: Not engaged. This slice reads and validates JSON
  files; it opens no process, no handle, no capture. PASS.
- **P-2 Core Stays Platform-Neutral**: The schema and validation live in
  `fragcap-profile`, not `fragcap-core`; the new `serde_json` runtime dependency
  is added to `fragcap-profile` only. `fragcap-core` gains no dependency. PASS.
- **P-3 Capture And Attribution Stay Separate**: Not engaged; no capture or
  attribution code changes. PASS.
- **P-4 No Silent Loss**: Directly served. Validation accumulates and reports
  every structural violation rather than stopping at the first, mirroring the
  section 15.4 discipline; a hint that omits its fidelity is refused rather than
  silently accepted. PASS.
- **P-5 Compatibility Outranks Richness**: The published schema is a standard
  Draft 2020-12 document any external validator reads; internal validation adds
  no incompatible extension. PASS.
- **P-6 Glossary First**: New terms (fidelity tier names, provenance, the four
  `kind` variants) get glossary entries in this change. PLANNED (tasks include
  the glossary entries).
- **P-7 Wrappers Stay Thin**: The `fragcap schema` CLI subcommands are thin over
  library functions in `fragcap-profile`; no output parsing. PASS.
- **P-8 House Standards Apply**: The one new runtime dependency (`serde_json`,
  promoted from dev) carries a dependency-inventory justification; UTF-8/LF/no
  dashes enforced; `cargo xtask ci` is the gate. PLANNED (inventory update is a
  task).
- **P-9 The Instrument Does Not Lie**: Central. Fidelity is a required structured
  field the resolver will read; a guess is stamped as a guess. The published
  schema and the enforcing validator are bound by a conformance test so the
  contract the tool advertises is the contract it enforces. PASS.

No gate violations. The single dependency-discipline risk (a heavy validator
crate) is resolved in research by not taking one.

## Project Structure

### Documentation (this feature)

```text
specs/025-master-json-schema/
├── plan.md              # This file
├── research.md          # Phase 0: validator-crate vs hand-roll decision, dialect, discriminator, MSRV
├── data-model.md        # Phase 1: shared core, kind variants, fidelity, provenance
├── quickstart.md        # Phase 1: runnable validation scenarios
├── contracts/
│   ├── cli-schema-command.md    # `fragcap schema validate|print` contract
│   └── master-schema.contract.md# the schema document's shape and variant rules
└── checklists/
    └── requirements.md  # spec quality checklist (from /speckit-specify)
```

### Source Code (repository root)

```text
crates/fragcap-profile/
├── Cargo.toml                 # + serde_json promoted to runtime dependency
├── assets/
│   └── target-schema.v1.json  # the master JSON Schema (Draft 2020-12), single source of truth, embedded
└── src/
    ├── schema.rs              # existing profile types (unchanged by this slice; #76 migrates parsing)
    ├── jsonschema/            # NEW: hand-rolled structural validation surface
    │   ├── mod.rs             # public entry: validate_value(kind-aware) -> Vec<Diagnostic>
    │   ├── document.rs        # embedded schema access + `print`
    │   ├── diagnostic.rs      # Diagnostic { json_pointer, message } accumulation
    │   └── variants.rs        # kind discriminator + per-variant required/optional rules
    └── lib.rs                 # re-export the schema validation surface

crates/fragcap-cli/
└── src/commands/
    └── schema.rs              # NEW: `fragcap schema validate <file>` and `schema print`

docs/
├── fragcap-specification.md   # section 15 reconciled to JSON + generalized beyond the profile
├── glossary/                  # + fidelity tiers, provenance, kind-variant terms
└── (docs site)               # field-level schema reference page

changelog.d/
├── 025-master-json-schema.md              # feature fragment
└── 025-master-json-schema.decisions.md    # dated decision: hand-roll + publish, serde_json promotion
```

**Structure decision**: The schema asset and its hand-rolled validator live in
`fragcap-profile` (which already owns profile schema and validation), keeping
`fragcap-core` dependency-free (P-2). The CLI subcommands live in `fragcap-cli`
as thin wrappers (P-7). No new crate is introduced.

## Complexity Tracking

No constitution gate is violated, so no complexity justification is required. The
one judgment call with lasting consequence (publish-schema + hand-roll-validation
instead of a validator crate) is recorded in research.md and as a dated changelog
decision, and is surfaced at the pre-push halt.
