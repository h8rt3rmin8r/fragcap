---
description: "Task list for S033 target-hint-record schema revision"
---

# Tasks: Target-Hint-Record Schema Revision

**Input**: Design documents from `specs/033-hint-record-schema/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Included. The conformance corpus is the test surface; fixtures are
written alongside the schema/validator changes (TDD per the constitution).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Parallelizable (different files, no dependency on incomplete tasks)
- **[Story]**: US1..US3 map to the spec's user stories
- Paths are repository-relative.

## Phase 1: Setup

- [X] T001 Confirm the branch builds and the conformance corpus is green before
  editing (`cargo test -p fragcap-profile --test schema_conformance`).

## Phase 2: Foundational (blocking prerequisites for all stories)

- [X] T002 Extend the embedded schema `crates/fragcap-profile/assets/target-schema.v1.json`: add `$defs/launch_entry` (free-string filters `os`/`osarch`/`launch_type`/`beta_branch`, required non-empty `executable`, optional `arguments`/`description`, `additionalProperties:false`) and `$defs/engine` (optional `name`, required `source` enum, required `confidence` enum, `additionalProperties:false`) per `contracts/hint-record-schema.md`.
- [X] T003 In the same embedded schema, add top-level optional `launch` (array of `launch_entry`), `launcher_mediated` (boolean), and `engine` (`$ref`); add the three to `$defs/record.properties`; and add the `allOf` gate forbidding them on `profile`/`package`/`export` top level.
- [X] T004 Mirror the identical change to the published copy `docs/schema/target-schema.v1.json` (byte-identical, so the drift check stays green).
- [X] T005 Add `InvalidEngineSource` and `InvalidEngineConfidence` to `SchemaCode` in `crates/fragcap-profile/src/jsonschema/diagnostic.rs` (enum + `as_str`), and map both to `DiagnosticCode::WrongType` in `crates/fragcap-profile/src/parse.rs`.

## Phase 3: User Story 1 - A hint carries the full launch array, flag, and engine (P1)

**Goal**: the loose subschema validates a full hint (and export records) carrying
the three new structures.

**Independent test**: `hint-loose-valid.json` validates with no diagnostics.

- [X] T006 [US1] Extend the hand-rolled validator `crates/fragcap-profile/src/jsonschema/variants.rs`: add `launch`/`launcher_mediated`/`engine` to the `Hint` arm of `allowed_top_keys` (not `Strict`/`Export`), and to the allowed-key set in `check_records`; add `TECHNOLOGY`-style enum consts for engine source and confidence.
- [X] T007 [US1] Implement `check_launch`, `check_launch_entry`, and `check_engine` in `variants.rs` (shape-check per data-model.md), plus an inline `launcher_mediated` boolean check; call them at the hint top level in `check` and for each record in `check_records`.
- [X] T008 [P] [US1] Add fixture `crates/fragcap-profile/tests/fixtures/schema/hint-loose-valid.json` (hint with a multi-entry launch array with filters + required executables, `launcher_mediated: true`, a valid engine object) and assert it `Valid` in `schema_conformance.rs`.
- [X] T009 [P] [US1] Add an export-envelope fixture whose record carries the three fields (or extend the existing export fixture) and assert it `Valid`, proving the fields work inside export records.

## Phase 4: User Story 2 - Vocabularies reconciled and honest (P1)

**Goal**: bad engine enums and a missing executable are rejected; engine
confidence and record fidelity are independent.

**Independent test**: the rejection fixtures carry their named diagnostics.

- [X] T010 [P] [US2] Add `engine-bad-source.json` (engine `source` out of enum) asserting `Invalid(InvalidEngineSource)`, and `engine-bad-confidence.json` (engine `confidence` out of enum) asserting `Invalid(InvalidEngineConfidence)`.
- [X] T011 [P] [US2] Add `launch-no-executable.json` (a launch entry with no `executable`) asserting `Invalid(MissingField)`.
- [X] T012 [P] [US2] Add (or reuse `hint-loose-valid.json`) a case with record `fidelity: heuristic-unverified` and an independent `engine.confidence: low`, asserting `Valid`, proving the two fields do not interact (SC-005).

## Phase 5: User Story 3 - The strict authored format is unchanged (P1)

**Goal**: strict profile/package reject the new fields; every pre-existing fixture
keeps its outcome.

**Independent test**: `profile-with-launch.json` is rejected; the corpus is
otherwise unchanged.

- [X] T013 [P] [US3] Add `profile-with-launch.json` (a strict profile carrying a `launch` array) asserting `Invalid(UnknownKey)`, proving the strict variant rejects hint-seeding fields.
- [X] T014 [US3] Run the full conformance corpus and confirm every pre-existing fixture (profile/package/hint/export and the S031 technologies fixtures) keeps its expected outcome and the embedded/published drift test passes.

## Phase 6: Polish & Cross-Cutting Concerns

- [X] T015 [P] Add glossary entries for the launch array/entry, launcher-mediated, and engine attribution (source + confidence) under `docs/glossary/`, note the confidence-vs-fidelity distinction, and regenerate the index (`bash scripts/lint-docs.sh fix`); run `bash scripts/lint-docs.sh check` (P-6).
- [X] T016 [P] Document the revised hint-record subschema and the vocabulary reconciliation in `docs/fragcap-specification.md` (the schema/section that describes the master schema variants), FR-010.
- [X] T017 Add changelog fragments `changelog.d/033-hint-record-schema.added.md` and `changelog.d/033-hint-record-schema.decisions.md` recording the additive extension, the loose-only gating, the engine-confidence-is-not-a-fidelity-tier reconciliation, the free-string filters, and the two new diagnostic codes.
- [X] T018 Run `cargo xtask ci` and `cargo xtask msrv` in the foreground and resolve any findings (fmt, clippy, test, lint, deps, license, docs, MSRV).

## Dependencies & Execution Order

- **Phase 1 -> Phase 2 -> Phases 3-5 -> Phase 6.**
- Foundational (Phase 2) blocks all stories: the schema (T002-T004) and the two
  diagnostic codes (T005) are prerequisites for the validator and fixtures.
- US1 (Phase 3): T006 before T007; T007 before the fixtures assert cleanly.
- US2/US3 fixtures (Phase 4/5) depend on the validator (T007) and codes (T005);
  T010-T013 are [P] with each other (distinct fixture files).
- Polish (Phase 6) after the stories; T018 is the final gate.

## Parallel Opportunities

- Fixture tasks T008/T009/T010/T011/T012/T013 are [P] (distinct files), gated only
  on the validator (T007) and codes (T005).
- Doc tasks T015/T016 are [P].

## Implementation Strategy

- **MVP = User Story 1** (Foundational + Phase 3): the loose subschema validates a
  full hint carrying the launch array, launcher flag, and engine object. That is
  the shape #78 needs to emit.
- Layer US2 (the honesty rejections and vocabulary independence), then US3 (the
  strict boundary and backward compatibility), then Polish and the full gate.
