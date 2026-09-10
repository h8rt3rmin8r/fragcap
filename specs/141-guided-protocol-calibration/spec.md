# Feature Specification: Guided Protocol Calibration Attempt

**Feature Branch**: `codex/s141-guided-protocol-calibration`

**Created**: 2026-09-10

**Status**: Complete

**Input**: User description: "Implement S141 as the next guided protocol calibration slice beneath issue #380 under spec-kit autopilot."

## Overview

S141 extends the registered-target `fragcap calibrate <TARGET>` front door from S140 through one exact protocol calibration attempt per invocation. A current exact positive reachability fact remains a prerequisite. Candidate protocols come only from final-client traffic classifications observed by a completed authorized session or from an explicit advanced measurement request, and a request is never itself evidence. The S139 proposal remains the sole sequencing authority, the existing Deep Capture coordinator remains the sole effect authority, and only directly observed results enter the append-only compatibility store.

## Clarifications

### Session 2026-09-10

- Q: How many protocol sessions may one S141 invocation start? -> A: At most one; any remaining candidate is carried in the next paste-ready command and re-evaluated in a new process.
- Q: Which observations may become automatic protocol candidates? -> A: Only concrete S120 classifications attributed to the final client in the completed session; unknown, unrouted, launcher, and intermediate observations remain visible but cannot authorize a candidate.
- Q: Does an operator-supplied `--protocol` value prove support? -> A: No; it requests one bounded measurement, while only the resulting direct observation may become compatibility evidence.
- Q: What happens when routing is current but no protocol candidate is available? -> A: Start no effect, report protocol coverage as unknown, and preserve a paste-ready ordinary Deep Capture command because routing readiness and protocol calibration coverage are separate facts.
- Q: Is candidate progress persisted? -> A: No; a bounded deduplicated candidate list is carried in the exact generated continuation command, while durable evidence remains exclusively in the existing append-only fact store.

## User Scenarios & Testing

### User Story 1 - Run One Exact Protocol Attempt (Priority: P1)

An authorized operator evaluates a registered cold target with current exact reachability evidence and one or more bounded protocol candidates. fragcap asks S139 for the exact next step, presents the complete authorization plan, and runs only the first useful protocol attempt.

**Why this priority**: This is the smallest safe step beyond reachability that reduces manual low-level calibration while retaining explicit trust, launch, artifact, deadline, cleanup, and evidence boundaries.

**Independent Test**: Use a controlled registered target with current exact positive routing evidence and a concrete candidate, invoke the guided command with exact authorization, and observe one protocol session whose direct result is appended under the proposed case.

**Acceptance Scenarios**:

1. **Given** a registered cold target with current exact positive routing evidence and one candidate lacking a current positive protocol fact, **When** the operator authorizes the complete plan, **Then** exactly one S139-selected protocol attempt runs through the existing bounded Deep Capture session.
2. **Given** several candidates, **When** S139 proposes several protocol steps, **Then** only the first deterministic useful step runs and the next command retains every still-unresolved candidate.
3. **Given** the operator declines or returns the wrong plan identifier, **When** authorization completes, **Then** no trust, proxy, routing, launch, Capture, artifact, or fact effect begins.

---

### User Story 2 - Carry Only Observed Candidates Forward (Priority: P2)

After a successful reachability session, the operator receives a continuation command containing only concrete protocol candidates derived from final-client observations produced by that same completed session.

**Why this priority**: Automatic guidance must be useful without turning guesses, configuration, or unrelated launcher traffic into evidence.

**Independent Test**: Feed the controlled session a mixture of final-client concrete classifications, unknown traffic, launcher traffic, and duplicates, then verify that the continuation is stable, bounded, deduplicated, and contains only eligible final-client protocols.

**Acceptance Scenarios**:

1. **Given** a completed reachability session with eligible final-client observations, **When** guided completion is reported, **Then** the next command carries the sorted deduplicated protocol candidates without persisting workflow state.
2. **Given** observations attributed only to launchers, intermediate processes, unknown traffic, or unrouted traffic, **When** completion is reported, **Then** none becomes a candidate or compatibility fact.
3. **Given** an explicit protocol candidate, **When** no matching direct observation occurs, **Then** the request remains distinguishable from evidence and no positive fact is invented.

---

### User Story 3 - Report Coverage and Continue Safely (Priority: P3)

An operator or local automation receives stable human and JSON guidance describing completed, refused, unavailable, and remaining protocol coverage, plus the exact next command when one exists.

**Why this priority**: One-attempt execution is usable only when the boundary between route readiness, observed coverage, and remaining work stays explicit.

**Independent Test**: Exercise ready-without-candidates, already-positive, limitation, authorization refusal, successful attempt, and non-observation outcomes in human and JSON modes and compare stable fields and effect counts.

**Acceptance Scenarios**:

1. **Given** current routing evidence and no eligible candidate, **When** the command evaluates the target, **Then** it starts no effect, reports protocol coverage unknown, and prints the ordinary Deep Capture command without claiming protocol calibration completion.
2. **Given** every supplied candidate already has a current exact positive fact, **When** evaluation completes, **Then** it starts no effect and reports those candidates complete.
3. **Given** a proposal limitation or a changed target, launch, route, or process authority, **When** evaluation completes, **Then** it refuses before effects and reports the typed reason.
4. **Given** JSON mode, **When** any guided outcome occurs, **Then** stable fields distinguish requested candidates, observed candidates, completed candidates, remaining candidates, action, status, reason, and next command while retaining existing detailed Deep Capture events.

### Edge Cases

- A missing, stale, legacy, mismatched, negative, or conflicting exact routing fact still selects reachability before any protocol attempt.
- A warm target remains effect-free unless `--restart-warm` explicitly selects the existing operator-owned close-and-retry workflow.
- Repeated, differently ordered, or mixed-case candidate input normalizes to one deterministic closed-set sequence.
- Unsupported protocol text is rejected by argument parsing before any repository, process, or session effect.
- A session that completes without an eligible final-client observation must not mark the requested protocol positive.
- A protocol observed after the candidate command was created may suppress the now-unnecessary attempt when the next invocation reads current exact facts.
- A candidate requiring unavailable routing, trust, launch, family, or protocol authority remains a typed proposal limitation and cannot fall back to a weaker path.
- Generated commands use the durable stable target identifier and preserve the PowerShell-quoted effective local-store path.
- Existing low-level `deep-capture --calibrate` invocations remain unchanged.

## Requirements

### Functional Requirements

- **FR-001**: The top-level `calibrate` command MUST accept a repeatable bounded protocol candidate option drawn from the existing closed calibration protocol vocabulary.
- **FR-002**: Candidate input MUST be normalized into a deterministic deduplicated sequence before proposal construction, and unsupported text MUST fail before effects.
- **FR-003**: The command MUST continue to use the existing stored-target resolver, query-only process inventory, exact compatibility facts, native backend identity, target version, child-environment route, IPv4 family, and S139 proposal authority.
- **FR-004**: A protocol attempt MUST require current exact positive routing evidence for the same target, launch case, route, address family, backend, backend version, fragcap version, and target version applicability dimensions.
- **FR-005**: Missing or non-positive routing evidence MUST continue to select at most one reachability attempt before any protocol attempt.
- **FR-006**: With current exact positive routing evidence, the command MUST execute at most the first deterministic useful S139 protocol step in one invocation.
- **FR-007**: The selected step MUST execute through the existing low-level plan-bound Deep Capture calibration path, including explicit exact-plan authorization, target-authority recheck, visible session trust, bounded artifacts, managed launch, deadlines, cleanup, and append-only direct fact behavior.
- **FR-008**: A protocol candidate supplied by the operator or a generated command MUST remain a measurement request and MUST NOT itself create, upgrade, or imply a compatibility fact.
- **FR-009**: The guided layer MUST derive automatic candidates only from concrete S120 classifications attached to observations owned by the final client of the completed authorized session.
- **FR-010**: Unknown, unrouted, launcher, intermediate, ambiguous, unavailable, and uncorrelated observations MUST NOT become automatic candidates.
- **FR-011**: Automatic candidate derivation MUST be bounded, deterministic, deduplicated, and based on the completed session result rather than a second capture, heuristic scan, target metadata, or persisted workflow record.
- **FR-012**: After any executed attempt, the command MUST re-read current exact facts, re-evaluate the target through S139, and report which requested candidates are complete, remaining, refused, or unavailable.
- **FR-013**: When unresolved candidates remain, the next command MUST use the durable target identifier, effective local-store path, required warm-restart selection if applicable, and the bounded remaining candidate list.
- **FR-014**: When routing is current and no eligible candidate is available, the command MUST start no effect, report protocol coverage unknown, and retain a paste-ready ordinary Deep Capture command without claiming full protocol coverage.
- **FR-015**: When all supplied candidates have current exact positive facts, the command MUST start no effect and report those candidates complete without claiming coverage of protocols that were never supplied or observed.
- **FR-016**: Every proposal limitation, authorization refusal, authority change, session failure, cleanup failure, or missing direct observation MUST retain its typed non-success outcome and MUST NOT silently downgrade or manufacture positive evidence.
- **FR-017**: Human and JSON modes MUST expose stable requested, observed, completed, remaining, action, status, reason, and next-command facts without suppressing existing detailed Deep Capture events.
- **FR-018**: JSON execution MUST retain the existing same-process `--authorize-stdin` exact-plan-identifier requirement, and no blanket confirmation flag may be introduced.
- **FR-019**: The existing low-level `deep-capture --calibrate`, ordinary Deep Capture, Capture, target storage schema, fact applicability, and public Rust API contracts MUST remain backward compatible except for a narrowly additive facade result needed to return observed candidates to the CLI.
- **FR-020**: Offline and controlled tests MUST cover direct, Steam, and publisher targets; reachability precedence; one-attempt selection; candidate normalization; final-client filtering; no-candidate coverage; already-positive candidates; warm guidance and retry; every limitation; authorization refusal; successful direct evidence; and missing observation without a live game, elevation, hidden trust, system proxy, prohibited process access, or target key extraction.
- **FR-021**: S141 MUST NOT add automatic registration, an internal multi-attempt loop, persisted calibration workflow state, interruption resume, a storage migration, a dependency, hidden or system-wide trust, certificate-pinning bypass, target process access, target TLS key extraction, or a Deep Capture completion claim, and MUST leave parent issue #380 open.

### Key Entities

- **Protocol candidate set**: A bounded deterministic list of concrete protocol dimensions requested for measurement, not a claim of support.
- **Observed protocol candidate**: A concrete S120 protocol classification from a final-client observation in one completed authorized session.
- **Guided protocol decision**: The S139 proposal projected into one selected action, exact status and reason, requested and completed coverage, remaining candidates, and next command.
- **Protocol attempt result**: One low-level calibration session outcome plus the eligible candidates directly observed during that session and the exact facts appended by existing authorities.
- **Continuation command**: A paste-ready durable-identifier invocation carrying only unresolved bounded candidates and the effective local-store path, with no durable workflow state outside the fact store.

## Success Criteria

### Measurable Outcomes

- **SC-001**: A registered cold target with current routing evidence and one unresolved candidate reaches one complete protocol authorization plan from one top-level command and runs no more than one session.
- **SC-002**: A completed reachability session produces a deterministic continuation containing every and only eligible final-client concrete protocol candidate observed in that session.
- **SC-003**: Requested, observed, and positively evidenced protocol sets remain distinguishable in every tested human and JSON outcome.
- **SC-004**: Every no-effect or refused state makes zero new session effect-adapter calls and reports one stable reason and next action.
- **SC-005**: Every generated continuation parses successfully, selects the same durable target from the same effective local store, and preserves the bounded unresolved candidate set.
- **SC-006**: The complete repository verification gate passes with no new dependency, lockfile package, storage migration, prohibited capability, or change to existing low-level calibration behavior.

## Assumptions

- S139 remains the accepted pure proposal and ordering authority.
- S140 remains the accepted registered-target front door and reachability authority.
- S120 classification, the existing session observation model, final-client launch binding, S121 fact persistence, and S134 authorization plan remain authoritative for their respective facts.
- Ordinary Deep Capture readiness depends on exact routing evidence, while protocol calibration coverage is supplemental and must be reported separately.
- Automatic registration, internal multi-attempt execution, persistent workflow state, resume, and final coverage remain later work under #380.
