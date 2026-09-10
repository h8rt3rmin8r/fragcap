# Feature Specification: Guided Steam Client Setup

**Feature Branch**: `codex/s143-guided-steam-topology`

**Created**: 2026-09-10

**Status**: Draft

**Input**: Work slice S143 adds a confirmation-bound Steam client setup step to `fragcap calibrate`, then continues through the existing S139-S142 guided calibration path.

## User Scenarios & Testing

### User Story 1 - Offer One Exact Steam Client Setup (Priority: P1)

An operator who selects a registered Steam target with no launch declaration can receive one complete proposal identifying the locally recorded Steam launch executable that may be authored as the socket-holding client. Existing usable or explicitly unresolved launch declarations remain authoritative and are never overwritten by this setup path.

**Why this priority**: S142 removes the registration prerequisite but a newly registered Steam target still stops at `missing-launch-declaration`, so the normal installed-Steam journey cannot reach calibration from the one-game front door.

**Independent Test**: Drive the command against controlled stored targets and Steam discovery results covering one exact executable, no executable, changed install roots, non-Steam anchors, existing client declarations, and existing unresolved declarations. Verify that only the exact missing-declaration Steam case produces a setup proposal and that proposal creation causes no store or session effect.

**Acceptance Scenarios**:

1. **Given** an exact positive Steam anchor, no launch declaration, and one current non-empty Steam executable hint under the same install authority, **when** guided calibration evaluates the target, **then** it emits one complete client setup proposal before requesting input.
2. **Given** a usable, unresolved, malformed, or otherwise present launch declaration, **when** guided calibration evaluates the target, **then** it preserves that declaration and reports through the existing proposal path without offering replacement.
3. **Given** a non-Steam target, missing current Steam candidate, multiple matching Steam candidates, missing executable hint, or changed install authority, **when** setup is considered, **then** no proposal or mutation occurs and the exact limitation remains visible.

---

### User Story 2 - Confirm and Revalidate Authored Client Authority (Priority: P2)

The operator sees the durable target identity, Steam application identifier, stored and discovered install authority, proposed executable, source and fidelity, target store, and exact resulting client declaration before choosing whether to attest that the executable holds the target's sockets. Human input defaults to decline. Structured input requires the exact current setup-plan identifier. After confirmation, both the stored target and local Steam evidence are read again and must remain unchanged before one conditional target-store operation records the authored client declaration.

**Why this priority**: Steam appinfo describes what Steam invokes, which can be a publisher launcher rather than the socket holder. Only an explicit operator assertion may turn that hint into client authority.

**Independent Test**: Confirm, decline, close input, supply the wrong identifier, interrupt, change the target, change the Steam candidate, remove the candidate, and race a prior update. Verify that only one unchanged confirmed authority changes the existing row and that no appinfo hint is presented as observed socket ownership.

**Acceptance Scenarios**:

1. **Given** a displayed setup proposal, **when** the operator declines, input closes, input is malformed, or interruption is observed, **then** no target field changes and no calibration session begins from that proposal.
2. **Given** a structured setup proposal, **when** the response differs by any byte, casing, whitespace, suffix, or missing line terminator, **then** the update is refused.
3. **Given** a confirmed proposal whose target row or Steam discovery authority changed before persistence, **when** revalidation runs, **then** the update is refused as drift and the changed authority remains untouched.
4. **Given** one confirmed unchanged proposal, **when** the conditional update runs, **then** the existing row retains its durable identity and non-launch fields while gaining exactly one authored client declaration.

---

### User Story 3 - Resume Guided Calibration by Durable Identity (Priority: P3)

After client setup succeeds, the same invocation re-resolves the target by durable identifier and re-enters the existing guided calibration proposal. Setup authorization is consumed only by the target update and never authorizes a later launch, trust, proxy, capture, artifact, or compatibility-fact effect.

**Why this priority**: The setup step is useful only when it removes the current dead end without creating a parallel calibration workflow or weakening the existing session authorization boundary.

**Independent Test**: Complete one controlled setup, verify that the refreshed proposal derives the cold Steam topology, and exercise ready, warm, reachability, protocol, and separately authorized session outcomes through the unchanged S139-S142 path.

**Acceptance Scenarios**:

1. **Given** a successful setup, **when** guided calibration continues, **then** the target is re-read by its durable identifier and the refreshed proposal recognizes exactly one Steam client declaration.
2. **Given** the refreshed proposal selects an effectful attempt, **when** execution reaches the existing Deep Capture authorization boundary, **then** a separate current plan response is required.
3. **Given** setup succeeds but current process, compatibility, or protocol evidence is warm, ready, incomplete, or refused, **when** guidance is emitted, **then** existing S139-S142 semantics and durable next commands remain unchanged.

### Edge Cases

- A zero, malformed, or non-Steam anchor never enters Steam client setup.
- A stored target and current Steam candidate must name the same positive application identifier and exact install root; a missing or changed root is not silently reconciled.
- A current Steam candidate whose executable is empty, whitespace, lacks one Windows executable image, or is otherwise unsuitable for an exact client declaration produces no plan.
- Appinfo may name a launcher. The proposal asks for a positive socket-holder attestation and never interprets discovery metadata, engine evidence, executable naming, or operator silence as that attestation.
- Any present launch declaration, including a prior `no` or `unsure` answer and malformed historical data, is preserved for explicit repair outside this slice.
- A target deleted, replaced, imported, promoted, or otherwise changed after proposal display is drift, even if its Steam anchor remains the same.
- A concurrent identical client update is reported as changed authority rather than silently consuming an obsolete plan.
- Sequential registration, Steam setup, and session authorization each consume their own complete input line.
- Output failure or flush failure before input results in no mutation.

## Clarifications

### Session 2026-09-10

- Q: Which existing launch states may the setup path replace? → A: Only an absent launch declaration is eligible; every present value remains authoritative for this slice.
- Q: What turns a Steam executable hint into client authority? → A: Only the operator's explicit positive socket-holder attestation; appinfo remains proposal evidence and the resulting declaration is authored.
- Q: How is a confirmation protected from concurrent target or discovery changes? → A: Rebuild the complete plan from fresh authorities and apply the update only if the stored target still exactly matches the planned row.

## Requirements

### Functional Requirements

- **FR-001**: The command MUST run Steam client setup only after ordinary stored resolution or S142 registration yields one exact durable target.
- **FR-002**: Eligibility MUST require a canonical positive `steam:<appid>` anchor and an absent launch declaration.
- **FR-003**: Every present launch declaration MUST bypass setup unchanged, including usable, unresolved, malformed, empty-array, and other historical values.
- **FR-004**: The command MUST obtain current Steam proposal evidence through the same bounded discovery composition used by S142 and MUST preserve its complete accounting and warnings.
- **FR-005**: Current proposal evidence MUST contain exactly one candidate with the target's Steam application identifier; zero or multiple matches MUST be explicit non-mutating limitations.
- **FR-006**: The stored and discovered install roots MUST both be present and exactly equal before a setup plan may be offered.
- **FR-007**: The proposed executable MUST be non-empty and reduce to one exact Windows executable image suitable for a client declaration; commands, URLs, placeholders, and ambiguous values MUST be refused.
- **FR-008**: The setup proposal MUST bind the complete stored target, complete selected candidate, conserved discovery account and warnings, effective local-store identity, proposed executable, exact resulting client declaration, authoring operation version, and no-effect boundaries.
- **FR-009**: The setup plan MUST have a deterministic versioned identifier that changes when any bound authority or proposed result changes.
- **FR-010**: Human and structured output MUST emit stable setup-plan and setup-outcome records distinct from registration, calibration guidance, and Deep Capture session events.
- **FR-011**: The complete proposal MUST be written and flushed before confirmation input is read.
- **FR-012**: Human confirmation MUST ask whether the exact executable is the socket-holding client and MUST default to decline for empty, negative, unsure, malformed, incomplete, closed, or interrupted input.
- **FR-013**: Structured confirmation MUST require `--authorize-stdin` and accept only the exact current setup-plan identifier followed by one line terminator.
- **FR-014**: Setup confirmation MUST be separate from registration confirmation and from every later Deep Capture plan authorization.
- **FR-015**: After confirmation, the command MUST re-read the target by durable identifier, repeat bounded Steam discovery, and rebuild the complete setup plan.
- **FR-016**: Any changed, missing, ambiguous, unconserved, or otherwise unreproducible authority after confirmation MUST produce a drift outcome and no target update.
- **FR-017**: The target update MUST be conditional on exact equality with the planned stored row so another writer cannot be overwritten between revalidation and persistence.
- **FR-018**: A successful update MUST retain the existing row identity, handle, name, classification, classification source, provenance, anchor, install root, evidence, detection coverage, folder name, and executable hint.
- **FR-019**: A successful update MUST write exactly one client launch declaration from the operator-attested executable and MUST stamp the resulting target fidelity as authored rather than observed or verified.
- **FR-020**: A successful update MUST NOT create a target, compatibility fact, listing snapshot, workflow record, artifact, or second storage shape.
- **FR-021**: After success, the command MUST re-resolve the target by durable identifier and rebuild the existing S139 proposal from current store, process, protocol, and compatibility evidence.
- **FR-022**: Existing S139-S142 ready, warm, reachability, protocol, continuation, and refusal semantics MUST remain unchanged after the new setup boundary.
- **FR-023**: Tests MUST prove eligibility, proposal completeness and field sensitivity, exact input, output and flush ordering, decline, interruption, target drift, discovery drift, conditional-update races, field preservation, authored fidelity, durable continuation, and separate sequential authorizations.
- **FR-024**: The slice MUST add no process handle, automatic process control, topology observation run, hidden trust, system proxy change, pinning bypass, target key extraction, multi-attempt loop, workflow persistence, dependency package, storage schema, non-Steam topology authoring, ordinary Deep Capture eligibility change, or completion claim for parent issue #380.

### Key Entities

- **Steam client setup eligibility**: A decision over one exact stored target and fresh Steam discovery that either preserves existing authority, offers one proposal, or explains why no proposal is safe.
- **Steam client setup plan**: The complete deterministic authority shown before the operator attests that one executable is the socket-holding client.
- **Steam client setup decision**: Confirmed, declined, closed, invalid, interrupted, drifted, changed, applied, or failed, without granting session authorization.
- **Conditional authored update**: One atomic replacement of an absent launch declaration on an otherwise byte-equivalent durable target row.
- **Durable calibration handoff**: The refreshed target and S139 proposal reached after setup succeeds.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Every eligible controlled Steam target produces exactly one complete setup plan and zero target or session effects before confirmation.
- **SC-002**: Every tested ineligible target or candidate state produces zero setup plans, zero target updates, and an explicit limitation.
- **SC-003**: Changing any plan-bound target, candidate, discovery, store, executable, or resulting-declaration field changes the setup-plan identifier.
- **SC-004**: Every decline, malformed response, output failure, interruption, and drift case preserves the full stored target value and creates no bundle or compatibility fact.
- **SC-005**: One confirmed unchanged plan changes exactly two target fields, the launch declaration and fidelity, while preserving every other field and the row count.
- **SC-006**: A successful controlled setup reaches the existing Steam proposal in the same invocation and any selected effectful attempt requires a separate additional authorization response.
- **SC-007**: The complete repository verification gate passes with no new dependency, lockfile package, schema migration, prohibited capability, existing launch declaration overwrite, or second calibration session.

## Assumptions

- S133 and S142 bounded discovery remain the sole current source of local Steam candidate evidence for this command.
- Steam appinfo's launch executable is a findability proposal only. It may name a publisher launcher, so positive operator attestation is required before it can become a client declaration.
- The existing resolved-client launch shape remains the single stored representation of an authored client.
- Raising the row fidelity to authored follows the existing interactive authoring contract; the classification source and all non-launch evidence remain unchanged.
- Existing present launch declarations require a future explicit repair workflow rather than implicit replacement.
- Direct and publisher topology authoring, topology observation, internal multi-attempt execution, persistent workflow state, richer operator pauses, final coverage, and ordinary Deep Capture handoff remain later work under #380.
