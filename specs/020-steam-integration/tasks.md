# Tasks: Steam integration and managed launch

**Feature**: S17 (specification section 16) | **Branch**: `020-steam-integration`
**Plan**: [plan.md](plan.md) | **Spec**: [spec.md](spec.md)

Tests are written before the code they cover (project TDD discipline). Tier-1 only in CI;
the live Steam launch is tier-2/manual and is never asserted as run. `[P]` marks tasks on
different files with no incomplete dependency between them.

## Phase 1: Setup

- [x] T001 Add `windows-sys` to `crates/fragcap-steam/Cargo.toml` under
  `[target.'cfg(windows)'.dependencies]` with `default-features = false` and features
  `Win32_Foundation`, `Win32_System_Registry`, `Win32_UI_Shell`; keep
  `fragcap-profile.workspace = true`. No new workspace dependency (D1).
- [x] T002 Replace `crates/fragcap-steam/src/lib.rs` skeleton doc-comment with the module
  layout (`mod vdf; mod library; mod scaffold; mod launch;`) and re-export the public API
  and `SteamError` per [contracts/crate-api.md](contracts/crate-api.md); keep
  `SPDX-License-Identifier: Apache-2.0`.
- [x] T003 Define `SteamError` (variants `NotInstalled`, `TitleNotFound { app_id }`,
  `Vdf { path, position, detail }`, `UnsupportedPlatform`, `Io { path, source }`) with
  `Display`/`Error` in `crates/fragcap-steam/src/lib.rs` per [data-model.md](data-model.md).

## Phase 2: Foundational (blocking prerequisites for all stories)

**VDF parser** (portable, unconditionally compiled and tested)

- [x] T004 [P] Write VDF parser unit tests in `crates/fragcap-steam/src/vdf.rs` (`#[cfg(test)]`):
  well-formed nested-quoted blocks, `"key" "value"` and `"key" { ... }`, `//` line comments,
  `\\`/`\"` escapes, and a malformed input asserting a positioned `VdfError` (no panic, no
  silent mis-parse). FR-003/FR-004.
- [x] T005 Implement the hand-rolled `vdf::parse(&str) -> Result<VdfValue, VdfError>` and
  `VdfError` (carries a byte position) in `crates/fragcap-steam/src/vdf.rs` to pass T004 (D3).

**Library discovery core** (registry read is `cfg(windows)`; parsing/assembly portable)

- [x] T006 [P] Write discovery unit tests in `crates/fragcap-steam/src/library.rs`
  (`#[cfg(test)]`) driving the portable path-assembly + manifest-parse logic from an
  in-tempdir fixture root: two libraries, resolves `installdir` under
  `steamapps/common/`, returns app_id + name + absolute install path.
- [x] T007 Implement `library` module: `libraryfolders.vdf` -> library list,
  `appmanifest_*.acf` -> `InstalledTitle`, factored so the file-walk/parse is portable and
  only the registry lookup for the Steam root is `#[cfg(windows)]` (non-Windows arm ->
  `SteamError::UnsupportedPlatform`). Malformed manifest -> reported and skipped (FR-004).
- [x] T008 Implement `discover() -> Result<SteamInstallation, SteamError>` in
  `crates/fragcap-steam/src/lib.rs`: registry root (cfg windows) -> `libraryfolders.vdf` ->
  every `appmanifest_*.acf`; `NotInstalled` when the registry entry/root is absent.

## Phase 3: User Story 1 - Scaffold a profile for an installed title (P1) [MVP]

**Goal**: `fragcap steam profile <app_id>` emits a section-15.4-valid profile skeleton.
**Independent test**: point the scaffolder at a fixture install directory and assert the
emitted profile names the platform/app_id, proposes a plausible client stage, and passes
`Profile::parse` unedited.

- [x] T009 [P] [US1] Write classifier unit tests in `crates/fragcap-steam/src/scaffold.rs`
  (`#[cfg(test)]`): launcher-token image -> launcher; largest non-launcher -> client;
  two proposals sharing a basename -> a `path_contains` disambiguator added; degenerate
  scans (no non-launcher; all launcher-tokened) still yield a client. FR-006, D7, Edge Cases.
- [x] T010 [P] [US1] Write the scaffold-self-validation test in
  `crates/fragcap-steam/src/scaffold.rs`: every rendered scaffold parses cleanly through
  `fragcap_profile::Profile::parse`; the header comment is present and states the
  classification is heuristic. FR-007, FR-008, SC-001, D4.
- [x] T011 [US1] Implement the executable scan + heuristic classifier (`ExecutableImage`,
  `StageProposal`) in `crates/fragcap-steam/src/scaffold.rs` to pass T009.
- [x] T012 [US1] Implement the TOML render + `scaffold(&InstalledTitle) -> Result<String,
  SteamError>` that round-trips through `Profile::parse` before returning, to pass T010
  (D4). Emit `game.platform = "steam"`, `game.app_id`, `exe` predicates, and any
  `path_contains` disambiguator.
- [x] T013 [US1] Replace `Steam(StubArgs)` with `Steam(SteamArgs)` carrying a
  `profile <app_id>` subcommand in `crates/fragcap-cli/src/cli.rs`; remove the reachable
  `commands::stub::run(Stub::Steam)` dispatch in `crates/fragcap-cli/src/lib.rs`. FR-013.
- [x] T014 [US1] Add `crates/fragcap-cli/src/commands/steam.rs`: run discovery, resolve the
  app_id (`TitleNotFound` -> named error, nothing on stdout, FR-009), scaffold, and print to
  stdout; wire it into the command dispatch. Re-export the crate API through the `fragcap`
  facade if not already visible.
- [x] T015 [P] [US1] Write a CLI-level test in `crates/fragcap-cli` (or the `fragcap` facade
  test dir) that scaffolds against a fixture library and asserts the printed profile passes
  `Profile::parse`; assert the not-installed app_id path errors and prints no profile.

## Phase 4: User Story 2 - Managed launch without the acquisition race (P2)

**Goal**: `fragcap run --launch` is wired, validates config before capture, and sequences
the launch after the watcher is armed.
**Independent test**: a profile with platform+app_id yields a `steam://run/<app_id>` request
sequenced after `Watching`; a profile missing either is refused before capture.

- [x] T016 [P] [US2] Write `launch_request` unit tests in
  `crates/fragcap-steam/src/launch.rs` (`#[cfg(test)]`): missing `game.platform` or
  `game.app_id` -> typed refusal; both present -> `LaunchRequest { url:
  "steam://run/<app_id>" }`. FR-011, D5.
- [x] T017 [US2] Implement `launch_request(&Profile) -> Result<LaunchRequest, SteamError>`
  (portable) and the `#[cfg(windows)]` `launch(&LaunchRequest)` issuing
  `ShellExecuteW(steam://run/<app_id>)`, with a non-Windows stub returning
  `UnsupportedPlatform`. FR-010.
- [x] T018 [US2] Replace the `assemble.rs` refusal `"managed launch (--launch) is not yet
  supported (slice S17)"` in `crates/fragcap-cli/src/assemble.rs` with real validation that
  reads the loaded profile: refuse with a named usage error when platform/app_id are absent
  or on a non-Windows build, before capture starts. FR-011, Edge Cases.
- [x] T019 [US2] Wire the launch into the run path so it is issued after `session.attach()`
  reaches `Watching` and the sinks/capture handle are open ([session.rs:221](../../crates/fragcap/src/session.rs)),
  behind the same `#[cfg(windows)]`/live gating the capture backend uses. FR-010.
- [x] T020 [P] [US2] Write an ordering/decision test (in `crates/fragcap-cli` or the facade)
  asserting the assembled run requests a managed launch of the correct app_id and that the
  launch is sequenced after arm - without spawning a live Steam process (D5, P-9).

## Phase 5: User Story 3 - Enumerate installed titles across libraries (P3)

**Goal**: discovery returns every installed title from every library and survives malformed
entries.
**Independent test**: a fixture root with two libraries returns all titles; a malformed
manifest is skipped; a duplicate app_id resolves deterministically.

- [x] T021 [P] [US3] Add the `crates/fragcap-steam/tests/discovery.rs` integration test:
  build a tempdir Steam root (`libraryfolders.vdf` + two libraries of `appmanifest_*.acf`),
  assert all titles across both libraries are returned with resolved install paths.
- [x] T022 [US3] Extend `tests/discovery.rs` with the malformed-manifest-skipped case
  (well-formed siblings survive, FR-004) and the duplicate-app_id case (first wins, collision
  reported, Edge Cases). Adjust the `library`/`discover` implementation only if a case fails.

## Phase 6: Polish & Cross-Cutting Concerns

- [x] T023 [P] Add `docs/glossary.md` entries for `managed launch`, `library discovery`,
  `profile scaffolding`, and `VDF` (Valve key-value), each cross-linked (P-6).
- [x] T024 [P] Add `changelog.d/S17-steam-integration.added.md` (the feature lines) and
  `changelog.d/S17-steam-integration.decisions.md` recording D1 (the dated
  `windows-sys`-over-`winreg` architecture-of-record deviation) and D5/D6/D7.
- [x] T025 [P] Update `crates/fragcap-steam/README.md` from "skeleton" to the shipped
  capability summary (discovery, scaffolding, managed launch; Windows-only internals).
- [x] T026 Update `AGENTS.md` current-state prose and `CLAUDE.md` slice-status line to note
  S17 complete (fragcap-steam filled in), keeping the dependency-inventory table honest
  (windows-sys features extended, no new package).
- [x] T027 Run `cargo xtask ci` in the foreground and watch it to completion; fix any
  fmt/clippy/test/lint/deps/license finding until green. Confirm `cargo xtask lint` still
  passes the OpenProcess/ReadProcessMemory/WriteProcessMemory ban.

## Dependencies & Execution Order

- **Setup (T001-T003)** blocks everything.
- **Foundational (T004-T008)** blocks all three stories (they need VDF + discovery).
- **US1 (T009-T015)** is the MVP; depends only on Foundational.
- **US2 (T016-T020)** depends on Foundational (uses `Profile`); independent of US1.
- **US3 (T021-T022)** depends on Foundational; sharpens discovery's edges.
- **Polish (T023-T027)** last; T027 is the gate and must be green before the pre-push halt.

## Parallel Opportunities

- Within Foundational: T004 (VDF tests) and T006 (discovery tests) are `[P]`.
- Within US1: T009, T010, T015 are `[P]` (distinct test files/targets).
- US1 and US2 can proceed in parallel once Foundational lands (different modules).
- Polish T023/T024/T025 are `[P]` (distinct files).

## MVP Scope

**User Story 1 + its Foundational prerequisites** (T001-T015): `fragcap steam profile
<app_id>` producing a section-15.4-valid scaffold. Managed launch (US2) and the discovery
edge tests (US3) layer on without reworking it.
