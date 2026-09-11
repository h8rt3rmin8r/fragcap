# Feature Specification: Explicit Guided Calibration Choice

**Feature Branch**: `codex/s146-guided-calibration-choice`

**Created**: 2026-09-11

**Status**: Draft

**Input**: Work slice S146 makes discovered target and Steam executable ambiguity an explicit stable choice, and binds advanced exact-case overrides to the durable guided-calibration workflow before any effect.

## User Scenarios & Testing

### User Story 1 - Select One Exact Candidate (Priority: P1)

An operator who encounters more than one exact discovered target or Steam executable candidate receives a bounded list with stable choice identifiers and can rerun the same command with one identifier. fragcap re-discovers and revalidates that exact candidate before registration or launch authoring.

**Why this priority**: The existing front door refuses ambiguity safely but leaves no guided continuation, so the operator must leave the workflow or edit metadata outside it.

**Independent Test**: Produce two controlled discovery candidates, verify that no target or workflow is written, rerun with one emitted identifier, confirm the exact registration plan, and verify only that candidate is registered.

**Acceptance Scenarios**:

1. **Given** multiple exact discovered targets, **when** no candidate identifier is supplied, **then** every candidate is emitted with a stable identity and no target, workflow, plan, or effect is created.
2. **Given** one emitted target candidate identifier, **when** discovery reproduces the same candidate, **then** the existing exact registration plan and confirmation boundary operate on only that candidate.
3. **Given** multiple exact Steam metadata candidates for one stored target, **when** one valid identifier is supplied, **then** the existing Steam client plan binds that exact candidate and revalidates it after confirmation.

---

### User Story 2 - Bind Advanced Exact-Case Intent (Priority: P2)

An advanced operator can explicitly select the launch case, routing strategy, and loopback family for a fresh guided workflow. Those dimensions are stored as immutable intent, shown in guidance, and reused only to build fresh proposals and plans on resume.

**Why this priority**: Exact case dimensions already determine fact applicability and authorization, but the guided front door currently hard-codes them or hides the inferred value.

**Independent Test**: Start a controlled IPv6 workflow with explicit launch and routing dimensions, pause it, resume it without repeating the flags, and verify that every fresh proposal and low-level plan retains the same exact values.

**Acceptance Scenarios**:

1. **Given** a fresh supported target, **when** the operator supplies compatible exact-case overrides, **then** the workflow persists them before any attempt and guidance reports their source as explicit.
2. **Given** a resumed workflow, **when** no override flags are supplied, **then** current authority is rebuilt using the workflow's immutable dimensions.
3. **Given** an unsupported routing strategy or a launch case inconsistent with the freshly inferred topology, **when** calibration is requested, **then** the workflow is refused before a plan or effect.

---

### User Story 3 - Refuse Stale or Misapplied Choice (Priority: P3)

An operator receives a precise no-effect refusal when a candidate identifier is malformed, unknown, duplicated, stale, supplied where no candidate choice is consumed, or combined with resume-time intent mutation.

**Why this priority**: An opaque choice is safe only if it cannot silently become positional, select a changed candidate, or widen a durable workflow after creation.

**Independent Test**: Exercise malformed, missing, stale, duplicate-identity, unused, and resume-conflicting choices and verify that each writes no target, workflow, plan, bundle, trust action, launch action, or compatibility fact.

**Acceptance Scenarios**:

1. **Given** a candidate whose authoritative fields changed after its identifier was emitted, **when** the old identifier is supplied, **then** it matches nothing and current choices are emitted again.
2. **Given** a candidate identifier with no ambiguous discovery boundary to consume it, **when** calibration proceeds, **then** the command refuses it as unused before workflow creation.
3. **Given** a durable workflow, **when** resume is combined with candidate or exact-case overrides, **then** argument parsing refuses the mutation.

### Edge Cases

- Candidate identities are content-derived from every candidate authority field that can affect registration or Steam client authoring, with evidence canonically ordered.
- Discovery accounting and warning changes do not rename an unchanged candidate, but the separately confirmed plan continues to bind the complete current discovery account and warnings.
- Two byte-identical candidates produce one choice identity and are refused as duplicate authority rather than selected by position.
- A supplied candidate identity must be consumed exactly once by either discovery registration or Steam client setup.
- Stored-target name ambiguity remains resolved by the existing durable `--id` surface.
- Direct and publisher targets with already exact launch declarations accept validated case overrides; S146 does not author or rewrite non-Steam topology.
- Only child-environment routing is currently executable. Other closed routing values may be requested for inspection but are refused by the existing proposal authority.
- IPv4 remains the default; IPv6 is explicit and flows through the existing S119 listener and readiness authority.
- A launch-case override is an assertion against the current inferred cold case, never permission to contradict target topology or warm-state truth.

## Requirements

### Functional Requirements

- **FR-001**: The command MUST expose a repeatable workflow-safe candidate selector as one exact content-derived identifier, not a row number or list position.
- **FR-002**: Candidate identity input MUST use a versioned prefix, lowercase fixed-length digest, bounded length, and strict parser.
- **FR-003**: Candidate canonicalization MUST bind identity, source, display name, classification, fidelity, install root, folder name, executable hint, detection coverage, and canonically ordered evidence.
- **FR-004**: An ambiguous discovery boundary MUST emit a stable structured choice event and equivalent human guidance containing scope, original selector or target identity, every choice identifier, and reviewable candidate fields.
- **FR-005**: No candidate supplied at an ambiguous boundary MUST be selected, registered, authored, or used to create a workflow.
- **FR-006**: A supplied candidate MUST match exactly one current canonical candidate and MUST be consumed exactly once.
- **FR-007**: An unknown, malformed, duplicate-authority, stale, or unused candidate identifier MUST stop before workflow creation or any session effect.
- **FR-008**: Target registration MUST retain its existing full discovery plan, exact confirmation, constant-time response comparison, and post-confirmation rediscovery checks after candidate selection.
- **FR-009**: Steam client authoring MUST accept an explicit candidate only when it matches the stored Steam app identifier and install root, names one valid executable, and survives existing post-confirmation revalidation.
- **FR-010**: The guided command MUST expose exact launch-case, routing-strategy, and proxy-family arguments for fresh workflows.
- **FR-011**: Fresh workflow creation MUST persist the optional launch-case assertion plus the selected routing strategy and address family before the first attempt.
- **FR-012**: Resume MUST use only the persisted exact-case intent and MUST conflict with candidate, launch-case, routing-strategy, proxy-family, and new protocol arguments.
- **FR-013**: The workflow store MUST migrate additively, validate every closed token, retain version refusal, and round-trip the new intent without storing authorization, responses, secrets, or effects.
- **FR-014**: Every fresh proposal MUST apply the workflow routing strategy and address family, then require any stored launch-case assertion to equal the current inferred cold case.
- **FR-015**: Every delegated low-level attempt MUST use the workflow address family and the proposal-selected launch case, routing strategy, and protocol under a fresh S134 plan.
- **FR-016**: Guidance MUST report candidate choice state, current launch case, optional launch-case assertion, routing strategy, and address family in stable human and JSON output.
- **FR-017**: Existing direct, Steam, publisher, warm-close, pause, resume, current-fact, no-repeat, recovery, confirmation, and structured-output behavior MUST remain intact.
- **FR-018**: Tests MUST cover identifier determinism and field sensitivity, ambiguous target and Steam choices, stale and unused identifiers, duplicate authority, resume conflicts, schema migration, workflow round trip, IPv6 propagation, every supported topology, and pre-effect refusal.
- **FR-019**: Public and architecture documentation MUST distinguish choosing existing authority from authoring topology, and MUST leave non-Steam topology authoring plus the final parent completion gate outside S146.

### Key Entities

- **Calibration Candidate Choice**: A versioned content-derived reference to one complete current discovery candidate.
- **Choice Set**: One bounded ambiguous selection boundary with a scope, context, and reviewable candidate projections.
- **Exact-Case Intent**: Immutable workflow values for optional launch-case assertion, routing strategy, address family, and requested protocol set.
- **Choice Consumption**: Proof that one supplied candidate identifier selected exactly one current candidate at exactly one permitted boundary.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Every controlled two-candidate target and Steam ambiguity emits two stable identifiers and performs zero writes or effects until one is supplied.
- **SC-002**: Changing any authority-bearing candidate field changes its identifier, while input order and evidence order do not.
- **SC-003**: A selected candidate survives full plan confirmation and rediscovery, while every stale, malformed, duplicate, or unused value stops before workflow creation.
- **SC-004**: An explicit IPv6 workflow resumes in a second process and all fresh proposal, guidance, and delegated plan dimensions remain IPv6.
- **SC-005**: Direct, Steam, and publisher exact topologies accept their matching launch assertion and refuse every mismatched assertion before effects.
- **SC-006**: The complete repository gate passes with no new dependency or lockfile package, no process control, no topology rewrite outside the existing Steam authoring operation, and no parent completion claim.

## Assumptions

- BLAKE3 and canonical JSON are already direct runtime authorities in the relevant crates.
- Candidate choice identities are ephemeral command inputs and are not durable database identifiers.
- The target snapshot already binds the consequence of a chosen registration or Steam executable candidate.
- Child-environment remains the only supported routing strategy until another strategy gains its own shipped plan and effect authority.
- Candidate output may contain local paths already present in current ambiguity diagnostics and registration plans.

## Clarifications

### Session 2026-09-11

- Q: Is a candidate selected by its displayed order? A: No. The identifier is derived from canonical authority and position never participates.
- Q: Does selecting a Steam executable bypass confirmation? A: No. Selection narrows the current candidate set; the existing complete Steam client plan and confirmation still authorize the write.
- Q: Can a launch-case override force a topology? A: No. It is an exact assertion that must match the freshly inferred cold case.
- Q: Can resume change exact-case flags? A: No. Exact-case intent is immutable for one workflow; start a fresh workflow to change it.
- Q: Does S146 author ambiguous direct or publisher launch chains? A: No. It chooses only discovered target and existing Steam metadata candidates. Non-Steam topology authoring remains separate work under #380.
