# Tasks: Verified First Capture and Deep Capture Journeys

**Input**: Design documents from `specs/090-getting-started-journeys/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Required. This documentation correctness slice uses a failing retired-baseline audit before implementation, current clap help and focused CLI contract suites, synthetic golden comparison, documentation checks, a production site build, and the full repository gate.

**Organization**: Tasks are grouped by user story so the Capture journey, Deep Capture continuation, and refusal guidance can each be validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel because the task changes a different file or reads an independent validation surface.
- **[Story]**: Maps the task to its user story.
- Every task names the file or command surface it changes or validates.

## Phase 1: Setup

**Purpose**: Establish S090 requirements, decisions, contracts, validation steps, and release-note traceability.

- [X] T001 Create and validate the complete S090 artifact set in `specs/090-getting-started-journeys/`
- [X] T002 Update `.specify/feature.json` to point at `specs/090-getting-started-journeys`
- [X] T003 Add the issue #245 user-facing correction fragment in `changelog.d/245-getting-started.fixed.md`

---

## Phase 2: Foundational

**Purpose**: Pin current command, specimen, privacy, and scope authorities before changing the guide.

- [X] T004 Run the retired-baseline audit from `specs/090-getting-started-journeys/quickstart.md` and record the expected stale matches in `site/content/docs/getting-started.mdx`
- [X] T005 Compare the guide with current `doctor`, `targets`, `targets show`, `capture`, and `deep-capture` help plus `crates/fragcap-cli/tests/goldens/doctor-ready.txt`
- [X] T006 Compare the guide's Deep Capture claims with `site/content/docs/reference/deep-capture-compatibility.mdx` and `specs/090-getting-started-journeys/contracts/journey-contract.md`, and record why the stale `site/content/docs/reference/output-formats.mdx` cannot serve as the bundle authority before issue #248

---

## Phase 3: User Story 1 - Complete a First Capture (Priority: P1)

**Goal**: Give a new operator one current path from installation verification through a bounded Capture and an opened `.fcapng`.

**Independent Test**: Every Capture command is accepted by current grammar, the doctor and target specimens match current synthetic contracts, and the prose distinguishes packet observations, process attribution, payload scope, and encrypted traffic.

### Tests for User Story 1

- [X] T007 [US1] Run the Capture command, doctor specimen, target-column, labelled-next-command, optional-database-path, and payload-scope baseline checks against `site/content/docs/getting-started.mdx`

### Implementation for User Story 1

- [X] T008 [US1] Rewrite prerequisites, installation verification, doctor output, and target selection in `site/content/docs/getting-started.mdx`
- [X] T009 [US1] Rewrite the bounded first Capture and analyzer result sections in `site/content/docs/getting-started.mdx`

**Checkpoint**: The first Capture journey is independently complete and verifiable.

---

## Phase 4: User Story 2 - Complete a Known-Compatible Deep Capture (Priority: P1)

**Goal**: Continue from packet Capture into a consent-forward Deep Capture session for an already proven stored target.

**Independent Test**: The guide requires current launch-specific evidence, cold Steam managed launch, mitmdump, explicit current-user trust authorization through `--trust-ca`, a bounded run, and review of manifest and cleanup evidence.

### Tests for User Story 2

- [X] T010 [US2] Run the eligibility, managed-launch, proxy-backend, trust-authorization, traffic-limit, bundle-state, sensitivity, and cleanup baseline checks against `site/content/docs/getting-started.mdx`

### Implementation for User Story 2

- [X] T011 [US2] Add the read-only compatibility checkpoint and current eligibility conditions to `site/content/docs/getting-started.mdx`
- [X] T012 [US2] Add the bounded Deep Capture invocation, traffic expectations, self-contained bundle review, sensitive-artifact handling, and post-session cleanup verification to `site/content/docs/getting-started.mdx`

**Checkpoint**: The known-compatible Deep Capture continuation is independently complete and verifiable.

---

## Phase 5: User Story 3 - Recognize Unsupported or Unknown Paths (Priority: P2)

**Goal**: Stop operators safely when evidence, launch ownership, proxy support, trust acceptance, or traffic support does not satisfy the shipped path.

**Independent Test**: Unknown and stale facts, unsupported launch cases, unsupported traffic, pinning, and incomplete cleanup remain distinct and lead to an honest stop or diagnostic action without new side effects.

### Tests for User Story 3

- [X] T013 [US3] Run the unknown-evidence, automatic-calibration, system-wide-proxy, pinning-bypass, target-key-extraction, warm-Steam, direct-executable, and unsupported-traffic phrase audit against `site/content/docs/getting-started.mdx`

### Implementation for User Story 3

- [X] T014 [US3] Add explicit unknown, stale, conflicting, wrong-launch, unsupported-traffic, and incomplete-cleanup stop guidance to `site/content/docs/getting-started.mdx`

**Checkpoint**: Refusal and recovery guidance is independently complete and verifiable.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Prove the two journeys across their authoritative contracts and finish the integration packet.

- [X] T015 Run every focused audit and help comparison in `specs/090-getting-started-journeys/quickstart.md`
- [X] T016 Run `cargo test -p fragcap-cli --test cli_args`, `cli_help`, `cli_targets`, `cli_doctor`, and `cli_deep_capture`
- [X] T017 Run `cargo xtask docs check` and `cargo xtask docs build`
- [X] T018 Run `cargo fmt --all -- --check` and `cargo xtask ci`
- [X] T019 Review `git diff --check`, prohibited punctuation, UTF-8 decoding, mojibake, links, synthetic-content privacy, and scope containment across all changed files
- [X] T020 Mark every completed task in `specs/090-getting-started-journeys/tasks.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: Depends on setup and establishes the current factual baseline.
- **User Story 1 (Phase 3)**: Depends on foundational command and specimen comparison.
- **User Story 2 (Phase 4)**: Depends on the Deep Capture contract comparison and uses the completed Capture journey as context.
- **User Story 3 (Phase 5)**: Depends on the same contract comparison and can be validated independently after foundational work.
- **Polish (Phase 6)**: Depends on all three stories.

### User Story Dependencies

- **User Story 1 (P1)**: Establishes the baseline product journey and can ship independently as an accurate Capture guide.
- **User Story 2 (P1)**: Uses User Story 1 as a narrative starting point but has its own eligibility, invocation, and outcome contract.
- **User Story 3 (P2)**: Can be authored alongside User Story 2 after the foundational comparison because it changes separate conceptual sections of the same page; edits remain sequential because they share one file.

### Parallel Opportunities

- T005 and T006 can be researched in parallel because they inspect separate command and reference authorities.
- Baseline assertions for T007, T010, and T013 are conceptually independent, but their commands run sequentially because they inspect one shared file.
- Focused test binaries in T016 are independent, but verification runs sequentially in the foreground under the autopilot protocol.

## Implementation Strategy

### MVP First

1. Establish the current output and command baseline.
2. Replace the stale first Capture path and validate it independently.
3. Preserve the working Capture path while adding the Deep Capture continuation.

### Incremental Delivery

1. Deliver a truthful first Capture.
2. Add the fact-backed, known-compatible Deep Capture session.
3. Add explicit refusal and recovery guidance.
4. Build the production site and run the complete repository gate.
