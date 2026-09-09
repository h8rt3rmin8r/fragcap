# Tasks: Target Discovery Integrity

**Input**: Design documents from `/specs/133-target-discovery-integrity/`

**Tests**: Required. Every behavior change begins with a failing focused test.

## Phase 1: Shared policy foundations

- [x] T001 [P] Add failing automatic-registration policy tests in `crates/fragcap-targets/src/register.rs` for authoritative Steam identity, strong engine evidence, location-only refusal, and conserved decision counts
- [x] T002 Implement `AutomaticRegistrationDecision`, refusal reasons, and the conserved batch policy in `crates/fragcap-targets/src/register.rs`
- [x] T003 [P] Add failing normalized excluded-subtree tests in `crates/fragcap-targets/src/sources/known_roots.rs`, including the excluded root itself, descendants, case, separator, and component-boundary cases
- [x] T004 Implement exact excluded-subtree pruning in `crates/fragcap-targets/src/sources/known_roots.rs`
- [x] T005 Export the new registration-policy vocabulary from `crates/fragcap-targets/src/lib.rs` and run the focused `fragcap-targets` tests

## Phase 2: User Story 1 - Safe automatic discovery and registration

- [x] T006 [US1] Add failing facade inventory tests in `crates/fragcap/tests/steam_source.rs` proving exact Steam client and title roots remain distinct, including unknown-engine titles
- [x] T007 [US1] Export the exact Steam inventory from `crates/fragcap/src/discovery.rs` and inject its client root into known-roots exclusions in `crates/fragcap-cli/src/commands/targets.rs`
- [x] T008 [US1] Add failing CLI registration tests in `crates/fragcap-cli/src/commands/targets.rs` for hero and Doctor admission/refusal accounting
- [x] T009 [US1] Route every automatic discovery caller through the shared high-precision admission policy in `crates/fragcap-cli/src/commands/targets.rs`
- [x] T010 [US1] Run focused facade and CLI tests and confirm discovery conservation remains intact

## Phase 3: User Story 2 - Explicit broad discovery without persistence

- [x] T011 [US2] Add failing CLI argument and rendering tests for `targets discover --summary` and detailed automatic-eligibility reasons in `crates/fragcap-cli/src/cli.rs` and `crates/fragcap-cli/src/commands/targets.rs`
- [x] T012 [US2] Implement privacy-safe aggregate discovery output and detailed eligibility rendering without changing persistence in `crates/fragcap-cli/src/commands/targets.rs`
- [x] T013 [US2] Add output privacy assertions proving summary mode emits no target names, paths, application identifiers, or path-bearing warnings
- [x] T014 [US2] Run focused CLI tests for detailed and count-only discovery modes

## Phase 4: User Story 3 - Conservative reconciliation of historical rows

- [x] T015 [P] [US3] Add failing pure reconciliation-planner tests in `crates/fragcap-targets/src/reconcile.rs` for exact platform infrastructure, authoritative duplicates, multi-product aggregates, user-authored rows, ambiguous provenance, anchored rows, refusal counts, and bounded/truncated inventory
- [x] T016 [US3] Implement the typed platform inventory, reconciliation dispositions, reasons, plan, and conserved counts in `crates/fragcap-targets/src/reconcile.rs`
- [x] T017 [P] [US3] Add failing atomic unchanged-row deletion tests in `crates/fragcap-targets/src/store.rs`, including stale and missing rows causing all-or-nothing refusal
- [x] T018 [US3] Implement transactional compare-and-delete for exact `TargetEntry` values in `crates/fragcap-targets/src/store.rs`
- [x] T019 [US3] Export reconciliation types and run focused `fragcap-targets` planner/store tests
- [x] T020 [US3] Add failing CLI parser and command tests for `targets reconcile`, preview-only default, `--yes`, exact row reporting, and stale-plan refusal in `crates/fragcap-cli/src/cli.rs` and `crates/fragcap-cli/src/commands/targets.rs`
- [x] T021 [US3] Implement Steam inventory composition, reconciliation preview, explicit confirmation, recomputation, and atomic application in `crates/fragcap-cli/src/commands/targets.rs`
- [x] T022 [US3] Run focused reconciliation CLI tests and verify no preview path mutates the store

## Phase 5: User Story 4 - Auditable specification and validation evidence

- [x] T023 [P] [US4] Correct target discovery and reconciliation requirements in `docs/fragcap-specification.md` and its navigation in `docs/fragcap-spec-outline.md`
- [x] T024 [P] [US4] Record the approved S133 issue-order deviation and dependencies in `docs/plans/README.md` and the architectural state in `AGENTS.md`
- [x] T025 [P] [US4] Add S133 changelog fragments for the corrected automatic-registration and reconciliation behavior
- [x] T026 [US4] Run the count-only real-machine validation command and record aggregate results only in `specs/133-target-discovery-integrity/real-machine-validation.md`

## Phase 6: Analysis, verification, and delivery

- [x] T027 Run the spec-kit cross-artifact analysis and resolve every critical or high finding
- [x] T028 Run formatting, focused tests, workspace tests, Clippy, and all repository xtask gates from `specs/133-target-discovery-integrity/quickstart.md`
- [x] T029 Verify UTF-8 without BOM, LF integrity, mojibake absence, no dependency/schema drift, and a clean intended diff
- [x] T030 Mark the S133 specification and validation evidence complete, then commit the completed slice with repository trailers
- [ ] T031 Push `codex/133-target-discovery-integrity`, publish the official pull request closing issue #375, and move the project item to PR review
- [ ] T032 Wait for CI and first-round external reviews, address and resolve every comment, and push corrections
- [ ] T033 Trigger at most one explicit second `@Codex` review round, address and resolve every new comment, and confirm all CI checks are green
- [ ] T034 Hand off the green, review-satisfied pull request for the user's final review and merge ritual

## Dependencies and execution order

- Phase 1 blocks every user story.
- US1 establishes the automatic path used by US2 reporting and US3 inventory composition.
- US2 and US3 are independently testable after Phase 1, but execute chronologically in this slice.
- Documentation can proceed after contracts settle; real-machine evidence requires the implemented summary mode.
- Delivery begins only after analysis and all local gates pass.
