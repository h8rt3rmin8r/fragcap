---
description: "Task list for S031 technology-detection surface"
---

# Tasks: Technology-Detection Surface

**Input**: Design documents from `specs/031-technology-detection/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Included. The constitution mandates test-driven discipline and each
user story in spec.md carries an Independent Test; test tasks are written before
the implementation they cover within each phase.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on incomplete tasks)
- **[Story]**: US1..US4 map to the spec's user stories
- Paths are repository-relative.

## Phase 1: Setup

- [X] T001 Create the vendored-asset directory `crates/fragcap-profile/assets/steamdb/` and add module declarations `mod technologies;` and `mod sha256;` (plus any needed `pub use`) to `crates/fragcap-profile/src/lib.rs`, with empty SPDX-headed `crates/fragcap-profile/src/technologies.rs` and `crates/fragcap-profile/src/sha256.rs` stubs.

## Phase 2: Foundational (blocking prerequisites for all stories)

- [X] T002 Vendor the ruleset: download `rules.ini` at pinned commit `243cf741921d2c8fd6b844f83831edf4692cf788` from `SteamDatabase/FileDetectionRuleSets`, normalize to UTF-8 without BOM and LF line endings, and write it verbatim to `crates/fragcap-profile/assets/steamdb/rules.ini`.
- [X] T003 [P] Write the third-party attribution `crates/fragcap-profile/assets/steamdb/THIRD_PARTY_NOTICES.md` containing the full MIT license text and `Copyright (c) 2021 SteamDB`, naming the source repository and the pinned commit.
- [X] T004 Implement a hand-rolled SHA-256 in `crates/fragcap-profile/src/sha256.rs` (no new dependency), exposing a function that hashes a byte slice to lowercase hex.
- [X] T005 [P] Add SHA-256 self-tests in `crates/fragcap-profile/src/sha256.rs` against the published NIST vectors (empty string, "abc", the 448-bit multi-block vector).
- [X] T006 Compute the SHA-256 of the committed `rules.ini` bytes and write `crates/fragcap-profile/assets/steamdb/rules.lock.json` with fields `source`, `commit`, `license` (`"MIT"`), `sha256`, and a `note` documenting the LF/UTF-8/no-BOM normalization used for the hash.
- [X] T007 Implement the ruleset line parser in `crates/fragcap-profile/src/technologies.rs`: section headers `[Name]`, `;` comments, blank lines, and `Key = pattern` / `Key[] = pattern` rules with trailing inline-comment stripping; map the applied sections (`Engine`, `AntiCheat`, `SDK`, `Emulator`, `Container`, `Launcher`) to the `Category` enum and recognize (but do not apply) `Evidence`.
- [X] T008 Implement `Category`, `TechnologyFinding` (category, name, marker_path, fidelity), and the `heuristic-unverified` fidelity stamping in `crates/fragcap-profile/src/technologies.rs`, reusing the schema fidelity vocabulary.
- [X] T009 Implement `CompiledRuleset` in `crates/fragcap-profile/src/technologies.rs`: compile each applied pattern independently with the `regex` crate case-insensitively, skip a pattern that fails to compile while recording its category/technology/error, and expose `compiled_count`, `skipped`, and `total_count`.
- [X] T010 Implement the bounded install-directory scan and `ScanOutcome` in `crates/fragcap-profile/src/technologies.rs`: a depth-bounded `read_dir` walk producing forward-slash relative paths, matching compiled rules, deduplicating per (category, technology) with a deterministic representative marker path, and surfacing unreadable paths distinctly from an empty result.

## Phase 3: User Story 1 - See a target's technologies, including anti-cheat (P1)

**Goal**: An operator scans an install directory and gets a grouped,
marker-cited, heuristic-stamped technology report on the command line.

**Independent test**: `cargo test -p fragcap-profile technologies` plus
`cargo run -p fragcap-cli -- technologies --path <dir>` on a temp install tree.

- [X] T011 [P] [US1] Unit test in `crates/fragcap-profile/src/technologies.rs`: a temp install directory with an anti-cheat marker (e.g. `EasyAntiCheat/`) and an engine marker (e.g. `.../Binaries/Win64/Game-Win64-Shipping.exe`) yields findings listing both under their categories, each with the matched marker path and `heuristic-unverified` fidelity.
- [X] T012 [P] [US1] Unit test: a technology revealed by several marker files is reported once (dedup), and multiple technologies matching one file are each reported.
- [X] T013 [P] [US1] Unit test: a directory with no markers yields empty findings and empty unreadable; an unreadable subtree yields a surfaced unreadable path; the two are never conflated (FR-010, SC-005).
- [X] T014 [US1] Add the `technologies` subcommand: create `crates/fragcap-cli/src/commands/technologies.rs`, register it in `crates/fragcap-cli/src/commands/mod.rs`, and wire argument parsing (`--path <INSTALL_DIR>`) in `crates/fragcap-cli/src/args.rs` / `crates/fragcap-cli/src/cli.rs`.
- [X] T015 [US1] Implement the grouped report output (fixed category order, technology name + marker path per finding, heuristic banner), the non-zero skipped-patterns note, and the exit contract (0 on clean/empty scan; surfaced non-zero only when the target directory itself is unreadable) per `contracts/cli-technologies.md`.
- [X] T016 [US1] CLI-level test (in `crates/fragcap-cli` tests) asserting the subcommand prints the expected grouped shape for a temp install tree and a clear "no technologies" line for an empty one.

## Phase 4: User Story 2 - Technologies in output metadata (P1)

**Goal**: The detected set materializes into a target artifact as the schema's
`technologies` structure and validates against the master schema.

**Independent test**: `cargo test -p fragcap-profile --test schema_conformance`
and the scaffold test.

- [X] T017 [US2] Extend the embedded schema `crates/fragcap-profile/assets/target-schema.v1.json`: add the optional top-level `technologies` array and `$defs/technology` (category enum, `name`, optional `marker_path`, `fidelity` ref) per `contracts/technologies-schema.md`.
- [X] T018 [US2] Mirror the identical change to the published copy `docs/schema/target-schema.v1.json` so the embedded/published drift check stays green (byte-identical).
- [X] T019 [US2] Extend the hand-rolled variant validator in `crates/fragcap-profile/src/jsonschema/variants.rs` to accept `technologies` as a known optional top-level array and shape-check each item (required `category` from the enum, non-empty `name`, required `fidelity`, optional `marker_path` string, no additional properties).
- [X] T020 [P] [US2] Add conformance fixtures under `crates/fragcap-profile/tests/fixtures/schema/` (valid technologies array, empty array accepted, missing `category` rejected, out-of-enum `category` rejected) and assert them in `crates/fragcap-profile/tests/schema_conformance.rs`.
- [X] T021 [US2] Enrich the Steam scaffold in `crates/fragcap-steam/src/scaffold.rs` to run detection on the classified install directory and serialize the findings into the materialized target's `technologies` array; add a test that the scaffolded artifact carries the detected set and still validates.

## Phase 5: User Story 3 - Incompatible patterns skipped, counted, surfaced (P1)

**Goal**: The RE2 incompatibility is a counted, surfaced skip over the real
vendored ruleset, with conservation guaranteed.

**Independent test**: `cargo test -p fragcap-profile` skip/conservation tests.

- [X] T022 [P] [US3] Test over the real embedded ruleset asserting `compiled_count + skipped.len() == total_count` (FR-006, SC-002).
- [X] T023 [P] [US3] Test asserting the skipped count is exposed to the caller and each skip identifies its affected category and technology, so reduced coverage is visible rather than implied-absent (FR-005, US3 acceptance 3).

## Phase 6: User Story 4 - Vendored asset attributed and integrity-locked (P2)

**Goal**: The asset is a faithful, attributed, hash-locked copy.

**Independent test**: the lock-hash and notice tests.

- [X] T024 [P] [US4] Test that the hand-rolled SHA-256 of the embedded `rules.ini` bytes equals the `sha256` recorded in `rules.lock.json` (SC-003), keeping the integrity check inside `cargo test`.
- [X] T025 [P] [US4] Test that `rules.lock.json` records `source`, `commit`, `license` (`MIT`), and `sha256`, and that `THIRD_PARTY_NOTICES.md` contains the MIT text and the SteamDB copyright.

## Phase 7: Polish & Cross-Cutting Concerns

- [X] T026 [P] Add glossary entries for the new terms (technology detection, the SteamDB detection ruleset, marker path) under `docs/glossary/` and update `docs/glossary/index.md` (P-6).
- [X] T027 [P] Add a technology-detection surface note/section reference to `docs/fragcap-specification.md` (or the outline) so the specification documents the capability (P-6, FR-017).
- [X] T028 Add changelog fragments `changelog.d/031-technology-detection.added.md` and `changelog.d/031-technology-detection.decisions.md` recording: the vendored asset + NOTICE + lock (pinned artifact), the additive backward-compatible schema-v1 extension, the hand-rolled SHA-256 (no new lockfile crate), the RE2 skip-and-count handling, and the deferral of the Evidence deduction pass.
- [X] T029 Confirm the documentation/convention linters do not flag the verbatim vendored `rules.ini`; if they do, add a scoped exclusion for `assets/steamdb/**` (recorded in the decisions fragment) rather than editing the third-party bytes (R8).
- [X] T030 Run `cargo xtask ci` and `cargo xtask msrv` in the foreground and resolve any fmt/clippy/test/lint/deps/license/MSRV findings.

## Dependencies & Execution Order

- **Phase 1 -> Phase 2 -> Phases 3-6 -> Phase 7.**
- Foundational (Phase 2) blocks all user stories: the vendored asset (T002),
  the SHA-256 (T004) and lock (T006), and the detection engine (T007-T010) are
  prerequisites.
- Within Phase 2: T004 before T005/T006; T002 before T006 (hash needs the bytes);
  T007 before T009; T008 before T009/T010.
- US1 (Phase 3) is the MVP and depends only on Foundational.
- US2 (Phase 4) depends on Foundational; T017 before T018/T019; T019 before T020.
- US3 (Phase 5) depends on Foundational only (tests over the engine + real asset).
- US4 (Phase 6) depends on T004/T006 (hash + lock) and T002/T003 (bytes + notice).
- Polish (Phase 7) after the stories; T030 is the final gate.

## Parallel Opportunities

- T003 and T005 run parallel to other Phase 2 work (different files).
- Within US1, T011/T012/T013 (unit tests in the module) are [P] with each other;
  the CLI tasks T014/T015 are sequential (same files) and follow the engine.
- US3 tests (T022/T023) and US4 tests (T024/T025) are all [P] and can run
  alongside US1/US2 work once Foundational is done.
- Doc tasks T026/T027 are [P] with each other.

## Implementation Strategy

- **MVP = User Story 1** (Foundational + Phase 3): a working, tested on-demand
  technology report from an install directory, including anti-cheat, each finding
  heuristic-stamped and marker-cited. This alone is a shippable increment.
- Layer US2 (durable schema-validated metadata + scaffold), then US3 (the
  explicit skip-count conservation guarantee over the real asset), then US4 (the
  attribution/lock acceptance), then Polish and the full gate.
