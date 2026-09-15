# Feature Specification: Deep Capture Session UX Completion

**Feature Branch**: `codex/s149-deep-capture-session-ux`

**Created**: 2026-09-15

**Status**: Draft

**Input**: S149 completes issue #332 after S148's offline workflow discovery, preserving exact plan-bound consent and controlled implementation acceptance.

## User Scenarios & Testing

### User Story 1 - Understand Authorization (Priority: P1)

An authorized operator can follow Doctor and calibration guidance, review the selected target and exact session effects, and distinguish CA trust consent from sensitive-output consent before authorizing the displayed plan.

**Why this priority**: Active inspection requires informed exact-scope consent before any effect.

**Independent Test**: Render controlled session plans with and without trust, key logging, and client identity, then decline or supply invalid input and verify zero session effects.

**Acceptance Scenarios**:

1. **Given** a clean controlled first-run journey, **When** the operator reaches the session authorization plan, **Then** the plan identifies Deep Capture as active target-scoped inspection, lists effects and sensitive artifacts, and states that Capture remains passive.
2. **Given** optional trust or sensitive controls, **When** the plan is displayed, **Then** each selected consequence is independently visible and the current exact identifier remains the authorization authority.
3. **Given** declined, closed, invalid, or interrupted input, **When** authorization ends, **Then** no session effect occurs and the outcome remains explicit.

### User Story 2 - Follow Observed Progress (Priority: P1)

The operator sees readiness, native proxy start, trust state, managed launch, observation, finalization, and cleanup as distinct stages, with live packet counters and separately delivered application counters describing only observed evidence.

**Why this priority**: Proxy readiness, traffic observation, decryption, and process correlation are different facts.

**Independent Test**: Feed typed lifecycle and observation events into the production presentation adapter and assert their human and structured projections.

**Acceptance Scenarios**:

1. **Given** a started proxy without traffic, **When** progress is shown, **Then** it says only that the proxy is ready, not that the target is inspectable.
2. **Given** full, metadata-only, opaque, unknown, or lost observations, **When** counters are emitted, **Then** counts retain those distinctions without inferring final-client ownership or universal protocol support.
3. **Given** calibration, **When** phases advance, **Then** human guidance names observation and fact persistence without equating a request with evidence.

### User Story 3 - Recover From Partial Results (Priority: P1)

The operator can distinguish completed, interrupted, partial, and failed sessions, locate retained evidence, and learn whether exact owned resources were released or need Doctor recovery.

**Why this priority**: A failed session can retain useful evidence and unresolved residue independently.

**Independent Test**: Render controlled terminal and cleanup outcomes, including no effects, released resources, and failed or unsupported cleanup.

**Acceptance Scenarios**:

1. **Given** interruption or partial collection, **When** a terminal report is shown, **Then** the report names retained bundle evidence and does not describe incomplete artifacts as complete.
2. **Given** unresolved cleanup, **When** recovery guidance is shown, **Then** it names observed owned residue and directs the operator to exact confirmation-gated Doctor recovery without deleting or broadening authority.
3. **Given** successful cleanup, **When** terminal guidance is shown, **Then** retained sensitive evidence remains distinct from resource residue and is never silently removed.

### Edge Cases

- No observations, launcher-only or uncorrelated observations, omitted evidence, and observation loss cannot establish decryption or target compatibility.
- Quiet suppresses progress but retains terminal outcomes and warnings; silent preserves required authorization and failures while suppressing optional output; JSON is deterministic and prompt-free.
- At 40 through 80 columns, human prose wraps without changing exact non-secret values; indivisible identifiers and paths may exceed width.
- A failure before effects must not invent retained artifacts or cleanup obligations.
- Presentation failure before authorization must fail closed, and optional progress cannot acquire effect authority.

## Requirements

### Functional Requirements

- **FR-001**: Preserve the existing Doctor-to-calibration-to-session first-run journey and exact target, case, plan identifier, deadlines, and default-no consent authority.
- **FR-002**: Before authorization, summarize active mode, target-scoped launch/routing, trust selection, sensitive output selection, artifact sensitivity, retention, and recovery consequences while retaining the complete canonical plan.
- **FR-003**: Preserve separate CA, sensitive-output, registration, and target-authoring consent; summaries MUST NOT replace or broaden any required consent.
- **FR-004**: Project typed readiness, proxy, trust, launch, observation, finalization, and cleanup events into truthful bounded human progress without new policy or secret-bearing output.
- **FR-005**: Application counters MUST distinguish delivered inspection classes and unavailable ownership without calling post-collection observations live; existing live packet counters and terminal classification remain independently authoritative.
- **FR-006**: Terminal human output MUST preserve actual complete, partial, interrupted, or failed state, identify only actually retained evidence, report exact cleanup outcomes, and provide recovery guidance only for unresolved owned resources.
- **FR-007**: Human, quiet, silent, JSON, and 40-through-80-column contracts MUST be covered by controlled tests; JSON MUST retain its existing schemas and never receive human prose or prompts.
- **FR-008**: All verification MUST use synthetic events, controlled fixtures, or injected adapters, with no live capture, game execution, real trust mutation, or installed-build testing by the agent.
- **FR-009**: Trace every issue #332 acceptance criterion to existing or new executable controlled evidence; leave #331, #333, #334, #372, and #278 open.

### Key Entities

- **Authorization plan**: Complete immutable scope and consent authority displayed before effects.
- **Session progress**: Presentation of an observed lifecycle transition or counted observation, not an inspectability promise.
- **Terminal report**: Independent session, artifact, observation, and cleanup facts used to explain outcome and next action.

## Success Criteria

### Measurable Outcomes

- **SC-001**: All five issue #332 acceptance criteria have exact passing controlled test references.
- **SC-002**: Every selected trust and sensitive-output consequence is visible before input, and all non-positive authorization tests prove zero session effects.
- **SC-003**: Every supported typed lifecycle stage and terminal outcome has a tested human projection; no readiness stage claims traffic or decryption.
- **SC-004**: All five output contracts preserve their defined disclosure and prompt behavior, including narrow terminal semantics.
- **SC-005**: Full repository verification and required PR checks pass, and every arrived review finding is answered and resolved before operator merge review.

## Assumptions

- Native readiness, plan-bound authorization, protocols, artifact retention, exact cleanup, packaging, and guided calibration are implemented by prior slices and remain policy authority.
- Controlled implementation acceptance does not claim live game compatibility. Optional live validation belongs to the operator against published product bytes.
- No new dependency name, storage, artifact schema, structured-event schema, trust policy, or selected protocol change is needed. The online PR audit requires one version-only exception, exact Rustls 0.23.43 to upstream-patched 0.23.45 for RUSTSEC-2026-0285, recorded in the plan and research.

## Clarifications

### Session 2026-09-15

- Q: Replace canonical authorization with a short summary or retain both? A: Retain the complete canonical plan and add a human consequence summary; exact consent authority must remain inspectable.
- Q: Is live installed or game execution required to close implementation UX? A: No. Existing controlled acceptance policy applies; live compatibility remains optional operator-owned release evidence.
- Q: Add structured lifecycle schemas or project existing events? A: Project existing typed events and preserve structured schemas, avoiding unnecessary contract expansion.
- Q: Are application observations delivered during collection? A: No. They arrive after collection and reconciliation, so application counters explicitly describe delivered records while existing packet counters retain live authority.
