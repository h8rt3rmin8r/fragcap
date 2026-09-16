# Tasks: S154 regression reliability and native documentation readiness

**Input**: [Specification](spec.md), [plan](plan.md), [research](research.md), [data model](data-model.md), [contracts](contracts/readiness.md) and [quickstart](quickstart.md).

## Phase 1: Setup

- [x] T001 Confirm clean main baseline, create slice branch and feature pointer in specs/154-regression-documentation/spec.md.
- [x] T002 Complete specification, ten-category clarification and requirements checklist in specs/154-regression-documentation/checklists/requirements.md.

## Phase 2: Foundation

- [x] T003 Consolidate independent read-only planning research and existing StreamSink semantics in specs/154-regression-documentation/research.md.
- [x] T004 Run blocking spec/plan/tasks analysis and retain its result in specs/154-regression-documentation/verification.md before implementation.

## Phase 3: US1, trustworthy stalled-consumer evidence

**Independent test**: Twenty fresh acknowledged-stall runs, full ordered independent files, exact loss and active TCP transport coverage.

- [x] T005 [US1] Add controlled registration/arming/blocked acknowledgement with finite cleanup and twenty isolation scenarios in crates/fragcap-sink/tests/streaming_backpressure.rs (FR-001 to FR-003, SC-001).
- [x] T006 [US1] Keep real TCP coverage as an accepting read consumer and full independent file with exact reports in crates/fragcap-sink/tests/streaming_tcp.rs (FR-004).
- [x] T007 [US1] Run focused controlled sink tests, including timeout and idle cases, and record outcomes in specs/154-regression-documentation/verification.md.

## Phase 4: US2, verified native documentation

**Independent test**: Eleven-topic complete inventory, negative drift diagnostics, existing example readers and production site checks.

- [x] T008 [US2] Add failing missing/duplicate/unsafe/absent/ignored evidence contracts and small coverage validator in xtask/src/docs_coverage.rs (FR-005, SC-002).
- [x] T009 [US2] Add closed page/test inventory in docs/audits/native-documentation-coverage.v1.json and invoke validation from xtask/src/docs.rs (FR-005).
- [x] T010 [US2] Author discoverable engineering readiness reference in site/content/docs/reference/native-documentation.mdx, link site/content/docs/meta.json, and correct bounded actual protocol gaps in site/content/docs/reference/deep-capture-compatibility.mdx (FR-006 to FR-007).
- [x] T011 [US2] Run default/net CLI examples, applicable artifact readers and production unit/build/accessibility gates; record outcomes in specs/154-regression-documentation/verification.md (SC-003).

## Phase 5: Verification and review handoff

- [x] T012 Add dated decisions and fixes/docs fragments in changelog.d/ and append chronological S154 narrative to docs/plans/README.md.
- [x] T013 Run proportionate complete local gates, inspect diff/encoding and record evidence in specs/154-regression-documentation/verification.md.
- [ ] T014 Commit, push and publish official PR closing #420/#421, linking open #331; reconcile GitHub Project slice/stage tracking.
- [ ] T015 Address every actual review, request at most one second round, verify all required final-head CI and provide human merge handoff with review evidence in specs/154-regression-documentation/verification.md (FR-008, SC-004).

## Dependencies and execution order

Setup precedes foundation and blocking analysis. US1 and US2 are independently testable after foundation; execute US1 first for the reliability MVP, then documentation. Each story's contracts precede its integration. Final verification and PR monitoring follow both stories.

## Parallel opportunities

Independent read-only documentation research ran alongside source inspection under the planning skill. Sink and documentation tests may run in separate verified hidden sessions once changes are complete; no concurrent edits to shared files are scheduled.

## Implementation strategy

Preserve the existing runtime and validators, add only the missing controlled and traceability coverage, verify each story independently, then complete hosted checks and review reconciliation. Explicit push authorization removes the default pre-push pause, not human merge or truthful acceptance boundaries.
