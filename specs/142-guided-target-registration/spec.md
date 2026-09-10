# Feature Specification: Guided Target Discovery and Registration

**Feature Branch**: `codex/s142-guided-target-registration`

**Created**: 2026-09-10

**Status**: Draft

**Input**: Work slice S142 adds confirmation-gated discovery and registration of an unregistered installed target to `fragcap calibrate`, then continues through the existing S139-S141 guided calibration path.

## User Scenarios & Testing

### User Story 1 - Find One Unregistered Installed Game (Priority: P1)

An operator can pass the same one-game argument to `fragcap calibrate` whether the target is already registered or is only present in current installed-game discovery. A stored target always wins. A clean stored miss triggers bounded discovery and returns either one exact candidate, an explicit small ambiguity, or a truthful no-match result.

**Why this priority**: The current front door stops before the workflow begins when a target has not already been registered, which is the first manual prerequisite named by parent issue #380.

**Independent Test**: Drive the command against a controlled discovery fixture with stored matches, stored ambiguity, exact Steam application identifiers, exact installed names, multiple same-name candidates, and misses. Verify that only one exact candidate reaches a registration proposal and that discovery causes no session effect.

**Acceptance Scenarios**:

1. **Given** an existing stored target, **when** its handle, name, row, or stable identifier is selected, **then** the command skips discovery and preserves S140-S141 behavior.
2. **Given** no stored match and one discovered candidate whose Steam application identifier or case-insensitive display name exactly matches the selector, **when** calibration starts, **then** the command presents that exact candidate for registration.
3. **Given** no stored match and several discovered candidates matching the selector, **when** calibration starts, **then** every match is listed with stable observed identity and no candidate is selected or persisted.
4. **Given** a numeric selector that names an existing listing row, **when** calibration starts, **then** the stored row wins; only a numeric stored miss may be interpreted as an exact discovered Steam application identifier.

---

### User Story 2 - Confirm One Exact Registration (Priority: P2)

Before a target row is written, the operator sees the candidate's observed identity, display name, source, fidelity, classification, install root, executable hint, evidence, the complete conserved discovery account and warnings, and the exact durable identity that registration will create when one is derivable. Interactive mode defaults to decline. Structured mode requires the exact emitted registration-plan identifier. After confirmation, discovery is repeated and the complete candidate plus discovery authority must still match before the existing one-path registration operation may run.

**Why this priority**: Registration changes the durable target store and can otherwise turn an ambiguous or stale discovery observation into long-lived false authority.

**Independent Test**: Confirm, decline, close, supply the wrong identifier, mutate the candidate between proposal and confirmation, and race an idempotent prior registration. Verify that only the unchanged confirmed candidate reaches the shared registration operation and that every outcome is explicit.

**Acceptance Scenarios**:

1. **Given** one exact candidate in interactive mode, **when** the operator declines or input closes, **then** no target row is inserted and the command exits without a calibration session.
2. **Given** one exact candidate in structured mode, **when** input does not equal the complete current registration-plan identifier plus one newline, **then** no target row is inserted.
3. **Given** a confirmed candidate whose discovery authority changed before persistence, **when** registration revalidates it, **then** the command refuses the write and reports drift.
4. **Given** a confirmed unchanged candidate, **when** the shared registration operation runs, **then** exactly one target is inserted or an exact already-present target is reused without duplication.

---

### User Story 3 - Continue Guided Calibration by Durable Identity (Priority: P3)

After registration, the same command resolves the resulting stored row by durable identity and enters the existing S139-S141 proposal machinery. Discovery candidates intentionally carry no authoritative launch entries, so a newly inserted row normally reaches the existing `missing-launch-declaration` limitation. If an exact already-present row has independently acquired authoritative topology, the remaining warm-state, reachability, protocol, authorization, fact, and continuation machinery stays available. Registration authorization never authorizes a Deep Capture plan, and the command never starts more than one calibration session.

**Why this priority**: Registration is useful here because it removes a prerequisite from the one-game workflow, not because it creates a second target-management command.

**Independent Test**: Confirm a controlled unregistered candidate, re-resolve the inserted row, and verify that the existing limitation is reached with the resulting stable identifier and no session effect. Separately retain the existing exact authorization behavior for already registered targets with authoritative topology.

**Acceptance Scenarios**:

1. **Given** a confirmed discovered candidate, **when** registration completes, **then** calibration continues through the existing guided proposal using the new durable stable identifier without promoting its hints into launch topology.
2. **Given** a confirmed candidate whose stored topology cannot yet support calibration, **when** the existing proposal evaluates it, **then** its typed limitation and durable next action are reported without inventing topology or session effects.
3. **Given** a later invocation reaches an effectful calibration plan, **when** it runs with `--authorize-stdin`, **then** the prior registration response grants no authority and the current Deep Capture plan still requires its own exact complete input line.
4. **Given** any registration outcome, **when** the workflow reports machine-readable output, **then** registration plan, outcome, and existing guidance remain distinguishable stable events.

### Edge Cases

- Stored ambiguity is terminal and never falls through to discovery.
- A discovered display name can match across Steam and path sources; all exact matches are listed and none is guessed.
- A discovered path candidate has no durable stable identifier until registration; `--id` therefore resolves stored targets only.
- A candidate removed, renamed, reclassified, moved, or changed in evidence between preview and confirmation is drift, not an equivalent result.
- Discovery warnings and conservation counts remain visible and cannot be treated as a clean complete search.
- An exact candidate already inserted by another process after preview is reused only when its identity is the same; a conflicting row refuses.
- Failure to flush the proposal before reading confirmation is a no-write error.
- Registering a candidate may still leave launch topology unresolved. Registration success is not calibration readiness.

## Clarifications

### Session 2026-09-10

- Q: How does a bare numeric selector interact with existing row numbers and Steam application identifiers? → A: Existing stored row semantics keep precedence. Only a stored no-match may use the same token as an exact discovered Steam application identifier.
- Q: What authorizes registration in human and structured execution? → A: Human execution shows the complete plan and accepts an explicit affirmative response, defaulting to no. `--authorize-stdin` requires the exact emitted registration-plan identifier on its own complete line. Any later Deep Capture plan requires a separate line and authorization.
- Q: Which discovery matches are safe to select automatically? → A: Only an exact Steam application identifier or exact case-insensitive candidate display name with one result. Fuzzy, substring, path-prefix, folder, executable-hint, and inferred-handle matching are excluded.
- Q: What happens after a confirmed path candidate lacks calibration topology? → A: It is registered through the common operation, re-resolved by its new durable identifier, and allowed to reach the existing typed limitation. S142 does not fabricate launch entries.
- Q: Does confirmation permit discovery to persist all eligible candidates? → A: No. The plan binds one candidate and only that candidate may be passed to the existing single-candidate registration operation.

## Requirements

### Functional Requirements

- **FR-001**: The command MUST attempt existing stored-target resolution before any discovery and MUST preserve every resolved and ambiguous stored result unchanged.
- **FR-002**: The command MUST run the existing bounded discovery composition only after a clean stored no-match.
- **FR-003**: Discovery selection MUST accept only one exact Steam application identifier or one exact Unicode-aware case-insensitive display name and MUST NOT use fuzzy, substring, inferred-handle, folder, executable-hint, or path-prefix matching.
- **FR-004**: A bare numeric selector MUST retain stored listing-row precedence and MAY become a Steam application identifier only after that row resolution returns no match.
- **FR-005**: `--id` MUST remain a stored durable-identifier selector and MUST NOT invent an identifier for an unregistered path candidate.
- **FR-006**: Every discovery ambiguity MUST list all exact matches with source, observed identity, display name, fidelity, classification, and install root where available, then exit with no target registration or session effect.
- **FR-007**: Discovery failure, incomplete coverage, warnings, and conservation counts MUST remain visible and MUST NOT be collapsed into an ordinary no-match.
- **FR-008**: The registration proposal MUST bind the complete selected `CandidateTarget`, conserved discovery account and warnings, effective local-store path, registration operation version, and the predicted anchored stable identifier when one exists.
- **FR-009**: The registration plan identifier MUST be a versioned domain-separated BLAKE3 digest of deterministic canonical JSON and MUST change when any bound candidate, discovery, or destination field changes.
- **FR-010**: The complete registration proposal MUST be emitted and flushed before confirmation input is read.
- **FR-011**: Interactive registration MUST require an affirmative complete line and MUST default to decline on empty, negative, invalid, incomplete, closed, or interrupted input.
- **FR-012**: JSON execution MUST require `--authorize-stdin`; structured registration MUST accept only the exact current plan identifier followed by one newline using constant-time comparison.
- **FR-013**: Registration confirmation MUST be separate from and MUST NOT authorize any later Deep Capture plan, trust change, launch, proxy, capture, artifact, or compatibility-fact effect.
- **FR-014**: After confirmation and before persistence, the command MUST repeat discovery, preserve exact no-match or ambiguity diagnostics, and require byte-equivalent canonical candidate authority and destination authority.
- **FR-015**: Only the confirmed unchanged candidate MAY be passed to the existing `register_candidate` operation; S142 MUST NOT add a storage shape, source-specific insert, or bulk persistence path.
- **FR-016**: Registration MUST remain idempotent. A same-identity row created concurrently MAY be reused, while a conflicting or unresolvable post-write identity MUST refuse continuation.
- **FR-017**: After successful or idempotent registration, the command MUST replace the original discovery selector with the resulting durable stable identifier for every downstream resolution and continue through the existing S139-S141 guided path.
- **FR-018**: A candidate with unsupported or unresolved launch topology MUST retain the existing typed proposal limitation; S142 MUST NOT synthesize launch authority from discovery hints.
- **FR-019**: Human output MUST state candidate identity, destination store, registration decision, resulting stable target identity, and whether calibration continued.
- **FR-020**: Machine-readable output MUST add stable registration-plan and registration-outcome events without changing existing guidance or low-level event meanings.
- **FR-021**: Tests MUST prove stored precedence, exact matching, ambiguity, refusal, plan integrity, revalidation drift, idempotency, separate sequential authorizations, durable handoff, and zero pre-confirmation target-row/session effects.
- **FR-022**: The slice MUST add no process handle, automatic process control, hidden or system trust, system proxy change, pinning bypass, target TLS key extraction, internal multi-attempt loop, workflow persistence, dependency package, storage schema, or Deep Capture completion claim, and MUST leave parent issue #380 open.

### Key Entities

- **Discovery selection**: One stored resolution result or one exact candidate match, preserving ambiguity and incomplete-search evidence.
- **Registration plan**: A deterministic complete preview of one candidate, one target-store destination, and one shared registration operation, with a versioned exact identifier.
- **Registration decision**: Declined, closed, invalid, interrupted, drifted, registered, already-present, or failed, with no implication for session authorization.
- **Registered-target handoff**: The resulting stable identifier used to re-enter the existing guided calibration path in the same invocation.

## Success Criteria

### Measurable Outcomes

- **SC-001**: One unregistered installed Steam target selected by exact name or application identifier reaches a complete registration plan from one top-level command with zero target-row or session effects before confirmation.
- **SC-002**: Every tested ambiguous selector produces all and only its exact candidates, selects none, and performs zero target registration and session-effect calls.
- **SC-003**: Changing any bound candidate or store field changes the registration-plan identifier, and every tested stale confirmation performs zero registration writes.
- **SC-004**: A confirmed unchanged candidate produces exactly one durable target or reuses one exact target, after which the existing guided decision names the same stable identifier.
- **SC-005**: Interactive decline and every malformed structured response preserve the target count and create no calibration bundle.
- **SC-006**: A structured registration consumes exactly one registration-plan response and starts no session from unresolved discovery hints; existing effectful calibration tests continue to require the separate current Deep Capture plan identifier.
- **SC-007**: The complete repository verification gate passes with no new dependency, lockfile package, storage migration, prohibited capability, bulk auto-registration, or existing low-level calibration behavior change.

## Assumptions

- S133 discovery precision and the existing shared `TargetSource` composition remain authoritative.
- S051/S055 `register_candidate` remains the sole target creation operation and may register a deliberately selected candidate even when S133 withholds it from automatic registration.
- S139-S141 remain authoritative for topology, process state, proposal ordering, one-attempt execution, fact reassessment, and continuation.
- The confirmation boundary governs insertion or reuse of the selected target row. Existing store opening, schema maintenance, and volume-eligibility accounting are repository infrastructure and do not grant candidate persistence.
- Internal multi-attempt execution, persistent workflow state and resume, richer operator-pausing state, advanced case overrides, final coverage, and ordinary Deep Capture handoff remain later work under #380.
