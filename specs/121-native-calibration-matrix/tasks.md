# Tasks: Complete Native Calibration Matrix

**Input**: Design documents from `specs/121-native-calibration-matrix/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Required. S121 changes security-sensitive eligibility, persistent evidence, and migration behavior, so each story begins with failing contract or regression tests.

**Organization**: Tasks are grouped by independently testable user story and executed chronologically.

## Phase 1: Setup and Baseline

**Purpose**: Establish the exact branch, active feature, issue scope, and green starting point.

- [X] T001 Confirm clean `codex/121-native-calibration-matrix`, issue #317 scope, and `.specify/feature.json` pointer
- [X] T002 Run focused existing compatibility, Deep Capture session, CLI, and native conformance tests
- [X] T003 Record the S121 no-new-dependency and no-#318 scope boundary in `specs/121-native-calibration-matrix/plan.md`

---

## Phase 2: Foundational Case Vocabulary and Migration

**Purpose**: Establish the shared stored-case model required by every story.

- [X] T004 Add failing closed-token and applicability tests in `crates/fragcap-targets/src/compatibility.rs`
- [X] T005 Add failing version-9 migration preservation and version-10 round-trip tests in `crates/fragcap-targets/src/store.rs`
- [X] T006 Implement routing-strategy, address-family, protocol-family, case, and applicability types in `crates/fragcap-targets/src/compatibility.rs`
- [X] T007 Implement schema version 10 and additive migration in `crates/fragcap-targets/src/schema.rs` and `crates/fragcap-targets/src/store.rs`
- [X] T008 Implement new-row validation, reads, writes, matrix projection, and latest-applicable selection in `crates/fragcap-targets/src/compatibility.rs` and `crates/fragcap-targets/src/store.rs`
- [X] T009 Run the focused `fragcap-targets` model and migration tests

**Checkpoint**: Existing rows survive, new rows carry exact dimensions, and applicability has one shared authority.

---

## Phase 3: User Story 1 - Calibrate One Exact Native Case (Priority: P1)

**Goal**: Make every shipped native protocol family addressable by one bounded exact calibration plan.

**Independent Test**: Controlled tests select each protocol over IPv4 and IPv6, reject invalid combinations before effects, and produce positive facts only for matching classifications.

### Tests for User Story 1

- [X] T010 [P] [US1] Add CLI parsing and invalid-combination tests in `crates/fragcap-cli/src/cli.rs` and `crates/fragcap-cli/tests/cli_deep_capture.rs`
- [X] T011 [P] [US1] Add facade protocol-selection and mismatch tests in `crates/fragcap/src/deep_capture/policy.rs`
- [X] T012 [P] [US1] Add controlled IPv4/IPv6 protocol matrix rows in `crates/fragcap/tests/native_conformance.rs`

### Implementation for User Story 1

- [X] T013 [US1] Add `--calibration-protocol` and closed CLI mapping in `crates/fragcap-cli/src/cli.rs` and `crates/fragcap-cli/src/commands/deep_capture.rs`
- [X] T014 [US1] Carry immutable calibration-case identity through `crates/fragcap/src/deep_capture/model.rs`, `crates/fragcap/src/deep_capture/session.rs`, and CLI adapters
- [X] T015 [US1] Filter positive compatibility candidates through exact S120 protocol classifications in `crates/fragcap/src/deep_capture/policy.rs`
- [X] T016 [US1] Refuse unsupported phase/protocol and unavailable routing combinations before effects in facade and CLI preflight
- [X] T017 [US1] Run focused policy, CLI parsing, and controlled matrix tests

**Checkpoint**: Every supported protocol and loopback family has an explicit bounded calibration case.

---

## Phase 4: User Story 2 - Preserve an Append-Only Evidence History (Priority: P1)

**Goal**: Persist complete case identity without rewriting legacy or conflicting evidence.

**Independent Test**: Version-9 rows survive exactly, S121 rows round-trip every dimension, and conflicting retests remain chronological.

### Tests for User Story 2

- [X] T018 [P] [US2] Add append conservation and conflicting-retest tests in `crates/fragcap-targets/src/store.rs`
- [X] T019 [P] [US2] Add fact-adapter context and append-failure tests in `crates/fragcap-cli/src/commands/deep_capture.rs`
- [X] T020 [P] [US2] Add compatibility artifact reconciliation tests in `crates/fragcap-cli/tests/cli_deep_capture.rs`

### Implementation for User Story 2

- [X] T021 [US2] Enrich every calibration fact append with routing, family, protocol applicability, versions, and target-version evidence in `crates/fragcap-cli/src/commands/deep_capture.rs`
- [X] T022 [US2] Preserve proposed, successful, and failed append results with complete case identity in `crates/fragcap/src/deep_capture/model.rs` and artifact adapters
- [X] T023 [US2] Extend `compatibility.json` and manifest calibration fields without changing raw proxy detail in CLI artifact finalization
- [X] T024 [US2] Run migration, append, artifact, and manifest tests

**Checkpoint**: Calibration history is append-only, loss-accounted, and self-describing.

---

## Phase 5: User Story 3 - Consume Only Exact Current Evidence (Priority: P2)

**Goal**: Make ordinary eligibility and presentation consume the shared exact applicability model.

**Independent Test**: Every single-dimension permutation refuses, the latest exact row governs, and every output surface reports the same applicability.

### Tests for User Story 3

- [X] T025 [P] [US3] Add exhaustive single-dimension eligibility permutations in `crates/fragcap/src/deep_capture/policy.rs`
- [X] T026 [P] [US3] Add legacy, stale, mismatch, and target-detail rendering tests in `crates/fragcap-cli/src/commands/targets.rs`
- [X] T027 [P] [US3] Add calibration event and terminal case-identity tests in `crates/fragcap-cli/src/events.rs` and `crates/fragcap-cli/tests/cli_deep_capture.rs`

### Implementation for User Story 3

- [X] T028 [US3] Replace launch-case-only eligibility with latest exact current routing evidence in `crates/fragcap/src/deep_capture/policy.rs` and CLI target resolution
- [X] T029 [US3] Render complete case dimensions and applicability in `crates/fragcap-cli/src/commands/targets.rs`
- [X] T030 [US3] Extend calibration plan, phase, terminal, and JSON events in `crates/fragcap-cli/src/events.rs` and `crates/fragcap-cli/src/commands/deep_capture.rs`
- [X] T031 [US3] Run eligibility permutation, presentation, event, and end-to-end CLI tests

**Checkpoint**: No stale, legacy, or mismatched evidence authorizes a prepared case.

---

## Phase 6: Documentation and Cross-Cutting Verification

**Purpose**: Reconcile the architecture of record, public vocabulary, and full repository gate.

- [X] T032 Update `docs/fragcap-specification.md`, `docs/fragcap-spec-outline.md`, and `docs/plans/README.md` for S121 and retain #318 and #334 as open
- [X] T033 Add or update glossary entries and regenerate `docs/glossary/index.md`
- [X] T034 Add S121 feature and dated decision fragments under `changelog.d/`
- [X] T035 Update `AGENTS.md` with the landed S121 boundary and no dependency change
- [X] T036 Re-run spec-kit analyze and resolve every cross-artifact coverage or consistency finding
- [X] T037 Run focused quickstart commands and `cargo xtask ci` in the foreground
- [X] T038 Audit UTF-8 without BOM, LF, whitespace, Unicode dashes, mojibake, issue #317 scope, no staged `.specify/feature.json`, and dependency lock stability
- [X] T039 Mark every completed task `[X]`, review the complete diff, commit with a conventional S121 message, push the authorized branch, and open the official PR closing #317

---

## Dependencies and Execution Order

- Phase 1 establishes baseline and scope.
- Phase 2 blocks every story because it defines the shared schema and applicability contract.
- User Story 1 establishes immutable case selection before User Story 2 persists it.
- User Story 2 establishes complete append records before User Story 3 consumes and presents them.
- Documentation and full verification follow all three stories.

## Parallel Opportunities

- T010, T011, and T012 touch independent CLI, facade, and conformance test files after Phase 2.
- T018, T019, and T020 cover separate persistence, adapter, and artifact surfaces.
- T025, T026, and T027 cover separate policy, target-detail, and event surfaces.
- Documentation tasks T032 through T035 touch separate primary files but are applied sequentially in this single-agent run to preserve chronological review.

## Implementation Strategy

1. Make migration and exact applicability independently green.
2. Add selected-protocol planning and filtering without changing transport forwarding.
3. Persist the immutable case through existing append-only and artifact authorities.
4. Replace coarse eligibility and presentation with the shared applicability result.
5. Close documentation, hygiene, and full-gate requirements before commit.
