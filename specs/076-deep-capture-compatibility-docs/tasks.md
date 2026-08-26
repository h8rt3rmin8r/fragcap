# Tasks: Deep Capture Compatibility Documentation

**Input**: Design documents from
`specs/076-deep-capture-compatibility-docs/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, and
`contracts/`

**Tests**: Required by the feature specification. Write focused tests before
implementation and observe the intended failure.

## Phase 1: Spec Kit Foundation

**Purpose**: Establish reviewed scope and contracts before code changes.

- [x] T001 Create `spec.md` and validate it with
  `checklists/requirements.md`.
- [x] T002 Resolve material ambiguities in the 2026-08-26 clarification
  session in `spec.md`.
- [x] T003 Create and pass `checklists/truth-and-privacy.md`.
- [x] T004 Create `plan.md`, `research.md`, `data-model.md`,
  `contracts/compatibility-matrix.md`, `contracts/traffic-support.md`, and
  `quickstart.md`.

---

## Phase 2: Foundational Projection

**Purpose**: Define one truthful compatibility projection shared by CLI
consumers.

- [x] T005 Add failing unit tests for empty, current, stale-marker,
  stale-source, repeated, conflicting, and deterministic unsaved facts in
  `crates/fragcap-targets/src/compatibility.rs`.
- [x] T006 Implement `CompatibilityMatrix`, `CompatibilityMatrixRow`, and
  `CompatibilityFreshness` in
  `crates/fragcap-targets/src/compatibility.rs`.
- [x] T007 Export the projection types from
  `crates/fragcap-targets/src/lib.rs` and run the focused target-crate tests.

**Checkpoint**: A pure projection produces every display row without guessing
or side effects.

---

## Phase 3: User Story 1 - Understand Traffic Coverage (Priority: P1)

**Goal**: Publish the exact Capture and Deep Capture boundary for all seven
required traffic families.

**Independent Test**: The documentation reference contains every required row
and agrees with the S075 application-record fields and limitations.

- [x] T008 [US1] Audit the S075 proxy addon and application JSONL writer against
  the seven-family contract in `contracts/traffic-support.md`; stop planning if
  the documented fields or limitations do not match shipped behavior.
- [x] T009 [US1] Add
  `site/content/docs/reference/deep-capture-compatibility.mdx` with the complete
  traffic-support table and no universal decryption claim.
- [x] T010 [US1] Add the reference page to
  `site/content/docs/meta.json` and link it from
  `site/content/docs/reference/cli.mdx`.
- [x] T011 [US1] Run the documentation linter and focused site checks.

**Checkpoint**: A user can choose the correct mode and understand protocol
limitations without local compatibility data.

---

## Phase 4: User Story 2 - Inspect Local Target Evidence (Priority: P1)

**Goal**: Render the local compatibility matrix in `targets show`.

**Independent Test**: Temporary stores with empty, current, stale, repeated,
and conflicting facts produce deterministic, additive target detail output.

- [x] T012 [US2] Add failing CLI integration tests in
  `crates/fragcap-cli/tests/cli_targets.rs` for unknown, current, stale-source,
  explicit-stale, repeated, conflicting, and launch-less facts.
- [x] T013 [US2] Read compatibility facts for the resolved target in
  `crates/fragcap-cli/src/commands/targets.rs` and fail rather than substitute
  unknown on store errors.
- [x] T014 [US2] Render the matrix contract in
  `crates/fragcap-cli/src/commands/targets.rs` without notes, executable names,
  paths, endpoints, or aggregate verdicts.
- [x] T015 [US2] Run focused CLI target tests and confirm existing target fields,
  selectors, ambiguity handling, and exits remain unchanged.

**Checkpoint**: The target detail command exposes all local evidence and remains
read-only.

---

## Phase 5: User Story 3 - Refresh And Contribute Safely (Priority: P2)

**Goal**: Explain evidence provenance, freshness, explicit refresh, and public
artifact privacy.

**Independent Test**: A reader can distinguish every source and freshness state
and can identify the explicit measurement path without being told that viewing
refreshes facts.

- [x] T016 [US3] Add the evidence and freshness legend, conflict behavior,
  read-only guarantee, and explicit refresh guidance to
  `site/content/docs/reference/deep-capture-compatibility.mdx`.
- [x] T017 [US3] Add placeholder-only contribution guidance and prohibited data
  classes to the same reference page.
- [x] T018 [US3] Scan changed public artifacts for local title names, personal
  paths, accounts, tokens, private endpoints, and host identifiers.

**Checkpoint**: Compatibility evidence can be interpreted and refreshed without
turning local research into public data.

---

## Phase 6: Specification And Release Record

**Purpose**: Keep the architecture of record and release narrative aligned.

- [x] T019 Update section 15 of `docs/fragcap-specification.md` with the runtime
  projection, source, ordering, stale, unknown, and no-inference rules.
- [x] T020 Update section 19.6 of `docs/fragcap-specification.md` with the exact
  seven-family traffic boundary and current application-record limits.
- [x] T021 Update the issue roadmap in `docs/fragcap-specification.md` to record
  #220 as resolved.
- [x] T022 Add
  `changelog.d/220-deep-capture-compatibility.added.md` with matching spec-impact
  sections.

---

## Phase 7: Verification And Local Commit

**Purpose**: Prove the implementation and prepare one auditable PR commit.

- [x] T023 Run formatting, clippy, locked workspace tests, dependency, lint,
  specification, changelog, documentation, site-build, and diff checks.
- [x] T024 Re-run the quickstart contract using a temporary placeholder-only
  target store and inspect the exact output.
- [x] T025 Review the complete diff for scope, privacy, encoding, line endings,
  and accidental `.specify/feature.json` staging; then create one conventional
  local commit.

---

## Dependencies And Execution Order

- Phase 1 is complete and gates all implementation.
- Phase 2 gates the CLI matrix in Phase 4.
- Phase 3 can proceed independently after Phase 1.
- Phase 4 depends on Phase 2.
- Phase 5 depends on the documentation page from Phase 3 and matrix behavior
  from Phase 4.
- Phase 6 follows the implemented contracts so the specification records real
  behavior.
- Phase 7 follows every user story and record update.

## Implementation Strategy

1. Establish the pure projection with failing tests first.
2. Publish the protocol reference and wire it into site navigation.
3. Add the target detail matrix with CLI integration tests first.
4. Complete refresh, privacy, master-spec, and changelog records.
5. Run all gates and commit locally.
6. Stop before push for the autopilot authorization gate.
