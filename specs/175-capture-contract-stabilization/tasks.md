# Tasks: Capture Contract Stabilization

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/capture.md](contracts/capture.md), and [quickstart.md](quickstart.md).

**Tests**: Required for every included active-release defect. Write each focused regression before its implementation and confirm it fails for the expected reason.

## Phase 1: Setup

- [x] T001 Confirm S175 branch, six open issue contracts, existing gate commands, and no conflicting checkout changes in `specs/175-capture-contract-stabilization/plan.md`.
- [x] T002 Validate the requirement-quality checklist in `specs/175-capture-contract-stabilization/checklists/requirements.md`.

## Phase 2: Foundational design

- [x] T003 Record the live and replay deadline time-domain seam and pcapng header-boundary decision in `specs/175-capture-contract-stabilization/research.md`.
- [x] T004 Confirm direction and payload decisions can be applied at one shared gate/factory without new dependency edges in `specs/175-capture-contract-stabilization/plan.md`.
- [x] T005 Run the read-only Spec-Kit analyze gate over `specs/175-capture-contract-stabilization/spec.md`, `plan.md`, and `tasks.md`; resolve every finding before implementation.

## Phase 3: User Story 1 - Elapsed capture duration (P1)

**Goal**: #484 stops on elapsed time without corrupting absolute packet and process evidence.

**Independent Test**: Epoch-stamped controlled replay and live source cases cover busy, sparse, silent, pending-terminal, and terminal-stage outcomes.

- [x] T006 [US1] Add epoch-stamped replay and controlled live-driver deadline regressions in `crates/fragcap-cli/tests/cli_capture.rs` and `crates/fragcap-cli/src/orchestrator.rs`.
- [x] T007 [US1] Separate live monotonic and replay virtual deadline ticks from absolute packet timestamps in `crates/fragcap-cli/src/orchestrator.rs`, and enforce replay duration before sinks in `crates/fragcap/src/session.rs`.
- [x] T008 [US1] Verify #484 stop causes and bounded replay output without changing packet evidence timestamps in `crates/fragcap-cli/tests/cli_capture.rs`.

## Phase 4: User Story 2 - Enforced scope and metadata retention (P1)

**Goal**: #485, #486, and #490 make accepted role, direction, and payload options true of every output path.

**Independent Test**: The bidirectional fixture and known payload marker produce exact role errors, selected-direction records, accounting, and payload-free pcapng and JSON Lines artifacts.

- [x] T009 [US2] Add empty, interior-empty, case-mismatch, and custom-role regressions in `crates/fragcap-cli/tests/cli_capture.rs` and `crates/fragcap-cli/src/assemble.rs`.
- [x] T010 [US2] Validate exact profile roles before source and sink setup in `crates/fragcap-cli/src/assemble.rs`.
- [x] T011 [US2] Add bidirectional direction and conservation regressions in `crates/fragcap/src/session.rs` and `crates/fragcap-cli/tests/cli_capture.rs`.
- [x] T012 [US2] Apply requested direction in `crates/fragcap/src/session.rs`, and surface separate known-opposite and unknown counts in `crates/fragcap-cli/src/orchestrator.rs`, `crates/fragcap-cli/src/events.rs`, `crates/fragcap-cli/src/output.rs`, and `crates/fragcap-cli/src/live_status/mod.rs`.
- [x] T013 [US2] Add conservative header-boundary tests in `crates/fragcap-core/src/parse/mod.rs` and pcapng length and marker regressions in `crates/fragcap-sink/src/pcapng/mod.rs`.
- [x] T014 [US2] Implement the bounded header-only prefix decision in `crates/fragcap-core/src/parse/mod.rs` using its existing link, network, and transport parsers.
- [x] T015 [US2] Thread payload mode through pcapng writer and `SinkFactory` in `crates/fragcap-sink/src/pcapng/mod.rs` and `crates/fragcap-sink/src/transport/mod.rs`.
- [x] T016 [US2] Map global and sink-specific payload settings through ordinary, rotated, streamed, FIFO, and ring paths in `crates/fragcap-cli/src/assemble.rs`.
- [x] T017 [US2] Verify markers and EPB lengths through ordinary, explicit, and ring output in `crates/fragcap-cli/tests/cli_capture.rs`; verify Deep Capture argument inheritance in `crates/fragcap-cli/src/commands/deep_capture.rs` and independently read a controlled 24-frame metadata-only artifact with installed TShark.

## Phase 5: User Story 3 - Usable sink names and accurate help (P2)

**Goal**: #496 and #497 make valid filenames and Capture guidance work as described.

**Independent Test**: Comma-path cases produce exact files with optional modifiers; malformed syntax fails; help and fixture examples match actual mode and bound counters.

- [x] T018 [US3] Add comma destination and malformed separator regressions in `crates/fragcap-cli/src/args.rs` and `crates/fragcap-cli/tests/cli_capture.rs`.
- [x] T019 [US3] Implement doubled-comma destination decoding while preserving existing modifiers in `crates/fragcap-cli/src/args.rs`.
- [x] T020 [US3] Add ring, stream, and volume-bound help expectations in `crates/fragcap-cli/tests/cli_help.rs` and fixture count checks in `crates/fragcap-cli/tests/cli_capture.rs`.
- [x] T021 [US3] Correct Capture and shared bound help in `crates/fragcap-cli/src/cli.rs`, with matching current-source contract text in `docs/fragcap-specification.md`.

## Phase 6: Polish and delivery

- [x] T022 Reconcile the S175 sequence and issue mapping in `docs/plans/README.md` and add a user-visible fragment under `changelog.d/`; the architecture rationale remains in this slice's `research.md`.
- [x] T023 Run focused tests and `cargo xtask ci` in the foreground, inspect all output, and perform the UTF-8 and mojibake check on changed files.
- [ ] T024 Review the exact diff and six issue acceptance criteria, commit the slice on `codex/s175-capture-contract-stabilization`, push the authorized branch, and open the official PR with six `Closes` references.
- [ ] T025 Wait for exact-head CI and bot reviews, resolve every finding and review thread within at most two review rounds, then hand off the green PR for owner merge.

## Dependencies and execution order

T001-T005 precede implementation. US1, US2, and US3 are independently testable, but each uses shared Capture fixtures and is implemented in the listed order for stable regression interpretation. T006, T009, T011, T013, T018, and T020 are test-first tasks. T021-T025 follow all three stories. Do not treat a green unrelated test or an unperformed field run as closure evidence for any included issue.
