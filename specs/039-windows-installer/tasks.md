# Tasks: Windows installer (MSI) and hint-database default with first-run bootstrap

**Feature**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md) | **Branch**: `039-windows-installer`

Tests are written under the project's test-driven discipline for the binary
bootstrap (the only executable-logic change). The installer, release-workflow,
and documentation deliverables are verified by the release-tag run and the manual
checklist (quickstart.md), mirroring the live-capture honesty posture.

## Phase 1: Setup

- [ ] T001 Create the `wix/` directory and the `assets/` directory at the repo root (empty placeholders removed once their files land).

## Phase 2: Foundational (blocking prerequisites for all stories)

- [ ] T002 Amend `docs/fragcap-specification.md` section 24.5 (Artifacts) to list the portable archive now carrying `hint.db`, the loose `hint.db`, and the unsigned MSI, while keeping "ships no game profiles" and stating a hint database is not a game profile.
- [ ] T003 [P] Amend `docs/fragcap-specification.md` section 20.2 (No bundling) to state the obligation binds only the capture driver, and section 15.3 to document the per-user hint-database default `%APPDATA%\fragcap\hint.db` and its first-run bootstrap alongside the profile directory.
- [ ] T004 Write `changelog.d/039-windows-installer.decisions.md` (dated) authorizing the pinned `release.yml` change, the archive-contract amendment (extending `release-packaging.decisions.md`), the frozen UpgradeCode GUID, unsigned-by-design (#79 out of scope), the best-effort Defender exclusion, and default-on local accumulation.
- [ ] T005 [P] Add `assets/hint-seed.json`: a `kind:"export"` document with an empty `records` array (the barebones-database source), matching the shape of `crates/fragcap-targets/tests/fixtures/seed.json` but empty.
- [ ] T006 Verify offline that `fragcap targets import assets/hint-seed.json --db <tmp>` produces a valid v2 store that `fragcap targets export` round-trips to a valid empty `kind:"export"` document; confirm the store is a single at-rest file (no `-wal`/`-shm` side-car).

## Phase 3: User Story 1 - Hints work with zero configuration (Priority: P1)

**Goal**: A `run` with no hint-database option creates and consults a per-user
default database; explicit paths keep their current semantics.

**Independent test**: `paths.rs` and `run.rs` unit tests pass for default-path
resolution and the three bootstrap cases; an explicit absent path is neither
created nor fatal.

- [ ] T007 [P] [US1] Add failing unit tests in `crates/fragcap-cli/src/paths.rs` for `default_hint_db_path()`: returns `<APPDATA>\fragcap\hint.db` when `APPDATA` is set, `None` when unset (guard env-mutating tests appropriately).
- [ ] T008 [P] [US1] Add failing unit tests in `crates/fragcap-cli/src/commands/run.rs` for `ensure_default_hint_db(default, template)`: absent+no-template creates an empty schema store; absent+template copies the template; present leaves the file unchanged (tempdirs).
- [ ] T009 [US1] Implement `default_hint_db_path() -> Option<PathBuf>` in `crates/fragcap-cli/src/paths.rs`, mirroring `user_profile_dir()`; leave `hint_db_path` unchanged. Make T007 pass.
- [ ] T010 [US1] Implement the pure `ensure_default_hint_db(default, template) -> io::Result<()>` helper in `crates/fragcap-cli/src/commands/run.rs` (no-op if present; create parent then copy template else `Store::open` empty). Make T008 pass.
- [ ] T011 [US1] Wire the default into `crates/fragcap-cli/src/commands/run.rs::run`: layer `default_hint_db_path()` under the explicit source with a `from_default` flag, call `ensure_default_hint_db` only when `from_default` (template = `current_exe()` sibling `hint.db` if present), warn (never fatal) on error, then run the unchanged `accumulate_launch` and `build_resolver` sequence.
- [ ] T012 [P] [US1] Update the `--hint-db` doc comment in `crates/fragcap-cli/src/cli.rs` to state the new `%APPDATA%\fragcap\hint.db` default, that it is created on first `run` when absent, and that a supplied flag/env overrides it and (if absent) is not created.

## Phase 4: User Story 2 - One-click Windows install (Priority: P1)

**Goal**: An unsigned MSI installs per-machine, adds fragcap to the system PATH,
ships the database template, best-effort excludes its own directory from Defender,
and links the capture driver on exit.

**Independent test**: `cargo wix` builds the `.msi` from `main.wxs` where WiX is
available; the manual checklist (quickstart.md Tier 2) covers runtime behavior.

- [ ] T013 [US2] Author `wix/License.rtf` from `LICENSE` (RTF for the WixUI license screen), UTF-8, LF, no em/en dashes.
- [ ] T014 [US2] Author `wix/main.wxs`: per-machine `ProgramFiles64Folder\fragcap`; components for `fragcap.exe` (keypath + system-PATH `Environment` element, `Part=last`), the read-only `hint.db` template, `LICENSE`, `NOTICE`; frozen `UpgradeCode` GUID (generated once); `ProductVersion` from the crate version; `MajorUpgrade`; Add/Remove About URL.
- [ ] T015 [US2] Add the Defender custom actions to `wix/main.wxs`: deferred elevated `Add-MpPreference -ExclusionPath` on install and `Remove-MpPreference` on uninstall, `Return="ignore"`, `INSTALLDIR` passed via `CustomActionData`.
- [ ] T016 [US2] Add the UI to `wix/main.wxs`: `WixUI_InstallDir` with the exit-dialog optional checkbox opening `https://npcap.com` via `WixShellExec`, referencing `wix/License.rtf`.
- [ ] T017 [US2] Add the `cargo-wix` configuration (e.g. `[package.metadata.wix]` in `crates/fragcap-cli/Cargo.toml`) needed to build `main.wxs`, without bumping any version.

## Phase 5: User Story 3 - Choose what to download (Priority: P2)

**Goal**: A tagged release emits the portable zip (with `hint.db`), the unsigned
MSI, and a loose `hint.db`, each with a checksum.

**Independent test**: inspection of the `artifacts` job output at tag time; the
optional `msiexec /qn` smoke step.

- [ ] T018 [US3] In `.github/workflows/release.yml` (pinned) `artifacts` job, add steps after Build: install WiX v3 + `cargo install cargo-wix`; build the barebones DB via `fragcap targets import assets/hint-seed.json --db <stage>/hint.db`.
- [ ] T019 [US3] In `.github/workflows/release.yml`, add the `cargo wix` step producing the `.msi` into `dist/`.
- [ ] T020 [US3] In `.github/workflows/release.yml`, amend "Assemble the distribution archive" to copy `hint.db` into the zip stage beside `fragcap.exe`, and copy a loose `hint.db` into `dist/`.
- [ ] T021 [US3] In `.github/workflows/release.yml`, broaden "Generate checksums" from the `*.zip` filter to also cover `.msi` and `.db`, one `.sha256` per artifact.
- [ ] T022 [P] [US3] Add the optional scoped `msiexec /i ... /qn` install + `fragcap` PATH check + `msiexec /x ... /qn` smoke step to the `artifacts` job (does not assert Defender state).

## Phase 6: User Story 4 - An honest, self-consistent record (Priority: P3)

**Goal**: Docs and glossary describe the installer, the unsigned handling, the
bundled empty database, and the new default; every new term is defined.

**Independent test**: `scripts/lint-docs.sh check` passes; the glossary index
reproduces exactly.

- [ ] T023 [P] [US4] Add a `## Windows installer (MSI)` section to `README.md` before `## Prerequisite: npcap`, modeled on the npcap section, covering what the MSI does and unsigned/SmartScreen handling (warning expected by design; how to proceed; checksum is the integrity check; signing is #79); add a Quick-links row.
- [ ] T024 [P] [US4] Add an "Install fragcap" step to `site/content/docs/getting-started.mdx` ahead of the capture-driver step, with the SmartScreen note, renumbering the following steps.
- [ ] T025 [US4] Add three glossary entries (MSI installer, unsigned installer, Windows Defender exclusion) to `docs/glossary/platform-and-distribution.md` in the `{: .matters }` format with cross-links.
- [ ] T026 [US4] Regenerate `docs/glossary/index.md` via `scripts/lint-docs.sh fix` (do not hand-edit).
- [ ] T027 [P] [US4] Add `changelog.d/039-windows-installer.added.md` (MSI installer; bundled barebones hint database; hint-db default + first-run bootstrap) and `changelog.d/039-windows-installer.changed.md` (archive now includes `hint.db`; `--hint-db` now defaults to `%APPDATA%\fragcap\hint.db`).

## Phase 7: Polish & cross-cutting

- [ ] T028 Run `cargo xtask ci` (fmt, clippy `--all-targets --all-features -D warnings`, `test --workspace --locked`, lint, deps, license, wrappers, docs check) in the foreground; fix to green.
- [ ] T029 [P] Run `cargo xtask neutral` and `cargo xtask msrv` (1.82) in the foreground; confirm green and that `Cargo.lock` is unchanged (no new dependency).
- [ ] T030 Confirm the house-standards sweep: UTF-8 no BOM, LF, no em/en dashes across every new/edited file including `wix/main.wxs` and Rust comments; the workspace version is unchanged.
- [ ] T031 Assemble the manual MSI verification checklist (quickstart.md Tier 2) into the PR body as the recorded honesty step, and note the release-tag verification items.

## Dependencies & completion order

- Phase 1 (setup) then Phase 2 (foundational: spec amendments, decisions fragment, and the empty seed) precede the story phases. The decisions fragment (T004) must exist before the pinned `release.yml` change (US3). The seed (T005/T006) is consumed by US3's DB build and by the shipped template.
- US1 (Rust bootstrap) is independent and can proceed in parallel with US2 (wix authoring) and US4 (docs) since they touch disjoint files.
- US3 (release.yml) depends on the seed (foundational) and on the wix authoring (US2) existing.
- Phase 7 (verification) is last and gates the commit.

## Parallel execution examples

- After Phase 2: run US1 (T007-T012), US2 (T013-T017), and US4 doc drafts
  (T023, T024, T027) concurrently; they touch disjoint files.
- Within US1: T007 and T008 (tests) are `[P]`; T012 (cli.rs doc) is `[P]` with the
  paths/run implementation.

## Implementation strategy

MVP is US1 (the zero-config hint database) plus US2 (the installer), the two P1
stories; US3 wires them into the release and US4 makes the record honest. The
whole slice ships as one PR that `Closes #96`. Verification (Phase 7) runs the
full gate in the foreground before the single pre-push halt.
