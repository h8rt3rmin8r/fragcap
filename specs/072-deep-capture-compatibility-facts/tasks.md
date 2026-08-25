# Tasks: Deep Capture compatibility facts

**Input**: Design documents from `/specs/072-deep-capture-compatibility-facts/`

**Prerequisites**: plan.md, research.md, data-model.md, analysis.md, `contracts/compatibility-facts-schema.md`, quickstart.md

Tests are written or extended alongside implementation because this slice changes durable local storage.

## Phase 1: Setup

- [X] T001 Re-read `docs/plans/deep-capture.md`, `docs/plans/steam-launcher-proxy-inheritance.md`, issue #217, `docs/fragcap-specification.md` section 15.8, `crates/fragcap-targets/src/schema.rs`, and `crates/fragcap-targets/src/store.rs`.

## Phase 2: Model

- [X] T002 Add `crates/fragcap-targets/src/compatibility.rs` with compatibility fact keys, launch cases, evidence sources, `CompatibilityFact`, and key/value validation.
- [X] T003 Add model tests for fact-key round trips and invalid key/value rejection.
- [X] T004 Export the compatibility model from `fragcap-targets`.

## Phase 3: Schema

- [X] T005 Bump the targets schema version from 8 to 9.
- [X] T006 Add `deep_capture_facts` to fresh-store DDL.
- [X] T007 Add `MIGRATE_8_TO_9` that creates the same table and inserts no rows.
- [X] T008 Include structured proxy backend, backend version, proxy mode, final-owner executable, and final-owner handoff columns.
- [X] T009 Keep launch case mutually exclusive and remove owner-handoff from launch-case tokens.

## Phase 4: Store APIs

- [X] T010 Add `Store::insert_compatibility_fact`.
- [X] T011 Add `Store::compatibility_facts_for_target`.
- [X] T012 Map SQLite constraint violations to model errors rather than panicking or leaking low-context failures.

## Phase 5: Tests

- [X] T013 Add a v8-to-v9 migration test proving no facts are invented.
- [X] T014 Add a full round-trip test including provenance, freshness, proxy backend, final-owner executable, handoff, stale state, and note.
- [X] T015 Add invalid value tests through both model construction and direct SQLite insertion.
- [X] T016 Add cascade delete test proving target deletion removes compatibility facts.

## Phase 6: Documentation and Verification

- [X] T017 Update `docs/fragcap-specification.md` section 15.8 and Q-13.
- [X] T018 Add a changelog fragment with spec impact.
- [X] T019 Add this numbered spec-kit slice and checklist.
- [X] T020 Run `cargo fmt --check`.
- [X] T021 Run `git diff --check`.
- [X] T022 Run `cargo test --workspace --quiet`.
- [X] T023 Run `cargo xtask lint`.
- [X] T024 Run `cargo xtask deps`.
- [X] T025 Run `cargo xtask spec`.
- [X] T026 Run `cargo xtask changelog --check`.
- [X] T027 Scan new/touched public artifacts for fact-finding PII.
