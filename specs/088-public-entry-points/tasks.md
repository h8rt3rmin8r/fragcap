# Tasks: Public Entry Point Reconciliation

**Input**: Design documents from `specs/088-public-entry-points/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Required. This documentation correctness slice uses baseline stale-claim audits, issue-form parsing, current CLI help comparison, documentation and specification gates, a production site build, and the full repository gate.

**Organization**: Tasks are grouped by user story so each public audience can be validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel because the task changes a different file and has no dependency on another incomplete task.
- **[Story]**: Maps the task to its user story.
- Every task names the file or external metadata surface it changes or validates.

## Phase 1: Setup

**Purpose**: Establish S088 metadata, requirements, decisions, and release-note traceability.

- [X] T001 Create S088 specification artifacts in `specs/088-public-entry-points/`
- [X] T002 Update `.specify/feature.json` to point at `specs/088-public-entry-points`
- [X] T003 Add the issue #244 user-facing correction fragment in `changelog.d/244-public-entry-points.fixed.md`

---

## Phase 2: Foundational

**Purpose**: Pin the current product claims and measure the stale baseline before editing public surfaces.

- [X] T004 Run the retired-claim audit from `specs/088-public-entry-points/quickstart.md` and record the expected failing surfaces
- [X] T005 Compare current `fragcap`, `capture`, and `doctor` help with examples in `README.md` and `.github/ISSUE_TEMPLATE/`
- [X] T006 Reconcile present-tense v0.7.0 status, mode, release, and roadmap statements in `docs/fragcap-specification.md`

---

## Phase 3: User Story 1 - Understand the Shipped Product (Priority: P1)

**Goal**: A repository or documentation visitor receives one accurate, bounded definition of shipped Capture and Deep Capture.

**Independent Test**: Read `README.md`, `site/content/docs/index.mdx`, and the GitHub repository description; each names both shipped modes, distinguishes their posture, and avoids universal claims.

### Tests for User Story 1

- [X] T007 [US1] Run the focused planned-versus-shipped and release-status phrase audit against `README.md`, `site/content/docs/index.mdx`, and `docs/fragcap-specification.md`

### Implementation for User Story 1

- [X] T008 [P] [US1] Reconcile product summary, status, capabilities, security boundary, Npcap policy, command overview, and repository map in `README.md`
- [X] T009 [P] [US1] Reconcile product introduction, prerequisites, start links, and security posture in `site/content/docs/index.mdx`
- [X] T010 [US1] Set the exact mode-aware description on GitHub repository `h8rt3rmin8r/fragcap`

---

## Phase 4: User Story 2 - Contribute Against the Current Repository (Priority: P1)

**Goal**: Contributors see the current workspace, security posture, Npcap policy, and development workflow.

**Independent Test**: Read both contributor entry points and compare their shared claims with the constitution while treating `CONTRIBUTING.md` as the canonical workflow.

### Tests for User Story 2

- [X] T011 [US2] Run the pre-implementation, passive-only, and stale-workflow phrase audit against `CONTRIBUTING.md` and `site/content/docs/contributing.mdx`

### Implementation for User Story 2

- [X] T012 [P] [US2] Reconcile product boundary, current state, Npcap policy, checks, and workflow in `CONTRIBUTING.md`
- [X] T013 [P] [US2] Reconcile the concise contributor summary and canonical-guide link in `site/content/docs/contributing.mdx`

---

## Phase 5: User Story 3 - File an Actionable Current Issue (Priority: P2)

**Goal**: Bug and feature reporters receive current commands, environment questions, planning pointers, and scope confirmations.

**Independent Test**: Parse both issue forms and compare their examples and constraints with v0.7.0 help, the constitution, and the Npcap policy.

### Tests for User Story 3

- [X] T014 [US3] Parse `.github/ISSUE_TEMPLATE/bug_report.yml` and `.github/ISSUE_TEMPLATE/feature_request.yml` and audit their retired command, release, roadmap, and Npcap values

### Implementation for User Story 3

- [X] T015 [P] [US3] Update reproduction, version, Npcap, and confirmation fields in `.github/ISSUE_TEMPLATE/bug_report.yml`
- [X] T016 [P] [US3] Update product boundary, planning guidance, and scope confirmations in `.github/ISSUE_TEMPLATE/feature_request.yml`

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Prove the correction across every surface and finish the integration packet.

- [X] T017 Run all focused S088 checks from `specs/088-public-entry-points/quickstart.md`
- [X] T018 Run `cargo xtask docs check`, `cargo xtask docs build`, and `cargo xtask spec`
- [X] T019 Run `cargo fmt --all -- --check` and `cargo xtask ci`
- [X] T020 Review `git diff --check`, changed-file punctuation, UTF-8 decoding, mojibake, local links, repository metadata, and scope containment
- [X] T021 Mark all completed tasks in `specs/088-public-entry-points/tasks.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: Depends on setup and establishes the factual baseline.
- **User Story 1 (Phase 3)**: Depends on the master-spec current-status correction.
- **User Story 2 (Phase 4)**: Depends on the shared mode and Npcap contracts, but is independently testable after foundational work.
- **User Story 3 (Phase 5)**: Depends on the shared security and command contracts, but is independently testable after foundational work.
- **Polish (Phase 6)**: Depends on all user stories and external repository metadata.

### Parallel Opportunities

- T008 and T009 can be authored in parallel after T006 because they change separate product-entry files.
- T012 and T013 can be authored in parallel because the canonical-versus-summary relationship is already decided.
- T015 and T016 can be authored in parallel because the issue forms are independent files.
- Verification commands run sequentially in the foreground.

## Implementation Strategy

### MVP First

1. Establish the current product and release baseline.
2. Correct the master specification and repository/documentation front doors.
3. Validate User Story 1 independently before contributor and reporter surfaces.

### Incremental Delivery

1. Deliver accurate product and release positioning.
2. Deliver current contributor guidance.
3. Deliver current issue intake.
4. Build the production site and run the complete repository gate.
