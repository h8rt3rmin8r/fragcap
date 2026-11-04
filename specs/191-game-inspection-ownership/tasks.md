# Tasks: Exact Game Inspection Ownership

**Slice**: S191. Complete #552 and #558 with repository-controlled evidence. Private installed-product observation follows release.

## Setup

- [x] T001 Read the constitution, architecture, issue reports, S189/S190 records, and current native process/socket/proxy boundaries in `docs/fragcap-specification.md` and `crates/`.
- [x] T002 Create S191 Spec-Kit specification, clarification, checklist, plan, research, data model, contract, and quickstart under `specs/191-game-inspection-ownership/`.

## User Story 1: Owned game handoff (#552)

- [x] T003 [US1] Add controlled multi-stage managed handoff and negative identity regressions in `crates/fragcap/tests/session.rs`, plus a Windows production orchestrator regression in `crates/fragcap-cli/src/orchestrator.rs`.
- [x] T004 [US1] Verify the landed handoff, packet-window, terminal-result, and shutdown authority in `crates/fragcap/src/session.rs` and `crates/fragcap-cli/src/orchestrator.rs`; the controlled path reproduced no further source defect.
- [x] T005 [US1] Verify exact cleanup/recovery state after owned helpers exit in `crates/fragcap-cli/src/commands/deep_capture/shutdown.rs` without rewriting the original partial session verdict.

## User Story 2: Exact pre-inspection decision (#558)

- [x] T006 [US2] Add typed requested/effective scope, immutable action, stable reason, and proxy-facing decision contract in `crates/fragcap/src/deep_capture/` and `crates/fragcap-proxy/src/`.
- [x] T007 [US2] Publish bounded live process and role evidence from `crates/fragcap-cli/src/orchestrator.rs` into `crates/fragcap/src/deep_capture/native.rs` without holding process locks through socket lookup.
- [x] T008 [US2] Reuse exact client-side socket, process generation, canonical path, account/session, launch receipt, ancestry, and selected-game role proof in `crates/fragcap/src/deep_capture/native.rs`; add identity and loss countercases.
- [x] T009 [US2] Integrate the immutable decision in native connection and association paths before TLS leaf, content, or key-log work in `crates/fragcap-proxy/src/runtime.rs` and transport modules; retain launch-chain default until #559/#561.
- [x] T010 [US2] Add bounded decision evidence and hook-order regressions in `crates/fragcap-proxy/src/` and `crates/fragcap/src/deep_capture/native.rs`.

## Closeout

- [x] T011 Update `docs/fragcap-specification.md`, `docs/plans/README.md`, `crates/fragcap/src/deep_capture/api.rs`, and `changelog.d/` for exactly the delivered behavior.
- [x] T012 Run Spec-Kit analysis and resolve findings in `specs/191-game-inspection-ownership/`.
- [x] T013 Run applicable format, lint, test, documentation, conventions, and wrapper checks; inspect encoding and diff in the repository root.
- [ ] T014 Commit S191 on `codex/s191-game-ownership-inspection`, push, publish the official PR, resolve reviews within the two-round cap, and wait for green CI.

## Dependencies

T003 precedes T004 and T005. T006 precedes T007 through T010. T011 follows implementation. T012 is performed before implementation and repeated after material spec changes. T013 and T014 follow all code and documentation work.
