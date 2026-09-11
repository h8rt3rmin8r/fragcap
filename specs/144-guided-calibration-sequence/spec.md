# Feature Specification: Bounded Guided Calibration Sequence

**Feature Branch**: `codex/s144-guided-calibration-sequence`

**Created**: 2026-09-11

**Status**: Draft

**Input**: Work slice S144 lets one `fragcap calibrate` invocation advance through a finite sequence of useful reachability and protocol attempts while preserving one fresh plan and one confirmation for every effectful attempt.

## User Scenarios & Testing

### User Story 1 - Advance Through Useful Calibration Work (Priority: P1)

An operator can start guided calibration once and let the command advance from missing reachability evidence into the minimum useful requested or directly observed protocol attempts. The command reports progress and current coverage after every attempt instead of requiring a new process invocation whenever the preceding attempt succeeds.

**Why this priority**: S140 and S141 prove one exact attempt at a time, but their mandatory copy-and-rerun boundary prevents the parent workflow from behaving as one guided session.

**Independent Test**: Run controlled targets through missing reachability, multiple requested protocols, and protocols discovered from eligible final-client observations. Verify deterministic attempt order, current fact re-evaluation, exact progress, and an ordinary Deep Capture command after all current candidates complete.

**Acceptance Scenarios**:

1. **Given** missing current reachability and two requested protocols, **when** each separately authorized attempt records its required positive fact, **then** one command runs reachability followed by each still-useful protocol attempt in deterministic order.
2. **Given** a reachability attempt reveals an eligible final-client protocol not explicitly requested, **when** current routing evidence is recorded, **then** that protocol becomes a distinct observed candidate for a later attempt in the same invocation.
3. **Given** all requested and observed candidates already have current positive evidence, **when** the refreshed proposal contains no work, **then** the command starts no further session and reports exact completed coverage plus the ordinary Deep Capture command.

---

### User Story 2 - Authorize Every Attempt Separately (Priority: P2)

Before each effectful attempt, the operator receives the complete current S134 authorization plan for that one session and confirms it separately. The command rebuilds authority from the current target, process state, compatibility facts, proposal, deadlines, bundle, and session effects rather than extending a prior decision.

**Why this priority**: A convenient sequence cannot become blanket authorization for later trust, launch, proxy, capture, artifact, or fact-write effects.

**Independent Test**: Supply exact responses for some plans and then decline, close input, provide an invalid identifier, change authority, or interrupt before the next response. Verify that only already confirmed attempts run and that every unconfirmed attempt applies zero effects.

**Acceptance Scenarios**:

1. **Given** a successful first attempt and a newly selected second attempt, **when** the second plan is emitted, **then** it has its own current identifier and requires a new complete response line.
2. **Given** the operator declines or does not exactly confirm a later plan, **when** the decision is processed, **then** no effect from that attempt or any later attempt occurs and prior evidence remains valid.
3. **Given** target, process, fact, proposal, or bundle authority changes between attempts, **when** the next step is considered, **then** the command rebuilds or refuses from current authority and never reuses the preceding plan.

---

### User Story 3 - Stop Safely With Exact Continuation (Priority: P3)

When the sequence cannot safely advance, it stops at the first boundary, names the reason, reports completed and remaining coverage, and provides an exact continuation when current evidence supports one. It never retries a case inside the invocation or loops without a newly proven fact.

**Why this priority**: Calibration commonly encounters no traffic, warm processes, user prompts, pinning, interruption, or partial evidence. These are workflow states, not permission to retry or fabricate completion.

**Independent Test**: Exercise partial evidence, terminal failure, warm-state change, limitations, repeated proposals, bundle collisions, and the complete supported candidate set. Verify finite termination, zero silent retries, and truthful final guidance.

**Acceptance Scenarios**:

1. **Given** an attempt returns without its required current positive fact, **when** the refreshed proposal is evaluated, **then** the sequence stops and preserves that case as remaining.
2. **Given** a session error, warm-state transition, proposal limitation, or repeated exact case, **when** the boundary is observed, **then** no later attempt starts and the terminal guidance distinguishes the cause.
3. **Given** every supported candidate is requested, **when** the sequence runs, **then** it executes no more than one reachability case and one case per concrete protocol and terminates within the closed bound.

### Edge Cases

- A plan decision authorizes only the exact attempt whose complete plan was displayed; registration and Steam setup confirmations remain separate lines and grant no session authority.
- Requested candidates remain distinguishable from observed candidates even when the same protocol appears in both sets.
- Only eligible final-client terminal observations may add protocol candidates; launcher, proxy, infrastructure, ambiguous, and unowned observations add none.
- A refreshed proposal may remove work because another writer recorded current evidence. The command skips that now-unnecessary case without treating it as an attempt.
- A refreshed proposal may change launch case, become warm, gain limitations, lose the target, or become otherwise unready. The sequence stops before another plan is prepared.
- An exact case selected twice without an intervening new fact is a no-progress cycle. It is reported and never executed twice.
- When `--bundle` is supplied, the first attempt retains that exact destination and later attempts use distinct deterministic sibling destinations. A pre-existing non-empty destination is refused by the existing bundle authority.
- Without `--bundle`, every delegated session retains the existing unique default session destination.
- A declined interactive plan is a clean stop. Invalid structured input, operational failure, and terminal session failure retain their existing non-success exits.
- Process interruption is checked before selecting and before authorizing every later attempt.

## Clarifications

### Session 2026-09-11

- Q: Does one guided invocation create one authorization envelope for all attempts? → A: No. Every effectful attempt retains one freshly built S134 plan and a separate exact confirmation.
- Q: When may the sequence advance after an attempt? → A: Only when a fresh exact proposal proves the attempted case now has the required current positive fact and selects different useful work.
- Q: How is the loop bounded? → A: One invocation may execute reachability once and each concrete supported protocol at most once; selecting an attempted exact case stops as a no-progress cycle.
- Q: How does an explicit bundle path behave across attempts? → A: The first attempt retains the exact path for compatibility; later attempts receive deterministic sibling paths and existing non-empty-path refusal remains authoritative.
- Q: Does this slice persist or resume workflow state? → A: No. S144 is a finite in-process sequence; exact continuation remains the cross-process handoff.

## Requirements

### Functional Requirements

- **FR-001**: The existing registration, Steam client setup, warm restart, target resolution, proposal, plan authorization, session execution, fact persistence, and ordinary eligibility authorities MUST remain the only authorities for their respective decisions and effects.
- **FR-002**: After front-door setup, the command MUST maintain one in-process sequence containing the original requested protocol set, accumulated eligible observed protocol set, attempted exact case set, current attempt number, and closed attempt bound.
- **FR-003**: Before selecting every effectful attempt, the command MUST freshly open the effective store, re-resolve the target by durable identifier, take a current complete process snapshot, read current compatibility facts, and rebuild the S139 proposal from the accumulated candidates.
- **FR-004**: The command MUST stop before preparing another session when the target cannot be re-resolved, process evidence is unavailable, the proposal has limitations, launch readiness is unavailable or warm, interruption is pending, or current authority otherwise refuses.
- **FR-005**: The command MUST select only the first valid useful step from the current deterministic S139 proposal.
- **FR-006**: One invocation MUST execute reachability at most once and each concrete supported protocol at most once for one exact launch, routing, and family case.
- **FR-007**: The sequence MUST have a closed bound equal to one reachability attempt plus the finite supported concrete protocol candidate set, currently fourteen total attempts.
- **FR-008**: A selected exact case already present in the invocation's attempted set MUST stop the sequence as an explicit no-progress cycle before another session plan or effect.
- **FR-009**: Every selected attempt MUST emit stable progress carrying its one-based attempt number, closed maximum, phase, protocol, selected launch case, completed candidates, remaining candidates, and distinct requested and observed candidate sets.
- **FR-010**: Every effectful attempt MUST delegate through the unchanged S134 executor, which MUST build, display, flush, and separately confirm one complete current authorization plan for that attempt.
- **FR-011**: Registration, Steam setup, warm-restart confirmation, and every session plan MUST consume separate input and MUST NOT authorize any later operation.
- **FR-012**: Human input MUST preserve default-decline semantics for every attempt; structured input MUST require the exact current plan identifier on a separate complete line for every attempt.
- **FR-013**: A decline, closed input, or interruption MUST stop the sequence without applying the pending attempt or selecting a later one.
- **FR-014**: Invalid structured input, plan drift, operational error, and terminal session failure MUST preserve their existing non-success exits and MUST prevent every later attempt.
- **FR-015**: After each executed attempt, the command MUST accumulate only protocols derived by the existing eligible final-client observation authority and MUST retain requested and observed sets separately.
- **FR-016**: Candidate merging and attempt selection MUST be deterministic, duplicate-free, and independent of observation, input, store-row, or map iteration order.
- **FR-017**: After each executed attempt, the command MUST freshly re-resolve the target and rebuild the proposal before classifying the attempt as completed, incomplete, refused, or failed.
- **FR-018**: The sequence MUST continue only if that fresh proposal proves the just-attempted case has the required current positive evidence, the session has no terminal error, and a different valid useful step remains.
- **FR-019**: Missing positive evidence, partial evidence, changed launch readiness, new limitations, or a still-required attempted case MUST stop the sequence without retry and MUST remain visible in completed and remaining coverage.
- **FR-020**: When no current useful steps remain, ordinary Deep Capture prerequisites MUST be revalidated before guidance claims readiness or requested coverage completion.
- **FR-021**: Terminal guidance MUST distinguish complete, declined, incomplete, refused, failed, interrupted, and no-progress outcomes and MUST carry an exact continuation command whenever current authority supports one.
- **FR-022**: If `--bundle` is supplied, attempt one MUST use the exact supplied path and every later selected attempt MUST use a deterministic sibling path containing its attempt number, phase, and protocol without weakening existing empty-directory validation.
- **FR-023**: If `--bundle` is omitted, every attempt MUST retain the existing unique default session bundle behavior.
- **FR-024**: Bundle derivation MUST be path-safe, must not depend on untrusted display text, and must fail before authorization when a distinct destination cannot be represented.
- **FR-025**: Human and JSON output MUST preserve existing event identities while exposing enough stable attempt and coverage fields to reconstruct the in-process sequence in order.
- **FR-026**: Tests MUST prove multi-attempt success, deterministic requested ordering, observed-candidate growth, separate confirmations, decline and invalid-input stopping, partial and terminal failure stopping, fresh fact and target reads, repetition prevention, closed bounds, bundle separation, and truthful final handoff.
- **FR-027**: Documentation MUST explain that one invocation can run multiple separately authorized sessions and MUST describe bundle layout, stop conditions, coverage output, and the remaining explicit continuation boundary.
- **FR-028**: The slice MUST add no persistent workflow or resume state, storage or artifact schema, dependency package, automatic process control, system-wide proxy effect, pinning bypass, non-Steam topology authoring, ambiguous topology choice, ordinary Deep Capture eligibility change, or completion claim for parent issue #380.

### Key Entities

- **Guided calibration sequence**: Finite in-process orchestration state for one durable target, including requested, observed, attempted, completed, and remaining work.
- **Exact attempted case**: One launch case, routing strategy, address family, phase, and protocol identity that may execute only once per invocation.
- **Attempt progress**: Stable ordered guidance before and after one separately authorized delegated session.
- **Current completion proof**: A fresh proposal and exact current compatibility facts showing that the attempted case produced the required positive evidence.
- **Attempt bundle destination**: The unique evidence root bound into one attempt's authorization plan.

## Success Criteria

### Measurable Outcomes

- **SC-001**: A controlled target with missing routing and two missing requested protocols completes three separately authorized attempts in one invocation and reports no remaining requested coverage.
- **SC-002**: Every effectful attempt emits exactly one distinct authorization plan and consumes exactly one separate confirmation response.
- **SC-003**: A declined, invalid, interrupted, incomplete, refused, or failed attempt starts zero later sessions and reports no false completion.
- **SC-004**: Every exact case executes at most once and an invocation terminates after no more than fourteen attempts in the current supported matrix.
- **SC-005**: Permuting requested or observed input order produces the same candidate order, attempt order, coverage sets, and terminal guidance.
- **SC-006**: A supplied bundle root remains the first attempt destination, every later attempt has a distinct deterministic sibling, and no completed attempt bundle is reused or overwritten.
- **SC-007**: After all current requested and observed work succeeds, the final guidance carries a parseable ordinary Deep Capture command and current exact completion coverage.
- **SC-008**: The complete repository verification gate passes with no new dependency, lockfile package, storage migration, prohibited capability, persistent workflow state, or parent completion claim.

## Assumptions

- S139 remains the deterministic authority for useful exact case ordering, S121 remains the current compatibility-fact authority, and S134 remains the per-session authorization authority.
- The concrete supported protocol set is closed and enumerable in this product version; expanding it also expands the mechanical sequence bound.
- Directly observed candidates describe work worth proposing, not evidence that the protocol is compatible.
- A successful delegated session is insufficient for progression until a fresh fact read proves the attempted case's required positive evidence.
- An operator may be asked to confirm several plans in one invocation because each plan binds different session, bundle, protocol, trust, and possible fact-write authority.
- Persistent state, interruption resume after process exit, non-Steam topology authoring, ambiguous topology choices, richer operator exercise pauses, and final parent completion remain later work under #380.
