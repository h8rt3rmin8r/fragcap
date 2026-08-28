# Tasks: Deep Capture Architecture and Trust Boundaries

**Input**: Design documents from `specs/091-deep-capture-architecture/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Required. This documentation correctness slice uses pre-implementation phrase and structure audits, source-to-prose comparisons, Mermaid node-count review, documentation checks, a production site build, and the full repository gate.

**Organization**: Tasks are grouped by user story so the two mode views, trust boundaries, and evidence/dependency interpretation can each be validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel because the task changes a different file or reads an independent validation surface.
- **[Story]**: Maps the task to its user story.
- Every task names the file or command surface it changes or validates.

## Phase 1: Setup

**Purpose**: Establish S091 requirements, decisions, contracts, validation steps, and release-note traceability.

- [X] T001 Create and validate the complete S091 artifact set in `specs/091-deep-capture-architecture/`
- [X] T002 Update `.specify/feature.json` to point at `specs/091-deep-capture-architecture`
- [X] T003 Add the issue #247 user-facing correction fragment in `changelog.d/247-deep-capture-architecture.fixed.md`

---

## Phase 2: Foundational

**Purpose**: Pin current execution, trust, artifact, and dependency authorities before changing the page.

- [X] T004 Run the passive-only, Npcap, diagram, and security-language baseline audit against `site/content/docs/architecture.mdx`
- [X] T005 Compare Capture scope, attribution, loss, and output claims with `docs/fragcap-specification.md`, `docs/glossary/capture-and-networking.md`, and current Capture implementation
- [X] T006 Compare Deep Capture eligibility, trust, execution, correlation, artifact, and cleanup claims with `crates/fragcap-cli/src/commands/deep_capture.rs` and `site/content/docs/reference/deep-capture-compatibility.mdx`
- [X] T007 Compare Npcap acquisition claims with `crates/fragcap-cli/src/doctor/action.rs`, `crates/fragcap-cli/src/doctor/fix.rs`, and the constitution licensing carveout

---

## Phase 3: User Story 1 - Understand the Two Capture Modes (Priority: P1)

**Goal**: Give readers separate, accurate, readable execution views for passive Capture and explicit Deep Capture.

**Independent Test**: A reader can classify each node as Capture-only, Deep-Capture-only, shared, external, or output evidence and identify the modes' activation, data paths, side effects, and limits.

### Tests for User Story 1

- [X] T008 [US1] Run the separate-diagram, mode-activation, shared-packet-foundation, scope-accounting, and twelve-node baseline checks against `site/content/docs/architecture.mdx`

### Implementation for User Story 1

- [X] T009 [US1] Rewrite the page overview and passive Capture execution view in `site/content/docs/architecture.mdx`
- [X] T010 [US1] Add the compatibility-gated Deep Capture execution view and mode-comparison prose in `site/content/docs/architecture.mdx`

**Checkpoint**: The two execution views are independently accurate and readable.

---

## Phase 4: User Story 2 - Evaluate Trust and Security Boundaries (Priority: P1)

**Goal**: Make consent, ownership, scope, refusal, and cleanup explicit for every active Deep Capture boundary.

**Independent Test**: A reviewer can trace `--trust-ca` authorization through the current-user Root change and cleanup, then verify all routing, trust, pinning, protocol, and denylist limits without consulting source code.

### Tests for User Story 2

- [X] T011 [US2] Run the authorization, owner, store-scope, cleanup, system-wide fallback, pinning, target-key, and denylisted-technique baseline checks against `site/content/docs/architecture.mdx`

### Implementation for User Story 2

- [X] T012 [US2] Add the Deep Capture trust-boundary and current eligibility explanation to `site/content/docs/architecture.mdx`
- [X] T013 [US2] Add routing, CA acceptance, pinning, protocol, system-wide fallback, and prohibited-technique limits to `site/content/docs/architecture.mdx`

**Checkpoint**: Every active boundary names its trigger, scope, owner, and cleanup or refusal behavior.

---

## Phase 5: User Story 3 - Interpret Outputs and Operational Dependencies (Priority: P2)

**Goal**: Distinguish artifact authority and dependency roles without absorbing issue #248's exhaustive artifact matrix.

**Independent Test**: A reader can classify each named bundle output by producer and authority, understand bounded correlation, and distinguish Npcap, mitmdump, and an unmodified analyzer.

### Tests for User Story 3

- [X] T014 [US3] Run the artifact-authority, correlation, Npcap-acquisition, dependency-role, and required-link baseline checks against `site/content/docs/architecture.mdx`

### Implementation for User Story 3

- [X] T015 [US3] Add the output-authority and structured-correlation sections to `site/content/docs/architecture.mdx`
- [X] T016 [US3] Correct Npcap acquisition language, separate runtime dependency roles, and add canonical cross-references in `site/content/docs/architecture.mdx`

**Checkpoint**: Packet truth, proxy evidence, analyzer material, audit records, and dependencies remain distinct.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Prove the architecture page across its authoritative contracts and finish the integration packet.

- [X] T017 Run every focused audit in `specs/091-deep-capture-architecture/quickstart.md`
- [X] T018 Run `cargo xtask docs check` and `cargo xtask docs build`
- [X] T019 Run `cargo fmt --all -- --check` and `cargo xtask ci`
- [X] T020 Review `git diff --check`, prohibited punctuation, UTF-8 decoding, mojibake, links, Mermaid node counts, synthetic-content privacy, and scope containment across all changed files
- [X] T021 Mark every completed task in `specs/091-deep-capture-architecture/tasks.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: Depends on setup and establishes the current factual baseline.
- **User Story 1 (Phase 3)**: Depends on all foundational comparisons.
- **User Story 2 (Phase 4)**: Depends on the Deep Capture execution view so its active boundaries can be explained in context.
- **User Story 3 (Phase 5)**: Depends on both execution views and can then classify their outputs and dependencies.
- **Polish (Phase 6)**: Depends on all three stories.

### User Story Dependencies

- **User Story 1 (P1)**: Establishes the mode split and can be validated as a complete architecture correction.
- **User Story 2 (P1)**: Builds on the Deep Capture view and makes its active security boundaries reviewable.
- **User Story 3 (P2)**: Builds on the two views but has an independent evidence and dependency classification contract.

### Parallel Opportunities

- T005, T006, and T007 inspect separate authority surfaces and can be researched in parallel.
- Baseline assertions for T008, T011, and T014 are conceptually independent, but their commands run sequentially because they inspect one shared page.
- Documentation gates in T018 run sequentially in the foreground under the autopilot protocol.

## Implementation Strategy

### MVP First

1. Pin the shipped behavior and stale baseline.
2. Replace passive-only framing with two accurate execution views.
3. Validate the diagrams and mode split before adding deeper trust and artifact prose.

### Incremental Delivery

1. Deliver separate Capture and Deep Capture architecture.
2. Add explicit consent, trust, refusal, and cleanup boundaries.
3. Add artifact authority, correlation, and dependency roles.
4. Build the production site and run the complete repository gate.
