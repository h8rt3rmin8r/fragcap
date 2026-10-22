# Tasks: Truthful CLI Action Boundaries

**Branch**: `codex/s178-command-action-truth`

## Phase 1: Specification and analysis

- [x] T001 Create S178 spec, clarify decisions, requirements and action checklists, plan, research, data model, CLI contract, and quickstart.
- [x] T002 Run Spec-Kit analyze against the constitution, specification, plan, tasks, and CLI contract; resolve findings before implementation.

## Phase 2: Bundle action scope and path guidance (US1)

- [x] T003 [US1] Write red missing/unowned/zero-work/eligible preview, human/JSON reason parity, retained scope, exact authorized collection, and cleanup/export missing-path tests in `crates/fragcap-cli/tests/cli_bundle.rs`.
- [x] T004 [US1] Update `crates/fragcap-cli/src/commands/bundle.rs` to gate Apply advice on existing preview facts, show existing row details, and give read-only guidance for non-actionable previews.
- [x] T005 [US1] Update bundle cleanup/export missing-path preflight in `crates/fragcap-cli/src/commands/bundle.rs` without changing facade authorization or deletion.
- [x] T006 [US1] Clarify deletion, retained selection, preview, and purge scope in `crates/fragcap-cli/src/cli.rs` and any relevant embedded help.

## Phase 3: Target selector integrity (US2)

- [x] T007 [US2] Write red two-target conflict, no-selector, single-selector, and action-specific missing-reference tests in `crates/fragcap-cli/tests/cli_targets.rs`.
- [x] T008 [US2] Add optional selector mutual exclusion in `crates/fragcap-cli/src/cli.rs` and a defensive direct-call guard in `crates/fragcap-cli/src/commands/targets.rs`.
- [x] T009 [US2] Make show/remove/export missing-reference advice action-specific in `crates/fragcap-cli/src/commands/target_resolve.rs` and `targets.rs` while preserving capture resolution.

## Phase 4: Catalog seed preflight and actual counts (US3)

- [x] T010 [US3] Write red absent/invalid source, unsupported launch, duplicate tier, default seed, valid bare-array, existing-store unchanged on bad input, and help tests in `crates/fragcap-cli/tests/cli_catalog.rs` and focused catalog tests.
- [x] T011 [US3] Prevalidate tier/source and parse named documents before `Store::open` in `crates/fragcap-cli/src/commands/catalog.rs`; deduplicate first-occurrence tier requests and count distinct actual fills.
- [x] T012 [US3] Correct parser/help and runtime source guidance in `crates/fragcap-cli/src/cli.rs` and `catalog.rs`, including feature-gated network flags and bare-array document shapes.

## Phase 5: Discovery side-effect truth (US4)

- [x] T013 [US4] Write red first-run help/output/store-effect tests in `crates/fragcap-cli/tests/cli_targets.rs` and `cli_help.rs`.
- [x] T014 [US4] Correct discover, scan, and bare-target short/full help in `crates/fragcap-cli/src/cli.rs` and `workflow_help.rs`, preserving existing seeding and registration behavior.

## Phase 6: Documentation and exact-head verification

- [x] T015 Update `docs/fragcap-specification.md` and chronological `docs/plans/README.md`; add an unreleased `changelog.d/S178-*.md` fragment.
- [x] T016 Re-run Spec-Kit analyze and convergence checks, focused tests, `cargo xtask ci`, and text hygiene; inspect the final diff for every acceptance scenario and unchanged authorization boundaries.

## Dependencies

T002 blocks implementation. Within each user story the test task precedes the implementation tasks. The final documentation and verification phase follows all four independent user stories. The owner-authorized commit, push, PR, CI, and review sequence follows the completed task list under the plan's workflow gate.
