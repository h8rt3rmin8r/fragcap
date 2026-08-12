# Tasks: Master JSON Schema for Targeting and Attribution

**Feature**: 025-master-json-schema | **Branch**: `feat/master-json-schema`
**Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

Test-driven: each user story writes its failing tests before implementation.
Security posture note: this slice opens no process and no capture (P-1 not
engaged); the required "safety" tests here are the honesty/no-silent-loss tests
(all-errors-at-once, refusal of a fidelity-less hint), which are mandatory and
never weakened.

## Phase 1: Setup

- [ ] T001 Promote `serde_json` from dev-dependency to runtime dependency in `crates/fragcap-profile/Cargo.toml` (keep `serde` as-is), and confirm `cargo build -p fragcap-profile` still resolves with no other new crate in `Cargo.lock`.
- [ ] T002 Update the dependency inventory table and current-state note in `AGENTS.md` to record `serde_json` moving dev -> runtime for `fragcap-profile`, with the one-line rationale (hand-rolled JSON validation; no validator crate taken).

## Phase 2: Foundational (blocking prerequisites)

- [ ] T003 Author the master schema document at `crates/fragcap-profile/assets/target-schema.v1.json` (Draft 2020-12) per `contracts/master-schema.contract.md`: top-level `schema`/`kind`/`fidelity`, `$defs` for `game`/`capture`/`stage`/`match`/`provenance`/`fidelity`, the four `kind`-discriminated variants (profile+package strict, hint+export loose requiring provenance), `additionalProperties:false` everywhere, `minProperties:1` on `match`. All `description` strings UTF-8, LF, no em/en dashes.
- [ ] T004 Create the validation module skeleton `crates/fragcap-profile/src/jsonschema/mod.rs`, `document.rs`, `diagnostic.rs`, `variants.rs`, and re-export the public surface from `crates/fragcap-profile/src/lib.rs`.
- [ ] T005 In `crates/fragcap-profile/src/jsonschema/document.rs`, embed the schema asset with `include_str!` and expose an accessor returning the exact embedded bytes; add a unit test asserting the embedded asset parses as valid JSON.
- [ ] T006 [P] Create the single authoritative fixture set under `crates/fragcap-profile/tests/fixtures/schema/` (tests and quickstart both reference this one location; no second copy): `profile-valid.json`, `profile-four-faults.json`, `hint-valid.json`, `hint-no-fidelity.json`, `hint-no-provenance.json`, `package-valid.json`, `export-valid.json`, `export-envelope.json`, `not-json.json`, `unknown-kind.json`, `unsupported-version.json`. Include a `notes` string in `profile-valid.json` so the notes field is exercised (FR-006).

## Phase 3: User Story 1 - Validate any file, every mistake at once (Priority: P1) [MVP]

**Goal**: `fragcap schema validate <file>` reports all structural violations in
one pass, exit non-zero on any, zero when clean.

**Independent test**: run the CLI against `profile-four-faults.json` and see four
located violations in one run; against `profile-valid.json` see none.

### Tests (write first, must fail)

- [ ] T007 [P] [US1] In `crates/fragcap-profile/tests/schema_validate.rs`, test that `profile-four-faults.json` yields exactly four diagnostics with distinct JSON-pointer locations, in stable document order, on repeated runs.
- [ ] T008 [P] [US1] Test that `profile-valid.json` yields zero diagnostics.
- [ ] T009 [P] [US1] Test that an unknown key anywhere yields a diagnostic naming its JSON pointer.
- [ ] T010 [P] [US1] Test that `unsupported-version.json` is refused with a diagnostic naming the supported version, and `unknown-kind.json` is refused as an undetermined variant.
- [ ] T011 [P] [US1] Test that `not-json.json` produces a syntax error distinguished from a schema violation.

### Implementation

- [ ] T012 [US1] Implement `Diagnostic { json_pointer, message }` and an accumulator in `crates/fragcap-profile/src/jsonschema/diagnostic.rs` that collects all findings (no `?` short-circuit), with deterministic ordering by pointer/document order.
- [ ] T013 [US1] Implement `kind`/`schema`-version detection and the strict-variant structural walk (types, required keys, enum ranges, `match` minProperties, unknown-key refusal) for `profile` in `crates/fragcap-profile/src/jsonschema/mod.rs` and `variants.rs`, accumulating diagnostics.
- [ ] T014 [US1] Distinguish a JSON syntax error (parse failure) from schema violations in the public entry point, returning a clearly-typed result.
- [ ] T015 [US1] Add the `schema` subcommand group and `schema validate <file>` in `crates/fragcap-cli/src/commands/schema.rs` (thin wrapper): read file, call the validator, print one line per violation with location, set exit code (0 valid, non-zero on any violation or read/parse failure). Wire it into the CLI command dispatch.
- [ ] T016 [US1] Verify US1 end to end against quickstart scenarios 1, 2, and 6; confirm output is byte-stable across runs.

**Checkpoint**: US1 is a usable MVP: any JSON target file can be validated with
all-errors-at-once feedback.

## Phase 4: User Story 2 - One vocabulary across all four artifact forms (Priority: P2)

**Goal**: profile, package, hint, and export all validate against the one schema;
a hint without fidelity or provenance is refused.

**Independent test**: validate one file of each form; confirm hint accepts fewer
required fields but rejects a missing fidelity/provenance; confirm export
round-trips.

### Tests (write first, must fail)

- [ ] T017 [P] [US2] In `crates/fragcap-profile/tests/schema_variants.rs`, test `package-valid.json` validates as the strict shape.
- [ ] T018 [P] [US2] Test `hint-valid.json` validates while omitting fields a profile requires.
- [ ] T019 [P] [US2] Test `hint-no-fidelity.json` is refused naming the missing `fidelity`, and `hint-no-provenance.json` is refused naming the missing `provenance`.
- [ ] T020 [P] [US2] Test `export-valid.json` (single) and `export-envelope.json` (array of hint records) both validate (round-trip conformance).
- [ ] T021 [P] [US2] Test that a `fidelity` value outside the closed enum is refused in every variant.

### Implementation

- [ ] T022 [US2] Implement the loose-variant rules (hint/export) in `crates/fragcap-profile/src/jsonschema/variants.rs`: relaxed required fields, mandatory `fidelity`, mandatory `provenance` with non-empty `source`.
- [ ] T023 [US2] Implement `export` envelope handling (single record and array-of-records) sharing the hint rules.
- [ ] T024 [US2] Enforce the closed `fidelity` enum across all variants at the shared-core level so a core change reaches all forms.
- [ ] T025 [US2] Verify US2 against quickstart scenarios 3 and 4.

**Checkpoint**: one schema, four forms, drift-proof by construction.

## Phase 5: User Story 3 - The schema is discoverable and authoritative (Priority: P3)

**Goal**: emit the enforced schema, publish it, render it, and bind it to the
validator so they cannot drift.

**Independent test**: `schema print` output equals the embedded asset; the
repository-published copy equals the embedded asset; the conformance corpus
passes.

### Tests (write first, must fail)

- [ ] T026 [P] [US3] In `crates/fragcap-profile/tests/schema_conformance.rs`, add the conformance corpus test: run every valid and invalid fixture through the validator and assert the expected accept/reject and variant, binding the published schema to the hand-rolled validator.
- [ ] T027 [P] [US3] Test that `schema print` emits bytes identical to `crates/fragcap-profile/assets/target-schema.v1.json`.
- [ ] T028 [P] [US3] Add a drift test asserting the embedded asset equals the repository-published copy (and, if a docs-site copy is a separate file, that too).

### Implementation

- [ ] T029 [US3] Implement `fragcap schema print` in `crates/fragcap-cli/src/commands/schema.rs` emitting the embedded schema exactly.
- [ ] T030 [US3] Publish the schema copy in the repository at its canonical published path and add the docs-site field-level reference page that includes or links the one published JSON file (the docs site does not hand-maintain a third copy; only the machine-readable copy is drift-checked in T028).
- [ ] T031 [US3] Reconcile master specification section 15 in `docs/fragcap-specification.md`: rewrite the format from TOML to JSON and generalize beyond the profile to the four artifact forms; note the structural/semantic seam.
- [ ] T032 [US3] Add glossary entries (P-6) for the new terms: the four `kind` variants, the `fidelity` tiers, and `provenance`, in `docs/glossary/`.
- [ ] T033 [US3] Verify US3 against quickstart scenario 5 and the gate.

**Checkpoint**: the schema is public, emitted, documented, and provably enforced.

## Phase 6: Polish & Cross-Cutting

- [ ] T034 Add the changelog feature fragment `changelog.d/025-master-json-schema.md` and the dated decision fragment `changelog.d/025-master-json-schema.decisions.md` recording: JSON+published-schema+hand-rolled-validation, the rejection of the `boon` validator crate (42 transitive crates), and `serde_json` promotion dev -> runtime.
- [ ] T035 [P] Text-hygiene sweep across all new and edited files (schema asset, Rust sources, docs, spec, changelog): UTF-8 without BOM, LF, no em/en dashes; run the repository doc/text checks.
- [ ] T036 Run the full gate in the foreground and make it green: `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --workspace --locked`, `cargo xtask lint`, `cargo xtask deps`, `cargo xtask license` (all via `cargo xtask ci`), plus `cargo xtask msrv` at 1.82.

## Dependencies & Execution Order

- Setup (T001-T002) -> Foundational (T003-T006) -> US1 (T007-T016) -> US2 (T017-T025) -> US3 (T026-T033) -> Polish (T034-T036).
- US1 is the MVP and is independently shippable. US2 and US3 build on US1's validator surface but are each independently testable.
- Within a story, `[P]`-marked test tasks touch distinct files and may be written in parallel; implementation tasks that share a file are sequential.

## Parallel Execution Examples

- Foundational: T006 (fixtures) runs parallel to T003-T005 (schema + module) since it touches only fixture files.
- US1 tests T007-T011 are all `[P]` (distinct test files/cases) and can be authored together before T012 begins.
- US2 tests T017-T021 and US3 tests T026-T028 are likewise `[P]` within their stories.

## Implementation Strategy

MVP = Phase 1 + Phase 2 + Phase 3 (US1): a working `fragcap schema validate` with
all-errors-at-once over the strict variant. Ship-quality for the slice adds US2
(all four forms) and US3 (publish + emit + bind), then Polish closes the gate,
the changelog, and the spec-section-15 reconciliation.
