# Tasks: S188 Managed stage path verification

**Issues**: #547
**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/stage-acquisition.md](contracts/stage-acquisition.md), [quickstart.md](quickstart.md)
**Tests**: Required for the exact managed stage, adverse identity, production packet gate, and composed CLI result contracts.

## Phase 1: Setup

- [x] T001 Confirm clean `origin/main`, `codex/s188-managed-stage-path-verification`, `.specify/feature.json`, issue #547, and the S187 merged baseline in `specs/188-managed-stage-path-verification/plan.md`.
- [x] T002 Specify, clarify, checklist, plan, research, data model, contract, and quickstart for #547 under `specs/188-managed-stage-path-verification/`.
- [x] T003 Review `crates/fragcap-core/src/process/tree.rs`, `crates/fragcap-profile/src/matching.rs`, `crates/fragcap/src/session.rs`, `crates/fragcap/src/deep_capture/process.rs`, `crates/fragcap-cli/src/orchestrator.rs`, and `crates/fragcap-cli/src/commands/deep_capture.rs` for one stage-proof path.

## Phase 2: Foundational identity seam

**Goal**: Give one process-tree generation a separately verified path without altering its observed image.

- [x] T004 Add failing basename-only verified-path and raw-image preservation tests in `crates/fragcap-core/src/process/tree.rs` and `crates/fragcap-profile/src/matching.rs`.
- [x] T005 Add optional verified path to the process-tree generation and evaluate exact path predicates through it in `crates/fragcap-core/src/process/tree.rs` and `crates/fragcap-profile/src/matching.rs`.
- [x] T006 Add failing event-to-process join tests for directional creation time, PID reuse, basename, account/session, path unavailability, and bounded limitation inventory in `crates/fragcap/src/deep_capture/process.rs`.
- [x] T007 Implement the query-only managed candidate join and typed limitation evidence in `crates/fragcap/src/deep_capture/process.rs`, without adding process-control rights or ordinary Capture queries.

## Phase 3: User Story 1 - Exact simple Steam client

**Goal**: A basename-only terminal client binds under its owned platform and opens scoped packet capture.
**Independent Test**: Synthesized simple platform profile plus real acquisition path binds only with an event-bound verified path.

- [x] T008 [US1] Extend the synthesized platform-profile test with failing basename-only acquisition and wrong-path cases in `crates/fragcap-cli/src/commands/target_resolve.rs`.
- [x] T009 [US1] Feed verified managed path evidence into `CaptureSession` before matching through `crates/fragcap/src/session.rs` and `crates/fragcap-cli/src/orchestrator.rs`; preserve original ETW text and S163 root authority.
- [ ] T010 [US1] Add controlled Windows production-path live capture with synthetic Steam root, loopback traffic, scoped pcapng, and unmodified TShark readback under `crates/fragcap-cli/tests/` or the existing isolated package-certification row.

## Phase 4: User Story 2 - Ordered publisher stages and refusals

**Goal**: The same exact evidence rule works for every declared stage and refuses ambiguous chains.
**Independent Test**: Ordered publisher stages bind from basename-only events; each adverse identity variant refuses the affected stage.

- [x] T011 [US2] Add failing ordered publisher, stale PID, escaped-parent, missing-path, and ambiguous-instance cases through `CaptureSession` in `crates/fragcap-cli/src/commands/target_resolve.rs` and the event join in `crates/fragcap/src/deep_capture/process.rs`.
- [x] T012 [US2] Preserve ordered ancestry and report bounded stage identity limitations through `crates/fragcap/src/deep_capture/process.rs` and `crates/fragcap-cli/src/orchestrator.rs` without creating a stage transition.
- [ ] T013 [US2] Extend the isolated Windows production-path fixture to the declared publisher chain and wrong-path/stale/missing/escaped variants under `crates/fragcap-cli/tests/` or the same package-certification row.

## Phase 5: User Story 3 - Truthful acquisition result

**Goal**: A failed acquisition names `acquisitiontimeout` at the top level while each artifact retains its own facts.
**Independent Test**: Controlled composed failure reads back CLI result, partial packet, proxy, manifest, and cleanup evidence.

- [ ] T014 [US3] Add failing no-acquisition live outcome and composed Deep Capture timeout tests in `crates/fragcap-cli/src/orchestrator.rs` and `crates/fragcap-cli/src/commands/deep_capture.rs`.
- [x] T015 [US3] Preserve `CaptureOutcome.stop_reason` on the no-acquisition path and map it to human and structured CLI errors in `crates/fragcap-cli/src/orchestrator.rs` and `crates/fragcap-cli/src/commands/deep_capture.rs`.
- [ ] T016 [US3] Assert independent packet, proxy, manifest, and cleanup truth after acquisition failure in the controlled Deep Capture test under `crates/fragcap-cli/tests/` or `crates/fragcap-cli/src/commands/deep_capture.rs`.

## Phase 6: Documentation, gates, and delivery

- [x] T017 Update shipped-behavior architecture and chronological slice ordering in `docs/fragcap-specification.md` and `docs/plans/README.md`; add `changelog.d/S188-managed-stage-path-verification.fixed.md` and a dated decisions fragment if architecture changes.
- [x] T018 Run non-destructive Spec-Kit analysis across `specs/188-managed-stage-path-verification/` and resolve all critical or high coverage, ambiguity, and constitution findings before implementation completion. All nine FRs and five SCs map to T004-T019; no critical or high finding remains.
- [ ] T019 Run focused tests, foreground `cargo xtask ci`, applicable isolated Windows/package gates, UTF-8 and mojibake checks; record exact passing and unavailable evidence in `specs/188-managed-stage-path-verification/verification.md`.
- [ ] T020 Commit only S188 files, push `codex/s188-managed-stage-path-verification`, and open an official PR with `Closes #547`; verify exact head and published diff.
- [ ] T021 Monitor exact-head CI, all Codex and security review threads and reactions, answer and resolve every finding, use at most one second `@Codex` request, and hand the green PR to the owner for final review and merge.

## Dependencies and Execution Order

T003 grounds the code changes. T004-T007 establish the shared identity seam before T008-T013 use it. T008-T010 complete the simple Steam path, then T011-T013 extend the same proof to ordered publishers. T014-T016 are independently testable but share final result artifacts with the first two stories. T017-T021 follow passing controlled acceptance. Every test task precedes its implementation task, and one agent executes shared-file edits sequentially.

## Implementation Strategy

The first complete increment is US1 with its production-path Windows row; #547 closes only when US2, US3, and all repository-controlled gates also pass. The production fixture uses synthetic identities and isolated CI resources, never the private field bundle. The owner retains the final review and merge ritual.
