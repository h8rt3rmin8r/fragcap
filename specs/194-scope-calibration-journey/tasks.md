# Tasks: Scope-Aware Calibration and Operator Journey

**Input**: Design documents from `specs/194-scope-calibration-journey/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Contract, migration, policy, CLI, presentation, artifact, and package checks are required.

## Phase 1: Setup and Contract Baseline

- [x] T001 Record S194 scope, issue mapping, chronology, and decisions in `docs/plans/README.md` and `changelog.d/S194-*.md`
- [x] T002 [P] Add S194 public terms and references in `docs/glossary/`
- [x] T003 Add failing scope applicability and migration tests in `crates/fragcap-targets/src/compatibility.rs`, `workflow.rs`, and `store.rs`

## Phase 2: Exact Scope and Durable Migration

- [x] T004 [US1] Add typed scope and middleware-handling dimensions to `crates/fragcap-targets/src/compatibility.rs`
- [x] T005 [US1] Add immutable scope and fallback intent to `crates/fragcap-targets/src/workflow.rs`
- [x] T006 [US1] Implement additive schema version 14 and migration in `crates/fragcap-targets/src/schema.rs` and `store.rs`
- [x] T007 [US1] Include scope in compatibility applicability, canonical case keys, and stored recommendation selection in `crates/fragcap-targets/src/compatibility.rs`
- [x] T008 [US1] Export the scope contract through `crates/fragcap-targets/src/lib.rs` and `crates/fragcap/src/deep_capture/api.rs`

## Phase 3: Guided Policy, Retry, and Fallback

- [x] T009 [US1] Add failing launcher-mediated auto-scope and legacy-inapplicability tests in `crates/fragcap/src/deep_capture/proposal.rs` and `policy.rs`
- [x] T010 [US1] Resolve requested scope before effects and keep inferred recommendations distinct from facts in `crates/fragcap/src/deep_capture/inspection.rs` and `proposal.rs`
- [x] T011 [US2] Add failing linked retry, refusal, and packet-fallback lifecycle tests in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T012 [US2] Persist immutable fallback authority and scoped attempt lineage in `crates/fragcap-targets/src/workflow.rs` and `store.rs`
- [x] T013 [US2] Implement one fresh game-scope successor and declared packet-only fallback in `crates/fragcap-cli/src/commands/calibrate.rs` and `deep_capture.rs`
- [x] T014 [US2] Project effective mode, evidence windows, omissions, and cleanup independently through `crates/fragcap/src/deep_capture/model.rs`, `manifest.rs`, and CLI assessment

## Phase 4: Public Operator Journey

- [x] T015 [US3] Add shared `--inspect-scope`, `--retry-game-scope`, and `--fallback-capture` parsing and conflicts in `crates/fragcap-cli/src/cli.rs`
- [x] T016 [US3] Bind scope and fallback to pre-effect authorization and generated commands in `crates/fragcap-cli/src/commands/calibrate.rs`, `deep_capture.rs`, and `workflow_help.rs`
- [x] T017 [US3] Reconcile human and JSON scope, result, lineage, and next-action output in `crates/fragcap-cli/src/events.rs` and `session_ux.rs`
- [x] T018 [US3] Add exact-scope readiness to target details and Doctor in `crates/fragcap-cli/src/commands/targets.rs` and `doctor/`
- [x] T019 [US3] Add parser/help/output/Doctor/target controlled matrices in `crates/fragcap-cli/src/` tests

## Phase 5: Documentation, Certification, and Verification

- [x] T020 [P] Update architecture, API, artifact, schema, first-run, calibration, Doctor, troubleshooting, and package documentation in `docs/` and `site/content/docs/`
- [x] T021 Add CI package artifact contract checks for the scope parser and controlled journey in existing package certification tests
- [x] T022 Run Spec-Kit analysis, resolve every finding, and mark all S194 tasks and checklists complete
- [x] T023 Run focused tests, full workspace tests, Clippy, formatting, conformance, lint, spec, docs, package certification, encoding, and mojibake checks

## Dependencies and Execution Order

- T001-T003 establish the contract and red tests.
- T004-T008 complete US1's durable exact-scope foundation.
- T009-T010 finish US1 policy.
- T011-T014 complete US2 and depend on durable scope.
- T015-T019 complete US3 and depend on US1/US2 contracts.
- T020-T023 reconcile documentation and evidence after behavior is stable.

## Independent Test Criteria

- **US1**: Migrate legacy data, propose launcher-mediated game scope, store exact scoped evidence, and prove mismatch inapplicable.
- **US2**: Produce one authorized linked retry or typed refusal and one exact packet fallback without positive inspection evidence.
- **US3**: Parse and render the same scope contract across calibration, Deep Capture, authorization, targets, Doctor, human output, JSON, help, and package artifact checks.

## Implementation Strategy

Complete the durable store and policy boundary first, then lifecycle fallback, then the public operator journey. Each phase preserves old readers and can be verified before the next one consumes it.
