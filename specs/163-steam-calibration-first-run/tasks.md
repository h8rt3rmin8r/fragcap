# Tasks: S163 Steam Calibration First Run

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/first-run-cli.md](contracts/first-run-cli.md)

## Phase 1: Setup

- [x] T001 Confirm clean S163 branch, existing ignore rules, current CLI architecture, and synthetic fixture boundaries in `specs/163-steam-calibration-first-run/plan.md`

## Phase 2: Foundational

- [x] T002 Add controlled receipt-gated process identity tests before runtime changes in `crates/fragcap/tests/session.rs`
- [x] T003 Add controlled setup, target detail, and failure output tests in `crates/fragcap-cli/tests/cli_calibrate.rs`, `crates/fragcap-cli/tests/cli_deep_capture.rs`, and CLI module tests

## Phase 3: User Story 1 - Launch an owned Steam target

**Goal**: Bind only the platform process created by this authorized launch, then dispatch once.

**Independent test**: Synthetic receipt and ETW basename event bind; same-name foreign, stale, reused, wrong-parent, changed-path, early-exit, and watcher-loss cases do not authorize dispatch.

- [x] T004 [US1] Add receipt-scoped platform authority and precise event matching in `crates/fragcap/src/session.rs`
- [x] T005 [US1] Supply prepared path, launch interval, receipt PID, and parent to the session before dispatch in `crates/fragcap-cli/src/orchestrator.rs`
- [x] T006 [US1] Keep raw process image evidence and add one-shot dispatch regression in `crates/fragcap-cli/src/orchestrator.rs`

## Phase 4: User Story 2 - Establish the actual client

**Goal**: A novice can use guided observation or an explicitly labeled manual declaration without a socket-ownership guess.

**Independent test**: Synthetic socketless launcher and one observed socket-owning descendant produce a reviewed exact client plan; zero, ambiguous, declined, interrupted, and drift cases write nothing.

- [x] T007 [US2] Join ETW owned-chain instances to IP Helper socket rows under a bounded confirmed Steam launch, with unique/ambiguous/no-evidence decisions in `crates/fragcap-cli/src/commands/calibrate/steam_client_observation.rs`
- [x] T008 [US2] Add evidence source and selected executable to exact drift-checked Steam client setup plan in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T009 [US2] Replace socket-holder prompt with plain-language guided and manual-choice interactions in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T010 [US2] Cover no-write outcomes and controlled first-run chain in `crates/fragcap-cli/tests/cli_calibrate.rs`

## Phase 5: User Story 3 - Understand authority and outcome

**Goal**: Show actual stored client separately from Steam hint and explain the earliest known failure.

**Independent test**: Distinct hint/client output labels, causal failure stages, observed zero versus unavailable counts, exact fact writes, and one cleanup detail per resource.

- [x] T011 [US3] Render ordered active launch entries and labeled Steam hint in `crates/fragcap-cli/src/commands/targets.rs`
- [x] T012 [US3] Derive and print prioritized failure diagnosis from session process, packet, proxy, and fact evidence in `crates/fragcap-cli/src/session_ux.rs` and `crates/fragcap-cli/src/commands/deep_capture.rs`
- [x] T013 [US3] Remove duplicate generic cleanup progress without removing structured events or terminal detail in `crates/fragcap-cli/src/session_ux.rs`
- [x] T014 [US3] Cover human and JSON output boundaries in `crates/fragcap-cli/tests/cli_calibrate.rs`, `crates/fragcap-cli/tests/cli_deep_capture.rs`, and CLI module tests

## Phase 6: User Story 4 - Follow published first-run guide

**Goal**: Publish a chronological first-run and recovery guide with correct command syntax and outcomes.

**Independent test**: Every sample command parses; each guidance branch maps to a controlled session state and uses synthetic identities.

- [x] T015 [US4] Update first-run guide and compatibility/CLI reference in `site/content/docs/getting-started.mdx`, `site/content/docs/reference/deep-capture-compatibility.mdx`, and `site/content/docs/reference/cli.mdx`
- [x] T016 [US4] Reconcile normative text and chronological slice index in `docs/fragcap-specification.md` and `docs/plans/README.md`

## Phase 7: Polish and cross-cutting

- [x] T017 Add S163 feature and dated architecture decisions fragments in `changelog.d/`
- [x] T018 Run controlled focused tests and full foreground `cargo xtask ci`, then check UTF-8, LF, no mojibake, synthetic public text, and `git diff --check` in `crates/`, `docs/`, `site/`, `specs/`, and `changelog.d/`
- [x] T019 Reconcile all FR/SC and issue acceptance criteria, mark tasks complete, and commit locally on `codex/s163-steam-calibration-first-run`

## Dependencies

T001 precedes test scaffolding. T002 precedes T004-T006. T003 precedes T007-T014. US1 makes owned-chain observation trustworthy for US2. US2 precedes the controlled reachability chain. US3 and US4 depend on the resulting behavior. T017-T019 follow all stories.

## Parallel opportunities

Read-only research for root identity, client setup, and report/docs can run independently. Tests and code touching `calibrate.rs` are sequential. Documentation files can be authored independently after the command behavior is settled.

## Implementation strategy

Implement US1 as the first independently tested increment, then US2, US3, and US4. Run focused checks after each phase, followed by the complete repository gate and a local commit. Stop before push.
