# Tasks: Extcap analyzer integration

**Feature**: roadmap slice S18 sub-slice A (specification section 14.5)
**Branch**: `021-extcap-integration`
**Input**: [plan.md](plan.md), [spec.md](spec.md), [data-model.md](data-model.md),
[research.md](research.md), [contracts/](contracts/), [quickstart.md](quickstart.md)

Test-driven: within each story a failing test is written before the code that
satisfies it. Verification is `cargo xtask ci`, run in the foreground, plus
`cargo xtask neutral` for the platform-neutral core build.

Path conventions: transport code in `crates/fragcap-sink/src/transport/`, CLI in
`crates/fragcap-cli/src/`, CLI integration tests in
`crates/fragcap-cli/tests/`.

## Phase 1: Setup

- [ ] T001 Add the `fifo` submodule to the transport module: declare
  `pub mod fifo;` in `crates/fragcap-sink/src/transport/mod.rs` with an empty
  `crates/fragcap-sink/src/transport/fifo.rs` carrying the SPDX header.
- [ ] T002 Add glossary entries (P-6) to `docs/glossary.md` for `extcap`,
  `DLT (link type)`, and `named pipe / FIFO`, following the existing entry
  template (blurb, detail, "why it matters here", references), cross-linking the
  FIFO entry to the streaming-sink and pcapng entries if present.
- [ ] T003 [P] Add changelog fragments `changelog.d/S18a-extcap.added.md` (the
  slice summary, present tense, citing specification 14.5 and roadmap slice S18)
  and `changelog.d/S18a-extcap.decisions.md` (the architecture-affecting
  decisions D1 to D9 from plan.md: the single-interface model, the
  `_for_extcap` reuse seam, the FIFO transport, the `open_fifo` platform policy,
  and the doctor read-only detection).

## Phase 2: Foundational (blocks US1, US2, US3)

- [ ] T004 Implement `open_fifo(path: &Path) -> io::Result<Box<dyn Write + Send>>`
  in `crates/fragcap-sink/src/transport/fifo.rs` per contracts/fifo-sink.md: a
  Windows `\\.\pipe\` path opens as a named-pipe client (write, no create, a
  short bounded retry on a busy pipe); any other path opens write+create+truncate.
  Re-export `open_fifo` from `crates/fragcap-sink/src/lib.rs`.
- [ ] T005 [P] Write a failing unit test in
  `crates/fragcap-sink/src/transport/fifo.rs` (`#[cfg(test)]`): `open_fifo` over a
  regular temp-file path returns a writer, and a pcapng `SinkFactory` built over
  it writes a Section Header Block magic as its first bytes (the non-pipe path is
  cross-platform and needs no server).
- [ ] T006 Add `SinkTransport::Fifo(PathBuf)` to `crates/fragcap-cli/src/args.rs`
  and the `fifo:` scheme to `parse_destination`; extend the scheme-list error
  messages to include `fifo:`. Add a parser unit test that `fifo:out.fcapng`
  parses to `SinkTransport::Fifo`.
- [ ] T007 Add `build_fifo_sink` in `crates/fragcap-cli/src/assemble.rs` and a
  `SinkTransport::Fifo` arm in `build_one_sink`: resolve pcapng (refuse
  `format=jsonl`), refuse rotation and streaming options, open the path with
  `fragcap::open_fifo`, build a pcapng `SinkFactory` encoder over it, push it.
- [ ] T008 Add `effective_config_for_extcap` to
  `crates/fragcap-cli/src/assemble.rs`, mirroring `effective_config_for_tap`: it
  takes the extcap options (profile-resolved, roles, direction, loopback) and the
  FIFO path, overlays roles/direction/loopback/payload on the profile `[capture]`
  defaults exactly as `run` does, sets `mode = File`, no ring, no launch, no
  volume bounds, and carries the FIFO as its single `SinkSpec`.
- [ ] T009 Replace `Extcap(StubArgs)` with `Extcap(Box<ExtcapArgs>)` in
  `crates/fragcap-cli/src/cli.rs` and define `ExtcapArgs` per data-model.md (the
  three declaration flags, `--capture`, `--fifo`, `--extcap-interface`,
  `--extcap-version`, the four config-call flags reusing the `run` value grammars,
  and a flattened `OfflineArgs`). Drop the `Extcap` variant from
  `crates/fragcap-cli/src/commands/stub.rs`. Register `pub mod extcap;` in
  `crates/fragcap-cli/src/commands/mod.rs` and route
  `Command::Extcap` to `commands::extcap::run` in `crates/fragcap-cli/src/lib.rs`.

**Checkpoint**: the workspace compiles, the extcap command is dispatched (even if
its handler is a skeleton), and the FIFO transport builds.

## Phase 3: US1 - An analyzer enumerates and starts fragcap (Priority: P1)

**Goal**: the four invocations, with the FIFO stream reproducing the golden.
**Independent test**: the declarations against the grammar, and a
`--capture --fifo` offline run read back as the committed pcapng golden.

- [ ] T010 [P] [US1] Write failing tests in `crates/fragcap-cli/tests/cli_extcap.rs`
  for the three declaration invocations, asserting each line matches the
  extcap control grammar and the interface/dlt/arg counts and key contents of
  contracts/extcap-cli-grammar.md (SC-001).
- [ ] T011 [US1] Implement the declaration emitters in
  `crates/fragcap-cli/src/commands/extcap.rs` (a pure `grammar` submodule):
  `--extcap-interfaces`, `--extcap-dlts`, `--extcap-config` per
  contracts/extcap-cli-grammar.md, writing to the command's stdout stream; make
  T010 pass.
- [ ] T012 [US1] Implement the `extcap` capture dispatch in
  `crates/fragcap-cli/src/commands/extcap.rs`: select the mode from the flags
  (declaration, or `--capture`), for `--capture` resolve the profile
  (`paths::search_path` + `resolve`), build the config with
  `effective_config_for_extcap` (FIFO from `--fifo`), assemble `components`, and
  run `orchestrator::capture`, the same back half `run` uses.
- [ ] T013 [P] [US1] Write a failing integration test in
  `crates/fragcap-cli/tests/cli_extcap.rs`: `extcap --capture --fifo <tempfile>`
  driven by the offline substrate (`--replay-source` a committed fixture) writes a
  valid pcapng reproducing that fixture's committed pcapng golden (SC-002), read
  back with the same parser the writer tests use.
- [ ] T014 [US1] Make T013 pass; assert the stream is byte-comparable to a plain
  `run --out` capture of the same fixture (FR-005), reusing the existing golden,
  and assert the extcap run's completion summary carries the conservation
  identity (received + buffer_dropped + refusals = captured) exactly as a file
  capture does (FR-011, SC-006).

**Checkpoint**: the whole extcap contract and FIFO stream are green at tier 1.

## Phase 4: US2 - The dialog options select the capture (Priority: P1)

**Goal**: the four declared options are applied at capture like the `run` flags.
**Independent test**: the config declaration names exactly the four options, and
each option value scopes the capture as the equivalent `run` flag.

- [ ] T015 [P] [US2] Extend `crates/fragcap-cli/tests/cli_extcap.rs`: assert
  `--extcap-config` declares exactly `--profile`, `--roles`, `--direction`,
  `--loopback` with the types in contracts/extcap-cli-grammar.md (SC-003 half 1).
- [ ] T016 [US2] Write a failing test in `crates/fragcap-cli/tests/cli_extcap.rs`:
  an `extcap --capture` carrying `--roles`, `--direction`, and `--loopback` over a
  fixture produces the same scoped stream as the equivalent
  `run --roles/--direction/--loopback` (SC-003 half 2); make it pass by confirming
  `effective_config_for_extcap` overlays identically (T008).
- [ ] T017 [US2] Write a failing test that an extcap capture with a profile that
  fails to resolve or validate exits 2 as a configuration error before capture
  (User Story 2 acceptance); make it pass.

**Checkpoint**: the dialog-to-capture fidelity is proven.

## Phase 5: US3 - doctor reports extcap installation (Priority: P2)

**Goal**: `doctor` names the extcap directory and its installed state.
**Independent test**: the classifier over a constructed `Inputs`.

- [ ] T018 [US3] Add `extcap_dir()` to `crates/fragcap-cli/src/paths.rs`
  (`%APPDATA%\Wireshark\extcap` on Windows, an XDG/HOME location on Unix, a
  `FRAGCAP_EXTCAP_DIR` override mirroring `FRAGCAP_PROFILE_DIR`).
- [ ] T019 [US3] Add `extcap_dir: Option<PathBuf>` to `Inputs` in
  `crates/fragcap-cli/src/doctor/mod.rs`; set it and the real `extcap_installed`
  (presence of a fragcap binary in the directory, read-only) in
  `crates/fragcap-cli/src/doctor/probe.rs` for both the Windows and non-Windows
  gather paths.
- [ ] T020 [P] [US3] Write failing classifier tests in
  `crates/fragcap-cli/src/doctor/checks.rs` (`#[cfg(test)]`): with a directory and
  a present binary, `integration()` reports installed and names the directory;
  with the binary absent, it reports not installed and names the same directory;
  with `extcap_dir = None` it says the location could not be determined (SC-004).
- [ ] T021 [US3] Update `integration()` in
  `crates/fragcap-cli/src/doctor/checks.rs` to name `extcap_dir` in both details;
  make T020 pass. Update the existing `doctor` fixture tests that construct
  `Inputs` to set the new field.

## Phase 6: Polish & Cross-Cutting

- [ ] T022 Confirm the offline substrate flags reach the extcap capture (the
  `cli_extcap.rs` tests already drive `--replay-source`); add a misuse test group
  in `crates/fragcap-cli/tests/cli_extcap.rs` for the error contract (no mode
  flag; `--capture` without `--fifo`; a declaration without `--extcap-interface`;
  an unknown `--extcap-interface`), each exit 2 naming the cause (SC-005).
- [ ] T023 Run `cargo xtask ci` in the foreground and watch it to completion;
  then run `cargo xtask neutral` to confirm the platform-neutral core still
  builds (SC-007). Fix any failure before proceeding.
- [ ] T024 Final review pass: confirm no em/en dashes, SPDX headers on the new
  files (`transport/fifo.rs`, `commands/extcap.rs`, `tests/cli_extcap.rs`),
  UTF-8/LF, that `cargo xtask lint` still finds no transmit/handle call, and that
  every new term used in code or docs has its glossary entry (P-6, P-8). Update
  the stale `Replay`/`Extcap` help doc-comments in `cli.rs`/`lib.rs` that named
  this slice.

## Dependencies

- Phase 1 (Setup) has no dependencies.
- Phase 2 (Foundational) depends on T001; T006 to T009 depend on T004.
- Phase 3 (US1) depends on Phase 2 (the FIFO transport, the config builder, and
  the dispatched command).
- Phase 4 (US2) depends on Phase 3 (it exercises the capture path US1 delivers).
- Phase 5 (US3) is independent of Phases 3 and 4 (doctor only) and can proceed
  after Phase 1; it is ordered last among the stories by priority (P2).
- Phase 6 (Polish) depends on all prior phases.

## Parallel opportunities

- T003 runs parallel to T001/T002 (different files).
- T005 is parallel to T006 (different crates).
- T010 and T013 are authored together (same test file, independent functions).
- Phase 5 (US3, doctor) can be developed in parallel with Phases 3 and 4 by a
  second worker: it touches only `paths.rs`, `doctor/`, and no capture code.

## Implementation strategy

MVP is US1 (Phase 3): the four invocations and the FIFO stream, reachable from
`fragcap extcap` and tier-1 testable against a regular temp file. US2 (Phase 4)
proves the dialog options actually scope the capture. US3 (Phase 5) is the
subordinate doctor report. Phase 6 covers the error contract and runs the gate.
