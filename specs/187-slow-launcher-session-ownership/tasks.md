# Tasks: S187 Slow launcher session ownership

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), [session-and-process.md](contracts/session-and-process.md)
**Issues**: #540 and #541

## Phase 1: Setup and blocking authority

- [x] T001 Confirm S187 branch, selector, spec directory, issue mapping, and merged #539 baseline.
- [x] T002 Run Spec-Kit analyze over `specs/187-slow-launcher-session-ownership/` and resolve all critical or high findings before implementation. The installed-path acceptance gap was corrected in the plan, quickstart, and T009; no critical or high finding remains.
- [x] T003 [P] Identify every deadline clamping and presentation path in `crates/fragcap/src/deep_capture/` and `crates/fragcap-cli/src/commands/deep_capture.rs`.
- [x] T004 [P] Identify Steam topology, exact shutdown scope, trace projection, and connection correlation paths in `crates/fragcap/src/`, `crates/fragcap-cli/src/`, and `crates/fragcap-core/src/`.

## Phase 2: User Story 1, explicit finite session timing

**Goal**: Exact two-minute calibration, ten-minute ordinary observation, and five-minute explicit launch wait with no silent clamp.
**Independent Test**: Controlled plans and production-path fixture retain requested/effective values, run through requested observation, and refuse over-cap values before effects.

- [x] T005 [US1] Add failing CLI and facade deadline tests in `crates/fragcap-cli/src/commands/deep_capture.rs` and `crates/fragcap/tests/deep_capture_session.rs` for 120/600/300 seconds, zero, over-cap, and independent release budgets.
- [x] T006 [US1] Replace deadline clamping with preflight validation and new ceilings in `crates/fragcap/src/deep_capture/model.rs`, `crates/fragcap/src/deep_capture/session.rs`, and `crates/fragcap-cli/src/commands/deep_capture.rs`.
- [x] T007 [US1] Give calibration the separate capture-return allowance in `crates/fragcap/src/deep_capture/session.rs` and update deadline tests so the full requested span remains available.
- [x] T008 [US1] Add requested/effective timing to canonical authorization, structured terminal event, human report, and artifact record in `crates/fragcap-cli/src/commands/deep_capture.rs` and `crates/fragcap-cli/src/events.rs`.
- [ ] T009 [US1] Exercise a controlled two-process slow launcher through `crates/fragcap-cli/tests/`, existing cancellation and cleanup paths, and an isolated `scripts/Test-PackageCertification.ps1` installed-binary fixture using that runner account's documented default paths with canonical-path readback. The hosted installed fixture remains to run.

## Phase 3: User Story 2, cold Steam publisher chain

**Goal**: Ordered exact intermediary roles in immutable preparation and pre-effect authorization, with observation separate from control.
**Independent Test**: Steam target with launcher, intermediates, and client prepares one exact profile and shutdown scope; unrelated same-name and missing-path cases refuse authority.

- [x] T010 [US2] Add failing topology and profile tests in `crates/fragcap/src/deep_capture/proposal.rs`, `crates/fragcap/src/managed_launch.rs`, and `crates/fragcap-cli/src/commands/target_resolve.rs`.
- [x] T011 [US2] Accept an ordered Steam publisher declaration and cold readiness in `crates/fragcap/src/deep_capture/proposal.rs` while retaining one-client compatibility.
- [x] T012 [US2] Canonicalize and retain every declared stage and exact shutdown path in `crates/fragcap/src/managed_launch.rs` and pre-effect authorization in `crates/fragcap-cli/src/commands/deep_capture.rs`.
- [x] T013 [US2] Synthesize ordered exact-path profile stages under the platform root in `crates/fragcap-cli/src/commands/target_resolve.rs`; preserve causal ancestry through transient shims.
- [x] T014 [US2] Verify `crates/fragcap-cli/src/commands/deep_capture/shutdown.rs` controls only exact preauthorized process objects and treats undeclared helpers and same-name processes as exit or unrelated observations.

## Phase 4: User Story 3, evidence and release reconciliation

**Goal**: Complete bounded descendant trace, truthful connection ownership from inception, and finite route-owner recovery.
**Independent Test**: Controlled first-second connection and late-helper fixture gives exact, ambiguous, or unavailable connection relation; trace and cleanup account for every relevant descendant.

- [x] T015 [US3] Add failing managed-descendant and PID-reuse tests in `crates/fragcap/src/deep_capture/process.rs` and review the existing `crates/fragcap/tests/loopback_correlation.rs` packet evidence boundary.
- [x] T016 [US3] Project exact-root descendants by process generation and event time into the bounded process trace with explicit projection and watcher omission accounting in `crates/fragcap/src/deep_capture/process.rs`.
- [x] T017 [US3] Add first-connection exact, ambiguous, and unavailable cases in `crates/fragcap/src/deep_capture/native.rs`, pre-client independent output coverage in `crates/fragcap/src/deep_capture/application.rs`, and a controlled routed launcher/client proxy test.
- [x] T018 [US3] Establish independent socket-to-process-instance correlation at native proxy connection inception in `crates/fragcap/src/deep_capture/native.rs` and relevant capture authority, preserving packet ScopeWithheld truth.
- [x] T019 [US3] Cover helper born during release, absent or late client, same-name unrelated process, proxy continuity, partial cleanup, and Doctor exit prerequisite in `crates/fragcap-cli/src/commands/deep_capture/shutdown.rs` and controlled CLI tests.

## Phase 5: Documentation, gates, and delivery

- [x] T020 Update `docs/fragcap-specification.md`, `docs/plans/README.md`, and `changelog.d/` with exact S187 behavior, evidence limits, and any architecture-affecting decisions.
- [ ] T021 Run focused tests, full foreground `cargo xtask ci`, applicable isolated Windows/package gates, and UTF-8/mojibake checks; inspect failures and fix within scope.
- [x] T022 Re-run Spec-Kit analysis if requirements or architecture changed, mark tasks accurately, and commit the exact S187 branch. Post-design analysis found no critical or high cross-artifact conflict; T009 and T021 remain open until hosted installed-package and exact-head gates finish.
- [ ] T023 Push the explicitly authorized branch, open an official PR with `Closes #540` and `Closes #541` only when both implementation gates pass, and verify the published diff.
- [ ] T024 Monitor exact-head CI, all Codex/security review comments and reactions, and at most one second `@Codex` review request; answer and close every finding, then hand the green PR to the owner before merge.

## Dependencies and execution order

T002 blocks implementation. T005-T009 establish the timing contract needed by the slow-chain fixture. T010-T014 establish preauthorized stage scope before T019 can test release. T015-T018 establish evidence truth; T019 integrates all three stories. T020-T024 follow passing controlled acceptance. Research checks T003 and T004 are independent. User Story 1 is the first independently shippable capability; issue closure and PR wording require all three stories.

## Parallel opportunities

The timing path (T005-T009) and read-only topology research (T004) can proceed independently. Topology/profiling tests and process trace tests can be developed independently before integration. All mutations to shared authorization and session code are serialized in this branch.
