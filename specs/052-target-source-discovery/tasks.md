---

description: "Task list for S052: TargetSource discovery seam and discovery tiers"
---

# Tasks: TargetSource discovery seam and discovery tiers

**Input**: Design documents from `specs/052-target-source-discovery/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Included. This repository's evidence discipline (claims require a run)
and the spec's fixture-driven acceptance scenarios make each story test-shaped;
every source test asserts `DiscoveryAccount::is_conserved()` (P-4).

**Organization**: By user story (US1 P1, US2 P2, US4 P2, US3 P3), each an
independently testable increment. US4 (volume eligibility) precedes US2 in file
order because US2's cross-volume walk consults it, but US2's walk stays
independently testable with an all-eligible fixture inventory.

**Environment note**: SQLite-backed crates build under the GNU host toolchain here
(`cargo +1.96.0-x86_64-pc-windows-gnu test ...`); the canonical gate is
`cargo xtask ci` under the pinned MSVC toolchain in CI.

## Phase 1: Setup (Shared Infrastructure)

- [X] T001 Scaffold the discovery modules in `crates/fragcap-targets/src/`: create empty `source.rs`, `classifier.rs`, `volume.rs`, and `sources/{mod.rs,known_roots.rs,directory.rs,interactive.rs}`; declare them in `lib.rs`; create the facade stub `crates/fragcap/src/discovery.rs` and declare it in `crates/fragcap/src/lib.rs`. All behind the existing default-off `targets` feature.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The seam, candidate, and account shapes every source story needs.

**⚠️ CRITICAL**: No source story can begin until this phase is complete.

- [X] T002 Define `CandidateTarget` (identity, display_name, fidelity reusing `fragcap_profile::FidelityTier`, classification reusing `catalog::Classification` with `unknown`, source_name) and `Discovery { candidates, account }` in `crates/fragcap-targets/src/source.rs` per data-model.md.
- [X] T003 Define the `TargetSource` trait (`name`, `discover -> Result<Discovery, TargetsError>`, `default_fidelity`) in `crates/fragcap-targets/src/source.rs` per contracts/target-source.md.
- [X] T004 Define `DiscoveryAccount` (produced, parse_failed, declined_by_user, considered_not_a_game, volume_skipped, access_error, considered) with `is_conserved()` in `crates/fragcap-targets/src/source.rs` per contracts/discovery-account.md.
- [X] T005 Implement test-only `FixtureSource` (canned candidates + account) in `crates/fragcap-targets/src/source.rs` (behind `#[cfg(test)]` or a test-support gate). T002-T005 share `source.rs`, so they are sequential, not parallel.
- [X] T006 Re-export the new public surface (`TargetSource`, `CandidateTarget`, `Discovery`, `DiscoveryAccount`) from `crates/fragcap-targets/src/lib.rs`.
- [X] T007 [P] Seam test (SC-006): a `FixtureSource` added to a discovery run yields its candidates and a conserved account with no change to any driver, in `crates/fragcap-targets/tests/source_seam.rs`.

**Checkpoint**: The seam compiles and the account conserves; source stories can begin.

---

## Phase 3: User Story 1 - Steam through the shared seam (Priority: P1) 🎯 MVP

**Goal**: The existing Steam walk becomes `SteamSource: TargetSource`, one
candidate per installed title at heuristic-unverified, appid joined to catalog,
with zero observable change (FR-006).

**Independent Test**: Drive `SteamSource` against committed Steam metadata
fixtures; assert candidate parity with the pre-refactor walk, catalog join, and a
counted `parse_failed` for a corrupt section.

- [X] T008 [P] [US1] Tests in `crates/fragcap/tests/steam_source.rs`: one candidate per installed title at heuristic-unverified; appid present in catalog fixture carries classification; absent appid is `unknown` and still produced; corrupt appinfo section counted `parse_failed` with the rest surviving; candidate-set parity with the prior `fragcap-steam` walk for the same fixture; `is_conserved()` holds.
- [X] T009 [US1] Implement `SteamSource` in `crates/fragcap/src/discovery.rs`: wrap `fragcap-steam` `library`/`appinfo` (unchanged) and map each `InstalledTitle` to a `CandidateTarget` at heuristic-unverified fidelity, carrying the appid as identity.
- [X] T010 [US1] Join each candidate's appid against `catalog.db` via `fragcap-targets::Store`/`catalog`, stamping the catalog `Classification`, `unknown` when absent (never dropped) (FR-005, P-9).
- [X] T011 [US1] Wire the `DiscoveryAccount` (count produced and `parse_failed`; conserve) in `SteamSource::discover`.
- [X] T012 [US1] Expose the facade discovery composition to the CLI so the target listing includes `SteamSource` output; keep `SteamWalkerProvider` in place (S054 owns the capture entry point).

**Checkpoint**: A Steam install lists its games through the seam, unchanged. MVP.

---

## Phase 4: User Story 4 - Volume eligibility keeps the cross-volume walk safe (Priority: P2)

**Goal**: A persistent, allowlist-shaped volume eligibility table in `local.db`,
permissive-seeded at first run, so a later-appearing or misreporting volume is not
walked without opt-in (FR-016/FR-016a/FR-017).

**Independent Test**: With a fixture inventory, assert first-run seeding marks
present volumes eligible, an excluded volume is enumerated zero times and counted
`volume_skipped`, re-inclusion re-enables it, and each decision's reason is
statable.

- [X] T013 [P] [US4] Tests in `crates/fragcap-targets/tests/volume_eligibility.rs`: first-run seeds present fixed volumes `seeded-first-run`; a volume appearing after seeding is unseen (not walked) until `user-added`; a `user-excluded` volume is never returned by `eligible_volumes()`; re-include works; the migration applies from a v3 store; `reason` is recoverable.
- [X] T014 [US4] Bump `SCHEMA_VERSION` 3 -> 4, add the `volume_eligibility` DDL (volume_id PK, mount_point, drive_type, eligible, reason, first_seen) and `MIGRATE_3_TO_4` in `crates/fragcap-targets/src/schema.rs` per data-model.md.
- [X] T015 [US4] Add the sequential migration step (v3 -> v4) to the store migration in `crates/fragcap-targets/src/store.rs`, preserving the v1->2->3 chain.
- [X] T016 [P] [US4] Define `Volume` (stable `identity` via volume GUID path, `mount_point`, `drive_type`) and the `VolumeInventory` seam (`fixed_volumes`) plus a fixture inventory in `crates/fragcap-targets/src/volume.rs` per research.md D2/D3.
- [X] T017 [US4] Implement `Store` eligibility ops in `crates/fragcap-targets/src/store.rs`: `seed_volume_eligibility(&[Volume])` (idempotent, first-run only), `eligible_volumes()`, `set_volume_eligibility(volume_id, eligible, reason)`, `volume_eligibility(volume_id)`.
- [X] T018 [US4] Implement the real `cfg(windows)` `VolumeInventory` adapter in `crates/fragcap/src/discovery.rs` over `GetLogicalDrives`/`GetDriveTypeW`/volume-GUID identity via the already-pinned `windows-sys` 0.36 (no lock delta).

**Checkpoint**: Eligibility persists and gates enumeration; reasons are auditable.

---

## Phase 5: User Story 2 - A machine without Steam still shows games (Priority: P2)

**Goal**: `KnownRootsSource` walks the fixed game-only root list across every
eligible fixed volume, classifying by directory shape with stop-on-hit descent
(FR-007/008/009/010/014/015).

**Independent Test**: With a fixture inventory and directory tree plus a fixture
classifier, assert cross-volume candidates, missing-root tolerance, stop-on-hit,
and eligibility-gated skips, all conserved.

- [X] T019 [P] [US2] Tests in `crates/fragcap-targets/tests/known_roots.rs`: two game dirs on volume A + one on volume B yield three candidates (cross-volume); a known root absent on a volume yields nothing and no error; a classifier `Hit` at top level emits one candidate and does not descend into its subtree; a `Miss` sibling is counted `considered_not_a_game`; an ineligible volume is enumerated zero times and counted `volume_skipped`; `is_conserved()` holds.
- [X] T020 [P] [US2] Define the `DirectoryClassifier` seam (`classify -> Hit{..}|Miss`) and a trivial/fixture classifier in `crates/fragcap-targets/src/classifier.rs` per research.md D6 (the signature matcher is S053).
- [X] T021 [P] [US2] Encode the fixed v0.5.0 known-root list (the 11 roots from FR-007) as a constant in `crates/fragcap-targets/src/sources/known_roots.rs`.
- [X] T022 [US2] Implement `KnownRootsSource` in `crates/fragcap-targets/src/sources/known_roots.rs`: for each eligible volume x each known root, walk with an injected directory lister, test each directory via the classifier, emit one candidate and stop on `Hit`, count `considered_not_a_game` on `Miss`; wire the account (FR-009: never enumerate executables first).
- [X] T023 [US2] Integrate eligibility: `KnownRootsSource` consults `eligible_volumes()` (injected/queried) and skips ineligible volumes, counting `volume_skipped`; a first run seeds the table via `seed_volume_eligibility` before walking.

**Checkpoint**: A no-Steam machine with a known root lists games (SC-001).

---

## Phase 6: User Story 3 - The user points discovery at a place they know (Priority: P3)

**Goal**: `DirectorySource` (one path -> one candidate) and `InteractiveSource`
(confirmation -> authored | declined) back `targets scan`/`targets add`
(FR-011/012/013), with persist-on-first-use (FR-021).

**Independent Test**: Fixture path yields one candidate; scripted yes stamps
authored; scripted no counts `declined_by_user`; non-interactive yields none and
says why; CLI `scan`/`add` behave accordingly.

- [X] T024 [P] [US3] Tests: `DirectorySource`/`InteractiveSource` behavior (authored stamp, `declined_by_user`, non-interactive no-candidate, conservation) in `crates/fragcap-targets/tests/user_pointed.rs`; CLI `targets scan`/`targets add` in `crates/fragcap-cli/tests/cli_targets.rs`.
- [X] T025 [P] [US3] Implement `DirectorySource` (one path, at most one candidate, source default fidelity) in `crates/fragcap-targets/src/sources/directory.rs`.
- [X] T026 [US3] Implement `InteractiveSource` wrapping `DirectorySource` with an injected confirmation seam: accept -> stamp `authored`; reject -> count `declined_by_user`; non-interactive context -> no candidate with a stated reason, in `crates/fragcap-targets/src/sources/interactive.rs`.
- [X] T027 [US3] Wire `targets scan <dir>` in `crates/fragcap-cli/src/commands/targets.rs` to `DirectorySource`/`InteractiveSource` and list candidates.
- [X] T028 [US3] SCOPE: `targets scan <dir>` wires `DirectorySource` (T027); authoring + persist-on-first-use is provided by the existing name-based `targets add`. The interactive `DirectorySource`-backed `add <exe>` (one-step scan->confirm->author) is folded into the S055 hero command, consistent with the S051 deferral discipline. `InteractiveSource`/`Confirm` seam is built and unit-tested; its CLI console wiring lands in S055.

**Checkpoint**: All four stories independently functional.

---

## Phase 7: Polish & Cross-Cutting Concerns

- [X] T029 [P] Add glossary entries (P-6) in `docs/glossary/` for: TargetSource, CandidateTarget, discovery tier, known-roots source, directory source, interactive source, discovery account, volume eligibility table, descent stop-on-hit; regenerate the glossary index and confirm the docs linter passes.
- [X] T030 [P] Reconcile master specification section 7 with what shipped (P-11) in `docs/fragcap-specification.md`: the `TargetSource` seam, the three tiers, the descent contract, and the volume eligibility layer; and record the three deferred v0.6.0 volume hazards (cloud placeholder hydration, reparse-point loops, within-volume skip list) so FR-018's "the specification MUST record" clause is satisfied in the master spec, not only the slice contract.
- [X] T031 Add changelog fragment `changelog.d/S052-target-source-discovery.added.md` beginning with `<!-- spec-impact: 7 -->`; record the two clarified decisions (permissive-seed allowlist; surface-live/persist-on-use) and the facade-composition placement.
- [X] T032 [P] Update the AGENTS.md current-state note: no new dependency, no new inter-crate edge; the discovery seam and tiers; `SteamWalkerProvider` retained pending S054.
- [X] T033 Run every quickstart.md scenario; confirm `is_conserved()` is asserted in every source test (the P-4 standing guard).
- [ ] T034 Run `cargo xtask ci` under the CI/MSVC toolchain. PENDING CI (this dev machine has no MSVC linker). VERIFIED LOCALLY under the GNU host toolchain: `cargo fmt --all --check` clean; `cargo clippy` clean (default features and `fragcap`/`fragcap-cli` with `targets`); `cargo test --workspace` green (0 failures); `cargo xtask lint` clean (P-1: no `OpenProcess`/`ReadProcessMemory`/`WriteProcessMemory`); `cargo xtask deps` clean; `cargo xtask license` clean. The remaining CI-only part is clippy `--all-features -D warnings` and the full MSVC build/link (the `live`/`net` features need npcap/http at link time).
- [ ] T035 `cargo xtask deps` reports the graph matches the architecture of record (no new inter-crate edge) - VERIFIED. `Cargo.lock` delta is one dependency-edge line (`windows-sys 0.36.1` under `fragcap`), no new package - VERIFIED. `cargo tree`/`cargo deny`/`cargo xtask msrv` PENDING a linker-capable machine / CI; no new crate was added, so the license and MSRV surface is unchanged from what CI already verified.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: none; start immediately.
- **Foundational (Phase 2)**: after Setup; BLOCKS all stories.
- **US1 (Phase 3)**: after Foundational; independent of US2/US3/US4.
- **US4 (Phase 4)**: after Foundational; independent (fixture inventory).
- **US2 (Phase 5)**: after Foundational; consults US4's eligibility store, so US4
  lands first, but US2's walk is testable with an all-eligible fixture even so.
- **US3 (Phase 6)**: after Foundational; independent of the automatic tiers.
- **Polish (Phase 7)**: after all desired stories.

### Within Each User Story

- Tests first (write, watch fail), then models/seams, then sources, then CLI wiring.
- Conservation (`is_conserved`) asserted in every source test.

### Parallel Opportunities

- Foundational: T002-T006 all touch `source.rs`/`lib.rs` and are sequential; only
  T007 (a separate test file) is [P] against them.
- US2: T020 (classifier), T021 (root list) are [P]; T022 depends on both.
- US4: T013 (tests), T016 (volume seam) are [P]; T017/T018 depend on T014/T015.
- Polish: T029, T030, T032 are [P] (distinct docs).

---

## Implementation Strategy

### MVP First (User Story 1)

1. Phase 1 Setup -> Phase 2 Foundational (seam + account) -> Phase 3 US1.
2. STOP and VALIDATE: Steam lists through the seam with parity. Demo the MVP.

### Incremental Delivery

1. Foundation ready (seam conserves).
2. US1 (Steam parity) -> validate -> demo.
3. US4 (eligibility) -> validate.
4. US2 (known-roots cross-volume, gated by eligibility) -> validate SC-001.
5. US3 (user-pointed + CLI) -> validate.
6. Polish: glossary (P-6), spec section 7 (P-11), changelog, `cargo xtask ci`.

---

## Notes

- [P] = different files, no incomplete-task dependency; tasks touching the same
  file (e.g. `source.rs`, `store.rs`) are sequential even when conceptually parallel.
- Every source test asserts `is_conserved()`; a new discard path with no counter
  fails there rather than shipping silently (P-4).
- No new crate and no new inter-crate edge: the facade already depends on both
  `fragcap-steam` and `fragcap-targets`; `windows-sys` 0.36 is already pinned.
- `SteamWalkerProvider` stays until S054's capture rework consumes the source form.
