# Implementation Plan: Target-Hint-Record Schema Revision

**Branch**: `feat/hint-record-schema` | **Date**: 2026-08-13 | **Spec**:
[spec.md](spec.md)

**Input**: Feature specification from `specs/033-hint-record-schema/spec.md`

## Summary

Extend the master target schema's loose subschema so a hint record (and each
record in an export envelope) can carry three optional structures the targets
hint database (#78) will emit: a `launch` array of Steam launch entries, a
`launcher_mediated` boolean, and an `engine` object with a name, a source enum,
and a confidence enum. The change is additive and backward compatible within
schema version 1 (no bump), applied identically to the embedded and published
schema copies; the strict profile and package variants and the export envelope's
top level do not carry the new fields (mirroring how `records` is gated). The
hand-rolled variant validator is extended to shape-check the new structures, two
new diagnostic codes name a bad engine source or confidence, conformance fixtures
cover the valid and rejected cases, and glossary and specification entries are
written. No SQLite, no seeding, no fetching (all #78); no new dependency.

## Technical Context

**Language/Version**: Rust, edition 2021, MSRV 1.82.

**Primary Dependencies**: none new. Reuses `serde_json` (already a
`fragcap-profile` runtime dependency) for the validator; the schema is a static
JSON asset embedded via `include_str!`.

**Storage**: two JSON schema assets (embedded + published), kept byte-identical.

**Testing**: `cargo test --workspace --locked`, primarily the conformance corpus
in `crates/fragcap-profile/tests/schema_conformance.rs` over committed JSON
fixtures, plus the embedded/published drift test.

**Target Platform**: platform-neutral (schema + validator).

**Project Type**: schema/library.

**Performance Goals**: none; validation is over small documents.

**Constraints**: additive within v1 (no version bump), both copies
byte-identical, closed property sets preserved, engine confidence not a fidelity
tier (P-9), strict variants unchanged, no new dependency, MSRV 1.82 green,
UTF-8/LF/no em or en dashes.

**Scale/Scope**: three new optional properties, two new `$defs`, two new
diagnostic codes, ~5 new fixtures, glossary + spec.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P-1 Passive Observation Only**: PASS. Schema and validator only; nothing
  observes a process, opens a handle, or touches the denylist.
- **P-2 Core Stays Platform-Neutral**: PASS. Changes are in `fragcap-profile`;
  `fragcap-core`'s allowlist is untouched.
- **P-3 Capture And Attribution Stay Separate**: PASS. No packet source or
  attributor is touched.
- **P-4 No Silent Loss**: PASS. Every new validation path reports a named
  diagnostic; nothing is silently accepted or dropped.
- **P-5 Compatibility Outranks Richness**: PASS. The change is additive and
  backward compatible; every pre-existing artifact still validates, and no
  version bump is made. The two schema copies stay byte-identical.
- **P-6 Glossary First**: PASS (planned). New terms gain glossary entries and the
  specification documents the revised subschema in the same change.
- **P-7 Wrappers Stay Thin**: PASS. No wrapper logic.
- **P-8 House Standards Apply**: PASS (planned). SPDX headers where source
  changes; UTF-8/LF/no dashes on JSON and prose.
- **P-9 The Instrument Does Not Lie (NON-NEGOTIABLE)**: PASS. The engine
  `confidence` is a within-record grading of one heuristic field, kept separate
  from the record `fidelity` so a guess cannot silently move overall trust; the
  launch array is never flattened to a single "the game binary" at seeding time;
  a failed engine lookup leaves the object absent rather than present-but-lying.

**Result**: All gates pass. No Complexity Tracking entries.

## Project Structure

### Documentation (this feature)

```text
specs/033-hint-record-schema/
├── plan.md, research.md, data-model.md, quickstart.md
├── contracts/hint-record-schema.md
├── checklists/{requirements.md, schema-and-honesty.md}
└── tasks.md
```

### Source Code (repository root)

```text
crates/fragcap-profile/
├── assets/target-schema.v1.json        # + launch/launcher_mediated/engine props,
│                                        #   $defs/launch_entry, $defs/engine, allOf gate
├── src/jsonschema/
│   ├── diagnostic.rs                    # + InvalidEngineSource, InvalidEngineConfidence
│   └── variants.rs                      # per-variant allowed keys + shape checks
├── src/parse.rs                         # map the two new SchemaCodes
└── tests/
    ├── schema_conformance.rs            # + fixtures in the corpus
    └── fixtures/schema/                 # + hint-loose-valid, engine-bad-source,
                                         #   engine-bad-confidence, launch-no-exe,
                                         #   profile-with-launch (rejected)

docs/schema/target-schema.v1.json        # byte-identical mirror of the embedded copy
docs/glossary/*.md                        # + launch array/entry, launcher-mediated,
                                          #   engine attribution; index regenerated
docs/fragcap-specification.md             # document the revised hint-record subschema
changelog.d/033-hint-record-schema.{added,decisions}.md
```

**Structure Decision**: Everything lives in `fragcap-profile` and the docs, the
same crate S025 and S031 extended. No new crate, no cross-crate change; the schema
is the shared vocabulary and the validator is bound to it by the conformance
corpus.

## Key Design Decisions

1. **The three fields live where a single loose record lives.** In the JSON
   Schema they are top-level `properties` (so they can appear on the hint variant)
   plus members of `$defs/record` (so they can appear inside each export record),
   and an `allOf` conditional forbids them on `profile`, `package`, and the
   `export` envelope top level (`properties: { launch: false, launcher_mediated:
   false, engine: false }`), mirroring exactly how `records: false` gates the
   records array off the non-export kinds. Net effect: valid on a hint top level
   and inside export records, rejected everywhere else.

2. **`$defs/launch_entry`**: object with optional free-string filters (`os`,
   `osarch`, `launch_type`, `beta_branch`), a required non-empty `executable`, and
   optional `arguments` and `description`, `additionalProperties: false`. The
   filters are free strings, not enums, because Steam's launch vocabularies evolve
   externally and an enum would reject valid new values (spec clarification).
   `launch` is an array of these with no `minItems` (an empty array is valid).

3. **`$defs/engine`**: object with an optional `name` (string), a required
   `source` enum (`pcgamingwiki`, `exe_heuristic`, `depot_filename_rules`), and a
   required `confidence` enum (`confirmed`, `high`, `medium`, `low`, `unknown`),
   `additionalProperties: false`. The record `fidelity` `$def` and enum are not
   touched; the engine `confidence` is a separate field, and the engine `source`
   is separate from the record's provenance `source`. This is the P-9
   reconciliation: a within-field grading, never a new fidelity rung.

4. **The hand-rolled validator gates by kind.** `allowed_top_keys` already keys on
   the `Kind` enum (`Strict`, `Hint`, `Export`); the `Hint` arm gains `launch`,
   `launcher_mediated`, `engine`, while `Strict` and `Export` do not (so a strict
   variant carrying one is an unknown key, satisfying the boundary), and
   `check_records`'s allowed set gains the three. New `check_launch`,
   `check_launch_entry`, and `check_engine` helpers shape-check wherever the
   fields are permitted (the hint top level via `check`, and each export record
   via `check_records`), and a `launcher_mediated` boolean check is inline. Two
   new diagnostic codes, `InvalidEngineSource` and `InvalidEngineConfidence`, name
   an out-of-enum value (mirroring `InvalidCategory` from S031); a launch entry
   missing `executable` reuses `MissingField`, an empty one `EmptyString`, an
   unknown key `UnknownKey`. `parse.rs` maps the two new codes to
   `DiagnosticCode::WrongType`, as `InvalidCategory` is mapped.

5. **Additive, no version bump, both copies.** Every addition is an optional
   property; prior artifacts validate unchanged, so schema version stays 1. The
   edit is applied byte-identically to the embedded `assets` copy and the
   published `docs/schema` copy, and the existing drift test enforces that. This
   is the same discipline S031's `technologies` extension followed.

6. **Fixtures extend the corpus, not a new harness.** New committed JSON fixtures
   are added to the `schema_conformance` corpus with declared expected outcomes;
   the pre-existing fixtures keep theirs, proving backward compatibility.

## Complexity Tracking

No constitution violations. No entries.
