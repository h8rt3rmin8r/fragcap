# Tasks: Session Causal Diagnostics

**Slice**: S190 | **Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), [contract](contracts/session-diagnostics.md)

## Phase 1: Setup and Analysis

- [x] T001 Confirm branch, selector, merged S189 baseline, #551/#553 issue text, governing context, and current diagnostic/phase owners in `specs/190-session-causal-diagnostics/plan.md`.
- [x] T002 Complete Spec-Kit specify, clarify, checklist, plan, tasks, and analyze for `specs/190-session-causal-diagnostics/`, resolving consistency findings before implementation.

## Phase 2: User Story 1, proxy failure cause (P1)

- [x] T003 [US1] Add controlled failing TLS and timeout assertions for the current missing connection facts in `crates/fragcap-proxy/src/runtime.rs` and `crates/fragcap-proxy/src/http1.rs`.
- [x] T004 [US1] Define bounded versioned connection stage, boundary, alert, and transport fields in `crates/fragcap-proxy/src/model.rs` and `crates/fragcap-proxy/src/http1.rs`.
- [x] T005 [US1] Record actual HTTP CONNECT stage transitions, client and upstream TLS completion, and failure boundary in `crates/fragcap-proxy/src/runtime.rs`, `crates/fragcap-proxy/src/tls.rs`, and `crates/fragcap-proxy/src/upstream.rs`.
- [x] T006 [US1] Preserve successful HTTP exchange facts through an idle header timeout and identify timeout phase in `crates/fragcap-proxy/src/http1.rs` and `crates/fragcap-proxy/src/runtime.rs`.
- [x] T007 [US1] Project bounded connection diagnostics, exact or unavailable process ownership, and loss counts through `crates/fragcap/src/deep_capture/diagnostics.rs`, `crates/fragcap/src/deep_capture/application.rs`, `crates/fragcap/src/deep_capture/native.rs`, and `crates/fragcap-cli/src/commands/calibrate/assessment.rs`.
- [x] T008 [US1] Retain and inspect the versioned diagnostic timeline in `proxy.jsonl` through `crates/fragcap/src/deep_capture/lifecycle.rs`, `crates/fragcap-cli/src/commands/deep_capture.rs`, and focused controlled tests.

## Phase 3: User Story 2, phase time (P2)

- [x] T009 [US2] Add controlled phase-timing assertions for late acquisition, no client, full observation, and interrupted completion in `crates/fragcap/tests/deep_capture_session.rs` and `crates/fragcap-cli/src/commands/deep_capture.rs`.
- [x] T010 [US2] Record monotonic phase starts, ends, and outcomes under their current budget owners in `crates/fragcap/src/deep_capture/session.rs`, `crates/fragcap-cli/src/orchestrator.rs`, and `crates/fragcap-cli/src/commands/deep_capture.rs`.
- [x] T011 [US2] Emit actual elapsed and remaining milliseconds with truthful never-started status through the additive event and artifact contracts in `crates/fragcap/src/deep_capture/model.rs`, `crates/fragcap/src/deep_capture/adapters.rs`, `crates/fragcap-cli/src/events.rs`, `crates/fragcap-cli/src/commands/calibrate/assessment.rs`, and `crates/fragcap-cli/src/commands/deep_capture.rs`.

## Phase 4: Reconciliation and PR

- [x] T012 Update the shipped contract and chronological slice ordering in `docs/fragcap-specification.md` and `docs/plans/README.md`; add `changelog.d/S190-session-causal-diagnostics.fixed.md`.
- [x] T013 Run source, documentation, conventions, and encoding checks; reconcile all scoped requirements and tests against `specs/190-session-causal-diagnostics/spec.md` and `quickstart.md`.
- [ ] T014 Commit and push `codex/s190-session-causal-diagnostics`, open the official PR, resolve every external finding within at most two Codex review rounds, and confirm applicable exact-head CI succeeds.
- [ ] T015 Give the owner the PR, exact head, review disposition, CI result, and remaining post-release #542/#552 questions for final review and merge.

## Dependencies

T001-T002 precede implementation. T003 precedes T004-T008. T009 precedes T010-T011. T012-T013 follow both stories. T014-T015 follow source completion. No source task is marked parallel because several owners cross the proxy and CLI projections.
