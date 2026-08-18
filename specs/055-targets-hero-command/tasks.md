# Tasks: The targets hero command and interactive authoring

**Feature**: S055 | **Branch**: `055-targets-hero-command` | **Input**: design docs in `specs/055-targets-hero-command/`

Tests are included: this repository's constitution and verification discipline
make tests and goldens non-negotiable for every slice. Local builds use
`cargo +1.96.0-x86_64-pc-windows-gnu`; the full gate is `cargo xtask ci` (CI, MSVC).

**Crates**: `fragcap-targets` (store, selector, export mapping), `fragcap-cli`
(command surface), `fragcap` (facade + fixture-pipeline test), `docs/`,
`changelog.d/`.

## Phase 1: Setup

- [ ] T001 Confirm branch `055-targets-hero-command` is checked out and `.specify/feature.json` points at `specs/055-targets-hero-command`; run baseline `cargo +1.96.0-x86_64-pc-windows-gnu test -p fragcap-targets -p fragcap-cli` to record a green starting point.

## Phase 2: Foundational (blocks all user stories)

- [ ] T002 Add the `listing_snapshot` table: bump `SCHEMA_VERSION` 5 -> 6, add the `CREATE TABLE listing_snapshot(position INTEGER PRIMARY KEY, stable_id INTEGER NOT NULL, handle TEXT NOT NULL)` to `DDL`, and add `MIGRATE_5_TO_6` plus the `if version == 5 { ...; stamp 6 }` step in the migration driver, in crates/fragcap-targets/src/schema.rs and crates/fragcap-targets/src/store.rs.
- [ ] T003 [P] Add `Store::write_listing_snapshot(&mut self, rows: &[(i64, &str)])` (DELETE then insert positions 1..n) and `Store::listing_snapshot_nth(&self, position: usize) -> Result<Option<i64>, TargetsError>` returning the stable_id at a 1-based position, in crates/fragcap-targets/src/store.rs.
- [ ] T004 Change the row-index branch of `resolve_positional` to resolve via `listing_snapshot_nth(position)` then `target_by_stable_id`, returning `Selection::NoMatch` for an out-of-range/removed position, in crates/fragcap-targets/src/selector.rs. Keep handle/name/`--id` resolution unchanged.
- [ ] T005 [P] Promote `is_row_index` to a shared `selector` helper and remove the duplicate at crates/fragcap-cli/src/commands/targets.rs:385, updating callers.
- [ ] T006 [P] Add a migration + snapshot unit test (fresh DB stamps 6; a v5 DB migrates; write_listing_snapshot then listing_snapshot_nth round-trips; out-of-range yields None) in crates/fragcap-targets/src/store.rs tests or crates/fragcap-targets/tests/.
- [ ] T007 Verify/add `fragcap::targets` facade re-exports for every new target surface the CLI will call (snapshot writer, `delete_target`, promotion, export/import, prompt seam), in crates/fragcap-targets/src/lib.rs and the `fragcap` facade module.

**Checkpoint**: schema v6, snapshot storage, and snapshot-backed row resolution exist and are tested; the rest builds on this.

## Phase 3: User Story 1 - The hero listing (P1, MVP)

**Goal**: `fragcap targets` / bare `fragcap` show the numbered CAPTURE/KNOWN table, write the snapshot, name the next command, and print an actionable empty case.

**Independent test**: seed a mixed store, assert table shape/ordering/footer and that `capture <n>` resolves the displayed row after a mutation; empty store asserts the next-commands block.

- [ ] T008 [P] [US1] Add a CAPTURE-readiness derivation (`ready` | `needs a target`) from an entry's `launch_entries` resolvability (reuse `entry_windows_clients`) + resolved `anchor`, as a pure function in crates/fragcap-targets/src/ (e.g. a `readiness` module) with unit tests over resolved / unresolved / anchor-only / bare entries.
- [ ] T009 [P] [US1] Add a KNOWN evidence-summary derivation (evidence findings -> "Denuvo, EasyAntiCheat"; else launcher-mediation + client image; else "no online mode recorded" / "no launch data known"), neutral phrasing (FR-021), as a pure function with unit tests, in crates/fragcap-targets/src/.
- [ ] T010 [US1] Rewrite the listing renderer in crates/fragcap-cli/src/commands/targets.rs to produce the columnar `#/TARGET/CAPTURE/KNOWN` table ordered by handle, ending with the `fragcap capture <n>` next-command line (first `ready` row), replacing the current tab-separated `list`.
- [ ] T010a [US1] Make the listing discovery-driven and registering: on a listing, run discovery across its tiers (reuse the `discover` composition) and register newly discovered titles idempotently via `insert_target` at the source fidelity, deduping on anchor/identity (reuse the `target_by_anchor` guard from `add`), never modifying or removing existing entries (FR-001, FR-007, P-10), in crates/fragcap-cli/src/commands/targets.rs. Existing-entry set stays byte-identical on a repeat listing.
- [ ] T011 [US1] Write the listing snapshot from every listing path (`list`, `list_default` for bare `fragcap` and `targets`) via `write_listing_snapshot`, after registration so the snapshot reflects the rows shown, in crates/fragcap-cli/src/commands/targets.rs.
- [ ] T012 [US1] Implement the empty case (no targets AND discovery finds nothing) printing actionable next commands and still naming a next command (FR-006, SC-006), preserving the bare-`fragcap` footer vs `targets` no-footer distinction, in crates/fragcap-cli/src/commands/targets.rs.
- [ ] T013 [US1] Add CLI tests + a golden for the listing: populated table shape and ordering, footer vs no-footer, empty-case next commands, and a snapshot-resolution case (list, mutate, `capture <n>` still hits the displayed row; out-of-range exits 2), in crates/fragcap-cli/tests/cli_targets.rs (+ cli_capture.rs) and tests/goldens/.

**Checkpoint**: US1 delivers all five hero criteria on its own (the MVP).

## Phase 4: User Story 2 - Interactive authoring + promotion (P2)

**Goal**: `targets add` authors a target with inline scan and the `Y/n/unsure` socket-holder question, never fabricating a holder; a capture promotes an unsure row to `verified`.

**Independent test**: scripted-prompt add for Y/n/unsure asserts the stored launch chain and no fabricated holder; a fixture-pipeline capture promotes an unsure row.

- [ ] T014 [P] [US2] Add the prompt seam: a `Prompt` trait (read line, read choice) with a console implementation and a scripted test double (mirroring `Confirm`/`ScriptedConfirm`), plus a `SocketHolderAnswer { Yes, No, Unsure }` type, in crates/fragcap-targets/src/ (or fragcap-cli) with the scripted double exercised by a unit test.
- [ ] T015 [P] [US2] Add the answer -> `launch_entries` JSON mapping (Y=resolved client; n=non-client stage, holder unresolved; unsure=unresolved marker, no holder) as a pure function asserting no answer records an unobserved holder (P-9, FR-012), with unit tests, in crates/fragcap-targets/src/.
- [ ] T016 [US2] Wire interactive `targets add`: resolve exe (path arg or Enter-to-browse), run inline detection (`SignatureSet::compile` + `detect`) and print engine/anti-cheat/drm findings before prompts, prompt name+handle (derived default, disambiguated), ask the socket-holder question, and persist via `insert_target` with `evidence` from the scan, in crates/fragcap-cli/src/commands/targets.rs.
- [ ] T017 [US2] Add the non-interactive fallback: when stdin is not a terminal use the flag-driven form with a `--socket-holder yes|no|unsure` flag; a required-but-missing value is a usage error (exit 2), never a blocking prompt (FR-015); update TargetsAddArgs in crates/fragcap-cli/src/cli.rs.
- [ ] T018 [US2] Keep `add --steam <app_id>` working through the shared path (installed -> `steam:<app_id>` anchor; not installed -> usage error; already registered -> report + exit 0), verifying no regression in crates/fragcap-cli/src/commands/targets.rs.
- [ ] T019 [US2] Add `Store::promote_target_launch(&mut self, id: i64, launch_entries: &Value, fidelity: FidelityTier)` rewriting an entry's launch chain and raising fidelity, plus the pure promotion function (observed image + unresolved entry -> resolved entry + Verified), with unit tests, in crates/fragcap-targets/src/store.rs and a promotion module.
- [ ] T020 [US2] Wire capture write-back: when `capture` runs against a target whose launch chain is unresolved, after the run take the observed dominant socket-holder image and call `promote_target_launch`; observe-nothing leaves it unresolved (no fabrication), in crates/fragcap-cli/src/commands/capture.rs.
- [ ] T021 [US2] Add tests: scripted-prompt add for Y/n/unsure (stored launch chain + no fabricated holder) in cli_targets.rs; a fixture-pipeline promotion test (unsure row -> capture -> `verified`, and observe-nothing stays unresolved) in crates/fragcap/tests/. If promotion proves to need a live backend, land the store+fn unit tests and mark the end-to-end Tier 2 (not CI) explicitly.

**Checkpoint**: authoring and the honest unsure->verified lifecycle work and are tested without a live driver.

## Phase 5: User Story 3 - Lifecycle: remove, export, import (P3)

**Goal**: curate and move targets between machines; export/import round-trips on `stable_id`.

**Independent test**: export a store, import into a fresh store, assert identical id set and no duplicates; remove deletes exactly one; ambiguous refuses.

- [ ] T021a [P] [US3] Make `targets scan <dir>` register the discovered titles (FR-016): persist each candidate via `insert_target` at the source fidelity, deduping on anchor/identity, with conserved and surfaced accounting (P-4), replacing the print-only behavior at crates/fragcap-cli/src/commands/targets.rs:84. Reuse the shared registration helper from T010a. Report registered vs skipped counts.
- [ ] T022 [P] [US3] Add `Store::delete_target(&mut self, id: i64) -> Result<bool, TargetsError>` (aliases cascade via FK), with a unit test, in crates/fragcap-targets/src/store.rs.
- [ ] T023 [US3] Add `targets remove <SELECTOR|--id N>`: resolve, delete exactly the match, ambiguous name lists matches and refuses (exit 2), clean handle/name miss exits 0, out-of-range/`--id` unknown exits 2; wire the `Remove` variant in crates/fragcap-cli/src/cli.rs and crates/fragcap-cli/src/commands/targets.rs.
- [ ] T024 [P] [US3] Add the target-entry array mapping: an explicit `TargetEntry` <-> JSON serializer/deserializer (the type has no serde derive) with required/optional fields per the export contract, plus structural validation of an imported element, in a new crates/fragcap-targets/src/targets_export.rs, with a round-trip unit test. Also add the store update path import needs for an existing `stable_id`: a focused `Store::update_target(&mut self, entry: &TargetEntry)` (overwriting name/classification/classification_source/fidelity/anchor/launch_entries/install_root/evidence for the row with that stable_id), with a unit test, in crates/fragcap-targets/src/store.rs.
- [ ] T025 [US3] Add `targets export [SELECTOR|--id N]`: no selector -> all entries (handle-ordered) as a pretty-printed array to stdout; selector -> one-element (or empty) array; ambiguous -> refuse (exit 2); wire the `Export` variant in crates/fragcap-cli/src/cli.rs and commands/targets.rs.
- [ ] T026 [US3] Add `targets import <FILE>`: parse + validate the array, reject a nonconforming file whole (all-or-nothing, FR-019), merge each element on `stable_id` (update in place | insert with handle disambiguation), report inserted/updated counts; wire the `Import` variant in crates/fragcap-cli/src/cli.rs and commands/targets.rs.
- [ ] T027 [US3] Add tests: export/import round-trip identity (fresh store, identical id set, no duplicates, idempotent second import) in crates/fragcap-targets/tests/ and cli_targets.rs; remove cases (exact, ambiguous, alias cascade); import reject-nonconforming.

**Checkpoint**: full target lifecycle curatable and portable.

## Phase 6: Polish and cross-cutting

- [ ] T028 [P] Add glossary entries (P-6) for the new terms (listing snapshot, capture readiness, unresolved launch chain, target-entry export) in the appropriate docs/glossary/ files, with index links.
- [ ] T029 Rewrite the master specification's targets-command section(s) to describe the shipped surface (listing, snapshot-backed row index, interactive add + unsure/promotion, remove/export/import) in docs/fragcap-specification.md (P-11), and run `cargo xtask spec` to confirm the Applies-To binding.
- [ ] T030 [P] Update the README quickstart to lead with `fragcap targets` as the hero command and reflect the new subcommands, in README.md.
- [ ] T031 [P] Add the changelog fragment changelog.d/S055-targets-hero-command.added.md with the spec-impact key for the sections touched in T029.
- [ ] T032 Run the full gate `cargo xtask ci` (fmt, clippy --all-targets --all-features, test --workspace --locked, lint, deps, license, fixture drift, spec) and the net/feature builds; fix to green. Record MSVC-only gates (clippy --all-features needing npcap, cargo deny, msrv) deferred to CI, and any Tier-2 boundary from T021, explicitly.

## Dependencies and order

- **Setup (T001)** -> **Foundational (T002-T007)** -> user stories.
- **US1 (T008-T013)** depends only on Foundational; it is the MVP and can ship alone.
- **US2 (T014-T021)** depends on Foundational; independent of US1 (though it reads the same store). T019 before T020; T014/T015 before T016.
- **US3 (T022-T027)** depends on Foundational; independent of US1/US2. T024 before T025/T026.
- **Polish (T028-T032)** last; T029 before T032 (spec gate).

## Parallel opportunities

- Foundational: T003, T005, T006 are [P] (distinct concerns) after T002.
- US1: T008, T009 [P] (pure functions) before the renderer T010.
- US2: T014, T015 [P]; T019's pure function parallel to T014/T015.
- US3: T022, T024 [P] before their wiring tasks.
- Polish: T028, T030, T031 [P].

## MVP scope

**User Story 1 (T001-T013)** is the minimum viable hero command: it satisfies all
five §9.5 hero acceptance criteria on a fresh install. US2 and US3 layer on
independently.
