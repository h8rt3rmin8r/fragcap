---

description: "Task list for S051: the target entry model"
---

# Tasks: The target entry model (handles, stable ids, selector resolution, cascade collapse)

**Input**: Design documents from `specs/051-target-entry-model/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: INCLUDED. The spec's Success Criteria and issue #138 acceptance
criteria mandate specific tests (Appendix A handle vectors, identifier equality,
selector ambiguity, the four preserved declines), so test tasks are first-class
here.

**Organization**: Tasks are grouped by user story (US1 through US5 from spec.md)
for independent implementation and testing.

**Environment note**: this dev machine has no MSVC linker; build and test
SQLite-backed crates with `cargo +1.96.0-x86_64-pc-windows-gnu ... --features targets`.
CI runs the real MSVC build. Tasks that require a linker-capable machine
(dependency-graph and license verification) are marked accordingly.

**Revision note**: this list incorporates the `/speckit-analyze` remediations, as
later narrowed by the implementation deferrals (spec Clarifications). I2: the
identifier primitives are Foundational (T009), so US1 registration no longer
forward-depends on US2. Two stories were then deferred and their tasks marked
DEFERRED below: US2 (the schema extension and entry export/import, originally C1)
to S055/S057, and US5 (retiring the `profile` command and capture surface,
originally I1) to S054's capture rework.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: US1 through US5; Setup/Foundational/Polish carry no story label

## Path Conventions

Single Rust workspace. New code lands in `crates/fragcap-targets/` (behind the
`targets` feature), with the schema extension in `crates/fragcap-profile/` and
retirement touching `crates/fragcap-cli/` and `crates/fragcap-profile/`, and docs
under `docs/`.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: dependency plumbing and module registration.

- [X] T001 Add `blake3` (`default-features = false`), `unicode-normalization`, the chosen general-category crate (candidate `unicode-properties`), and `getrandom` to `[workspace.dependencies]` in root `Cargo.toml`, and wire them into `crates/fragcap-targets/Cargo.toml` `[dependencies]` as `optional = true` gated by the existing `targets` feature.
- [X] T002 [P] Register new modules (`entry`, `handle`, `identifier`, `selector`) in `crates/fragcap-targets/src/lib.rs` with `pub` stubs, feature-gated on `targets`.

**Checkpoint**: crate compiles with `--features targets` and empty module stubs.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: the enums, schema, base entity, and identifier primitives every story depends on.

**CRITICAL**: No user story work can begin until this phase is complete.

- [X] T003 [P] Define the `Classification` (`game`/`launcher`/`tool`/`mod`/`emulator`/`unknown`) and `ClassificationSource` (`catalog`/`engine-signature`/`platform`/`user`/`unset`) enums with `as_str`/`parse` in `crates/fragcap-targets/src/entry.rs`, matching the schema CHECK sets.
- [X] T004 [P] Define the ordered `Fidelity` enum (`authored` > `verified` > `heuristic-unverified` > `observed`) with a derived `Ord` (highest-first ordering documented) and `as_str`/`parse` in `crates/fragcap-targets/src/entry.rs`.
- [X] T005 Define the `TargetEntry` struct with the FR-001 fields (`id`, `stable_id`, `handle`, `name`, `classification`, `classification_source`, `fidelity`, `provenance`, `anchor`, `launch_entries`, `install_root`, `evidence`) in `crates/fragcap-targets/src/entry.rs` (depends on T003, T004).
- [X] T006 Bump `SCHEMA_VERSION` 2 -> 3 and add the `targets` and `target_id_aliases` table DDL plus `MIGRATE_2_TO_3` (with the non-numeric-handle CHECK `handle GLOB '*[^0-9]*'` and all enum CHECKs) in `crates/fragcap-targets/src/schema.rs`.
- [X] T007 Add the v2 -> v3 migration arm to `Store::open` in `crates/fragcap-targets/src/store.rs`, transactional and mirroring the existing v1 -> v2 arm (apply DDL + stamp `user_version` in one transaction).
- [X] T008 [P] Migration test in `crates/fragcap-targets/tests/` (or `store.rs` unit tests): a fresh store opens at v3 with the `targets` table; a synthetic v2 store upgrades in place with existing catalog rows intact and the `targets` table empty.
- [X] T009 Implement the identifier primitives in `crates/fragcap-targets/src/identifier.rs` (depends on T001): anchor canonicalization (`steam:<appid>`, `epic:<catalogItemId>`, `gog:<productId>`); the anchored identifier (low 63 bits of BLAKE3 over the canonicalized anchor); the unanchored random 63-bit generator (via `getrandom`, no reserved bit). These are shared by US1 registration and US2 merge, so they are Foundational.

**Checkpoint**: schema v3 exists and migrates; enums, `TargetEntry`, and identifier primitives compile. User stories can begin independently.

---

## Phase 3: User Story 1 - Register a target and refer to it by a human handle (Priority: P1) 🎯 MVP

**Goal**: a target is a `local.db` row with an auto-derived handle; no profile file is written and the handle is selectable.

**Independent Test**: register the Appendix A names, confirm each handle matches the table, and confirm each is selectable by handle with no file on disk.

### Tests for User Story 1

- [X] T010 [P] [US1] Appendix A handle-vector unit tests (all 13 rows) in `crates/fragcap-targets/tests/handle_vectors.rs`, including the `Tom Clancy's(TM)`, `Pokemon`-with-accent, Roman-numeral, vulgar-fraction, degree-sign, and `S.T.A.L.K.E.R.` cases.
- [X] T011 [P] [US1] Handle edge-case unit tests in `crates/fragcap-targets/tests/handle_vectors.rs`: purely-numeric input falls back; whitespace-only falls back to exe stem then `target_<n>`; a 90-char title truncates to 64 with no trailing `_`; a combining mark with canonical combining class 0 is still stripped as category `Mn`.
- [X] T012 [P] [US1] Collision unit test in `crates/fragcap-targets/tests/handle_vectors.rs`: `Portal 2` registered twice yields `portal_2` then `portal_2_2`, and the first entry's handle is unchanged.

### Implementation for User Story 1

- [X] T013 [US1] Implement the FR-004 normalization algorithm in exactly the specified order (strip `So`/`Sk`/`Cf`; NFKD; strip `Mn`; lowercase; delete apostrophes/quotes; collapse non-`[a-z0-9]` runs to single `_`; trim; truncate 64 then trim trailing `_`) in `crates/fragcap-targets/src/handle.rs`.
- [X] T014 [US1] Implement the fallback chain (empty/invalid -> exe stem -> `target_<n>`) and the purely-numeric rejection, guaranteed to terminate, in `crates/fragcap-targets/src/handle.rs` (depends on T013).
- [X] T015 [US1] Implement collision auto-increment (`_2`, `_3`, ... on the new item, existing untouched) and the user-override path (same validity rules), with an override-specific unit test, in `crates/fragcap-targets/src/handle.rs` using a store handle-existence query (depends on T014).
- [X] T016 [US1] Add `Store` methods to insert a `TargetEntry` and to test handle existence/uniqueness in `crates/fragcap-targets/src/store.rs` (depends on T005, T007); the insert enforces the handle CHECK and returns a typed error a caller can map.
- [X] T017 [US1] Wire target registration from a name into the `targets` command surface in `crates/fragcap-cli/src/commands/targets.rs`: compute the handle, build an unanchored `TargetEntry` using the Foundational identifier generator (T009), and store it (depends on T009, T015, T016). No forward dependency on US2.

**Checkpoint**: registering Appendix A names yields the expected handles, persisted in `local.db` with a real unanchored `stable_id`, no files written. MVP-complete and independently testable.

---

## Phase 4: User Story 2 - Independent registrations merge instead of duplicating (Priority: P1)

**Goal**: anchored entries share a deterministic identifier and merge; unanchored entries supersede their random id with an alias when anchored; the exported entry validates against the published schema.

**Independent Test**: build two entries from the same Steam anchor and confirm equal `stable_id`; from different anchors, unequal; supersede an unanchored entry and confirm the alias resolves; export and re-import to one entry.

### Tests for User Story 2

- [X] T018 [P] [US2] Identifier determinism tests in `crates/fragcap-targets/tests/identifier.rs`: two entries from `steam:2221490` have equal `stable_id`; `steam:2221490` vs `steam:620` differ; every anchored id is non-negative; a non-canonical prefix (STEAM:620) yields the same id; two unanchored ids differ.
- [ ] T019 [P] [US2] DEFERRED to S055/S057 -- Supersession + export/import round-trip test in `crates/fragcap-targets/tests/identifier.rs`: an unanchored entry matched to an anchor adopts the anchored id, its old value lands in `target_id_aliases`, `--id <old>` still resolves it, and export-then-import into a fresh store yields one entry (no duplicate).

### Implementation for User Story 2

- [X] T020 [US2] Add `Store` merge-on-`stable_id` (an anchor already present merges rather than inserting) and the supersession write (adopt anchored id, insert former value into `target_id_aliases`) in `crates/fragcap-targets/src/store.rs` (depends on T009, T016).
- [ ] T021 [US2] DEFERRED to S055/S057 -- Include `stable_id` and `anchor` in JSON export and merge on the active id (consulting `target_id_aliases`) in import, in `crates/fragcap-targets/src/export.rs` and `crates/fragcap-targets/src/import.rs` (depends on T020).
- [ ] T022 [US2] DEFERRED to S055/S057 -- Extend the published master schema in `crates/fragcap-profile/` (`schema.rs` / `jsonschema/`) with the entry fields (`name`, `anchor`, `classification`, `classification_source`, `fidelity`, `launch_entries`; `handle` and `stable_id` optional on input), bumping the schema version if the extension requires it, and add a conformance test that an exported entry passes `schema validate` and that import(export(entry)) is one entry (depends on T021). Resolves C1: one document shape for export, import, and validation.

**Checkpoint**: identity is deterministic from the anchor, registrations merge, supersession and export/import preserve identity, and an exported entry validates against the published schema.

---

## Phase 5: User Story 3 - An ambiguous selector refuses to guess (Priority: P2)

**Goal**: a selector resolves to exactly one target or none; an ambiguous name lists matches and exits 2.

**Independent Test**: register two targets sharing a name, select by that name, confirm both are listed and the command exits non-zero without resolving.

### Tests for User Story 3

- [X] T023 [P] [US3] Selector resolution unit tests in `crates/fragcap-targets/tests/selector.rs`: exact handle resolves one; case-insensitive exact name resolves one; `--id` resolves by `stable_id` and by a superseded alias; a bare integer indexes the current listing.
- [X] T024 [P] [US3] Ambiguity + zero-match tests in `crates/fragcap-targets/tests/selector.rs`: a name matching >1 row returns the ambiguous outcome carrying each match's handle and id; a name matching 0 rows returns a distinct no-match outcome.

### Implementation for User Story 3

- [X] T025 [US3] Implement selector resolution (bare-int row index; exact handle then case-insensitive exact name; `--id` by `stable_id`/alias) returning a typed `Resolved | NoMatch | Ambiguous{matches}` in `crates/fragcap-targets/src/selector.rs` (depends on T016, T020).
- [X] T026 [US3] Wire the selector into the CLI: accept a positional token/int and `--id <N>`, print the ambiguous match list (handle + id) and exit 2, in `crates/fragcap-cli/src/cli.rs` and `crates/fragcap-cli/src/commands/targets.rs` (depends on T025).
- [X] T027 [US3] CLI integration test for the ambiguity exit code and listing output in `crates/fragcap-cli/tests/` (depends on T026).

**Checkpoint**: selection never guesses; ambiguity exits 2 with a helpful list.

---

## Phase 6: User Story 4 - Resolution is ordered by fidelity, and the declines are preserved (Priority: P2)

**Goal**: the store read is fidelity-ordered and the four hint declines remain declines expressed as fidelity-aware query conditions.

**Independent Test**: seed one title at different fidelities across the stores and confirm the highest wins; feed each declined shape and confirm none resolves.

### Tests for User Story 4

- [X] T028 [P] [US4] Fidelity-ordering tests extending `crates/fragcap-targets/tests/hint_cascade.rs`: `authored` in `local.db` beats `heuristic-unverified` in `catalog.db`; competing `local.db` rows resolve highest-first; a runtime match may promote to `verified`.
- [X] T029 [P] [US4] Decline-preservation tests in `crates/fragcap-targets/tests/hint_cascade.rs`: each of sparse, engine-only, launcher-mediated, and multi-executable rows declines and the cascade continues; the pre-existing mediation-merge and multi-exe ambiguity-note tests still pass.

### Implementation for User Story 4

- [X] T030 [US4] Make the store read fidelity-aware in `crates/fragcap-targets/src/hint_provider.rs`: a `local.db` target row resolves at its stored `fidelity`, a `catalog.db` row at `heuristic-unverified`, and the highest-fidelity competing row wins (depends on T005, T020).
- [X] T031 [US4] Re-express the four declines (sparse, engine-only, launcher-mediated, multi-exe) as fidelity-aware query conditions on the read, keeping each a decline recorded via `ResolutionNotes`, in `crates/fragcap-targets/src/hint_provider.rs` (depends on T030).

**Checkpoint**: fidelity ordering holds; every decline is preserved and explained. (EngineRule and PlatformWalker providers remain in the resolver this slice, per the operator decision; their removal is S052.)

---

## Phase 7: User Story 5 - Profiles stop being files (Priority: P3)

**Goal**: retire the `profile` command in full (validate/list/show) and the profile-file surface; keep `schema validate`.

**Independent Test**: confirm `--profile` and every `profile` subcommand are unrecognized, no profile directory is created or read, and `schema validate` still runs.

### Tests for User Story 5

- [ ] T032 [P] [US5] DEFERRED to S054 -- CLI tests in `crates/fragcap-cli/tests/`: `--profile <path>` is unrecognized (exit 2); `profile validate`, `profile list`, and `profile show` are unrecognized; `schema validate <file>` still validates against the entry-extended schema (exit 0 conformant, 1 non-conformant).
- [ ] T033 [P] [US5] DEFERRED to S054 -- Test in `crates/fragcap-cli/tests/` that a fresh run creates and reads no AppData profile directory (assert the former `user_profile_dir` path is neither created nor consulted).

### Implementation for User Story 5

- [ ] T034 [US5] DEFERRED to S054 -- Remove the `--profile` selector and the entire `profile` command (validate/list/show; its subject, profile files, is retired) from `crates/fragcap-cli/src/cli.rs` and delete `crates/fragcap-cli/src/commands/profile.rs`; ensure `schema validate`/`schema print` remain under the `schema` command (FR-023).
- [ ] T035 [US5] DEFERRED to S054 -- Remove `user_profile_dir`, `search_path`, and the profile-dir env override from `crates/fragcap-cli/src/paths.rs`, keeping the S050 `catalog_db`/`local_db` path functions.
- [ ] T036 [US5] DEFERRED to S054 -- Retire the `Profile` file provider and file-search path in `crates/fragcap-profile/` (remove the file-loading provider from the resolver registration) while keeping `FidelityTier`, `Target`, `validate_json`, `schema_document` (now entry-extended), and the resolver machinery (depends on T034).
- [ ] T037 [US5] DEFERRED to S054 -- Update CLI help/golden fixtures in `crates/fragcap-cli/tests/` to drop the retired surface and reflect the new selector args (depends on T034, T026).

**Checkpoint**: the profile-file surface is gone; `schema validate` works against the extended schema; selection is by handle/name/id.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: documentation, governance, dependency verification, and the full gate.

- [X] T038 [P] Add glossary entries for every new term (target entry, handle, anchor, stable identifier, fidelity ordering, superseded alias) under `docs/glossary/` in this same change (P-6).
- [X] T039 [P] Update master specification sections 5, 6, and 15 to describe the entry model, the fidelity-ordered store cascade, and the entry-extended schema as shipped, noting the provider reduction completes in S052 (P-11) in `docs/fragcap-specification.md`.
- [X] T040 [P] Add the four new dependencies (`blake3`, `unicode-normalization`, the category crate, `getrandom`) to the `AGENTS.md` dependency table with their justifications from research.md.
- [X] T041 Add changelog fragment(s) under `changelog.d/` for S051, each beginning with a `<!-- spec-impact: ... -->` header naming sections 5/6/15 (or `none`), so `cargo xtask spec` passes.
- [ ] T042 (CANNOT RUN HERE: needs MSVC linker) [REQUIRES LINKER-CAPABLE MACHINE] Verify the dependency delta: `cargo tree -p fragcap-targets --features targets` shows only the expected new crates; `cargo deny check licenses` passes (add a `deny.toml` exception if `constant_time_eq` resolves to CC0-only); `cargo xtask msrv` is unaffected (new crates behind default-off `targets`).
- [X] T043 Run `crates/fragcap-targets/tests/corpus`/`conformance` and confirm the S050 "catalog refresh leaves local.db byte-identical" and existing hint tests still pass (no regression from the v3 migration).
- [ ] T044 (CANNOT RUN HERE: MSVC/CI gate) Run the full gate `cargo xtask ci` (fmt, clippy `--all-targets --all-features`, `test --workspace --locked`, lint, deps, license) and the quickstart.md validation; capture output.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: no dependencies.
- **Foundational (Phase 2)**: depends on Setup; BLOCKS all user stories. Now includes the identifier primitives (T009), so no user story forward-depends on another.
- **User Stories (Phases 3-7)**: depend on Foundational.
  - US1 (P1) is self-contained: registration stores an unanchored entry using the Foundational generator (T009). This is the MVP.
  - US2 (P1) adds anchor-based merge, supersession, export/import, and the schema extension. Depends on the store CRUD (T016) and the identifier primitives (T009).
  - US3 (P2) depends on the store CRUD/merge (T016, T020).
  - US4 (P2) depends on the entry model + merge (T005, T020) and touches `hint_provider.rs` only.
  - US5 (P3) is largely independent (CLI + fragcap-profile retirement); its golden update (T037) depends on the US3 selector args (T026); its schema-validate test (T032) reflects the US2 schema extension (T022).
- **Polish (Phase 8)**: depends on all desired stories.

### Within Each User Story

- Tests first (they should fail before implementation).
- Algorithm/model before store methods before CLI wiring.

### Parallel Opportunities

- Setup: T002 after T001.
- Foundational: T003 ∥ T004; T008 after T007; T009 after T001 (parallel with T003-T008 except it needs only T001).
- US1 tests T010 ∥ T011 ∥ T012. US2 tests T018 ∥ T019. US3 tests T023 ∥ T024. US4 tests T028 ∥ T029. US5 tests T032 ∥ T033.
- Once Foundational lands, US1 and US2 implementation proceed in parallel; US4 (`hint_provider.rs`) and US5 (cli/paths/profile) touch disjoint files and can run alongside.
- Polish: T038 ∥ T039 ∥ T040 (disjoint docs).

---

## Parallel Example: User Story 1

```bash
# Tests together (write first, expect failure):
Task: "Appendix A handle vectors in crates/fragcap-targets/tests/handle_vectors.rs"
Task: "Handle edge cases (fallback, truncation, Mn) in the same test file"
Task: "Collision test (portal_2 -> portal_2_2)"
# Then implement handle.rs steps in order (T013 -> T014 -> T015).
```

---

## Implementation Strategy

### MVP First (User Story 1)

With the identifier primitives now Foundational, US1 is a true standalone MVP:
complete Setup -> Foundational -> US1 (register by handle, persisted with a real
unanchored `stable_id`), then STOP and validate against SC-001, SC-002, SC-003.

### Incremental Delivery

1. Setup + Foundational -> schema v3 and identifier primitives ready.
2. US1 -> register by handle (MVP). Validate.
3. US2 -> anchor identity, merge/dedup, export/import, schema extension. Validate SC-004.
4. US3 -> non-guessing selection. Validate SC-006.
5. US4 -> fidelity-ordered resolution + preserved declines. Validate SC-005.
6. US5 -> retire the profile-file surface. Validate SC-007.
7. Polish -> docs, governance, dependency verification, full gate.

### Notes

- [P] tasks touch different files with no incomplete dependencies.
- Verify each test fails before implementing.
- Commit after each task or logical group; stage only this slice's files.
- The dependency-graph/license verification (T042) and the real MSVC gate (T044)
  cannot be completed on this dev machine; do not report them green until run on a
  linker-capable machine / CI.
