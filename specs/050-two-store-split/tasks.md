---

description: "Task list for S050: the two-store split"
---

# Tasks: The two-store split (catalog.db + local.db)

**Input**: Design docs in `specs/050-two-store-split/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/stores.md](contracts/stores.md)

**Tests**: Included (Rust `#[cfg(test)]` and the crate `tests/`). Run under
`cargo +1.96.0-x86_64-pc-windows-gnu`.

## Format: `[ID] [P?] [Story] Description`

---

## Phase 1: Setup

- [x] T001 Confirm branch `050-two-store-split`, clean tree apart from `specs/050-two-store-split/`.
- [x] T002 Confirm the gnu-host toolchain builds the SQLite crates (`cargo +1.96.0-x86_64-pc-windows-gnu build -p fragcap-targets`).

## Phase 2: Foundational (provider layering + paths, shared by all stories)

- [x] T003 In `crates/fragcap-targets/src/hint_provider.rs`, change `HintDatabaseProvider` to hold `Vec<Store>`; keep `new(store)` (single) and add `new_layered(stores: Vec<Store>)`; refactor the per-store lookup into a helper and make `provide` iterate stores in order, returning the first usable `Target`.
- [x] T004 [P] Add a `fragcap-targets` test proving layering: a title present only in the second store resolves; a title present in both resolves from the first; both-absent yields `None`.
- [x] T005 In `crates/fragcap-cli/src/paths.rs`, add `default_catalog_db_path`/`default_local_db_path` (via a shared `default_db_from(appdata, filename)`), and `catalog_db_path(flag)`/`local_db_path(flag)` with `FRAGCAP_CATALOG_DB`/`FRAGCAP_LOCAL_DB`; remove `HINT_DB_ENV`, `hint_db_path`, `default_hint_db_path`.
- [x] T006 [P] Update the `paths.rs` unit tests for the two-store helpers (default joins, flag-over-env).

## Phase 3: User Story 2 - Fresh install yields both stores without elevation (P1)

- [x] T007 [US2] In `crates/fragcap-cli/src/commands/run.rs`, generalize `ensure_default_hint_db` to `ensure_store(path, template)` and `bundled_hint_db_template` to `bundled_catalog_template` (sibling `catalog.db`); keep `copy_writable_hint_db` (rename to `copy_writable_store`).
- [x] T008 [US2] In `run.rs`, resolve both store paths (flag/env/default), bootstrap `catalog.db` from the template and `local.db` empty (both only when defaulted), each failure a warning that drops that path.
- [x] T009 [US2] Update the `run.rs` bootstrap tests to cover both stores: seeded catalog copy is writable; empty local created; a present store untouched; missing template creates empty.

## Phase 4: User Story 3 - Learned data to local.db, resolution parity (P2)

- [x] T010 [US3] In `run.rs`, accumulate learned launch data into `local.db` (call `accumulate_launch(local_path, ...)`).
- [x] T011 [US3] In `run.rs`, change `build_resolver` to open the present stores and register one `HintDatabaseProvider::new_layered(vec![local, catalog])` (local first); preserve absent (non-fatal) and unopenable (loud error) handling per store.
- [x] T012 [US3] Update/extend the `run.rs` resolver tests: a learned-only title resolves from local; a seed-only title resolves from catalog; an unopenable store is a loud error.

## Phase 5: User Story 1 - The trust boundary holds (P1)

- [x] T013 [US1] Add a CLI/integration test (in `crates/fragcap-cli/tests/`) proving: after a run that accumulates, `catalog.db` is byte-identical; replacing `catalog.db` leaves `local.db` byte-identical.

## Phase 6: CLI surface, maintainer commands, doctor

- [x] T014 In `crates/fragcap-cli/src/cli.rs`, replace `--hint-db` with `--catalog-db` and `--local-db` (updated doc comments); update `RunArgs` fields and all references.
- [x] T015 In `crates/fragcap-cli/src/commands/targets.rs`, point the maintainer commands at `catalog.db` (default path + flag); update its tests.
- [x] T016 In `crates/fragcap-cli/src/doctor/*` (probe, checks, mod), report both `catalog.db` and `local.db` (path + presence) instead of a single hint-db line; update `crates/fragcap-cli/tests/cli_doctor.rs`.

## Phase 7: Packaging (pinned) and docs

- [x] T017 In `crates/fragcap-cli/wix/main.wxs`, install the seed as `catalog.db` (component id, file name, source, comment header).
- [x] T018 In `.github/workflows/release.yml`, build/stage/archive/checksum `catalog.db` instead of `hint.db`; add `changelog.d/S050-catalog-db-packaging.decisions.md` (pinned artifact, `spec-impact: none`).
- [x] T019 Update `docs/fragcap-specification.md` store sections (15.x default paths and bundled-store prose, 24.5 packaging) to describe `catalog.db` + `local.db`; version currency only.
- [x] T020 [P] Add `catalog.db` and `local.db` glossary entries to `docs/glossary/platform-and-distribution.md`, update the hint-database entry, and regenerate `docs/glossary/index.md` (`scripts/lint-docs.sh fix`).
- [x] T021 [P] Reconcile `site/content/docs/getting-started.mdx` and `site/content/docs/reference/target-schema.mdx` references from `hint.db` to the two-store layout.

## Phase 8: Changelog + verify

- [x] T022 Add `changelog.d/S050-two-store-split.changed.md` (`spec-impact` naming the edited spec sections) and `changelog.d/S050-cli-store-flags.changed.md` (`spec-impact: none`).
- [x] T023 Run the gate set under the gnu-host toolchain: `fmt --all --check`, `clippy --workspace --all-targets`, `test --workspace`, `xtask lint`, `xtask spec`, `xtask docs check`. Fix anything reported.
- [x] T024 Run the [quickstart.md](quickstart.md) scenarios and confirm outcomes.
- [x] T025 Final text-hygiene pass over every edited file (no BOM/CRLF/trailing-WS/em-en dashes).

## Dependencies

- Foundational (T003, T005) blocks everything.
- US2 (bootstrap) precedes US3 (accumulate/resolve) and US1 (byte-identical test).
- CLI/doctor/packaging/docs (Phases 6-7) after the core behavior; independent of each other.
- Verify last.

## Notes

- `spec-impact` on every new changelog fragment (S049 gate).
- Only `release.yml` is a pinned artifact here (decisions fragment); WiX is not.
- No migration; an old `hint.db` is ignored.
