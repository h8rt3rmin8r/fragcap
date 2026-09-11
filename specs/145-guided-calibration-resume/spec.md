# Feature Specification: Durable Guided Calibration Resume

**Feature Branch**: `codex/s145-guided-calibration-resume`

**Created**: 2026-09-11

**Status**: Draft

**Input**: Work slice S145 gives the bounded guided calibration sequence a durable,
explicitly selected workflow checkpoint so an operator can pause, leave the process,
and resume later without reusing authorization or stale execution authority.

## User Scenarios & Testing

### User Story 1 - Resume One Exact Workflow (Priority: P1)

An operator can leave an incomplete guided calibration and later resume its exact
target and protocol intent by workflow identifier. Current target, process, fact,
proposal, and recovery authority are rebuilt before any new attempt.

**Why this priority**: S144 can continue only while one process remains alive. Real
game setup commonly spans restarts, login, agreements, updates, and gameplay that
cannot be completed in one command lifetime.

**Independent Test**: Start a controlled calibration, stop after an incomplete
attempt, open a new process with the emitted resume command, and verify that only
remaining current work is proposed under a new plan.

**Acceptance Scenarios**:

1. **Given** an incomplete workflow, **when** its exact identifier is resumed from
   the same local store, **then** current authority is rebuilt and completed facts
   suppress already satisfied work.
2. **Given** a workflow whose previous attempt was interrupted in flight, **when**
   it is resumed, **then** the interruption is reported, recovery readiness is
   checked by the existing lifecycle authority, and no previous plan or response is
   reused.
3. **Given** a completed workflow, **when** it is resumed, **then** no session starts
   and current completion guidance is emitted.

---

### User Story 2 - Pause for Operator Work (Priority: P2)

An operator can record that a workflow is paused for login, agreement acceptance,
gameplay exercise, normal shutdown, or interruption. The pause is visible in human
and structured output and the resume command remains stable.

**Why this priority**: These actions belong to the operator and target application.
Making them explicit prevents fragcap from guessing, controlling the target, or
misreporting absence of evidence as incompatibility.

**Independent Test**: Mark each supported pause on a controlled workflow and verify
that no effect starts, the durable reason is exact, and a later resume revalidates
instead of treating the pause as evidence.

**Acceptance Scenarios**:

1. **Given** an active or paused workflow, **when** the operator selects one pause
   reason, **then** the checkpoint changes atomically and no capture, proxy, trust,
   launch, or process-control effect occurs.
2. **Given** a warm declared target, **when** calibration cannot continue, **then**
   the workflow records a shutdown pause and retains the normal close-and-retry
   command boundary.
3. **Given** a session that completes without the required positive fact, **when**
   guidance is finalized, **then** the workflow records a gameplay pause rather than
   claiming failure or compatibility.

---

### User Story 3 - Refuse Drift and Corruption Truthfully (Priority: P3)

An operator receives an exact refusal if the selected workflow is missing, malformed,
belongs to another target authority, or cannot be updated without overwriting a
concurrent revision.

**Why this priority**: A durable hint becomes dangerous if it can silently redirect a
target, widen requested work, or overwrite another process's progress.

**Independent Test**: Exercise missing identifiers, target edits, invalid stored
vocabulary, newer checkpoint versions, and concurrent revision changes. Verify that
all stop before a new plan or effect.

**Acceptance Scenarios**:

1. **Given** target authority that differs from the checkpoint snapshot, **when** the
   workflow is resumed, **then** it is refused and a fresh workflow is required.
2. **Given** a checkpoint from a newer unsupported record version, **when** it is
   read, **then** the command refuses it without modifying the row.
3. **Given** two writers using the same revision, **when** both try to advance it,
   **then** exactly one succeeds and the other reports concurrent drift.

### Edge Cases

- A resume identifier is local to the explicitly selected store; the command never
  searches other stores.
- Resume cannot be combined with a target selector or new protocol candidates.
- Operational bounds such as duration may be selected again because the next
  authorization plan binds their current values.
- A process crash after the in-flight checkpoint but before a terminal update leaves
  an interruption marker, not a completed attempt.
- A crash during an atomic checkpoint transaction leaves either the old complete row
  or the new complete row.
- Deleted targets cascade-delete their workflows, so a stale identifier is a clean
  missing-workflow refusal.
- Observed protocol candidates remain distinct from requested candidates and from
  current compatibility facts.
- Explicit bundle roots use the durable attempt ordinal so resumed work cannot reuse
  an earlier attempt destination.

## Requirements

### Functional Requirements

- **FR-001**: The local target store MUST persist versioned guided-calibration
  workflow checkpoints through an additive schema migration.
- **FR-002**: Each checkpoint MUST have a positive store-local identifier, target-row
  relationship, immutable target-authority snapshot, requested and observed protocol
  sets, exact attempted-case history, attempt ordinal, lifecycle state, pause reason,
  revision, and timestamps.
- **FR-003**: The store MUST constrain lifecycle and pause vocabularies and MUST
  validate every protocol token before returning a checkpoint.
- **FR-004**: A fresh guided sequence MUST create its checkpoint before its first
  effectful attempt and MUST emit the workflow identifier in stable guidance.
- **FR-005**: `fragcap calibrate --resume <WORKFLOW_ID>` MUST explicitly select one
  workflow from the effective local store and MUST conflict with all target selectors
  and new protocol candidates.
- **FR-006**: Resume MUST compare the complete stored target-authority snapshot with
  a freshly resolved target and MUST refuse any mismatch before a plan or effect.
- **FR-007**: Resume MUST freshly read the process snapshot, current compatibility
  facts, proposal, bundle destination, and recovery readiness before each attempt.
- **FR-008**: A checkpoint MUST NOT contain an authorization plan or response,
  capability, credential, private key, certificate authority, trust state, proxy
  endpoint, target secret, or authority to reuse a prior effect.
- **FR-009**: Every resumed effectful attempt MUST emit a fresh complete S134 plan and
  consume a separate exact confirmation.
- **FR-010**: Immediately before delegating an attempt, the workflow MUST atomically
  enter an in-flight state with that attempt's ordinal, phase, protocol, and exact
  S144 case key.
- **FR-011**: Resume of an in-flight workflow MUST classify the prior process as
  interrupted and MUST rely on the existing lifecycle recovery authority before any
  new effect.
- **FR-012**: After a terminal attempt, the workflow MUST atomically record the fresh
  requested, observed, completed, and remaining coverage projection plus its next
  lifecycle state.
- **FR-013**: Workflow updates MUST use revision-checked conditional writes so a
  concurrent writer cannot be overwritten silently.
- **FR-014**: The command MUST support explicit no-effect pauses for `login`, `eula`,
  `gameplay`, `shutdown`, and `interrupted` on an existing workflow.
- **FR-015**: An explicit pause MUST start no capture, proxy, trust, launch, cleanup,
  or process-control effect and MUST emit the exact pause reason and resume command.
- **FR-016**: Warm-state guidance MUST record `shutdown`; missing required positive
  post-session evidence MUST record `gameplay`; process interruption MUST record
  `interrupted`; decline MUST record `authorization`.
- **FR-017**: Completed workflows MUST remain readable and resumable as a no-effect
  current-status check, while refused target drift MUST require a fresh workflow.
- **FR-018**: Stable human and JSON guidance MUST include workflow identifier,
  revision, lifecycle state, pause reason, attempt ordinal, and exact resume command.
- **FR-019**: An explicit bundle root MUST retain attempt one compatibility and use
  the durable ordinal for every resumed sibling without overwriting a prior bundle.
- **FR-019a**: Exact attempted-case history MUST preserve S144's no-repeat bound
  across processes. A refusal before session effects MAY retry under a fresh plan and
  new ordinal, while an interrupted or failed effectful attempt MUST NOT repeat
  silently.
- **FR-020**: Existing S142 registration, S143 Steam setup, S144 bounded sequencing,
  S134 authorization, S121 fact applicability, and lifecycle recovery authorities
  MUST remain unchanged.
- **FR-021**: Tests MUST cover migration, round trip, vocabulary refusal, optimistic
  concurrency, crash-shaped in-flight resume, every pause reason, target drift,
  completed resume, bundle ordinal continuity, fresh plans, and stable rendering.
- **FR-022**: Public documentation MUST distinguish durable workflow progress from
  compatibility evidence and effect recovery, and MUST keep non-Steam topology
  authoring, ambiguity resolution, and final parent completion outside S145.

### Key Entities

- **Guided Calibration Workflow**: Store-local durable identity for one target-bound
  calibration intent and its latest safe progress checkpoint.
- **Target Authority Snapshot**: Immutable copy of the complete target fields that
  selected the workflow. It detects redirection or topology drift on resume.
- **Workflow Lifecycle State**: Closed state of `ready`, `in-flight`, `paused`,
  `completed`, or `refused`.
- **Pause Reason**: Closed reason owned by the operator or a truthful terminal
  boundary. It never counts as compatibility evidence.
- **Workflow Revision**: Positive monotonic value used for conditional atomic updates.

## Success Criteria

### Measurable Outcomes

- **SC-001**: An incomplete controlled workflow resumes in a second process and
  proposes only work still missing from fresh current facts.
- **SC-002**: All five explicit operator pauses perform zero session effects and
  round-trip through both human and JSON output with the same reason.
- **SC-003**: A crash-shaped in-flight row never completes work or authorizes a new
  session by itself; the next plan has a distinct identifier and response.
- **SC-004**: Every target-authority mismatch, invalid token, unsupported checkpoint
  version, and stale concurrent revision stops before plan emission or effects.
- **SC-005**: Resumed explicit bundle attempts use distinct deterministic destinations
  for every durable ordinal from one through fourteen.
- **SC-006**: The complete repository gate passes with no new dependency, lockfile
  package, process-control capability, system proxy, hidden trust, or parent
  completion claim.

## Assumptions

- SQLite is already the target and compatibility authority, and an additive local
  workflow table is preferable to a second sidecar persistence path.
- Workflow identifiers are unique only within the effective local store; generated
  resume commands always preserve that store path.
- Explicit pause selection records operator intent but does not inspect, automate, or
  attest to the target application's UI state.
- Existing Deep Capture recovery and Doctor behavior remain the sole authorities for
  unfinished system effects.
- S145 does not author non-Steam launch topology, select among ambiguous executable
  candidates, or close parent issue #380.

## Clarifications

### Session 2026-09-11

- Q: Is resume implicit when the same target is selected? A: No. Resume requires an
  explicit store-local workflow identifier so stale progress cannot silently alter a
  fresh calibration request.
- Q: Does a checkpoint preserve authorization or effect state? A: No. It preserves
  intent and progress only; every new attempt repeats recovery readiness, proposal,
  plan, and confirmation.
- Q: How is target drift handled? A: Any change to the complete target-authority
  snapshot refuses that workflow and requires a fresh one.
- Q: How are login, EULA, gameplay, and shutdown recognized? A: They are explicit
  operator-selected pause reasons, except warm state and missing positive evidence
  which map conservatively to shutdown and gameplay.
- Q: How are concurrent or crashed writers handled? A: Transactions and conditional
  revisions prevent overwrite, while an in-flight row is durable evidence of an
  interrupted process rather than successful work.
