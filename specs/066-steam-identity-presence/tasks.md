---

description: "Task list for S066: Steam install-path resolution, target presence, and multi-name identity"

---

# Tasks: Steam install-path resolution, target presence, and multi-name identity

**Input**: Design documents from `specs/066-steam-identity-presence/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Tests**: Included (test-driven discipline is mandatory per the constitution's
Development Workflow section).

**Organization**: Tasks are grouped by user story (US1 = #166, US2 = #167, US3 = #173)
so each is independently implementable and testable, per the source spec.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on an incomplete task)
- **[Story]**: US1, US2, or US3

## Phase 1: Setup

- [X] T001 Confirm the baseline is green: run `cargo xtask ci` in the foreground and
  read its output before any change, so a later failure is known to be this slice's
  (repo root)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The appinfo `common/type` and launch-executable extraction, and the new
`InstalledTitle` fields, are read once and consumed by both US1 (the type decides
`common/` vs `music/`, and gates the Music exclusion) and US3 (`installdir` and
`launch_executable` become `folder_name`/`executable_hint`). Building this once here
means neither story's phase re-parses appinfo.

**CRITICAL**: No user story implementation begins until this phase is complete.

- [X] T002 In `crates/fragcap-steam/src/appinfo.rs`, extend the per-section extraction
  (alongside the existing `extract_launch`) with an `extract_common_type(root:
  &VdfValue) -> Option<String>` that reads `common/type` verbatim, and extend
  `AppInfoApp` with `pub common_type: Option<String>`, populated in `parse_appinfo`'s
  per-section decode (same pass as launch entries, no second parse). Add a unit test
  fixture (a hand-built binary section, matching the existing test-fixture style
  already in this file) covering a `Music`-typed section, a `Game`-typed section, and
  a section with no `common/type` key at all.
- [X] T003 [P] In `crates/fragcap-steam/src/library.rs`, add `pub installdir: String`,
  `pub app_type: Option<String>`, and `pub launch_executable: Option<String>` to
  `InstalledTitle`. `installdir` is the raw `AppState.installdir` manifest value
  already read in `read_manifest` (kept alongside the existing joined `install_dir`);
  `app_type` and `launch_executable` are populated from the appinfo cross-reference
  built in T004.
- [X] T004 In `crates/fragcap-steam/src/library.rs`, have `discover_in(root)` read
  `appcache/appinfo.vdf` once via `crate::appinfo::read_appinfo(root)` (already
  handles "no cache is not an error"), build an `appid -> (common_type,
  first_launch_executable)` lookup, and thread it through `read_library_titles` into
  `read_manifest` so `InstalledTitle::app_type`/`launch_executable` are set per title.
  A missing or unparseable appinfo cache yields an empty lookup (every title's
  `app_type`/`launch_executable` stay `None`), not a `discover_in` error.
- [X] T005 Add a `discover_in` fixture test (in `crates/fragcap-steam/src/library.rs`'s
  test module) proving a title present in both the manifest and the appinfo cache
  carries the expected `installdir`, `app_type`, and `launch_executable`, and a title
  absent from the appinfo cache carries `app_type: None`, `launch_executable: None`
  with no error.

**Checkpoint**: `InstalledTitle` carries all three new fields correctly; user story
work can now begin.

---

## Phase 3: User Story 1 - A soundtrack no longer masquerades as a capturable game (Priority: P1) 🎯 MVP

**Goal**: A Music-typed Steam app resolves to its real `steamapps/music/` directory
(no spurious warning) and is never registered as a capture target.

**Independent Test**: Fixture Steam tree with one Music app and one ordinary game;
`discover_in` and `SteamSource::discover` both behave per spec.

### Tests for User Story 1 ⚠️ (write first, confirm failing)

- [X] T006 [P] [US1] In `crates/fragcap-steam/src/library.rs`'s test module, add a
  fixture-tree test: a title with `app_type == Some("Music")` (or a case variant, e.g.
  `"music"`) installed under `steamapps/music/<installdir>` resolves `install_dir` to
  that real path with no read/detection error, while a same-tree ordinary
  `common`-installed title's resolution is unchanged (covers spec acceptance
  scenarios 1 and 2).
- [X] T007 [P] [US1] In `crates/fragcap/src/discovery.rs`'s test module, add a test:
  `SteamSource::discover` over a fixture root with one Music-typed installed title and
  one ordinary game produces exactly one candidate (the game); the Music title is
  counted under `account.considered` and `account.considered_not_a_game`, and
  `discovery.account.is_conserved()` is `true`.

### Implementation for User Story 1

- [X] T008 [US1] In `crates/fragcap-steam/src/library.rs`, `read_manifest` (or its
  caller in `read_library_titles`, wherever the app-type lookup is threaded per T004)
  joins `steamapps/music/<installdir>` when `app_type` case-insensitively equals
  `"music"`, else `steamapps/common/<installdir>` exactly as today (depends on T002-T004).
- [X] T009 [US1] In `crates/fragcap/src/discovery.rs`, `SteamSource::discover` skips a
  title whose `app_type` case-insensitively equals `"music"` entirely: increments
  `account.considered` and `account.considered_not_a_game`, emits no `CandidateTarget`
  for it, and continues to the next title (depends on T003/T004 for `app_type`; must
  not disturb the existing `parse_failed`/classification logic for every other title).
  Also update `DiscoveryAccount::considered_not_a_game`'s doc comment in
  `crates/fragcap-targets/src/source.rs` to note this second use (a Steam title
  excluded by app type, not only a known-root directory matching no signature), so a
  future reader of the account does not have to infer the second cause.
- [X] T010 [US1] Run T006 and T007 and confirm both pass; run the existing
  `crates/fragcap-steam` and `crates/fragcap` test suites in full to confirm no
  regression to an ordinary `common`-installed title's resolution or classification.

**Checkpoint**: User Story 1 is fully functional and independently testable. This is
the MVP slice: `cargo test -p fragcap-steam -p fragcap` is green and a music app no
longer produces a warning or a bogus row.

---

## Phase 4: User Story 2 - A dead registration says so instead of pretending to be healthy (Priority: P1)

**Goal**: A registered target whose `install_root` no longer exists renders with a
warning-colored note in `fragcap targets`, without ever mutating the registration, and
unaffected rows stay byte-identical.

**Independent Test**: A fixture store with one target whose `install_root` points at a
nonexistent path; list targets in every color mode; confirm only that row changes and
the registration is untouched afterward.

### Tests for User Story 2 ⚠️ (write first, confirm failing)

- [X] T011 [P] [US2] In `crates/fragcap-cli/tests/cli_targets.rs`, add a golden test:
  a target with a recorded, nonexistent `install_root` renders its row's SENSITIVITIES
  cell prefixed with `install folder not found` (plain text, no ANSI, matching how the
  existing goldens already force `NO_COLOR`/non-terminal output); a second target in
  the same fixture with no `install_root` recorded renders with no such note (covers
  acceptance scenarios 1 and 3 in plain-text form; scenario 2's colorized form is
  covered by T012).
- [X] T012 [P] [US2] In `crates/fragcap-cli/src/color.rs`'s (new file's) test module,
  add a unit test asserting the shared `WARN`/`RESET` constants match doctor's
  existing values exactly (`\x1b[33m`/`\x1b[0m`) so the extraction in T013 is
  byte-identical to what `doctor` prints today.
- [X] T013 [P] [US2] In `crates/fragcap-targets/src/readiness.rs`'s test module, add
  unit tests for `install_presence`: `Present` for an existing path (a temp
  directory), `Missing` for a recorded nonexistent path, `NotRecorded` for
  `install_root: None`. Also assert (SC-005/FR-010) that computing
  `install_presence` over a target, then re-reading that same target back from an
  in-memory `Store`, yields a `TargetEntry` equal to what was inserted: the
  derivation never writes anything back to the store.

### Implementation for User Story 2

- [X] T014 [P] [US2] Create `crates/fragcap-cli/src/color.rs`: `pub(crate) fn
  use_color() -> bool` (the body moved verbatim from
  `crates/fragcap-cli/src/commands/doctor.rs`), and `pub(crate) const WARN: &str =
  "\x1b[33m"`, `pub(crate) const RESET: &str = "\x1b[0m"` (moved from
  `crates/fragcap-cli/src/doctor/mod.rs`'s `Status::Warn` arm and `ANSI_RESET`). Wire
  the new module into `crates/fragcap-cli/src/lib.rs` (or wherever modules are
  declared).
- [X] T015 [US2] Update `crates/fragcap-cli/src/commands/doctor.rs` and
  `crates/fragcap-cli/src/doctor/mod.rs` to call `crate::color::use_color()` and the
  shared `WARN`/`RESET` constants instead of their own private copies, deleting the
  now-duplicate definitions. Run the existing doctor tests to confirm zero output
  change (depends on T014).
- [X] T016 [US2] In `crates/fragcap-targets/src/readiness.rs`, add `pub enum
  InstallPresence { Present, Missing, NotRecorded }` and `pub fn
  install_presence(entry: &TargetEntry) -> InstallPresence` per data-model.md (depends
  on nothing beyond the existing `TargetEntry::install_root`).
- [X] T017 [US2] In `crates/fragcap-cli/src/commands/targets.rs`, `render_table`:
  compute `install_presence` per row; when `Missing`, prefix the SENSITIVITIES value
  with `install folder not found` (joined with `; ` to any existing non-`-` content,
  replacing a bare `-` outright), wrapped in `crate::color::WARN`/`RESET` when
  `crate::color::use_color()` is true, plain otherwise. Every other row's rendering
  path is untouched (depends on T014, T016).
- [X] T018 [US2] In `crates/fragcap-cli/src/commands/targets.rs`, `hero_listing`'s
  next-command selection: skip a row whose `install_presence` is `Missing` when
  choosing the suggested `fragcap capture <n>` position (falls through to the next
  `Ready` row, or the first row if none qualify) (depends on T016).
- [X] T019 [US2] Run T011-T013 and confirm they pass; run the full existing
  `crates/fragcap-cli` golden test suite to confirm every row not in the missing-root
  state is still byte-identical, in every color mode.

**Checkpoint**: User Story 2 is fully functional and independently testable, and does
not depend on User Story 1 or 3.

---

## Phase 5: User Story 3 - A renamed title can be found by the name the user actually sees (Priority: P2)

**Goal**: A target's raw installdir and observed launch executable are stored
verbatim; selector resolution matches on all three names; `targets show` surfaces a
genuine (non-cosmetic) divergence; `&` expands to `and` in handle derivation.

**Independent Test**: Register a candidate whose three names diverge; resolve it by a
substring of each; confirm the divergence note appears only for the genuinely
different pair.

### Tests for User Story 3 ⚠️ (write first, confirm failing)

- [X] T020 [P] [US3] In `crates/fragcap-targets/src/handle.rs`'s test module, add a
  test: `normalize("Trapped with Ivy & Piper")` (and `derive_handle` with the same
  input) yields `"trapped_with_ivy_and_piper"`, and `normalize("Warhammer 40,000:
  ...")`-style names with a bare comma/colon are unaffected by the new step.
- [X] T021 [P] [US3] In `crates/fragcap-targets/src/register.rs`'s test module, add a
  test: registering a `CandidateTarget` with `folder_name` and `executable_hint` set
  stores both verbatim on the resulting `TargetEntry`; registering one with both
  `None` leaves both `None` on the entry (no fabrication).
- [X] T022 [P] [US3] In `crates/fragcap-targets/src/selector.rs`'s test module, add
  tests: a token equal to an exact handle or exact name resolves via the existing
  tiers unchanged; a substring of `folder_name` or `executable_hint` alone resolves
  via the new third tier; two targets whose exact names collide still resolve
  `Ambiguous` at tier 2 (unchanged); a token matching two different targets only by
  substring resolves `Ambiguous` at tier 3.
- [X] T023 [P] [US3] In `crates/fragcap-targets/src/readiness.rs`'s (or a new
  colocated module's) test module, add tests for `name_divergence`: identical after
  normalization → `None`; a casing/whitespace-only difference → `Cosmetic`; a
  substring/prefix relationship (truncation) → `Cosmetic`; genuinely different words
  (the `Trapped with Ivy & Piper` / `Escape from Ivy & Piper` case) → `Semantic`; a
  target with `folder_name: None` → `None`.

### Implementation for User Story 3

- [X] T024 [P] [US3] In `crates/fragcap-targets/src/handle.rs`, add the `&` → `" and "`
  expansion step to `normalize`, positioned before the apostrophe/quote-deletion step
  (per data-model.md/research.md R-11), and update the doc comment's numbered step
  list to match.
- [X] T025 [US3] In `crates/fragcap-targets/src/entry.rs`, add `pub folder_name:
  Option<String>` and `pub executable_hint: Option<String>` to `TargetEntry`.
- [X] T026 [US3] In `crates/fragcap-targets/src/schema.rs`, bump `SCHEMA_VERSION` to
  `8`, add `folder_name TEXT` and `executable_hint TEXT` to the `DDL` constant's
  `targets` table, and add `MIGRATE_7_TO_8` with the two additive `ALTER TABLE`
  statements (depends on T025).
- [X] T027 [US3] In `crates/fragcap-targets/src/store.rs`, apply the `MIGRATE_7_TO_8`
  step in `Store::open`'s migration chain, and extend every `targets` row
  reader/writer (`insert_target`, `targets`, `target_by_handle`,
  `target_by_stable_id`/`target`, and any other `SELECT`/`INSERT` touching the table)
  to read and bind the two new columns (depends on T026).
- [X] T028 [P] [US3] In `crates/fragcap-targets/src/source.rs`, add `pub folder_name:
  Option<String>` and `pub executable_hint: Option<String>` to `CandidateTarget`.
- [X] T029 [US3] In `crates/fragcap-targets/src/register.rs`, `register_candidate`
  copies `candidate.folder_name`/`candidate.executable_hint` onto the new
  `TargetEntry` fields (depends on T025, T028).
- [X] T030 [US3] In `crates/fragcap/src/discovery.rs`, `SteamSource::discover` sets
  `folder_name: Some(title.installdir.clone())` and `executable_hint:
  title.launch_executable.clone()` on every produced `CandidateTarget` (depends on
  T003, T028).
- [X] T031 [US3] In `crates/fragcap-targets/src/store.rs`, add `targets_by_substring(&self,
  needle: &str) -> Result<Vec<TargetEntry>, TargetsError>`: case-insensitive (Unicode
  `to_lowercase`, matching `targets_by_name`'s existing fold) substring match against
  `name`, `folder_name`, and `executable_hint`, each target counted once (depends on
  T027).
- [X] T032 [US3] In `crates/fragcap-targets/src/selector.rs`, `resolve_positional`
  gains the third tier: when the exact-handle and exact-name tiers both miss, call
  `targets_by_substring` and map 0/1/>1 to `NoMatch`/`Resolved`/`Ambiguous` exactly as
  the exact-name tier already does (depends on T031).
- [X] T033 [P] [US3] In `crates/fragcap-targets/src/readiness.rs` (or a new colocated
  module), add `pub enum NameDivergence { None, Cosmetic, Semantic }` and `pub fn
  name_divergence(entry: &TargetEntry) -> NameDivergence` per data-model.md, reusing
  `crate::handle::normalize` (depends on T025).
- [X] T034 [US3] In `crates/fragcap-cli/src/commands/targets.rs`, `print_target`
  (the `targets show` renderer) prints both names in one note when
  `name_divergence(t) == Semantic` (depends on T033).
- [X] T035 [P] [US3] In `crates/fragcap-targets/src/targets_export.rs`, add
  `folder_name`/`executable_hint` as optional export keys, emitted only when present,
  following the `detection_scan` precedent; update `import.rs` to accept them
  (depends on T025).
- [X] T036 [US3] Run T020-T023 and confirm they pass; run the full
  `crates/fragcap-targets`, `crates/fragcap`, and `crates/fragcap-cli` test suites to
  confirm no regression to existing exact handle/name resolution or export/import
  round-tripping.

**Checkpoint**: All three user stories are independently functional.

---

## Phase 6: Polish & Cross-Cutting Concerns

- [X] T037 [P] Update `docs/fragcap-specification.md`'s schema-version reference to 8
  and, if Appendix A carries a handle-derivation vector table, update every
  `&`-bearing vector to match the new expansion (search for literal `&` in the
  appendix and in `crates/fragcap-targets/src/handle.rs`'s existing test vectors,
  per research.md R-11).
- [X] T038 [P] Add one `changelog.d/` feature fragment summarizing the three fixes,
  and one `changelog.d/*.decisions.md` fragment recording D-3 (dedicated columns
  instead of reusing `launch_entries`) and D-5 (the shared `crate::color` module
  extraction) from plan.md's decision log.
- [X] T039 Run `FRAGCAP_UPDATE_FIXTURES=1 cargo test -p fragcap-capture --test corpus`
  only if this slice's changes touch any committed fixture corpus (expected: no, this
  slice does not touch pcap/pcapng fixtures) - confirm and skip if not applicable.
- [X] T040 Run `cargo xtask ci` in the foreground to completion and read its output
  (fmt, clippy, test --workspace --locked, lint, deps, license).
- [X] T041 Walk through `quickstart.md` end to end and confirm every numbered scenario
  matches its expected output.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: Depends on Setup. Blocks User Story 1 and the
  Steam-sourced parts of User Story 3 (T030 depends on T003/T004).
- **User Story 1 (Phase 3)**: Depends on Foundational (T002-T004).
- **User Story 2 (Phase 4)**: Depends on nothing beyond the existing codebase; does
  not depend on Foundational or on User Story 1. Can be built in parallel with Phase 3.
- **User Story 3 (Phase 5)**: `handle.rs`/`entry.rs`/`schema.rs`/`store.rs`/`source.rs`
  work (T020, T021, T024-T029, T031-T033, T035) depends on nothing beyond the existing
  codebase and can start immediately after Setup; only T030 (wiring `SteamSource`)
  depends on Foundational (T003/T004).
- **Polish (Phase 6)**: Depends on all three user stories being complete.

### Parallel Opportunities

- T002 and T003 touch different files and can run in parallel; T004 depends on both.
- User Story 2 (Phase 4) has no dependency on User Story 1 (Phase 3) or the
  Steam-specific parts of User Story 3, and can be implemented and merged
  independently.
- Within User Story 3, the handle (T020, T024), export (T035), and
  divergence-derivation (T023, T033) work are independent of the schema/store/selector
  chain (T025-T027, T031-T032) and can proceed in parallel.

## Implementation Strategy

### MVP First

1. Phase 1 (Setup) -> Phase 2 (Foundational) -> Phase 3 (User Story 1, #166). This
   alone fixes the most concrete, reproducible defect and is independently shippable.

### Incremental Delivery

2. Phase 4 (User Story 2, #167) next: no dependency on Phase 3, addresses the
   findability consequence of the same underlying condition class.
3. Phase 5 (User Story 3, #173) last: the largest surface (schema migration, selector
   resolution), and the one issue explicitly framed as a findability improvement
   rather than a correctness defect.
4. Phase 6 (Polish) closes out documentation, changelog, and the full gate.
