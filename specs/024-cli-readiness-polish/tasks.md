---

description: "Task list for CLI readiness, help, and output-contract polish"
---

# Tasks: CLI readiness, help, and output-contract polish

**Input**: Design documents from `specs/024-cli-readiness-polish/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Included. The spec's success criteria and the repository's
verification discipline (`cargo xtask ci`) require them. The `doctor` classifier
already has a pure-function unit suite to extend.

**Organization**: By user story (US1..US7), priority order from spec.md. All
runtime changes are in `crates/fragcap-cli`.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: parallelizable (different file, no dependency on an incomplete task)
- **[Story]**: US1..US7

## File-contention notes (breaks naive [P])

- `crates/fragcap-cli/src/commands/profile.rs` is touched by US3, US4, US7 -  those edits are sequential, not parallel.
- `crates/fragcap-cli/src/cli.rs` is touched by US5 tasks - sequential.
- `crates/fragcap-cli/src/doctor/checks.rs` is touched by several US1 tasks -  sequential.

---

## Phase 1: Setup

**Purpose**: Establish a known-green baseline before changes.

- [x] T001 Run `cargo xtask ci` on the branch head and confirm it is green before any change (records the pre-change baseline).

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared manifest change needed by US1's npcap version read.

- [x] T002 [P] Add `Win32_Storage_FileSystem` to the `windows-sys` feature list in `crates/fragcap-cli/Cargo.toml` (enables the wpcap.dll FileVersion read; explicit rather than relying on cross-crate feature unification, per research R2).

**Checkpoint**: Manifest ready; user stories can proceed.

---

## Phase 3: User Story 1 - Honest doctor readiness (Priority: P1) 🎯 MVP

**Goal**: `doctor` reports live/socket-table backend presence, downgrades
loopback, points empty-interfaces at the real cause, and shows the real npcap
version (#63, #69, #70.2).

**Independent Test**: `doctor` on a backend-less build reports the live backend
absent and blocking ("not ready", exit 1); on a featured build reports present
with a real npcap version; loopback-missing does not force "not ready".

### Tests for User Story 1

- [x] T003 [P] [US1] Extend the pure-function unit tests in `crates/fragcap-cli/src/doctor/checks.rs`: absent-`live` → `Fail` and `Report::exit() == FAILURE`; absent-`socket-table` → `Warn` and still ready; loopback-absent → `Warn` and `report.ready()`; live-absent → interfaces message names the missing backend.

### Implementation for User Story 1

- [x] T004 [US1] Add `live_available` and `socket_table_available: Option<bool>` to `Inputs` in `crates/fragcap-cli/src/doctor/mod.rs`, and populate them via `#[cfg(feature = "live")]` / `#[cfg(feature = "socket-table")]` in `crates/fragcap-cli/src/doctor/probe.rs`, mirroring `tracing_availability()`.
- [x] T005 [US1] Add the live-backend check (`Fail` when `None`, with remediation) and the socket-table check (`Warn` when `None`) in `crates/fragcap-cli/src/doctor/checks.rs`, inserted in section order in `run()`.
- [x] T006 [US1] Downgrade the missing-loopback branch from `Check::fail` to `Check::warn` (keep the remediation text) in `crates/fragcap-cli/src/doctor/checks.rs`.
- [x] T007 [US1] Reword the empty-interface branch to name the missing live backend when `live_available` is `None`, in `crates/fragcap-cli/src/doctor/checks.rs`.
- [x] T008 [US1] Read the real npcap version from the wpcap.dll FileVersion (`GetFileVersionInfoSizeW`/`GetFileVersionInfoW`/`VerQueryValueW`, `#[cfg(windows)]`, graceful fallback) in `crates/fragcap-cli/src/doctor/probe.rs`, and reword the fallback so `crates/fragcap-cli/src/doctor/checks.rs` npcap line does not print "version installed" when no version is known.

**Checkpoint**: `doctor` is honest about capability and version.

---

## Phase 4: User Story 2 - Released binary can capture (Priority: P1)

**Goal**: The release binary is built with capability features (#62).

**Independent Test**: `release.yml` builds with `--features live,socket-table,etw`
and includes the npcap SDK step; a dated decision fragment exists.

- [x] T009 [US2] Add the dated decision fragment `changelog.d/release-features.decisions.md` recording the pinned-artifact change to `release.yml` (feature set + npcap SDK step), per the non-negotiable pinned-artifact rule.
- [x] T010 [US2] Update `.github/workflows/release.yml`: build the release binary with `--features live,socket-table,etw` and add the npcap SDK-acquisition step used by `platform.yml`.

**Checkpoint**: The shipped recipe produces a capable binary.

---

## Phase 5: User Story 3 - Profile `--json` (Priority: P2)

**Goal**: `profile list`/`validate` honor `--json` via the §17.5 event stream,
one diagnostic event per problem + a summary (#65).

**Independent Test**: `profile validate --json` emits N `diagnostic` events + a
`summary`; `profile list --json` emits structured counts; both parse as NDJSON.

### Tests for User Story 3

- [x] T011 [P] [US3] Integration test in `crates/fragcap-cli/tests/`: parse `profile validate --json` output line-by-line with the dev-only `serde_json`, asserting one `diagnostic` event per problem (with `code`/`path`/`line`/`col`/`message`) plus a terminal `summary`; assert `profile list --json` emits structured counts.

### Implementation for User Story 3

- [x] T012 [US3] Add `diagnostic` and `summary` event variants, plus a profile-list counts event, to `crates/fragcap-cli/src/events.rs`, rendered in the existing §17.5 `{"ts","event",...}` form via `render()`.
- [x] T013 [US3] Thread the `--json` flag into the `profile` dispatch: pass it through `crates/fragcap-cli/src/lib.rs` and into the handler signature in `crates/fragcap-cli/src/commands/profile.rs`.
- [x] T014 [US3] In `crates/fragcap-cli/src/commands/profile.rs`, emit one `diagnostic` event per diagnostic (not the collapsed newline-joined string) plus a `summary` for `validate`, and structured counts for `list`, when `--json` is set; leave human output unchanged.

**Checkpoint**: Structured output is machine-consumable for `profile`.

---

## Phase 6: User Story 4 - Consistent exit codes (Priority: P2)

**Goal**: A reference that resolves to no profile exits 1 from both `show` and
`validate`; an invalid profile file stays 2 (#68).

**Independent Test**: `show` and `validate` agree at exit 1 for an absent slug
and for an unresolvable path-shaped reference; an invalid profile file exits 2.

### Tests for User Story 4

- [x] T015 [P] [US4] Integration test in `crates/fragcap-cli/tests/`: `show` and `validate` both exit 1 for an absent slug and for `missing.toml` (unresolvable path); an existing-but-invalid profile file exits 2; a valid profile exits 0.

### Implementation for User Story 4

- [x] T016 [US4] In `crates/fragcap-cli/src/exit.rs`, reclassify `ResolveError::InvalidReference` from `CliError::Usage` to `CliError::Failure` (keep `Load { LoadError::Invalid }` at `Usage`); update the `From<ResolveError>` doc comment to state the contract.

**Checkpoint**: The exit contract is consistent and documented.

---

## Phase 7: User Story 5 - Trustworthy help text (Priority: P2)

**Goal**: No parser implementation note, no internal slice IDs, correct
`--launch` copy (#66, #67).

**Independent Test**: `run`/`extcap`/all help contains no `S1[0-9]` slice id and
no `value_parser`/`Vec<String>` note; `--launch` describes real behavior.

### Tests for User Story 5

- [x] T017 [P] [US5] Test in `crates/fragcap-cli/tests/` that renders help for `run`, `extcap`, and each subcommand and asserts no substring matching an `S`-followed-by-digits slice id and no `value_parser`/`Vec<String>` note appears.

### Implementation for User Story 5

- [x] T018 [US5] Move the `--roles` `value_delimiter`/`value_parser` rationale from the doc comment to a `//` source comment, on both the `run` and `extcap` roles fields in `crates/fragcap-cli/src/cli.rs`.
- [x] T019 [US5] Remove internal slice IDs (S15/S16/S17) from user-facing help and reword to "not yet implemented": in `crates/fragcap-cli/src/cli.rs` (`Replay`, `ModeArg::Stream`/`Ring`, `RunArgs.ring`), `crates/fragcap-cli/src/args.rs` (`RingWindow`), and `crates/fragcap-cli/src/commands/stub.rs`.
- [x] T020 [US5] Reconcile the `--launch` help in `crates/fragcap-cli/src/cli.rs` to describe the shipped managed-launch behavior (Windows-only; Steam app id from the profile) without a slice id.
- [x] T020a [US5] Update the global `--json` flag help in `crates/fragcap-cli/src/cli.rs` to state its scope - which surfaces emit structured events (`run`/`tap`/`extcap`/`steam` and now `profile`; `doctor` uses its own per-check JSON) - satisfying FR-010's discoverability requirement.

**Checkpoint**: Help text is clean and current.

---

## Phase 8: User Story 6 - Elevation gate (Priority: P3)

**Goal**: Live capture refuses cleanly without elevation, before the driver
opens, exit 1 (#56).

**Independent Test**: A non-elevated live-capture command refuses with an
actionable message and exit 1, before driver access; offline/read-only commands
still run unelevated.

### Tests for User Story 6

- [x] T021 [P] [US6] Test the refusal at a platform-neutral predicate seam (elevated=false + live-capture command → refusal, exit 1) in `crates/fragcap-cli/`, so CI covers the logic off-Windows.

### Implementation for User Story 6

- [x] T022 [US6] Expose the current-process elevation predicate (`is_elevated`, `crates/fragcap-cli/src/doctor/probe.rs`) for reuse by the assembly path via a Windows-only shared function; it reads only the current-process token (P-1).
- [x] T023 [US6] Add the elevation refusal gate to `crates/fragcap-cli/src/assemble.rs` for `run`/`tap`/`extcap` capture: when not elevated, return `CliError::failure(...)` (exit 1) with the elevation message before the capture source is built; `#[cfg(windows)]`; do not auto-relaunch; leave offline/read-only paths untouched.

**Checkpoint**: Live capture guards elevation clearly.

---

## Phase 9: User Story 7 - Validate output polish (Priority: P3)

**Goal**: `profile validate` success line names the path once (#70.1).

**Independent Test**: `profile validate <valid-by-path>` prints the path once.

- [x] T024 [US7] Remove the redundant trailing `(path ...)` from the `profile validate` success line in `crates/fragcap-cli/src/commands/profile.rs` (sequence after US3's edits to the same file).

**Checkpoint**: Success output is clean.

---

## Phase 10: Polish & Cross-Cutting Concerns

**Purpose**: Governance and final verification (FR-020, FR-021, SC-007).

- [x] T025 [P] Update master specification `docs/fragcap-specification.md`: §17.4/§17.5 (exit-code alignment note; `diagnostic`/`summary` events) and §26.3 (capability readiness lines, loopback severity, npcap version).
- [x] T026 [P] P-6 glossary check: add a section 4.3 glossary entry for any newly introduced term; confirm no term is used without an entry.
- [x] T027 Add the slice changelog fragment `changelog.d/S024-cli-readiness-polish.md` summarizing the nine fixes and their issue numbers.
- [x] T028 Run `cargo xtask ci` in the foreground to green; on a Windows box run the `quickstart.md` manual smokes (featured `doctor`, unelevated-refusal, `profile --json`).

---

## Dependencies & Execution Order

- **Setup (T001)** → **Foundational (T002)** → user stories.
- **US1 (T003-T008)**: T004 before T005/T007 (checks read the new `Inputs`
  fields); T005/T006/T007/T008 all edit `checks.rs` → sequential among
  themselves; T008 also needs T002 (feature) and edits `probe.rs`.
- **US2 (T009-T010)**: T009 (decision fragment) before T010 (pinned change).
- **US3 (T011-T014)**: T012 (events) and T013 (dispatch) before T014 (emit).
- **US4 (T015-T016)**: independent (edits `exit.rs`).
- **US5 (T017-T020)**: T018/T019/T020 edit `cli.rs` → sequential.
- **US6 (T021-T023)**: T022 before T023.
- **US7 (T024)**: sequence after US3 (shared `profile.rs`).
- **Polish (T025-T028)**: after all stories; T028 is the final gate.

### Parallel opportunities

- Test tasks T003, T011, T015, T017, T021 are `[P]` (distinct files) and can be
  written up front.
- US2, US4, US5, US6 are largely independent of US1 and of each other (distinct
  files), so different agents/developers can take them in parallel once
  Foundational completes; US3 and US7 must coordinate on `profile.rs`.
- Polish T025/T026 are `[P]` (different files).

---

## Implementation Strategy

- **MVP**: US1 + US2 together - they fix the headline P-9 defect (a binary that
  lies about being ready) and its root cause (a binary that cannot capture).
- **Incremental**: add US3, US4, US5 (scriptability + trust), then US6, US7.
- **Governance rides with the code**: the decision fragment (T009) lands with
  the release.yml change; spec/glossary/changelog (T025-T027) land before the
  final gate (T028).

## Notes

- Commit per story or logical group; stage only this slice's files. Never stage
  `.specify/feature.json` (gitignored local state).
- Verify each story's tests fail before implementing where practical.
- The final gate `cargo xtask ci` must run in the foreground and be read to
  completion (verification discipline); `cargo xtask lint` also enforces the P-1
  no-handle guard the elevation gate must not trip.
