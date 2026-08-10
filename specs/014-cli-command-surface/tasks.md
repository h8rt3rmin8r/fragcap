---

description: "Task list for slice S14: CLI Command Surface (run, tap, doctor, profile)"
---

# Tasks: CLI Command Surface (run, tap, doctor, profile)

**Input**: Design documents from `specs/014-cli-command-surface/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md,
contracts/cli-command-surface.md, quickstart.md

**Tests**: Included. This project builds test-first; the spec's Independent Tests
are the acceptance tests, and the constitution requires verification with evidence.

**Organization**: Grouped by user story. Story order follows spec priority:
US1 doctor (P1, MVP), US2 run (P1), US3 profile (P2), US4 tap (P2), US5 surface (P3).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- File paths are repository-relative.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Turn the CLI crate into a testable library plus thin binary and
declare the new dependencies.

- [X] T001 Convert `crates/fragcap-cli` to lib+bin: add a `[lib]` target, keep
  `[[bin]] name = "fragcap"`, and shrink `crates/fragcap-cli/src/main.rs` to a shim
  that calls `fragcap_cli::run(std::env::args_os())` and exits with its code.
- [X] T002 Add dependencies to `crates/fragcap-cli/Cargo.toml`: `clap` (features
  `["derive"]`) and `ctrlc` as runtime; `serde_json` and `tempfile` as
  dev-dependencies. Pin `clap`/`ctrlc` in `[workspace.dependencies]` and reference
  by workspace. No new runtime dep on any other crate.
- [X] T003 [P] Create the empty module skeleton in `crates/fragcap-cli/src/`:
  `lib.rs`, `cli.rs`, `args.rs`, `exit.rs`, `emit.rs`, `events.rs`, `output.rs`,
  `orchestrator.rs`, `assemble.rs`, `paths.rs`, `commands/mod.rs`, and the
  `commands/` and `doctor/` submodule files, each with the SPDX header and a doc
  comment, compiling as stubs.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The core/facade additions and the shared CLI scaffolding every
command depends on: the size grammar, the attribution/session bridge, the arg
grammar, the exit contract, and the emitter.

**⚠️ CRITICAL**: No command phase can begin until this phase is complete.

- [X] T004 [P] Add `crates/fragcap-core/src/size.rs`: a pure `parse(&str) ->
  Result<u64, SizeError>` for an integer plus required unit (`b`/`kb`/`mb`/`gb`),
  binary (1024-based), rejecting zero and missing/unknown units, mirroring
  `duration.rs`. Register `pub mod size` in `crates/fragcap-core/src/lib.rs`.
- [X] T005 [P] Unit-test `size::parse` in `crates/fragcap-core/src/size.rs`
  (`#[cfg(test)]`): each unit, overflow, zero rejection, missing/unknown unit,
  whitespace. Tests fail before T004 is complete, pass after.
- [X] T006 [P] Promote `write_json_string` to `pub` in
  `crates/fragcap-sink/src/json/escape.rs` and re-export it through the facade
  `sink` module in `crates/fragcap/src/lib.rs`.
- [X] T007 Add `CaptureSession::role_bindings() -> Vec<(u32, Option<Arc<str>>,
  Option<StageId>)>` in `crates/fragcap/src/session.rs`, reading the bound stages
  from the process tree.
- [X] T008 Add `RoleStampingAttributor` in `crates/fragcap/src/session.rs`: a
  `FlowAttributor` wrapping an inner `Arc<dyn FlowAttributor>`, holding an
  `Arc`-swapped `pid -> (role, stage)` snapshot, applying
  `Attribution::with_role`/`with_stage` after the inner `resolve`; add a
  `publish(bindings)` method. Re-export from the facade `session` module.
- [X] T009 [P] Unit-test `RoleStampingAttributor` in
  `crates/fragcap/src/session.rs` (`#[cfg(test)]`): an unstamped resolve passes
  through; a published binding stamps role and stage; a pid with no binding is
  unchanged; republish swaps the snapshot.
- [X] T010 [P] Re-export `ScriptedWatcher`, `ProcessScript`, and (behind `etw`)
  `EtwWatcher` from the facade `attr` module and top level in
  `crates/fragcap/src/lib.rs`.
- [X] T011 Define `Exit(u8)` (0/1/2) with `code()` and a `CliError` enum plus
  `From<ResolveError|LoadError|PipelineError|ConfigError|SizeError>` mappings in
  `crates/fragcap-cli/src/exit.rs`, per the contract's exit-code table.
- [X] T012 Implement the value parsers in `crates/fragcap-cli/src/args.rs`: `Dur`
  (delegates to `fragcap_core::duration::parse`), `Size` (delegates to
  `fragcap_core::size::parse`), `SinkSpec`, `ProfileRef`, `Direction`, `Roles`,
  `RingWindow`, each a clap `value_parser` mapping errors to a usage message.
- [X] T013 Define the clap derive types in `crates/fragcap-cli/src/cli.rs`: `Cli`
  (name `fragcap`, `version`), a `Command` enum with all seven subcommands, and
  `RunArgs`/`TapArgs`/`DoctorArgs`/`ProfileArgs`/`StubArgs` mirroring section 17.2.
- [X] T014 Implement `crates/fragcap-cli/src/lib.rs` `pub fn run<I>(args: I) ->
  Exit`: parse with clap (parse errors -> exit 2), dispatch to each command's
  entry, and map the `Result<(), CliError>` to `Exit` at one site.
- [X] T015 [P] Implement the emitter in `crates/fragcap-cli/src/emit.rs`,
  `events.rs`, and `output.rs`: an `Emitter` with Human (stderr, honoring
  quiet/silent, errors never suppressed) and Json (NDJSON on stderr) variants; the
  `Event` enum and its hand-rolled writer over the sink escaper; RFC3339 `Z`
  timestamp formatting from `SystemTime`; the completion-summary renderer.
- [X] T016 [P] Implement `crates/fragcap-cli/src/paths.rs`: resolve the user
  profile directory from `%APPDATA%` via `std::env` and assemble the `SearchPath`
  from it plus repeatable `--profile-dir` values; an empty `BundledSet` for this
  release.
- [X] T017 [P] Scaffold `crates/fragcap-cli/tests/cli_args.rs`: drive `run()` over
  the value grammars and assert the exit-code table (bad args -> 2, and the parse
  paths). Extended for help/stubs in US5.

**Checkpoint**: the crate compiles, parses arguments, and dispatches; commands are
empty. Command phases can begin.

---

## Phase 3: User Story 1 - Diagnose environment readiness (Priority: P1) 🎯 MVP

**Goal**: `fragcap doctor` classifies environment readiness, names remediations,
and returns the correct exit code, detect-never-install.

**Independent Test**: `cargo test -p fragcap-cli --test cli_doctor` over
constructed `Inputs`.

### Tests for User Story 1

- [X] T018 [P] [US1] Write `crates/fragcap-cli/tests/cli_doctor.rs`: for each
  constructed `Inputs` (ready; npcap absent; loopback option absent; WinPcap API
  mode absent; not elevated; no interfaces; tracing unavailable while elevated),
  assert the classification, the human-render golden (including "Ready to
  capture."), the `--json` per-check records, that every `Fail` has a remediation,
  that the two npcap options fail independently, and the exit code (0 vs 1). Tests
  fail before implementation.

### Implementation for User Story 1

- [X] T019 [P] [US1] Implement the pure model in
  `crates/fragcap-cli/src/doctor/mod.rs`: `Inputs`, `Status` (Ok/Warn/Skip/Fail),
  `Check`, `Report`, and `Report::exit()` (Fail present -> 1, else 0).
- [X] T020 [US1] Implement the per-check classifiers in
  `crates/fragcap-cli/src/doctor/checks.rs` as `fn(&Inputs) -> Check` for platform
  (OS, subsystem, privilege), capture driver (npcap present/version, loopback
  adapter, WinPcap API mode as two checks), tracing (severity per research D-f),
  interfaces, integration (extcap warn), and profiles; each Fail carries its exact
  remediation; npcap options named individually.
- [X] T021 [US1] Implement the human and `--json` renderers for the report (aligned
  columns; one record per check) in `crates/fragcap-cli/src/doctor/mod.rs` or
  `output.rs`.
- [X] T022 [US1] Implement `crates/fragcap-cli/src/commands/doctor.rs`: call
  `probe::gather()`, run `checks::run`, render per `--json`, return `Report::exit()`.
- [X] T023 [US1] Implement the thin `crates/fragcap-cli/src/doctor/probe.rs`
  (`cfg(windows)` and feature-gated) gathering real `Inputs`: npcap detection is
  read-only (registry/service/adapter), never installing; a non-windows build
  returns a minimal `Inputs` so the command still runs. Not unit-tested.

**Checkpoint**: `doctor` works and is fully covered offline. MVP deliverable.

---

## Phase 4: User Story 2 - Capture a game with a profile (Priority: P1)

**Goal**: `fragcap run --profile <ref>` produces an attributed capture, stops on a
bound or interrupt, and surfaces its counters.

**Independent Test**: `cargo test -p fragcap-cli --test cli_run` over
`ReplaySource` + `ScriptedAttributor` + `ScriptedWatcher`.

### Tests for User Story 2

- [X] T024 [P] [US2] Write `crates/fragcap-cli/tests/cli_run.rs`: drive `run()`
  over the offline substrate and assert the `.fcapng` and `.jsonl` goldens, the
  stamped role/stage, the `--json` event-sequence golden, the completion-summary
  counters and the conservation identity, exit 0 on a fired interrupt, each bound
  (duration/packet/byte) stopping for its named reason, and exit 1 on an
  acquisition timeout with no target. Tests fail before implementation.

### Implementation for User Story 2

- [X] T025 [P] [US2] Implement `EffectiveConfig` and the overlay (command line
  over `CaptureDefaults`) and `SinkSpec` handling in
  `crates/fragcap-cli/src/assemble.rs`, including rejecting `stream`/`ring` modes,
  transport sinks, and `--launch` with exit 2 naming the slice.
- [X] T026 [US2] Implement source, attributor, sink, and watcher assembly in
  `crates/fragcap-cli/src/assemble.rs`: offline (`ReplaySource` +
  `ScriptedAttributor` + `ScriptedWatcher`) and feature-gated live/socket-table/etw
  paths; wrap the attributor in `RoleStampingAttributor`; declare all interfaces up
  front; prepend the `TeeCountingSink`.
- [X] T027 [US2] Implement the `TeeCountingSink` and the `SessionDriver` in
  `crates/fragcap-cli/src/orchestrator.rs`: a thread owning the `CaptureSession`,
  selecting over the watcher events (republishing bindings to the stamper), the
  tee's retained-packet channel (`on_packet`), a ~50ms tick (`on_tick`), and the
  ctrlc interrupt flag (`on_interrupt`); stop the pipeline via `StopHandle` on
  Draining/Complete; emit lifecycle events.
- [X] T028 [US2] Implement `crates/fragcap-cli/src/commands/run.rs`: resolve the
  profile, build `EffectiveConfig`/`SessionConfig`, assemble the pipeline and
  session, install the ctrlc handler, run to completion, and render the completion
  summary; map outcomes to the exit contract.

**Checkpoint**: `run` captures end to end offline with full accounting.

---

## Phase 5: User Story 3 - Manage and validate profiles (Priority: P2)

**Goal**: `fragcap profile validate/list/show`.

**Independent Test**: `cargo test -p fragcap-cli --test cli_profile`.

### Tests for User Story 3

- [X] T029 [P] [US3] Write `crates/fragcap-cli/tests/cli_profile.rs`: a valid
  profile validates (exit 0) with its source; an invalid profile reports every
  diagnostic in one pass (exit 2); `list` reports bundled and user counts over a
  temp directory; `show` reports the resolved profile and source, and a well-formed
  unresolvable reference exits 1. Tests fail before implementation.

### Implementation for User Story 3

- [X] T030 [US3] Implement `crates/fragcap-cli/src/commands/profile.rs`: the
  `validate`/`list`/`show` subcommands over `resolve`, `load`, and `Diagnostics`,
  printing every diagnostic on failure and mapping to the exit contract.

**Checkpoint**: profile management works and reports all diagnostics at once.

---

## Phase 6: User Story 4 - Capture a running process ad hoc (Priority: P2)

**Goal**: `fragcap tap --process <name> --duration <dur>` reusing the run engine.

**Independent Test**: extend `cli_run.rs` with a tap case.

### Tests for User Story 4

- [X] T031 [P] [US4] Add a `tap` case to `crates/fragcap-cli/tests/cli_run.rs`:
  a synthesized one-stage capture of a named process over the offline substrate
  produces attributed output and the same completion summary and exit contract as
  `run`; a missing `--process` is a usage error (exit 2). Test fails first.

### Implementation for User Story 4

- [X] T032 [US4] Implement `crates/fragcap-cli/src/commands/tap.rs`: build a
  one-stage profile TOML for the named process and construct it through the real
  `load`/`parse` validation path (no unvalidated construction), then hand it to the
  shared `orchestrator::capture` used by `run`.

**Checkpoint**: `tap` captures a named process through the same engine.

---

## Phase 7: User Story 5 - Discover the whole tool from its help (Priority: P3)

**Goal**: all seven commands visible; the three stubs exit 2 naming their slice.

**Independent Test**: extend `cli_args.rs` with help and stub assertions.

### Tests for User Story 5

- [X] T033 [P] [US5] Add help and stub assertions to
  `crates/fragcap-cli/tests/cli_args.rs`: `--help` lists all seven commands; each
  of `replay`/`steam`/`extcap` reports "not yet implemented", names its delivering
  slice, and exits 2. Test fails first.

### Implementation for User Story 5

- [X] T034 [US5] Implement `crates/fragcap-cli/src/commands/stub.rs`: the
  `replay`/`steam`/`extcap` handlers printing "not yet implemented (slice SNN)"
  with the correct slice (S15/later, S17, S18) and returning exit 2.

**Checkpoint**: the full surface is discoverable and honest.

---

## Phase 8: Polish & Cross-Cutting Concerns

- [X] T035 [P] Add glossary entries in `docs/glossary.md` (P-6) for the readiness
  status vocabulary and the lifecycle event names introduced this slice.
- [X] T036 [P] Add `changelog.d/S14-cli.added.md` (the feature line) and
  `changelog.d/S14-cli.decisions.md` (clap and ctrlc on the CLI crate; the
  size grammar home and base; the RoleStampingAttributor bridge and its home;
  hand-rolled events and stream routing; the doctor probe split and severity/npcap
  rules).
- [X] T037 Run `cargo xtask ci` in the foreground and read it to completion; fix
  any failure within the slice. Then run the `quickstart.md` commands.

---

## Dependencies & Execution Order

- **Setup (Phase 1)**: no dependencies.
- **Foundational (Phase 2)**: depends on Setup; blocks all stories. Within it,
  T004 before T005/T012; T006 before T015; T007/T008 before US2; T011/T012/T013
  before T014.
- **US1 (Phase 3)**: after Foundational. Independent of other stories.
- **US2 (Phase 4)**: after Foundational (needs T007/T008 stamping, T015 events,
  T016 paths). Independent of US1/US3.
- **US3 (Phase 5)**: after Foundational (needs T016 paths). Independent.
- **US4 (Phase 6)**: after US2 (reuses `orchestrator::capture`).
- **US5 (Phase 7)**: after Foundational (needs the command enum/dispatch).
- **Polish (Phase 8)**: after all desired stories.

## Parallel Opportunities

- Setup: T003 parallel with dependency edits.
- Foundational: T004/T005, T006, T009, T010, T015, T016, T017 are largely
  independent files and can proceed in parallel once T013/T014 land the dispatch.
- Stories US1, US3, and US5 can proceed in parallel after Foundational; US2 before
  US4.

## Implementation Strategy

MVP is US1 (`doctor`): Setup + Foundational + Phase 3, validated offline. Then US2
(`run`) as the product core, US3 and US4, US5 last. Each story is independently
testable. The whole slice must pass `cargo xtask ci` before the pre-push halt.

## Notes

- Live, socket-table, and ETW paths compile under the `--all-features` clippy gate
  but are `#[ignore]`d; nothing in CI needs a driver, elevation, or a game.
- Commit after each logical group; stage only this slice's files; never stage
  `.specify/feature.json`.
