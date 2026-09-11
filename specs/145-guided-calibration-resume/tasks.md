# Tasks: Durable Guided Calibration Resume

**Input**: Design documents from `specs/145-guided-calibration-resume/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Required by FR-021 and the autopilot TDD protocol. Test tasks precede
implementation and must demonstrate the expected red state.

## Phase 1: Setup and Specification

**Purpose**: Establish the tracked slice and complete its reviewable intent.

- [x] T001 Create the S145 child issue of #380, add it to the repository delivery
  Project and milestone, and set its Slice, Stage, and Status fields
- [x] T002 Author and validate the S145 specification plus requirements and security
  checklists in `specs/145-guided-calibration-resume/`
- [x] T003 Complete clarification, research, data model, command contract, quickstart,
  and implementation plan in `specs/145-guided-calibration-resume/`
- [x] T004 Run `/speckit-analyze`, resolve every blocking or critical inconsistency,
  and record a clean implementation gate

---

## Phase 2: Foundational Store Contract

**Purpose**: Freeze durable vocabulary, migration, validation, and concurrency before
command integration.

- [x] T005 Run focused baseline target-store, guided calibration, event, and reference
  tests
- [x] T006 Add failing workflow domain tests for lifecycle, pause reason, protocol-set,
  target-snapshot, and record-version validation in `crates/fragcap-targets/src/workflow.rs`
- [x] T007 Add failing version 10 to 11 migration and fresh-schema tests in
  `crates/fragcap-targets/src/store.rs`
- [x] T008 Add failing store tests for create, read, cascade, conditional revision,
  in-flight allocation, attempted-case history, terminal checkpoint, corruption, and unsupported version in
  `crates/fragcap-targets/src/store.rs`
- [x] T009 Implement the closed workflow domain in
  `crates/fragcap-targets/src/workflow.rs` and export it from
  `crates/fragcap-targets/src/lib.rs`
- [x] T010 Add schema version 11 and its additive workflow table migration in
  `crates/fragcap-targets/src/schema.rs`
- [x] T011 Implement transactional create, validated read, and revision-checked update
  operations in `crates/fragcap-targets/src/store.rs`
- [x] T012 Make the focused `fragcap-targets` workflow and migration suite pass

**Checkpoint**: A durable checkpoint is valid, atomic, target-bound, and unable to
overwrite a concurrent revision.

---

## Phase 3: User Story 1 - Resume One Exact Workflow (Priority: P1)

**Goal**: Resume current remaining work in a new process without reusing authority.

**Independent Test**: A controlled incomplete workflow resumes by identifier, rereads
current facts and target state, emits a fresh plan, and continues with the next durable
attempt ordinal.

### Tests

- [x] T013 [US1] Add failing CLI parse and help tests for mutually exclusive explicit
  `--resume` selection in `crates/fragcap-cli/src/cli.rs` and
  `crates/fragcap-cli/tests/cli_calibrate.rs`
- [x] T014 [US1] Add failing fresh-start identity, second-process resume, current-fact
  suppression, completed resume, and missing-workflow tests in
  `crates/fragcap-cli/tests/cli_calibrate.rs`
- [x] T015 [US1] Add failing in-flight crash-shaped resume, target drift, revision
  drift, fresh-plan, and bundle ordinal continuity tests in
  `crates/fragcap-cli/tests/cli_calibrate.rs`

### Implementation

- [x] T016 [US1] Add explicit resume arguments and closed pause values to
  `crates/fragcap-cli/src/cli.rs`
- [x] T017 [US1] Split fresh workflow creation from resume loading and perform complete
  target-authority revalidation in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T018 [US1] Persist ready and in-flight boundaries, carry durable protocol intent
  and ordinal, and rebuild current S139 work before every effect in
  `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T019 [US1] Persist fresh terminal coverage and lifecycle outcomes without storing
  plan, response, or effect authority in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T020 [US1] Make the focused resume tests pass while retaining S142-S144 and S134
  behavior

**Checkpoint**: One exact workflow resumes safely across processes and current facts
remain the completion authority.

---

## Phase 4: User Story 2 - Pause for Operator Work (Priority: P2)

**Goal**: Record explicit operator-owned pause boundaries without performing effects.

**Independent Test**: Every supported pause atomically updates one existing workflow,
emits exact guidance, and calls no controlled Deep Capture session.

### Tests

- [x] T021 [US2] Add failing explicit login, EULA, gameplay, shutdown, and interruption
  pause tests in `crates/fragcap-cli/tests/cli_calibrate.rs`
- [x] T022 [US2] Add failing automatic warm-to-shutdown, partial-to-gameplay,
  interruption, decline, and failure mapping tests in
  `crates/fragcap-cli/tests/cli_calibrate.rs`

### Implementation

- [x] T023 [US2] Implement the no-effect explicit pause path before registration,
  proposal, or executor entry in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T024 [US2] Map terminal sequence boundaries to closed durable pause reasons and
  stable resume commands in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T025 [US2] Make all pause tests pass and verify no pause produces a plan, bundle,
  session, trust, launch, or process-control event

**Checkpoint**: Operator work is durable and visible but never confused with evidence
or automation.

---

## Phase 5: User Story 3 - Stable Truthful Guidance (Priority: P3)

**Goal**: Project the same validated workflow identity and lifecycle in human and JSON
output and refuse malformed or drifted state.

**Independent Test**: Every terminal workflow state renders stable identity, revision,
state, optional pause, ordinal, and resume command; corruption stops before effects.

### Tests

- [x] T026 [US3] Add failing event serialization and human rendering tests for the
  additive workflow fields in `crates/fragcap-cli/src/events.rs`
- [x] T027 [US3] Add failing stable terminal guidance, parseable resume-command, invalid
  stored vocabulary, and unsupported record-version tests in
  `crates/fragcap-cli/tests/cli_calibrate.rs`

### Implementation

- [x] T028 [US3] Extend `calibration.guidance` with nullable workflow identity,
  revision, state, pause, ordinal, and resume command fields in
  `crates/fragcap-cli/src/events.rs`
- [x] T029 [US3] Route every post-creation terminal boundary through one validated
  workflow guidance projection in `crates/fragcap-cli/src/commands/calibrate.rs`
- [x] T030 [US3] Make stable rendering and corruption refusal tests pass without
  changing evidence artifact schemas

**Checkpoint**: Durable progress is exact and equally visible to human and structured
consumers.

---

## Phase 6: Documentation, Audit, and Completion

**Purpose**: Reconcile shipped truth, verify scope, and prepare the authorized PR.

- [x] T031 Update the architecture record and outline in
  `docs/fragcap-specification.md` and `docs/fragcap-spec-outline.md`
- [x] T032 Record the S145 boundary and remaining #380 work in `docs/plans/README.md`
  and `AGENTS.md`
- [x] T033 Update the public CLI reference in `site/content/docs/reference/cli.mdx`
- [x] T034 Add changed and decisions fragments in
  `changelog.d/S145-guided-calibration-resume.changed.md` and
  `changelog.d/S145-guided-calibration-resume.decisions.md`
- [x] T035 Run the S145 quickstart, complete a requirement-to-test audit, and mark all
  checklists and tasks complete
- [x] T036 Run `/speckit-converge`, append and implement any remaining traceable work,
  or record a clean convergence result
- [x] T037 Run `cargo xtask ci`, encoding, punctuation, mojibake, diff, dependency, and
  worktree checks
- [ ] T038 Commit, push the authorized branch, open a PR closing the S145 child issue,
  and set Project Stage to PR review
- [ ] T039 Resolve every first-round review finding, trigger at most one `@Codex review`
  second round, resolve every second-round finding, and wait for all CI checks to pass

## Dependencies and Execution Order

- Phase 1 gates all implementation.
- Phase 2 is the durable authority foundation and blocks every user story.
- User Story 1 establishes resume, User Story 2 adds no-effect pause control, and User
  Story 3 stabilizes the shared projection.
- Phase 6 follows all stories; convergence is blocking before the repository gate.

## Scope Guardrails

- No non-Steam topology authoring, ambiguous candidate selection, advanced override,
  or parent #380 completion.
- No persisted plan, authorization response, capability, credential, key, certificate
  authority, trust state, proxy endpoint, target secret, or effect obligation.
- No hidden trust, system proxy, pinning bypass, target-process access, process control,
  or new network behavior.
- No new dependency or lockfile package and no evidence artifact schema change.
