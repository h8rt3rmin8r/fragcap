# Tasks: Deep Capture Bundle and Artifact Reference

**Input**: Design documents from `specs/092-deep-capture-artifacts/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/output-reference-contract.md`

**Tests**: Focused source-contract audits are required before and after documentation implementation, followed by documentation and complete repository gates.

**Organization**: Tasks are grouped by user story. Work within `output-formats.mdx` is sequential because each story edits the same authority page.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel because it affects a different file and has no dependency on an incomplete task.
- **[Story]**: Maps a task to its user story.

## Phase 1: Setup and Specification

**Purpose**: Establish the clean branch, active feature, behavioral authority, and complete S092 design artifacts.

- [X] T001 Confirm `codex/092-deep-capture-artifacts` starts clean from current `main` and issue #248 remains open in the `Post-v0.7.0 documentation` milestone
- [X] T002 Point `.specify/feature.json` at `specs/092-deep-capture-artifacts` without staging the local pointer
- [X] T003 Read `AI_CONTEXT.md`, the constitution, master specification section 13.7, current Deep Capture writer and doctor cleanup source, controlled bundle tests, and current public output references
- [X] T004 Create and validate `specs/092-deep-capture-artifacts/spec.md` and `specs/092-deep-capture-artifacts/checklists/requirements.md`
- [X] T005 Run clarification coverage analysis against `specs/092-deep-capture-artifacts/spec.md` and confirm that shipped source and issue #248 resolve all material S092 questions without adding an unnecessary clarification section
- [X] T006 Create and complete the artifact-contract requirements checklist in `specs/092-deep-capture-artifacts/checklists/artifact-contract.md`
- [X] T007 Create the S092 plan and design set in `specs/092-deep-capture-artifacts/plan.md`, `research.md`, `data-model.md`, `contracts/output-reference-contract.md`, and `quickstart.md`

**Checkpoint**: The artifact set defines exact shipped paths, authorities, sensitivity labels, states, omissions, correlation limits, lifecycle, and validation.

---

## Phase 2: Foundational Contract Audits

**Purpose**: Prove the existing page is stale and bind the implementation to current v0.7.0 source before rewriting prose.

- [X] T008 Record the expected failing baseline phrase audit for the retired two-equivalent-formats claim in `site/content/docs/reference/output-formats.mdx`
- [X] T009 Record expected failing baseline coverage for Deep Capture artifact paths, manifest states, omission tokens, correlation anchors, and cross-page links in `site/content/docs/reference/output-formats.mdx`, `deep-capture-compatibility.mdx`, and `cli.mdx`
- [X] T010 Reconcile the public contract inventory against `crates/fragcap-cli/src/commands/deep_capture.rs`, `crates/fragcap-cli/src/doctor/fix.rs`, `crates/fragcap-cli/src/har.rs`, and `crates/fragcap-cli/tests/cli_deep_capture.rs`

**Checkpoint**: The stale claim fails and the expected replacement vocabulary is traceable to shipped code and tests.

---

## Phase 3: User Story 1 - Identify the Authority of Every Output (Priority: P1)

**Goal**: Make the output reference the single truthful guide to ordinary Capture outputs and every Deep Capture bundle authority.

**Independent Test**: A reader can select the authoritative artifact for packet, application, HTTP projection, proxy, process, compatibility, and cleanup questions from one page.

- [X] T011 [US1] Rewrite the introduction and ordinary Capture sections in `site/content/docs/reference/output-formats.mdx` to distinguish output families while preserving pcapng and packet JSON Lines behavior
- [X] T012 [US1] Add the Deep Capture bundle, manifest read-first rule, three finalized states, and early cleanup exception to `site/content/docs/reference/output-formats.mdx`
- [X] T013 [US1] Add the complete nine-role artifact matrix with exact paths, authorities, emitted sensitivity labels, required status, production conditions, and lifetimes to `site/content/docs/reference/output-formats.mdx`
- [X] T014 [US1] Add a shortened synthetic `manifest.json` example that demonstrates artifact declarations, omissions, correlation, and cleanup without local or secret material to `site/content/docs/reference/output-formats.mdx`
- [X] T015 [US1] Run the artifact-role and state coverage audit against `site/content/docs/reference/output-formats.mdx`

**Checkpoint**: User Story 1 is independently readable and all output authorities are explicit.

---

## Phase 4: User Story 2 - Handle Sensitive Artifacts Deliberately (Priority: P1)

**Goal**: Explain actual content, sensitivity, live use, retention, sharing, and cleanup behavior without changing or overstating emitted labels.

**Independent Test**: A reader can decide how to handle each artifact and can explain the proxy-owned key-log lifecycle and cleanup boundary accurately.

- [X] T016 [US2] Document application JSON Lines record families, metadata-only and unsupported observations, and HAR projection limits in `site/content/docs/reference/output-formats.mdx`
- [X] T017 [US2] Document final-path creation, live analyzer use, nonempty-only retention, secret-adjacent classification, certificate-pinning limit, and no target extraction for `tls-keylog.log` in `site/content/docs/reference/output-formats.mdx`
- [X] T018 [US2] Add private-by-default retention, original-preservation, reviewed-copy sharing, session cleanup, confirmation-gated doctor residue-cleanup guidance, and links to existing artifact glossary terms in `site/content/docs/reference/output-formats.mdx`
- [X] T019 [US2] Audit `ordinary`, `sensitive`, and `secret-adjacent` wording plus every prohibited universal, automatic-deletion, and target-extraction claim in `site/content/docs/reference/output-formats.mdx`

**Checkpoint**: User Story 2 gives exact handling guidance without relabeling emitted artifacts or promising cleanup the runtime does not perform.

---

## Phase 5: User Story 3 - Interpret State, Omissions, and Correlation (Priority: P2)

**Goal**: Make missing artifacts, per-observation reasons, cleanup results, and cross-file joins interpretable without converting absent evidence into negative claims.

**Independent Test**: A reader can interpret every current omission token and follow only correlation anchors that the current artifacts actually carry.

- [X] T020 [US3] Add the exact finalized-manifest omission table and distinguish it from application observation reasons and cleanup resource statuses in `site/content/docs/reference/output-formats.mdx`
- [X] T021 [US3] Add the correlation-anchor matrix, join guidance, and missing-anchor semantics to `site/content/docs/reference/output-formats.mdx`
- [X] T022 [P] [US3] Link `site/content/docs/reference/deep-capture-compatibility.mdx` to the output contract without copying its artifact matrix
- [X] T023 [P] [US3] Link `site/content/docs/reference/cli.mdx` to the output contract beside the Deep Capture command description
- [X] T024 [US3] Run exact omission-token, correlation-field, and inbound-link audits across the three edited public references

**Checkpoint**: User Story 3 distinguishes absence from unavailability and makes the output contract reachable from both related references.

---

## Phase 6: Polish and Cross-Cutting Validation

**Purpose**: Close traceability, documentation, source hygiene, and repository gates before the pre-push halt.

- [X] T025 Add `changelog.d/248-deep-capture-artifacts.fixed.md` with a valid specification-impact marker and a concise user-visible correction
- [X] T026 Run `cargo xtask docs check` and resolve every glossary, link, structure, and generated-index finding
- [X] T027 Run `cargo xtask docs build` and resolve every production static-export failure
- [X] T028 Run `cargo fmt --all -- --check`, `cargo xtask lint`, `git diff --check`, and focused mojibake and prohibited-punctuation audits
- [X] T029 Run `cargo xtask ci` in the foreground and resolve every failure
- [X] T030 Re-run the complete contract audit from `specs/092-deep-capture-artifacts/contracts/output-reference-contract.md` and validate every checklist remains complete
- [X] T031 Review the final diff for issue #248 scope, synthetic-only content, exact runtime agreement, and exclusion of `.specify/feature.json`
- [X] T032 Mark all completed tasks in `specs/092-deep-capture-artifacts/tasks.md`, stage only S092 files, commit locally with the repository co-author trailer, and halt before `git push`

---

## Dependencies and Execution Order

### Phase Dependencies

- **Setup and Specification (Phase 1)**: Starts from the clean branch.
- **Foundational Contract Audits (Phase 2)**: Depends on the completed design inventory and blocks public prose changes.
- **User Story 1 (Phase 3)**: Depends on Phase 2 and establishes the page structure used by later stories.
- **User Story 2 (Phase 4)**: Depends on User Story 1 because it expands the same artifact sections.
- **User Story 3 (Phase 5)**: Depends on User Story 1; its two inbound-link edits can run in parallel after the output contract exists.
- **Polish (Phase 6)**: Depends on all three stories.

### User Story Dependencies

- **User Story 1 (P1)**: Establishes the artifact authorities and is the minimum viable correction.
- **User Story 2 (P1)**: Uses User Story 1's inventory to add security and lifecycle handling.
- **User Story 3 (P2)**: Uses User Story 1's manifest vocabulary and can proceed independently of User Story 2 after the shared page sections are stable.

### Parallel Opportunities

- T022 and T023 edit different inbound reference pages and can run in parallel.
- Documentation checks are sequential because each validates the same completed tree.
- No `output-formats.mdx` tasks are marked parallel because concurrent edits to the authority page would conflict.

## Parallel Example: User Story 3

```text
Task: "Link deep-capture-compatibility.mdx to the output contract"
Task: "Link cli.mdx to the output contract"
```

## Implementation Strategy

### Authority First

1. Establish exact emitted tokens and the stale baseline.
2. Correct the two output families and nine artifact authorities.
3. Add sensitivity and lifecycle guidance.
4. Add omission and correlation interpretation.
5. Link inbound pages and validate the full contract.

### Review Boundary

The slice ends with one local commit on `codex/092-deep-capture-artifacts`. Do not push until the operator explicitly authorizes it after reviewing the autopilot breakdown.

## Notes

- `[P]` tasks affect different files and have no incomplete shared-file dependency.
- Exact machine tokens remain inline code; explanatory prose uses existing glossary terms.
- `.specify/feature.json` is local state and must never be staged.
