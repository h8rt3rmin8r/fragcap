# Tasks: Workspace Scaffold, Licensing, and CI Skeleton

**Feature**: S01 | **Branch**: `feat/workspace-scaffold` \
**Input**: [spec.md](spec.md), [plan.md](plan.md),
[data-model.md](data-model.md), [contracts/xtask-cli.md](contracts/xtask-cli.md),
[quickstart.md](quickstart.md)

**Tests are required for this slice.** FR-015a requires the conventions check
to be covered by tests, and SC-004 requires each check to be demonstrated
failing on known-bad input. A linter whose matcher never fires is
indistinguishable from a clean repository, so the tests are what make the
checks real rather than decorative.

## Phase 1: Setup

- [ ] T001 Create the workspace manifest at `Cargo.toml` with members
      `crates/*` and `xtask`, resolver 2, and the `workspace.package` and
      `workspace.dependencies` tables per plan.md
- [ ] T002 [P] Create `rust-toolchain.toml` pinning channel 1.96.0, the
      `x86_64-pc-windows-msvc` target, and the `rustfmt` and `clippy`
      components
- [ ] T003 [P] Create `deny.toml` declaring the permitted dependency license
      allowlist from specification section 20.4
- [ ] T004 [P] Create `profiles/README.md` explaining the directory's purpose
      and the slice that fills it
- [ ] T005 [P] Create `fixtures/README.md` explaining the directory's purpose,
      the slice that fills it, and the rule that fixtures are synthetic or
      self-generated
- [ ] T006 [P] Create `scripts/README.md` explaining the directory's purpose,
      the slice that fills it, and the open house-shell-standard gap

## Phase 2: Foundational

Blocking prerequisite for every user story. Nothing below compiles until the
crate skeletons exist.

- [ ] T007 Create `crates/fragcap-core/Cargo.toml` and `src/lib.rs` with the
      SPDX header, inheriting workspace metadata, with no dependencies
- [ ] T008 [P] Create `crates/fragcap-profile/` depending on `fragcap-core`
- [ ] T009 [P] Create `crates/fragcap-capture/` depending on `fragcap-core`
- [ ] T010 [P] Create `crates/fragcap-attr/` depending on `fragcap-core`
- [ ] T011 [P] Create `crates/fragcap-sink/` depending on `fragcap-core`
- [ ] T012 [P] Create `crates/fragcap-steam/` depending on `fragcap-profile`
- [ ] T013 Create `crates/fragcap/` (facade) depending on `fragcap-core` and
      the five mid-level crates per decision D-1
- [ ] T014 Create `crates/fragcap-cli/` binary depending on `fragcap`, with
      `src/main.rs` carrying the SPDX header
- [ ] T015 Create `xtask/Cargo.toml` and `xtask/src/main.rs` with the command
      dispatch skeleton and the 0/1/2 exit code contract from
      contracts/xtask-cli.md
- [ ] T016 Run `cargo build --workspace` and commit the resulting `Cargo.lock`

## Phase 3: User Story 1 - Build from a clean clone (P1)

**Goal**: A contributor clones, runs one command, and gets a working build.

**Independent test**: Build and test from a clean checkout with only `rustup`
present.

- [ ] T017 [US1] Verify `cargo build --workspace` succeeds from a clean target
      directory and record the output
- [ ] T018 [US1] Verify `cargo test --workspace --locked` succeeds and record
      the output
- [ ] T019 [US1] Verify every directory named in plan.md's Source Code tree
      exists and carries content explaining its purpose

## Phase 4: User Story 2 - Mistakes caught before review (P1)

**Goal**: Violations are reported mechanically, by name, rather than noticed in
review.

**Independent test**: Introduce each violation deliberately, confirm the named
failure, revert.

### Tests first

- [ ] T020 [P] [US2] Write failing tests in `xtask/src/lint.rs` covering each
      conventions rule: byte order mark, CRLF, trailing whitespace, missing
      final newline, em-dash, en-dash, missing SPDX header. Each test feeds
      known-bad input and requires the specific rule to be reported
- [ ] T021 [P] [US2] Write a failing test in `xtask/src/lint.rs` asserting that
      a clean input produces no findings, so the checks cannot pass by
      matching nothing
- [ ] T022 [P] [US2] Write failing tests in `xtask/src/deps.rs` covering an
      unexpected edge, a missing edge, and an exact match, against synthetic
      metadata

### Implementation

- [ ] T023 [US2] Implement the conventions check in `xtask/src/lint.rs`:
      binary detection by content sniffing, an explicit vendored-path exclusion
      list, and `path:line: rule: detail` output
- [ ] T024 [US2] Implement the dependency direction check in
      `xtask/src/deps.rs`, reading `cargo metadata` and comparing against the
      edge set encoded in one place, reporting unexpected and missing edges
      separately
- [ ] T025 [US2] Implement `cargo xtask neutral` in `xtask/src/main.rs`,
      building `fragcap-core` for a non-host target and **exiting 2 with the
      `rustup target add` line when the target is absent, never 0**
- [ ] T026 [US2] Implement `cargo xtask msrv` in `xtask/src/main.rs`, building
      at the declared minimum and **printing that the result does not yet
      constrain anything while the workspace has no external dependencies**
- [ ] T027 [US2] Implement `cargo xtask ci` in `xtask/src/main.rs`, running
      fmt, clippy, test, lint, and deps in order and propagating the first
      failure's exit code
- [ ] T028 [US2] Implement the `docs` and `publish` stubs, each naming its
      owning slice and exiting 2 rather than 0
- [ ] T029 [US2] Execute quickstart.md step 6 in full. **All four categories
      named in US2, not three**: misformatted code caught by `fmt --check`;
      a platform-specific dependency added to `fragcap-core` caught by
      `neutral`; a copyleft dependency caught by the license check; a missing
      SPDX header caught by `lint`. Confirm each names its specific violation
      and exits non-zero, then revert. Record the actual output
- [ ] T029a [US2] Install the non-host target with `rustup target add` so
      `cargo xtask neutral` can actually run, and record its passing output.
      Without this SC-006 is deferred to a workflow that cannot execute
- [ ] T029b [US2] Install and run the dependency license check locally once,
      proving `deny.toml` parses and its allowlist matches. If the tool cannot
      be installed, record FR-009 as scaffolded rather than passing

## Phase 5: User Story 3 - See what will ship (P2)

**Goal**: The crate graph and its metadata are inspectable and correct.

**Independent test**: Read workspace metadata and confirm it matches the
architecture of record.

- [ ] T030 [US3] Verify `cargo xtask deps` passes against the real workspace
      and record the output
- [ ] T031 [US3] Verify every crate manifest declares `license = "Apache-2.0"`
      and inherits workspace metadata rather than restating it
- [ ] T032 [US3] Verify with `cargo metadata` that no crate depends on
      `fragcap-cli` and that no mid-level crate depends on a sibling

## Phase 6: Workflows

Written to be correct when a remote exists. **None can execute during this
slice**, and each carries a comment saying so.

- [ ] T033 [P] Create `.github/workflows/ci.yml`: Linux and Windows matrix;
      fmt, clippy with `-D warnings`, test with `--locked`, `cargo xtask lint`,
      `cargo xtask deps`, and the native `fragcap-core` build on Linux that
      proves P-2
- [ ] T034 [P] Create `.github/workflows/platform.yml` for Windows
      capture-dependent tests, including the npcap SDK acquisition step,
      scaffolded and marked unexercised until S09
- [ ] T035 [P] Create `.github/workflows/audit.yml` running dependency
      vulnerability and license checks against `deny.toml`
- [ ] T036 [P] Create `.github/workflows/docs.yml` as a skeleton declaring its
      trigger and naming S18 as its owner
- [ ] T037 [P] Create `.github/workflows/links.yml` on a weekly schedule,
      naming S18 as its owner
- [ ] T038 [P] Create `.github/workflows/release.yml` as a tag-triggered
      skeleton that asserts no artifact contains npcap, per section 20.2
- [ ] T039 Validate every workflow file parses as YAML and record which ones
      have never executed

## Phase 7: Polish

- [ ] T040 Run `cargo xtask ci` in the foreground and record the complete
      output as the slice's verification evidence
- [ ] T041 [P] Add a `changelog.d/` fragment for the workspace scaffold, and a
      dated decisions fragment covering D-1, D-2, and D-3
- [ ] T042 [P] Update `README.md` to replace the pre-implementation status
      notice with build instructions, keeping the npcap prerequisite ahead of
      any usage instruction
- [ ] T043 [P] Update `AGENTS.md` to remove the "repository is
      pre-implementation" statement and name the real verification commands
- [ ] T044 Update `docs/plans/README.md` to mark the reconnaissance gate closed
      and S01 complete
- [ ] T045 Write the slice completion record listing every check that ran, its
      result, and every check that is scaffolded but unexercised, per FR-018
      and SC-007

## Dependencies

```text
Phase 1 (Setup)          -> Phase 2 (Foundational)
Phase 2                  -> Phase 3, Phase 4, Phase 5
Phase 4 (checks)         -> Phase 5 (uses the deps check)
Phase 2                  -> Phase 6 (workflows call xtask commands)
All                      -> Phase 7 (Polish)
```

Within Phase 2, T007 blocks T008 through T014, since every crate depends on
core directly or transitively. T013 blocks T014.

Within Phase 4, T020 through T022 precede T023 through T028: tests are written
failing first.

## Parallel Opportunities

- **Phase 1**: T002 through T006 are independent files.
- **Phase 2**: T008 through T012 are five independent crates, once T007 lands.
- **Phase 4**: T020, T021, and T022 are independent test modules.
- **Phase 6**: T033 through T038 are six independent files.
- **Phase 7**: T041, T042, and T043 touch different files.

## Implementation Strategy

**MVP scope**: Phases 1 through 3. That produces a workspace that builds and
tests, which is what every later slice actually needs.

**Why Phase 4 is not deferred despite being past the MVP line**: the checks
constrain seventeen slices of future work, and each one added after code exists
has to be introduced against a repository that already violates it. The cost of
adding them now is near zero; the cost later is a cleanup pass per check.

**Verification discipline**: T017, T018, T029, T030, T039, and T040 each
require recording actual command output. A task claiming a check passed without
that output is not complete, per P-9 and the slice's own SC-007.
