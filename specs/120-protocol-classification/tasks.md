# Tasks: Exhaustive Protocol Classification

**Input**: Design documents from `specs/120-protocol-classification/`

**Tests**: Required by the specification, constitution, and autopilot TDD discipline.

## Phase 1: Setup

- [x] T001 Record the active S120 scope and file inventory in `specs/120-protocol-classification/plan.md`
- [x] T002 [P] Add initial failing classification contract tests in `crates/fragcap/tests/protocol_classification.rs`
- [x] T003 [P] Add classification reconciliation expectations to `crates/fragcap/tests/application_stream.rs`
- [x] T004 [P] Add compatibility eligibility failure cases to `crates/fragcap/src/deep_capture/policy.rs`

## Phase 2: Foundational Classification Contract

- [x] T005 Implement schema version, traffic family, detection state, inspectability state, outcome reason, and validation in `crates/fragcap/src/deep_capture/classification.rs`
- [x] T006 Export the classification API from `crates/fragcap/src/deep_capture/mod.rs`
- [x] T007 Replace coarse compatibility observation labels with one validated classification plus retained raw evidence in `crates/fragcap/src/deep_capture/model.rs`
- [x] T008 Map every native proxy observation and refusal into the facade classification contract in `crates/fragcap/src/deep_capture/native.rs`

## Phase 3: User Story 1 - Understand Every Traffic Outcome

**Goal**: Every published traffic matrix cell has one valid versioned classification.

**Independent Test**: Run the exhaustive table and invalid-combination cases without a live target.

- [x] T009 [US1] Complete exhaustive traffic-family and state transition tests in `crates/fragcap/tests/protocol_classification.rs`
- [x] T010 [US1] Add proxy mapping coverage for HTTP, TLS, SOCKS, TCP, UDP, QUIC, HTTP/3, unknown, unsupported, and parser-failed evidence in `crates/fragcap/tests/native_proxy.rs`
- [x] T011 [US1] Update controlled conformance rows to assert exact classifications in `crates/fragcap/tests/native_conformance.rs`

## Phase 4: User Story 2 - Preserve Failure And Omission Authority

**Goal**: Protocol, trust, retention, and artifact failures remain separately attributable.

**Independent Test**: Inject each required reason and prove raw details survive classification and artifact projection.

- [x] T012 [US2] Emit additive versioned classification objects and header identity in `crates/fragcap/src/deep_capture/application.rs`
- [x] T013 [US2] Reconcile classification and bounded-loss counts in the application trailer and prefix reader in `crates/fragcap/src/deep_capture/application.rs`
- [x] T014 [US2] Add typed artifact omission construction and severity mapping in `crates/fragcap/src/deep_capture/manifest.rs`
- [x] T015 [US2] Use typed omission construction during bundle finalization in `crates/fragcap-cli/src/commands/deep_capture.rs`
- [x] T016 [US2] Cover not-routed, not-reached, opaque, pinned, client-auth, unsupported-version, parser-failed, truncated, and writer-failed authority separation in `crates/fragcap/tests/application_stream.rs` and `crates/fragcap-cli/src/commands/deep_capture.rs`

## Phase 5: User Story 3 - Derive Compatibility Facts Without Guessing

**Goal**: Durable compatibility facts require exact eligible classifications.

**Independent Test**: Feed every detection, inspectability, and reason state through candidate selection and calibration outcomes.

- [x] T017 [US3] Implement per-fact classification eligibility predicates in `crates/fragcap/src/deep_capture/policy.rs`
- [x] T018 [US3] Update calibration outcome selection to preserve parser and processing failures as inconclusive evidence in `crates/fragcap/src/deep_capture/policy.rs`
- [x] T019 [US3] Serialize classification and eligibility evidence in compatibility output in `crates/fragcap-cli/src/commands/deep_capture.rs`
- [x] T020 [US3] Complete exhaustive positive and negative fact-promotion tests in `crates/fragcap/src/deep_capture/policy.rs` and `crates/fragcap-cli/src/commands/deep_capture.rs`

## Phase 6: User Story 4 - Reconcile Human And Machine Summaries

**Goal**: Every displayed summary derives from the same conserved classification counts.

**Independent Test**: Compare application records, compatibility output, manifest omissions, human events, and JSON events for one mixed session.

- [x] T021 [US4] Add a conserved `ClassificationSummary` derived only from retained observations in `crates/fragcap/src/deep_capture/classification.rs`
- [x] T022 [US4] Carry the shared summary through Deep Capture terminal events in `crates/fragcap/src/deep_capture/model.rs` and `crates/fragcap/src/deep_capture/session.rs`
- [x] T023 [US4] Render the shared classification and omission counts in human and JSON output in `crates/fragcap-cli/src/events.rs` and `crates/fragcap-cli/src/commands/deep_capture.rs`
- [x] T024 [US4] Add human and JSON reconciliation integration coverage in `crates/fragcap-cli/tests/cli_deep_capture.rs`

## Phase 7: Documentation And Completion

- [x] T025 [P] Add glossary entries and regenerate the index in `docs/glossary/capture-and-networking.md` and `docs/glossary/index.md`
- [x] T026 [P] Update the normative architecture and traffic matrix in `docs/fragcap-specification.md` and `docs/fragcap-spec-outline.md`
- [x] T027 [P] Record S120 status in `docs/plans/README.md`, `crates/fragcap-proxy/README.md`, and `AGENTS.md`
- [x] T028 [P] Add S120 feature and dated decision fragments under `changelog.d/`
- [x] T029 Mark every completed task and set the slice status to Complete in `specs/120-protocol-classification/spec.md` and `specs/120-protocol-classification/tasks.md`
- [x] T030 Run focused tests and `cargo xtask ci` from the repository root
- [x] T031 Verify UTF-8 without BOM, LF, whitespace, dash, mojibake, dependency, and staged-file hygiene across the complete diff

## Dependencies

- Phase 1 precedes all implementation.
- Phase 2 is foundational and blocks all user stories.
- User Story 1 establishes the matrix required by User Stories 2 and 3.
- User Story 2 establishes serialized authority required by User Story 4.
- User Story 3 may proceed after User Story 1 while User Story 2 is underway.
- User Story 4 depends on User Stories 1 and 2.
- Documentation and completion follow all user stories.

## Parallel Opportunities

- T002 through T004 touch independent test surfaces.
- T010 and T011 can proceed after T005 through T008.
- T017 through T020 are independent from manifest construction once classification types exist.
- T025 through T028 touch separate documentation and changelog files.

## Implementation Strategy

1. Establish the typed matrix and invalid-state tests first.
2. Map native evidence without deleting raw fields.
3. Add artifact and compatibility projections only after the core contract passes.
4. Finish with one shared conserved summary and full repository verification.
