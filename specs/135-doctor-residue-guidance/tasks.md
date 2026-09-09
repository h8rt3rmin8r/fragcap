# Tasks: Doctor Residue Guidance

**Input**: Design documents from `/specs/135-doctor-residue-guidance/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/doctor-residue-output.md, quickstart.md

**Tests**: Human guidance, width, Unicode, structured-output, recovery-parity, and read-only regressions are mandatory and precede implementation tasks.

**Organization**: Tasks are grouped by independently testable user story after shared display and check-model foundations.

## Phase 1: Setup

**Purpose**: Establish change records without changing runtime behavior.

- [x] T001 [P] Add the user-visible S135 outcome fragment in `changelog.d/373-doctor-residue-guidance.fixed.md`
- [x] T002 [P] Add the S135 decision fragment covering presentation separation, structured context, width selection, and unchanged recovery authority in `changelog.d/s135-doctor-residue-guidance.decisions.md`

---

## Phase 2: Shared Presentation Foundation

**Purpose**: Create one display-width authority and additive check model before residue wording or layout changes.

**Critical**: No user story proceeds until existing target-table rendering is protected and human presentation cannot select a cleanup action.

- [x] T003 Add failing display-cell tests for ASCII, combining marks, selectors, CJK, fullwidth forms, emoji, and padding in `crates/fragcap-cli/src/display.rs`
- [x] T004 Extract `display_width`, `display_cell_width`, and `pad_display` into `crates/fragcap-cli/src/display.rs`, register the module in `crates/fragcap-cli/src/lib.rs`, and reuse it from `crates/fragcap-cli/src/commands/targets.rs`
- [x] T005 Add failing check-model tests proving machine fields remain unchanged, optional human presentation affects only human rendering, and non-native checks omit structured context in `crates/fragcap-cli/src/doctor/mod.rs`
- [x] T006 Add presentation-only and typed `NativeResourceContext` values to the existing Doctor `Check` in `crates/fragcap-cli/src/doctor/mod.rs`

**Checkpoint**: Display width has one owner, target tables retain their behavior, and a check can carry distinct human and machine contracts without changing action authority.

---

## Phase 3: User Story 1 - Understand a Blocking Residue Finding (Priority: P1)

**Goal**: Explain each native resource condition, ownership proof, readiness consequence, identity, and safe next action in plain language.

**Independent Test**: An abandoned session-owner report alone communicates the earlier incomplete session, unretired record, absent active-owner proof, blocking consequence, and exact confirmed Doctor cleanup path.

### Tests for User Story 1

- [x] T007 [US1] Add failing diagnosis tests for healthy, active, stale, cleanup-failed, unknown, unsupported, recoverable, and non-recoverable findings in `crates/fragcap-cli/src/doctor/checks.rs`
- [x] T008 [US1] Add the failing abandoned session-owner regression and assert all required meanings plus absence of internal key-value prose and `recovery authority` in `crates/fragcap-cli/tests/cli_doctor.rs`
- [x] T009 [US1] Add cleanup-action parity tests proving the before-and-after action set depends only on existing recoverability in `crates/fragcap-cli/src/doctor/checks.rs`

### Implementation for User Story 1

- [x] T010 [US1] Implement exact state and health to plain-language diagnosis mapping in `crates/fragcap-cli/src/doctor/checks.rs`
- [x] T011 [US1] Attach stable human label, secondary session/resource identity, and exact native context to every residue check in `crates/fragcap-cli/src/doctor/checks.rs`
- [x] T012 [US1] Replace internal human recovery terminology with review-and-confirm `fragcap doctor --fix` guidance while preserving machine remediation and `Action::Cleanup` selection in `crates/fragcap-cli/src/doctor/checks.rs`

**Checkpoint**: Every controlled residue class is understandable from human output and no wording broadens recovery eligibility.

---

## Phase 4: User Story 2 - Read a Stable Human Layout (Priority: P2)

**Goal**: Keep Doctor scannable at 80 columns and readable at the supported 40-column boundary without truncation or Unicode drift.

**Independent Test**: The same long and non-ASCII residue report renders in aligned and compact layouts, ordinary lines fit the selected display width, and plain versus color output has identical visible layout.

### Tests for User Story 2

- [x] T013 [US2] Add failing 80-column aligned and 40-column compact golden assertions for short and long identities in `crates/fragcap-cli/src/doctor/mod.rs`
- [x] T014 [US2] Add failing Unicode display-width, indivisible-token, continuation-indent, and no-truncation cases in `crates/fragcap-cli/src/doctor/mod.rs`
- [x] T015 [US2] Add failing plain versus color visible-layout parity and ordinary-line width assertions in `crates/fragcap-cli/src/doctor/mod.rs`
- [x] T016 [US2] Add failing command tests for terminal width clamping and deterministic non-terminal 80-column output in `crates/fragcap-cli/src/commands/doctor.rs`

### Implementation for User Story 2

- [x] T017 [US2] Replace fixed byte-length hanging wrap with display-cell-aware wrapping and layout-derived indentation in `crates/fragcap-cli/src/doctor/mod.rs`
- [x] T018 [US2] Implement aligned and compact report layouts through an injected-width renderer while preserving section and verdict ordering in `crates/fragcap-cli/src/doctor/mod.rs`
- [x] T019 [US2] Select stdout width once per human invocation, clamp it to 40 through 80, and pass it through read-only and fix reports in `crates/fragcap-cli/src/commands/doctor.rs` and `crates/fragcap-cli/src/doctor/fix.rs`

**Checkpoint**: Dynamic identities never move the status column, narrow reports retain every value, and ANSI color never affects visible alignment.

---

## Phase 5: User Story 3 - Preserve Exact Machine and Recovery Truth (Priority: P3)

**Goal**: Give automation exact structured residue facts while retaining existing common fields, verdicts, actions, and read-only behavior.

**Independent Test**: Every controlled native finding parses as one JSON record carrying the seven required facts, and action plus effect traces match the pre-S135 contract.

### Tests for User Story 3

- [x] T020 [US3] Add failing JSON assertions for all seven native-resource fields across every health class in `crates/fragcap-cli/tests/cli_doctor.rs`
- [x] T021 [US3] Add failing structured escaping, one-record-per-line, common-field compatibility, and non-native omission tests in `crates/fragcap-cli/src/doctor/mod.rs` and `crates/fragcap-cli/tests/cli_doctor.rs`
- [x] T022 [US3] Retain controlled read-only no-effect and exact cleanup confirmation, active-resource, partial-failure, and exit-semantic assertions in `crates/fragcap-cli/tests/cli_doctor.rs`

### Implementation for User Story 3

- [x] T023 [US3] Serialize the additive `native_resource` object on native check records without exposing bundle paths or secrets in `crates/fragcap-cli/src/doctor/mod.rs`
- [x] T024 [US3] Preserve existing common check and verdict serialization, machine names, machine details, remediation, and recovery execution paths in `crates/fragcap-cli/src/doctor/mod.rs` and `crates/fragcap-cli/src/doctor/fix.rs`

**Checkpoint**: Automation no longer scrapes human residue prose, while recovery and read-only contracts remain unchanged.

---

## Phase 6: Documentation, Analysis, Convergence, and Verification

**Purpose**: Synchronize shipped architecture, user guidance, evidence, and repository gates.

- [x] T025 [P] Update Doctor residue and output-width contracts in `docs/fragcap-specification.md`, `docs/fragcap-spec-outline.md`, and `docs/plans/README.md`
- [x] T026 [P] Update human and structured Doctor usage in `site/content/docs/reference/cli.mdx` without claiming Deep Capture completion
- [x] T027 Re-run `/speckit-analyze`, resolve every finding, and record complete requirement-to-task coverage against `specs/135-doctor-residue-guidance/spec.md`, `specs/135-doctor-residue-guidance/plan.md`, and `specs/135-doctor-residue-guidance/tasks.md`
- [x] T028 Run focused display, Doctor, residue, JSON, cleanup, and read-only tests from `specs/135-doctor-residue-guidance/quickstart.md`
- [x] T029 Run `/speckit-converge`, append and implement any remaining tasks, then rerun until every S135 requirement is satisfied
- [x] T030 Run `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all --locked`, and `cargo xtask ci` from the repository root
- [x] T031 Verify the lockfile package set is unchanged, changed text is UTF-8 without BOM and LF-only, no mojibake or forbidden dash exists, every checklist is complete, and the final diff matches issue #373 scope

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: Starts immediately.
- **Foundation (Phase 2)**: Depends on Phase 1 and blocks all user stories.
- **User Story 1 (Phase 3)**: Depends on the presentation model from Phase 2.
- **User Story 2 (Phase 4)**: Depends on the stable human label and shared display helpers.
- **User Story 3 (Phase 5)**: Depends on typed native context and keeps behavior from User Story 1 intact.
- **Documentation and Verification (Phase 6)**: Depends on all user stories.

### User Story Dependencies

- **User Story 1**: Delivers the minimum viable comprehension fix.
- **User Story 2**: Makes the User Story 1 output stable across supported widths.
- **User Story 3**: Adds exact automation fields without changing either human story.

### Within Each User Story

- Add and observe failing tests before implementation.
- Derive diagnoses from typed inventory facts.
- Keep presentation separate from action selection.
- Complete focused tests before advancing.

### Parallel Opportunities

- T001 and T002 affect separate changelog files.
- T025 and T026 affect separate documentation sets after behavior settles.
- Test cases within a phase may be authored together before implementation.

## Implementation Strategy

### Minimum Viable Guidance

1. Complete change-record setup and the presentation foundation.
2. Implement the abandoned-owner and residue health diagnoses.
3. Prove action parity before changing layout.

### Incremental Delivery

1. Make residue guidance understandable.
2. Make human layout width-aware and Unicode-correct.
3. Add exact structured machine context.
4. Synchronize documentation and run convergence plus full gates.

## Notes

- Mark a task `[x]` only after its implementation or verification evidence is observed.
- No test may perform a real trust mutation, launch a game, require Npcap or elevation, or invoke external cleanup.
- If implementation reveals an architecture deviation, record it in the S135 decision fragment and master specification before completion.
