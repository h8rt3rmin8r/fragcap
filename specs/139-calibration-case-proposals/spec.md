# Feature Specification: Guided Calibration Case Proposals

**Feature Branch**: `codex/s139-calibration-case-proposals`

**Created**: 2026-09-09

**Status**: Complete

**Input**: User description: "Implement S139 as the read-only calibration case-discovery and proposal slice beneath issue #380 under spec-kit autopilot."

## Overview

S139 gives the future guided calibration workflow one deterministic, read-only authority for deciding what can be measured next. Given one resolved stored target, a complete current process-image snapshot, exact native version context, existing append-only compatibility facts, and any observed protocol candidates, it identifies the declared Steam, direct-executable, or publisher topology, explains warm or ambiguous state, selects conservative defaults, and proposes only the next useful exact calibration attempts. It applies no launch, trust, artifact, fact, or storage effect.

## User Scenarios & Testing

### User Story 1 - Discover One Safe Launch Case (Priority: P1)

An authorized operator selects a stored target and receives one exact supported cold launch case or a precise explanation of the action or choice required before calibration can begin.

**Why this priority**: Every later calibration decision depends on knowing which declared process chain can be owned without guessing or widening target scope.

**Independent Test**: Synthetic stored targets and process snapshots cover Steam, direct-executable, publisher, warm, incomplete, unresolved, and ambiguous inputs without starting a process or reading machine state.

**Acceptance Scenarios**:

1. **Given** a stored target with one unambiguous supported topology and a complete snapshot in which every required image is absent, **When** a proposal is requested, **Then** the result names the exact cold launch case and declared image chain.
2. **Given** any required platform, launcher, intermediate, or client image is present, **When** a proposal is requested, **Then** the result names the exact warm case, cold counterpart, complete declared image set, and operator-owned normal-shutdown action.
3. **Given** missing, malformed, conflicting, or multiply plausible launch declarations, **When** a proposal is requested, **Then** the result preserves the candidates or limitation and never selects a launch case silently.

---

### User Story 2 - Propose Only Useful Exact Measurements (Priority: P1)

An operator sees the minimum next calibration work for the selected exact case, with reachability before any trust-bearing protocol attempt and with every retest explained from retained evidence.

**Why this priority**: Repeating current evidence wastes operator time, while skipping missing or uncertain reachability can offer a trust-bearing action without a proven route.

**Independent Test**: Fact histories independently vary missing, stale, legacy-incomplete, mismatched, negative, conflicting, and current positive rows for routing and protocol inspectability, and the proposed sequence remains exact and deterministic.

**Acceptance Scenarios**:

1. **Given** no current exact positive final-client routing fact, **When** work is proposed, **Then** exactly one reachability attempt is first and protocol attempts remain explicitly deferred.
2. **Given** current exact positive final-client routing evidence and observed protocol candidates, **When** work is proposed, **Then** only protocols lacking one unconflicted current exact positive inspectability fact receive a trust-bearing attempt.
3. **Given** stale, legacy-incomplete, mismatched, negative, or conflicting evidence, **When** a retest is proposed, **Then** the proposal names that exact reason without altering or aggregating the retained rows.
4. **Given** current exact positive evidence with no conflict, **When** the same case is proposed again, **Then** redundant work is omitted.

---

### User Story 3 - Keep Defaults and Overrides Explainable (Priority: P2)

An advanced caller can inspect the conservative default dimensions or supply an exact supported routing and loopback-family override before asking for a proposal.

**Why this priority**: A guided front door needs safe defaults, while diagnostic and automation callers must retain exact case control without a parallel policy path.

**Independent Test**: Default and override permutations produce the same complete case identity and stable ordering, while unsupported routing, address-family, and protocol inputs are refused before any proposal step is emitted.

**Acceptance Scenarios**:

1. **Given** no override, **When** a proposal is requested, **Then** it selects child-environment routing and IPv4 and explains both as conservative compatibility defaults.
2. **Given** an exact supported override, **When** a proposal is requested, **Then** every applicability decision and attempt uses the overridden dimension without fallback.
3. **Given** an unsupported or inapplicable protocol candidate, **When** a proposal is requested, **Then** it is reported as a limitation and is not converted into a runnable attempt.

### Edge Cases

- The process inventory is unavailable or incomplete, so absence cannot prove cold state.
- A Steam anchor is malformed, or a Steam target has no exact client declaration or several client declarations.
- A publisher declaration has one non-client role, lacks a terminal client, repeats an image under conflicting roles, or declares several plausible clients.
- Process image comparison differs only by case or repeats an image in the snapshot.
- Fact rows have no durable identifier, identical timestamps, conflicting values, or an explicit stale source.
- A protocol candidate list is empty, repeated, unordered, includes routing, or includes explicit inapplicability.
- Target version is newly available or unavailable, making otherwise similar facts legacy-incomplete or mismatched.

## Requirements

### Functional Requirements

- **FR-001**: The system MUST derive one declared topology from the existing stored target form and MUST distinguish Steam, direct-executable, publisher-chain, ambiguous, and unavailable topologies without reading or changing external state.
- **FR-002**: A Steam topology MUST require one exact declared client and MUST include both the platform image and client image in warm-state evaluation.
- **FR-003**: A direct topology MUST require one exact declared Windows client and MUST preserve all candidates when more than one is plausible.
- **FR-004**: A publisher topology MUST preserve declared order and roles, require one terminal client, and reject incomplete or conflicting chains rather than repairing them by inference.
- **FR-005**: Cold state MUST be concluded only from a complete process snapshot in which every image required by the selected topology is absent.
- **FR-006**: A warm result MUST name the observed warm case, exact cold counterpart, every declared image to close normally, and MUST NOT authorize or perform process control.
- **FR-007**: Proposal generation MUST default to child-environment routing and IPv4 when no exact override is supplied, and MUST never silently fall back from a supplied dimension.
- **FR-008**: Every proposed attempt MUST carry the exact launch case, routing strategy, address family, protocol, native backend name and version, fragcap version, and target version when available.
- **FR-009**: Reachability MUST be the only runnable proposal until one unconflicted current exact `reached-client` routing fact exists for the selected case.
- **FR-010**: Protocol attempts MUST be limited to caller-supplied observed protocol candidates from the shipped closed protocol set and MUST be ordered deterministically.
- **FR-011**: A current exact positive routing or inspectability fact MUST suppress the equivalent redundant attempt only when no current exact conflict exists.
- **FR-012**: Missing, stale, legacy-incomplete, context-mismatched, negative, and conflicting evidence MUST remain distinct proposal reasons.
- **FR-013**: Existing facts MUST remain individual append-only observations; proposal generation MUST NOT update, delete, append, aggregate, or select a title-wide verdict from them.
- **FR-014**: Unsupported topology, incomplete process inventory, invalid version context, and invalid protocol candidates MUST produce typed limitations with zero runnable attempts.
- **FR-015**: Proposal output MUST be deterministic for equivalent inputs, including case-insensitive image comparison, duplicate inputs, fact order, and protocol-candidate order.
- **FR-016**: The complete proposal operation MUST be offline-testable without a game, capture driver, elevation, network listener, certificate trust action, artifact creation, or local target database mutation.
- **FR-017**: S139 MUST NOT add the guided CLI command, resolve or register targets, execute calibration, mutate trust, append compatibility facts, create session artifacts, change ordinary Deep Capture eligibility, or close parent issue #380.

### Key Entities

- **Calibration Proposal Request**: One resolved target, complete-or-unavailable process inventory, exact native version context, optional exact routing and family overrides, observed protocol candidates, and retained compatibility facts.
- **Declared Calibration Topology**: The supported Steam, direct, or publisher launch shape and its ordered image identities, or a preserved ambiguity or limitation.
- **Launch Readiness**: The exact cold case, an operator-owned warm-to-cold action, or a pre-effect limitation.
- **Calibration Proposal Step**: One exact reachability or protocol attempt plus the evidence reason that makes it useful.
- **Deferred Protocol**: An observed protocol candidate that cannot yet become a runnable attempt because exact final-client routing has not been established.

## Success Criteria

### Measurable Outcomes

- **SC-001**: One hundred percent of controlled Steam, direct, publisher, warm, ambiguous, unavailable-snapshot, and malformed-topology cases produce their specified typed result with zero external effects.
- **SC-002**: Reachability precedes trust-bearing protocol work in 100 percent of evidence permutations, and zero protocol attempts are runnable while exact routing is missing, stale, legacy, mismatched, negative, or conflicting.
- **SC-003**: Changing any applicable case dimension prevents an otherwise positive fact from suppressing work in 100 percent of single-dimension permutations.
- **SC-004**: Equivalent requests with permuted facts, protocols, process-image case, or duplicate inputs produce identical proposal output in 100 percent of controlled permutations.
- **SC-005**: Every proposed retest carries exactly one stable reason from the specified six-state reason set, and current unconflicted positive evidence produces no redundant step.
- **SC-006**: The repository verification suite passes with no new dependency, lockfile package, storage migration, process-control capability, trust effect, or CLI behavior change.

## Assumptions

- The resolved stored target is supplied by the existing target-selection authority; target resolution and registration remain later #380 work.
- The caller supplies a complete query-only image snapshot or explicitly marks it unavailable. The proposal engine does not enumerate processes itself.
- The existing cold Steam, direct-executable, and publisher-chain launch cases are the only supported real-target topologies.
- Child-environment routing and IPv4 are the conservative defaults because both are already shipped and IPv4 remains the compatibility default.
- Protocol candidates come from retained S120 classifications or an exact advanced caller selection. Metadata does not imply traffic use.
- Existing S121 fact applicability and append-only chronology remain authoritative and are consumed rather than replaced.

## Clarifications

### Session 2026-09-09

- Q: Does S139 add the public guided command? -> A: No. It provides the pure proposal authority that the later command will orchestrate.
- Q: What proves cold state? -> A: Only a complete caller-supplied snapshot with every declared topology image absent.
- Q: Which protocol cases are proposed automatically? -> A: Only observed closed-set candidates supplied by the caller; target metadata never invents traffic use.
- Q: When does existing evidence suppress a proposal? -> A: Only when the latest exact evidence is positive and current exact facts do not conflict.
- Q: How are safe defaults selected? -> A: Child-environment routing and IPv4, with exact caller overrides and no fallback.
