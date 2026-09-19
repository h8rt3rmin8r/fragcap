# Tasks: S158 Windows gate determinism

**Input**: Design documents from `specs/158-windows-gate-determinism/`

**Prerequisites**: `spec.md`, `research.md`, `data-model.md`, `contracts/windows-gate-determinism.md`, `plan.md`

**Testing rule**: Each behavioral correction begins with a regression that fails against the pre-S158 implementation. No task may weaken an existing security, conservation, loss, deadline or cleanup assertion.

## Phase 1: Specification and analysis

- [x] T001 Create the S158 specification, clarification record and requirements checklist in `specs/158-windows-gate-determinism/`.
- [x] T002 Research retained #413 and #429 failures, current queue ownership, native terminal observations, deadline flow and calibration environment serialization.
- [x] T003 Define the writer readiness, observation drain, deadline ownership and test-isolation contracts.
- [x] T004 Produce the implementation plan and pass the blocking cross-artifact analysis gate.

## Phase 2: Writer readiness foundation

- [x] T005 [US1] Add a deterministic pre-ready stall regression and startup-failure ownership regression in `crates/fragcap/src/deep_capture/application.rs`; confirm failure against the original publication order.
- [x] T006 [US1] Implement the one-shot worker readiness handshake in `crates/fragcap/src/deep_capture/application.rs`, publishing the lease only after readiness and joining on startup failure.
- [x] T007 [US1] Run all application writer and application stream tests, confirming the 4,096 queue contract, ordered output and exact queue or storage loss reconciliation remain unchanged.

## Phase 3: Structured observation drain

- [x] T008 [US2] Add failing scripted-clock and proxy-result regressions in `crates/fragcap/tests/deep_capture_session.rs` for completed former-boundary crossing, incomplete drain, adapter error, genuine capture timeout, shutdown timeout, cancellation and cleanup continuation.
- [x] T009 [US2] Add the additive structured observation drain result to `crates/fragcap/src/deep_capture/adapters.rs` and update native plus controlled adapters in `crates/fragcap/src/deep_capture/native.rs` and tests.
- [x] T010 [US2] Correct `crates/fragcap/src/deep_capture/session.rs` so observation duration owns capture only and bounded complete drain evidence is collected under the remaining shutdown authority without the stale capture-clock check.
- [x] T011 [US2] Run the full facade Deep Capture session test target and verify every deadline, cancellation, observation and cleanup predicate.

## Phase 4: Calibration test isolation

- [x] T012 [US3] Add failing poison-recovery and unwind-restoration regressions in `crates/fragcap-cli/tests/cli_calibrate.rs`.
- [x] T013 [US3] Centralize controlled environment acquisition with poison recovery and exact RAII environment restoration in `crates/fragcap-cli/tests/cli_calibrate.rs`.
- [x] T014 [US3] Route every controlled-environment lock through the suite guard and cover every controlled variable with its exact unwind-safe snapshot, then run the exact #429 case and complete test binary.

## Phase 5: Reconciliation and verification

- [x] T015 [P] Record S158 behavior and decisions in `docs/fragcap-specification.md`, `docs/plans/README.md` and `changelog.d/` without changing the released baseline.
- [x] T016 Run `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all --locked`, `cargo xtask ci`, `cargo xtask msrv` and `cargo xtask neutral` in the foreground.
- [x] T017 Run text hygiene, UTF-8 without BOM and mojibake checks over every changed text file.
- [x] T018 Re-run spec-kit analysis and convergence, reconcile all tasks and record final verification evidence.

## Phase 6: Pull request and hosted acceptance

- [ ] T019 Commit S158, push `codex/s158-windows-gate-determinism`, publish the official pull request and attach it to this task.
- [ ] T020 Observe all first-attempt CI conclusions, including Windows native performance and platform, and preserve any failure as evidence rather than accepting a rerun.
- [ ] T021 Address every first-round review comment and CI defect, reply and resolve each thread, and push corrections.
- [ ] T022 Request at most one second Codex review with `@Codex review` when first-round findings are settled, then address and resolve every second-round comment without a third request.
- [ ] T023 Confirm every required check is green on the final head, every review is satisfied, both issues carry exact evidence and the branch is clean before requesting operator final review and merge.

## Dependencies and execution order

- T001 through T004 complete the mandatory spec-kit gate before code changes.
- T005 precedes T006 and T007.
- T008 precedes T009 through T011.
- T012 precedes T013 and T014.
- T015 follows settled behavior from T007, T011 and T014.
- T016 through T018 require all implementation and documentation tasks.
- T019 through T023 require local convergence and explicit user authorization, which is already present.

## Independent acceptance

- **US1**: The injected stalled worker cannot expose a sink, then releases into ordered zero-loss output under unchanged bounds.
- **US2**: Complete terminal drain remains valid across the former capture boundary, while incomplete, failed and genuinely late stages remain exact failures with cleanup.
- **US3**: One deliberate panic does not prevent a later controlled test, and environment values restore exactly on unwind.
