---

description: "Task list for S053: Data-driven detection signatures"
---

# Tasks: Data-driven detection signatures

**Input**: Design documents from `/specs/053-detection-signatures/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Included. This repository treats invariant tests (conservation,
stop-on-hit, neutral-evidence, fidelity) as load-bearing, and the spec's acceptance
criteria are test-shaped, so test tasks are generated.

**Organization**: By user story. US1 (the classifier seam) is the MVP; US2
(signatures as data) and US3 (the standalone command) build on the shared
foundation. The embedded-ruleset removal is sequenced last (Phase 6) so the build
stays green while consumers are repointed.

## Path Conventions

Single Rust workspace. Crates under `crates/`: `fragcap-profile` (matcher +
Signature type), `fragcap-targets` (table, seed, classifier), `fragcap-steam`
(scaffold), `fragcap` (facade), `fragcap-cli` (commands).

---

## Phase 1: Setup

**Purpose**: Module and asset skeletons so later tasks land in place.

- [X] T001 Add `crates/fragcap-targets/src/signatures.rs` (empty module) and wire
  `mod signatures;` in `crates/fragcap-targets/src/lib.rs`; create the
  `crates/fragcap-targets/assets/` directory for the seed document.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The shared matcher (fragcap-profile) and the signature store
(fragcap-targets) that all three user stories need. New code lands alongside the
existing `CompiledRuleset`, which is not removed until Phase 6.

**CRITICAL**: No user story work can begin until this phase is complete.

- [X] T002 [P] In `crates/fragcap-profile/src/technologies.rs`, add the `Signature`,
  `SignatureKind` (filename / directory-shape / pe-version-string / binary-marker),
  and `SignatureCategory` (engine / anti-cheat / drm) value types and the
  confidence-to-`FidelityTier` mapping (D4), independent of any database.
- [X] T003 In `crates/fragcap-profile/src/technologies.rs`, implement the generic
  per-directory match primitive over `&[Signature]` for the `filename` and
  `directory-shape` kinds, returning `DetectionFinding`s (category, product,
  evidence, fidelity). No per-product branches (FR-003).
- [X] T004 In `crates/fragcap-profile/src/technologies.rs`, add the hand-rolled PE
  version-resource reader (DOS/PE headers, section table, `.rsrc`, `VS_VERSIONINFO`
  string fields) and wire the `pe-version-string` kind into the primitive; reads the
  binary's own bytes only, no OS call, no process memory (FR-008b, D3).
- [X] T005 In `crates/fragcap-profile/src/technologies.rs`, implement the
  full-inventory entry point over `&[Signature]`: the bounded tree walk, dedup per
  (category, product), grouping by category, and unreadable-subtree surfacing
  (reusing the existing walk discipline).
- [X] T006 [P] In `crates/fragcap-profile/src/technologies.rs` tests (or a sibling
  test module), cover the matcher: filename and directory-shape matches, a
  pe-version-string match against a minimal PE image built by a test helper
  (fixtures are generated), fidelity from confidence, unreadable surfacing, and that
  a finding carries no status/gate field.
- [X] T007 In `crates/fragcap-targets/src/schema.rs`, bump `SCHEMA_VERSION` from 4 to
  5, add the `signature` table DDL (with CHECK on category and kind) to `DDL`, and
  add `MIGRATE_4_TO_5` creating the table (D5).
- [X] T008 In `crates/fragcap-targets/src/store.rs`, apply `MIGRATE_4_TO_5` in the
  open/migrate path, and add `Store::load_signatures() -> SignatureSet` partitioning
  rows into applied / inert (binary-marker) / skipped (malformed pattern) with the
  `applied + inert + skipped == total` invariant.
- [X] T009 In `crates/fragcap-targets/src/signatures.rs`, implement
  `seed_signatures(store, source)` (idempotent: replace prior rows) and the row
  <-> `Signature` mapping, parsing the seed document with `serde_json`.
- [X] T010 [P] In `crates/fragcap-targets/tests/`, add a signatures store test:
  migration v4->v5 applies, `load_signatures` partitions and preserves the count
  invariant, and re-seeding is idempotent.

**Checkpoint**: matcher and signature store exist and are tested; user stories can
begin.

---

## Phase 3: User Story 1 - Detection runs automatically inside every discovery scan (Priority: P1) MVP

**Goal**: The real `DirectoryClassifier` classifies a directory by signature shape,
stops descent on a hit, and stamps the detected engine `verified`, inside every
discovery source with no separate user action.

**Independent Test**: Point a source at a fixture tree with an engine marker; one
candidate is emitted, engine is `verified`, descent stopped, account conserved.

- [X] T011 [US1] In `crates/fragcap-targets/src/classifier.rs`, add
  `SignatureClassifier` implementing the S052 `DirectoryClassifier` over a loaded
  `SignatureSet`: an engine-signature shape match returns `Hit { Game }` carrying the
  detected engine at `verified`, stops descent, and records any anti-cheat/DRM as
  neutral evidence; otherwise `Miss` (considered-not-a-game). Replaces
  `KnownRootChildIsGame` as the production classifier.
- [X] T012 [US1] In `crates/fragcap/src/discovery.rs` (and facade wiring), load
  signatures from `catalog.db` and pass the `SignatureClassifier` into the discovery
  sources' scan phase, replacing the S052 placeholder classifier (FR-006).
- [X] T013 [P] [US1] In `crates/fragcap-targets/tests/`, test the classifier through
  `FixtureSource`: a shape hit emits exactly one candidate, stamps the engine
  `verified`, and stops descent (SC-004); a miss is considered-not-a-game; the
  discovery account stays conserved (SC-007).
- [X] T014 [P] [US1] In `crates/fragcap/tests/`, test end-to-end discovery over a
  fixture tree: a locally detected engine is presented `verified` and outranks a
  remote catalog `heuristic-unverified` attribution for the same candidate (FR-009,
  SC-005).

**Checkpoint**: discovery classifies real installs by signature; MVP complete.

---

## Phase 4: User Story 2 - Detection capability refreshes as data (Priority: P2)

**Goal**: The Appendix B set ships as seedable data; a signature added to the table
is honored with no code change.

**Independent Test**: Seed the table; confirm all 16 products present; add one
filename/dir-shape row and confirm it is detected with no rebuild.

- [X] T015 [US2] Author `crates/fragcap-targets/assets/signatures.json` with the full
  Appendix B set (FR-002): engines Unity, Unreal, Source, Godot, CryEngine, RE
  Engine; anti-cheat EAC, BattlEye, Vanguard, mhyprot, GameGuard, Xigncode3; DRM
  Denuvo, Steam DRM, Arxan, VMProtect (Denuvo/Arxan/VMProtect as inert
  binary-marker rows).
- [X] T016 [US2] In `crates/fragcap-cli/src/cli.rs` and
  `crates/fragcap-cli/src/commands/targets.rs`, add the `targets seed-signatures
  --db <catalog.db>` subcommand seeding from the bundled document (offline),
  alongside `targets seed`/`seed-engine`. This is the catalog-seed-family path
  FR-005 names; the seed is idempotent (re-running reloads the same table).
- [X] T017 [P] [US2] In `crates/fragcap-cli/tests/` (or fragcap-targets tests), test
  SC-001 (every Appendix B product represented after a fresh seed), SC-002 (a new
  filename/dir-shape row is honored on the next scan with no code change), SC-003
  (each implemented-kind Appendix B product is detected from a fixture directory
  carrying its marker), and that the inert binary-marker count is surfaced.

**Checkpoint**: signatures are data, refreshable through the catalog seed path.

---

## Phase 5: User Story 3 - A researcher inventories an unknown binary directory (Priority: P2)

**Goal**: `technologies --path <dir>` reports the technologies from the table-backed
matcher as neutral evidence, without registering a target; the Steam scaffold uses
the same matcher via injected signatures.

**Independent Test**: Run `technologies --path` over a fixture with an anti-cheat and
a DRM marker; both listed as neutral facts, nothing framed as a reason not to
capture; unreadable subtree surfaced.

- [X] T018 [US3] In `crates/fragcap-cli/src/commands/technologies.rs`, repoint the
  command at the table-backed matcher: add a `--catalog-db` argument, load signatures
  from it, run the full-inventory matcher, and keep the neutral grouped output and
  the unreadable/empty handling (FR-010).
- [X] T019 [US3] In `crates/fragcap-steam/src/scaffold.rs`, change scaffold
  enrichment to take an injected `&[Signature]` (or a prebuilt matcher) instead of
  `CompiledRuleset::embedded()`; update the facade caller to supply signatures loaded
  from `catalog.db` (no new `fragcap-steam -> fragcap-targets` edge, D1).
- [X] T020 [P] [US3] In `crates/fragcap-cli/tests/` and `crates/fragcap-steam` tests,
  test the neutral-evidence output (FR-011), the unreadable-subtree warning, the
  empty-is-not-an-error case, and scaffold enrichment through injected signatures.

**Checkpoint**: all three stories independently functional.

---

## Phase 6: Removal, Polish, and Cross-Cutting Concerns

**Purpose**: Remove the replaced embedded ruleset (only after consumers are
repointed) and satisfy the constitution's ACTION gates.

- [X] T021 Remove the embedded detector: delete `CompiledRuleset`, `RULES_INI`,
  `RULES_LOCK`, and `SkippedPattern` from
  `crates/fragcap-profile/src/technologies.rs`; delete
  `crates/fragcap-profile/assets/steamdb/` (`rules.ini`, `rules.lock.json`,
  `THIRD_PARTY_NOTICES.md`); drop the corresponding re-exports from
  `crates/fragcap-profile/src/lib.rs` and `crates/fragcap/src/lib.rs`; remove the
  `sha256` module if the ruleset lock test was its only user (verify with a grep
  first). Depends on T012, T018, T019.
- [X] T022 [P] Add the neutral-evidence audit test (D9): assert no detection output
  path emits a status, color token, or gating wording for a detected anti-cheat or
  DRM product, and that a title with no online multiplayer mode is still presented as
  capturable (FR-011, FR-012).
- [X] T023 [P] Glossary (P-6): add detection signature, signature table, signature
  kind, signature category, generic signature matcher, and neutral evidence to
  `docs/glossary/`; regenerate the index (`bash scripts/lint-docs.sh fix`, then
  `check`).
- [X] T024 Master specification (P-11): reconcile sections 3.6, 8, and Appendix B
  with what shipped (data-driven signature table, three implemented kinds, the
  removed embedded ruleset, local-outranks-remote fidelity).
- [X] T025 Add `changelog.d/S053-detection-signatures.added.md` with the
  `<!-- spec-impact: N -->` header; record the two clarified decisions, the
  fragcap-profile/fragcap-targets crate placement, the catalog-seed mapping of
  `catalog update`, and the embedded-ruleset removal.
- [X] T026 Update `AGENTS.md`: current-state note and dependency-inventory narrative
  (no new crate, no new inter-crate edge, embedded SteamDB ruleset removed, schema
  version 5).
- [X] T027 Run the quickstart scenarios and the full gate in the foreground:
  `cargo xtask ci` on CI (MSVC); locally `cargo +1.96.0-x86_64-pc-windows-gnu test
  --workspace` plus `cargo fmt --all -- --check` and clippy. Confirm `is_conserved`
  is asserted in every source test.

---

## Dependencies & Execution Order

- **Setup (T001)**: no dependencies.
- **Foundational (T002-T010)**: after Setup; blocks all user stories. Within it:
  T002 before T003/T004/T005; T007 before T008; T008/T009 before T010.
- **US1 (T011-T014)**: after Foundational. T011 before T012; T013/T014 after T012.
- **US2 (T015-T017)**: after Foundational. T015 before T016/T017.
- **US3 (T018-T020)**: after Foundational. T018/T019 before T020.
- **Phase 6**: T021 after T012, T018, T019 (consumers repointed). T022-T027 after the
  stories they audit; T027 last.

### Parallel Opportunities

- T002 and (T007) touch different crates and can proceed together.
- T006 and T010 (tests in different crates) are parallel.
- US1, US2, and US3 are independent once Foundational completes and can be worked in
  parallel by different developers.
- T022, T023 are parallel (different files) within Phase 6.

---

## Implementation Strategy

### MVP First (User Story 1)

1. Phase 1 Setup.
2. Phase 2 Foundational (matcher + signature store).
3. Phase 3 US1 (classifier seam wired into discovery).
4. STOP and VALIDATE: discovery classifies a fixture install by signature, stamps
   `verified`, stops descent, account conserved.

### Incremental Delivery

US1 (classifier) -> US2 (Appendix B seed as data) -> US3 (standalone command +
scaffold) -> Phase 6 (remove the embedded ruleset, glossary, spec, changelog,
gate). Each story is independently testable from fixtures.

---

## Notes

- [P] = different files, no dependency on an incomplete task.
- The removal (T021) is deliberately last so every prior task keeps the build green.
- Never stage `.specify/feature.json` (local gitignored state).
- The operator merges the pull request; never self-merge (integration workflow).
