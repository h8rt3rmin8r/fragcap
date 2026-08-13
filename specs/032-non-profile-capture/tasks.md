---
description: "Task list for S032 non-profile production capture path"
---

# Tasks: Non-Profile Production Capture Path

**Input**: Design documents from `specs/032-non-profile-capture/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Included (TDD per the constitution; each user story carries an
Independent Test).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Parallelizable (different files, no dependency on incomplete tasks)
- **[Story]**: US1..US3 map to the spec's user stories
- Paths are repository-relative.

## Phase 1: Setup

- [X] T001 No new files or dependencies; confirm the branch builds
  (`cargo build --workspace`) before wiring begins.

## Phase 2: Foundational (blocking prerequisites for all stories)

- [X] T002 Add the pure accessor `Target::identity(&self) -> Option<&MatchPredicates>` in `crates/fragcap-profile/src/target.rs`, returning the resolved identity for the `Observed`/`EngineRule`/`PlatformWalker` origins and `None` for `Profile`; re-export it as needed and add a unit test asserting `Some` for a non-profile origin and `None` for a profile origin.
- [X] T003 Change `RunArgs.profile` to `Option<String>` and add `--install-dir: Option<PathBuf>` and `--steam: Option<String>` in `crates/fragcap-cli/src/cli.rs`, all in one clap `ArgGroup` (`required = true`, `multiple = false`); update every existing read of `args.profile` to `args.profile.as_deref()`.

## Phase 3: User Story 1 - Capture a game from its install directory (P1)

**Goal**: `run --install-dir <dir>` resolves the client from install layout,
synthesizes a `heuristic-unverified` one-stage identity, and captures it.

**Independent test**: `cargo test -p fragcap-cli nonprofile` over an Unreal
fixture layout through the offline harness.

- [X] T004 [US1] Add a private `synthesize_profile(identity: &MatchPredicates, game_id, game_name, app_id: Option<&str>) -> Result<Profile, CliError>` helper in `crates/fragcap-cli/src/commands/run.rs` that serializes the identity's present predicates into a one-stage JSON profile stamped `heuristic-unverified` and parses it via `Profile::parse` (mirroring `watch::synthesize_profile`).
- [X] T005 [US1] Branch `run` in `crates/fragcap-cli/src/commands/run.rs`: for `--install-dir <path>` build `ResolutionRequest::for_install(path, ...)`, resolve; if the resolved target has a profile use it (unchanged), else read `Target::identity`, synthesize the one-stage profile, and capture via the same `orchestrator::capture` call, reusing `assemble::effective_config(args, &synthesized)`.
- [X] T006 [US1] Offline test in `crates/fragcap-cli` (a `tests/` file or module): build a fixture Unreal install directory plus a process script that starts the resolved shipping client, run `run --install-dir <fixture>` through the offline harness, and assert the client is captured through a synthesized `heuristic-unverified` identity, reproducing the attribution an equivalent authored one-stage identity produces.

## Phase 4: User Story 2 - Capture a Steam-installed game by app id (P1)

**Goal**: `run --steam <app_id>` resolves the install directory via the Steam
library lookup, then takes the US1 path.

**Independent test**: `cargo test -p fragcap-cli steam` over a fake Steam library
fixture.

- [X] T007 [US2] In `crates/fragcap-cli/src/commands/run.rs`, handle `--steam <app_id>`: call `fragcap::steam::install_root_for(app_id)`, and on success feed the resolved install directory into the same `for_install` + synthesize + capture path as US1; carry the app id onto the synthesized profile's `game.app_id`.
- [X] T008 [US2] Test that a not-installed app id (via a fake library fixture whose lookup returns not-found) yields a surfaced failure (exit 1) naming the missing title and captures nothing; and that an app id mapping to a recognized-engine directory drives the US1 non-profile path.

## Phase 5: User Story 3 - Honest fidelity and honest failure (P1)

**Goal**: the synthesized identity is `heuristic-unverified` and never
`authored`; declines are surfaced with their reason and capture nothing; the
`--profile` path is byte-identical.

**Independent test**: fidelity + decline + golden tests.

- [X] T009 [US3] In `crates/fragcap-cli/src/commands/run.rs`, render a non-profile `ResolutionError::Unresolved(u)` into a surfaced failure (exit 1) that names the resolver's notes (engine-rule ambiguity, walker ambiguity, unreadable path); keep the `--profile` branch's existing `From<ResolutionError>` mapping unchanged.
- [X] T010 [P] [US3] Test asserting the synthesized profile carries `heuristic-unverified` and never `authored` (assert on `synthesize_profile`'s output).
- [X] T011 [P] [US3] Tests: `run --install-dir <dir>` over an unrecognized layout, an ambiguous layout, and an unreadable directory each exit 1 with a message naming the reason and capture nothing (P-4).
- [X] T012 [P] [US3] Test that the `run --profile` capture output is byte-identical to the existing goldens (reuse/extend the corpus-pipeline or run golden coverage), proving the profile path is untouched.
- [X] T013 [P] [US3] Command-line parsing tests: none of the three inputs, and more than one, are usage errors (exit 2); exactly one parses (FR-005).

## Phase 6: Polish & Cross-Cutting Concerns

- [X] T014 [P] Add a glossary entry for "non-profile capture path" under `docs/glossary/` and regenerate the index (`bash scripts/lint-docs.sh fix`); run `bash scripts/lint-docs.sh check` (P-6).
- [X] T015 [P] Document the non-profile capture path in `docs/fragcap-specification.md` (under the resolution-cascade or run-command section) and reframe the "run cannot capture yet" note as resolved (P-6, FR-011).
- [X] T016 Add changelog fragments `changelog.d/032-non-profile-capture.added.md` and `changelog.d/032-non-profile-capture.decisions.md` recording the activation, the three-input arg group, the `heuristic-unverified` synthesis, the `Target::identity` accessor, and the decline-reason surfacing.
- [X] T017 Run `cargo xtask ci` and `cargo xtask msrv` in the foreground and resolve any findings (fmt, clippy, test, lint, deps, license, docs, MSRV).

## Dependencies & Execution Order

- **Phase 1 -> Phase 2 -> Phases 3-5 -> Phase 6.**
- Foundational (Phase 2) blocks all stories: `Target::identity` (T002) and the
  arg group (T003) are prerequisites.
- US1 (Phase 3) is the MVP; T004 before T005; T005 before T006.
- US2 (Phase 4) depends on US1's synthesize+capture path (T005).
- US3 (Phase 5): T009 depends on T005; T010/T011/T012/T013 are [P] once the
  branch (T005) and arg group (T003) exist.
- Polish (Phase 6) after the stories; T017 is the final gate.

## Parallel Opportunities

- US3 tests T010/T011/T012/T013 are [P] with each other (distinct assertions).
- Doc tasks T014/T015 are [P] with each other.

## Implementation Strategy

- **MVP = User Story 1** (Foundational + Phase 3): `run --install-dir` captures a
  resolved-but-unprofiled target through a synthesized heuristic identity. This
  alone activates the cascade and is a shippable increment.
- Layer US2 (`--steam` convenience), then US3 (the fidelity/decline/byte-identical
  guarantees), then Polish and the full gate.
