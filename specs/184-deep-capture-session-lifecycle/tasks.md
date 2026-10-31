# Tasks: S184 Deep Capture session lifecycle

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/session-lifecycle.md](contracts/session-lifecycle.md)

## Phase 1: Spec-Kit and regression setup

- [x] T001 Verify S184 branch, selector, issue mapping, requirements checklist, and non-destructive Spec-Kit analysis before code edits.
- [x] T002 [US1] Add failing controlled Windows regression in `crates/fragcap-cli/src/commands/deep_capture/shutdown.rs` for exact root and client plus an unproven same-name event, checking exact control and exit.
- [x] T003 [US2] Add failing deadline and installed-path regressions in `crates/fragcap-cli/src/commands/deep_capture.rs` and `crates/fragcap/src/deep_capture/session.rs` for ordinary duration above 60 seconds, calibration behavior, invalid values, and plan/runtime truth.

## Phase 2: Owner release and cleanup

- [x] T004 [US1] Separate proven owners and unresolved observations in `crates/fragcap-cli/src/commands/deep_capture/shutdown.rs`; continue bounded exact-owner release and late-event verification.
- [x] T005 [US1] Reconcile route-owner, route, target-route, proxy, and trust results in `crates/fragcap/src/deep_capture/session.rs`, `routing.rs`, and CLI trust/Doctor boundaries with exact recoverable obligations.

## Phase 3: Ordinary duration

- [x] T006 [US2] Validate mode-specific duration before effects, preserve ordinary accepted observation through the CLI and facade, and account separately for Capture return time in `deep_capture.rs` and `session.rs`.
- [x] T007 [US2] Reconcile authorization plan, terminal report, documentation, and controlled installed-path timing evidence for ordinary and calibration modes.

## Phase 4: Delivery

- [x] T008 Update `docs/fragcap-specification.md`, affected CLI documentation, `docs/plans/README.md`, and `changelog.d/` with #526/#527 behavior and explicit #528/#529 limits.
- [x] T009 Run focused tests, complete repository gates and applicable Windows/package checks in the foreground, and record exact results in `verification.md`.
- [x] T010 Commit only S184 files, push `codex/s184-deep-capture-session-lifecycle`, and open an official PR closing #526 and #527.
- [x] T011 Monitor exact-head CI and external Codex/security reviews, respond to and resolve every actionable comment, use no more than two total review rounds, and hand the final green PR to the owner before merge.
