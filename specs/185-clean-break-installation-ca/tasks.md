# Tasks: S185 Clean-break installation CA

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/ca-lifecycle.md](contracts/ca-lifecycle.md)

## Phase 1: Setup and foundations

- [x] T001 Confirm literal S185 branch, feature selector, issues #528/#529, constitution amendment, and Spec-Kit quality checklists.
- [x] T002 Add controlled identity, signer-pairing, signature, and exact machine-store assertions in certificate and CA test modules; hosted package checks own real store execution.
- [x] T003 Implement reloadable installation authority, key-protection/readback, and exact LocalMachine/Root operations in `crates/fragcap-proxy/src/`.

## Phase 2: User Story 1, install once and retain trust

- [x] T004 [US1] Add package assertions for first install, repeat, repair, upgrade, exact rollback, and unrelated-transaction retention; real MSI execution remains a hosted gate.
- [x] T005 [US1] Implement versioned ProgramData identity, exact pre-S185 cleanup, idempotent provisioning, and rollback in the CLI CA module.
- [x] T006 [US1] Add disclosed WiX deferred elevated install/rollback actions and package assertions in `crates/fragcap-cli/wix/main.wxs` and package checks; route portable `ca install` through the same backend.

## Phase 3: User Story 2, session continuity

- [x] T007 [US2] Add two controlled signer reloads with HTTP CONNECT and QUIC/H3 leaf issuer assertions, plus controlled owner-release failure with retained trust; no wire-level session claim.
- [x] T008 [US2] Replace production per-session authority creation and CurrentUser/Root trust mutation with a proven installation signer. Bind fingerprint to authorization and manifest; make session cleanup leave installation trust untouched.
- [x] T009 [US2] Keep proxy/route-owner partial status independent of retained CA trust and update controlled regression evidence for #528.

## Phase 4: User Story 3, diagnose and repair

- [x] T010 [US3] Add seven controlled health fixtures, actual hidden-binary startup probes for help/version/JSON/non-Deep status, and pre-effect refusal coverage.
- [x] T011 [US3] Implement read-only startup status, privileged pre-effect signing gate, Doctor status, and explicit `ca status/install/repair/remove` commands.
- [x] T012 [US3] Reconcile help, session presentation, and error guidance with installation trust.

## Phase 5: Delivery

- [x] T013 Update master specification, threat model, install/portable/upgrade docs, package inventory, `docs/plans/README.md`, and `changelog.d/` with the clean cutover and issue limits.
- [x] T014 Rerun focused tests, `cargo xtask ci`, Windows/package source checks, encoding and mojibake sanity checks on the review-fix head, and update exact evidence in `verification.md`; hosted MSI execution remains under T016.
- [x] T015 Commit the completed S185 slice, push the branch, open and attach the official PR referencing #528 and #529.
- [ ] T016 Watch exact-head CI and all Codex/security reviews, answer and resolve every actionable comment, request no more than one second review round, and hand the green PR to the owner before merge.

## Dependencies and execution

T002-T003 block US1 and US2. T004-T006 establish installation state before T007-T009. T010-T012 can follow T003, but the gate depends on the final transaction shape. T013-T016 follow all user-story tasks. Controlled test modules can be written before implementation under TDD; no independent agent work is required.
