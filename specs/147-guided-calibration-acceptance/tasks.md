# Tasks: Guided Calibration Acceptance

**Input**: Design documents from `specs/147-guided-calibration-acceptance/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Required by FR-003, FR-006, FR-007, and the autopilot TDD protocol. Validator mutation tests and focused behavior tests precede implementation.

## Phase 1: Setup and Specification

- [x] T001 Assign issue #380 to Slice S147 in the delivery Project and retain its In Progress state
- [x] T002 Author and validate the S147 specification plus requirements and security checklists
- [x] T003 Complete clarification, research, data model, registry contract, quickstart, and implementation plan
- [x] T004 Run the spec-kit consistency analysis and resolve every blocking inconsistency

## Phase 2: Acceptance Inventory and Validator

- [x] T005 Run focused baseline guided-calibration and xtask tests without live or sensitive effects
- [x] T006 Add failing validator tests for schema, exact thirteen-item inventory, required fields, duplicate identities, evidence classification, duplicate references, path confinement, Git tracking, missing functions, ignored tests, and conditional tests
- [x] T007 Implement `xtask` guided-calibration acceptance registry parsing and validation
- [x] T008 Author the version 1 registry mapping every issue #380 criterion to exact controlled tests
- [x] T009 Wire the acceptance command into `cargo xtask ci` and make validator tests pass

## Phase 3: Controlled Behavioral Gaps

- [x] T010 Audit all thirteen criteria against exact current tests and record the trace in the registry
- [x] T011 Add failing store and CLI tests for durable `update` and `anti-cheat` no-effect pauses
- [x] T012 Implement the two missing closed pause reasons through store parsing, CLI parsing, mapping, and schema version 13 migration
- [x] T013 Add a failing controlled test for explicit selection of one ambiguous non-Steam stored client
- [x] T014 Implement stable stored-client candidates, a separate complete confirmation plan, fresh reproduction, and conditional persistence without rewriting Steam or publisher chains
- [x] T015 Remove the explicit case assertion from the three-topology controlled test and prove inferred defaults plus durable handoff
- [x] T016 Add focused assertions that the guided authorization event exposes exact target, launch, route, deadline, artifact, trust, and cleanup authority
- [x] T017 Make all focused guided-calibration tests pass across direct, Steam, publisher, warm, ambiguity, partial, trust-refusal, interruption, cleanup, resume, and handoff states

## Phase 4: Evidence and Documentation Boundary

- [x] T018 Update the master testing strategy to make real-game validation operator-owned and published-release-only
- [x] T019 Update architecture, outline, slice ordering, agent narrative, and public documentation with the exact S147 acceptance and non-claim
- [x] T020 Add S147 changed and decisions changelog fragments
- [x] T021 Verify that issue #380 can close from controlled implementation evidence while #334 remains open

## Phase 5: Convergence and Review

- [x] T022 Run the quickstart audit, requirements trace, punctuation, UTF-8, BOM, mojibake, and dependency checks
- [x] T023 Run spec-kit convergence and implement every traceable omission
- [x] T024 Run `cargo xtask ci` and review the complete diff and worktree
- [ ] T025 Commit, push, open a PR closing #380, and move Project Stage to PR review
- [ ] T026 Resolve every first-round review finding, trigger at most one `@Codex review` second round, resolve every second-round finding, and wait for green CI

## Dependencies and Execution Order

- Phase 1 gates implementation.
- Phase 2 establishes the durable evidence authority before any completion claim.
- Phase 3 fills only audited behavior gaps and otherwise reuses existing product tests.
- Phase 4 documents the deliberate release-validation policy correction.
- Phase 5 closes only after complete controlled evidence, CI, and review converge.
