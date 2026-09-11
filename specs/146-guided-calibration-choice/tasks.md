# Tasks: Explicit Guided Calibration Choice

**Input**: Design documents from `specs/146-guided-calibration-choice/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Required by FR-018 and the autopilot TDD protocol. Test tasks precede implementation and must demonstrate the expected red state.

## Phase 1: Setup and Specification

- [x] T001 Create issue #406 as an S146 child of #380, add it to the delivery Project and milestone, and set Slice, Stage, and Status
- [x] T002 Author and validate the S146 specification plus requirements and security checklists
- [x] T003 Complete clarification, research, data model, command contract, quickstart, and implementation plan
- [x] T004 Run the spec-kit consistency analysis and resolve every blocking inconsistency

## Phase 2: Foundational Choice and Store Contract

- [x] T005 Run focused baseline target-store, guided-calibration, event, and CLI parse tests
- [x] T006 Add failing candidate identity tests for determinism, field sensitivity, evidence-order independence, parser strictness, and duplicate authority in `crates/fragcap-targets/src/choice.rs`
- [x] T007 Implement and export the canonical candidate identity domain in `crates/fragcap-targets/src/choice.rs` and `crates/fragcap-targets/src/lib.rs`
- [x] T008 Add failing workflow model and version 11 to 12 migration tests for launch, routing, and family intent in `crates/fragcap-targets/src/workflow.rs` and `crates/fragcap-targets/src/store.rs`
- [x] T009 Implement schema version 12, record version 1 additive intent, validated round trip, and immutable workflow intent in `crates/fragcap-targets/src/schema.rs`, `crates/fragcap-targets/src/store.rs`, and `crates/fragcap-targets/src/workflow.rs`
- [x] T010 Make the focused `fragcap-targets` choice, workflow, and migration suite pass

## Phase 3: User Story 1 - Select One Exact Candidate

- [x] T011 [US1] Add failing CLI parse, choice-event serialization, ambiguous target, ambiguous Steam client, selected target, selected Steam client, duplicate, stale, and structured-output tests
- [x] T012 [US1] Add `--candidate` and the stable `calibration.choice_required` event in `crates/fragcap-cli/src/cli.rs` and `crates/fragcap-cli/src/events.rs`
- [x] T013 [US1] Implement exact-once candidate consumption for registration and Steam client setup in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T014 [US1] Preserve existing confirmation and post-confirmation rediscovery authority after selection
- [x] T015 [US1] Make all focused candidate-choice tests pass with zero effects on refusal

## Phase 4: User Story 2 - Bind Advanced Exact-Case Intent

- [x] T016 [US2] Add failing parse, persistence, resume-conflict, IPv6 propagation, supported topology, unsupported routing, and launch-mismatch tests
- [x] T017 [US2] Add guided launch-case, routing-strategy, and proxy-family arguments and mappings in `crates/fragcap-cli/src/cli.rs`
- [x] T018 [US2] Persist fresh exact-case intent and rebuild every proposal from it in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T019 [US2] Validate launch assertions against fresh inferred topology and propagate family into every delegated low-level attempt
- [x] T020 [US2] Extend stable human and JSON guidance with exact-case values and the optional launch assertion
- [x] T021 [US2] Make all focused exact-case tests pass across direct, Steam, publisher, pause, and resume paths

## Phase 5: User Story 3 - Refuse Misapplied Choice

- [x] T022 [US3] Add failing malformed, unknown, unused, duplicate-use, resume-mutation, and drifted-authority tests
- [x] T023 [US3] Centralize strict candidate parsing, matching, consumption, and pre-workflow unused validation
- [x] T024 [US3] Verify every refusal creates no workflow, plan, bundle, launch, trust, or compatibility fact

## Phase 6: Documentation, Convergence, and Review

- [x] T025 Update architecture, outline, slice ordering, agent narrative, and public CLI reference
- [x] T026 Add S146 changed and decisions changelog fragments
- [x] T027 Run the quickstart audit, requirements trace, punctuation, encoding, mojibake, and dependency checks
- [x] T028 Run spec-kit convergence and implement every traceable omission
- [x] T029 Run `cargo xtask ci` and review the complete diff and worktree
- [ ] T030 Commit, push, open a PR closing #406, and move Project Stage to PR review
- [ ] T031 Resolve every first-round review finding, trigger at most one `@Codex review` second round, resolve every second-round finding, and wait for green CI

## Dependencies and Execution Order

- Phase 1 gates implementation.
- Phase 2 establishes shared identity and durable intent.
- User Story 1 consumes candidate identity, User Story 2 consumes durable intent, and User Story 3 closes misuse paths.
- Documentation and review begin only after focused and full gates pass.
