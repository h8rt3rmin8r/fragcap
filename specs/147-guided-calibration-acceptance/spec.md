# Feature Specification: Guided Calibration Acceptance

**Feature Branch**: `codex/s147-guided-calibration-acceptance`

**Created**: 2026-09-11

**Status**: Draft

**Input**: Work slice S147 closes the guided-calibration parent outcome in issue #380 with exhaustive automated and controlled-target acceptance evidence, while reserving real-game validation for an operator using a published release.

## User Scenarios & Testing

### User Story 1 - Trust the Complete Guided Workflow (Priority: P1)

An operator can select one target and rely on fragcap to guide registration, topology, cold-state preparation, exact calibration attempts, pauses, resume, coverage, and the ordinary Deep Capture handoff without first composing the internal case matrix.

**Why this priority**: S140 through S146 delivered the workflow incrementally. The parent outcome needs one durable proof that every promised behavior remains present together.

**Independent Test**: Run the controlled command and store suites against direct, Steam, and publisher fixtures, then validate that every parent acceptance criterion maps to at least one executed, non-ignored test.

**Acceptance Scenarios**:

1. **Given** a supported stored or discoverable target, **when** guided calibration starts from one target selector, **then** defaults and exact current state determine the next bounded action without requiring low-level case flags.
2. **Given** ambiguity, warm state, incomplete evidence, or an operator action, **when** the workflow cannot safely advance, **then** it records an exact resumable state and performs no unapproved next effect.
3. **Given** sufficient current positive facts, **when** the workflow completes, **then** human and structured output reconcile coverage and provide the exact ordinary Deep Capture command.

---

### User Story 2 - Audit Acceptance as Executable Evidence (Priority: P2)

A maintainer can inspect one versioned acceptance registry that maps every criterion in issue #380 to exact automated tests and can run one repository command that rejects missing, ignored, duplicate, malformed, or stale evidence references.

**Why this priority**: A prose claim can drift after the parent issue closes. A checked registry keeps the implementation and its completion evidence coupled.

**Independent Test**: Mutate a controlled copy of the registry to remove a criterion, reference a missing or ignored function, duplicate an identity, or name an untracked path, and verify that validation fails with the exact defect.

**Acceptance Scenarios**:

1. **Given** the canonical registry, **when** the acceptance gate runs, **then** all thirteen parent criteria are present exactly once and every evidence reference resolves to a tracked test function.
2. **Given** a missing, ignored, duplicated, malformed, or stale reference, **when** the gate runs, **then** it fails rather than accepting documentary intent as proof.
3. **Given** the full repository gate, **when** tests pass, **then** registry validation and the referenced controlled tests are both part of the same CI result.

---

### User Story 3 - Preserve the Released-Build Validation Boundary (Priority: P3)

An operator can distinguish automated implementation acceptance from optional real-world compatibility evidence. fragcap does not launch a real game, mutate the real trust store, or claim live compatibility as part of this work slice.

**Why this priority**: The product is security-sensitive, and the operator has explicitly reserved live testing for a published build under their control.

**Independent Test**: Validate that the acceptance registry permits only controlled automated evidence for implementation completion and that project documentation describes real-game evidence as post-publication, operator-owned compatibility validation.

**Acceptance Scenarios**:

1. **Given** S147 implementation acceptance, **when** the parent issue closes, **then** the record states that no real-game compatibility was demonstrated.
2. **Given** a future published release, **when** an operator elects to test a real game, **then** the scrubbed result is release compatibility evidence and does not retroactively alter the implementation gate.
3. **Given** automated acceptance, **when** the gate runs, **then** it uses controlled targets and synthetic traffic without a real account, remote service, capture driver, or real trust-store mutation.

### Edge Cases

- One criterion may require several tests because topology, output mode, and refusal behavior are separate authorities.
- One test may support several criteria, but each registry entry states the precise proposition it proves.
- A source file or function name that exists only in prose is not executable evidence.
- An ignored test, commented-out function, helper, or unit test without the test attribute is not accepted.
- Windows-only behavior may be referenced only where the ordinary CI suite executes it on Windows or where the existing controlled Windows matrix owns it.
- Historical slice specifications are supporting design records, never substitutes for current executable evidence.
- Controlled fixtures may model login, update, EULA, anti-cheat, no-traffic, gameplay, trust refusal, interruption, and cleanup without interacting with a real application.
- Closing issue #380 means its shipped orchestration contract is implemented and regression-protected. It does not mean any named commercial game has been certified.

## Requirements

### Functional Requirements

- **FR-001**: The repository MUST contain one versioned guided-calibration acceptance registry with exactly the thirteen criteria from issue #380 represented by stable identifiers.
- **FR-002**: Every criterion MUST carry a concise statement, a controlled-evidence classification, and one or more exact test references stating what each reference proves.
- **FR-003**: Registry validation MUST reject an unsupported schema version, missing required fields, duplicate criterion identifiers, incomplete criterion inventory, empty evidence, duplicate evidence references, untracked paths, missing test functions, and ignored tests.
- **FR-004**: Registry validation MUST distinguish automated implementation evidence from released-build real-game compatibility evidence.
- **FR-005**: The implementation acceptance set MUST contain only automated controlled evidence and MUST require zero real games, game accounts, remote services, capture drivers, real trust-store mutations, process termination, or sensitive live capture.
- **FR-006**: The complete repository CI command MUST execute the registry validator and every referenced test through the ordinary test suite.
- **FR-007**: Executable evidence MUST cover the one-target front door, direct/Steam/publisher topology, explicit ambiguity, safe defaults and exact overrides, reachability-first ordering, full authorization-plan visibility, operator-owned warm retry, actionable pause reasons, exact append-only fact history, completion handoff, selective retest, durable human/JSON resume, trust refusal, interruption, cleanup, and successful handoff.
- **FR-008**: Controlled tests MUST close any acceptance proposition that currently relies only on indirect or documentary evidence.
- **FR-009**: An ambiguous non-Steam stored target with two or more client-only launch entries MUST require one stable content-derived executable choice, a separate complete confirmation plan, fresh target reproduction, and a complete-row conditional update before proposal evaluation continues; Steam and publisher chains MUST remain outside this rewrite path.
- **FR-010**: The durable pause vocabulary MUST include update and anti-cheat as no-effect operator actions, and schema version 13 MUST migrate every version 12 workflow without changing its checkpoint authority.
- **FR-011**: Completion output MUST remain concise, coverage-reconciling, and paste-ready in both human and structured modes.
- **FR-012**: No S147 change MAY weaken authorization, target ownership, topology validation, append-only fact authority, cleanup, or effect boundaries established by S121 and S133 through S146.
- **FR-013**: The testing strategy MUST state that real-game validation is operator-owned and performed only against a published release when the operator chooses, and that its absence does not block controlled implementation work.
- **FR-014**: Project documentation MUST state explicitly that S147 did not demonstrate live compatibility with a real game.
- **FR-015**: Architecture, outline, slice ordering, agent narrative, and changelog records MUST describe the acceptance boundary without claiming general Deep Capture completion.
- **FR-016**: S147 MUST add no network behavior, system proxy configuration, automatic process control, pinning bypass, or new sensitive effect.

### Key Entities

- **Acceptance Criterion**: One stable parent requirement with a closed evidence classification and one or more executable proofs.
- **Evidence Reference**: A tracked source path, exact non-ignored test function, and bounded proposition proven by that test.
- **Implementation Acceptance**: The automated decision that the guided orchestration contract is implemented and regression-protected.
- **Released-Build Compatibility Evidence**: Optional operator-owned observation of a real game using published product bytes, kept separate from implementation acceptance.

## Success Criteria

### Measurable Outcomes

- **SC-001**: The canonical registry contains thirteen unique criteria and every criterion resolves to at least one tracked, executable, non-ignored automated test.
- **SC-002**: Validator self-tests reject every malformed, missing, duplicate, ignored, untracked, and stale-reference case named in FR-003.
- **SC-003**: Controlled evidence covers all three launch topologies and all required warm, ambiguity, partial, trust-refusal, interruption, cleanup, and completion states.
- **SC-004**: The repository gate passes without launching a real game, contacting a game service, mutating the real trust store, or running sensitive live capture.
- **SC-005**: Documentation and machine-readable acceptance evidence both state that live real-game compatibility was not demonstrated by S147.
- **SC-006**: S147 closes issue #380 without a new dependency, lockfile package, artifact-schema change, network path, or general Deep Capture completion claim; the sole product-storage change is the additive version 12 to 13 pause-vocabulary migration.

## Assumptions

- S140 through S146 are merged and constitute the implementation under acceptance.
- Existing controlled authorization adapters and synthetic protocol paths are the correct implementation evidence seams.
- `cargo xtask ci` remains the repository's complete local gate and executes all Rust tests.
- The operator may choose to gather real-game evidence after a release, but that work is outside this slice and outside automated CI.

## Clarifications

### Session 2026-09-11

- Q: Must a real game be run before S147 can complete? A: No. The operator explicitly requires real-game testing to use a new published release and forbids the agent from running the sensitive software.
- Q: Does closing #380 certify compatibility with a real Steam, direct, or publisher-launched game? A: No. It certifies the controlled implementation contract only; live compatibility remains unverified.
- Q: Can automated tests use the production orchestration path? A: Yes, through controlled authorities, synthetic traffic, temporary stores, and effect-recording adapters that do not touch the real environment.
- Q: What happens if the audit finds a behavioral gap? A: S147 adds the smallest traceable controlled test and implementation correction required by a parent criterion.
- Q: Does this redefine the entire release gate? A: No. It corrects the timing and ownership of real-game compatibility validation while leaving separate release, packaging, and Deep Capture completion gates intact.
