# Feature Specification: Guided Reachability Calibration Front Door

**Feature Branch**: `codex/s140-guided-calibration-front-door`

**Created**: 2026-09-09

**Status**: Complete

**Input**: User description: "Implement S140 as the registered-target guided reachability calibration front door beneath issue #380 under spec-kit autopilot."

## Overview

S140 gives an authorized operator one top-level `fragcap calibrate <TARGET>` entry point for an already registered target. The command resolves the target through the existing shared authority, obtains a complete read-only process image snapshot and exact retained compatibility facts, consumes the S139 proposal, and either runs the first required reachability measurement through the existing plan-bound Deep Capture executor or reports the exact next action. It does not register targets, select TLS protocols, run multiple attempts, or persist workflow state.

## Clarifications

### Session 2026-09-09

- Q: How much calibration execution belongs in S140? -> A: At most one exact reachability attempt; TLS and protocol sequencing remain later #380 work.
- Q: What happens when the selected target is warm? -> A: The default is effect-free guidance; `--restart-warm` explicitly enters the existing operator-owned close-and-retry workflow.
- Q: What happens when current exact routing evidence is already positive? -> A: Apply no calibration effect and print a paste-ready ordinary Deep Capture command.

## User Scenarios & Testing

### User Story 1 - Calibrate a Registered Cold Target (Priority: P1)

An authorized operator names a registered Steam, direct-executable, or declared publisher target without choosing a launch case, calibration phase, routing strategy, loopback family, or protocol. fragcap selects only the exact missing reachability case, presents the complete authorization plan, and runs that one bounded measurement after authorization.

**Why this priority**: This removes the internal case matrix from the normal first-run path while retaining the exact safety and evidence authorities already shipped.

**Independent Test**: Use a controlled registered target with missing routing evidence, invoke the new command, authorize the emitted plan, and observe one reachability session whose directly observed fact is appended under the exact proposed case.

**Acceptance Scenarios**:

1. **Given** a registered cold direct target with no exact routing fact, **When** the operator invokes `fragcap calibrate <TARGET>` and authorizes the complete plan, **Then** exactly one IPv4 child-environment reachability attempt runs through the existing bounded Deep Capture session.
2. **Given** a registered cold Steam or publisher target with non-positive, stale, legacy, mismatched, or conflicting routing evidence, **When** calibration begins, **Then** the exact cold launch case and retest reason come from the S139 proposal and no unsupported case is guessed.
3. **Given** the operator declines or returns the wrong plan identifier, **When** authorization completes, **Then** no proxy, trust, launch, Capture, bundle, or fact effect begins.

---

### User Story 2 - Receive Safe Guidance Without Effects (Priority: P2)

An operator whose target is already ready, currently warm, ambiguous, unavailable, or invalid receives a truthful typed outcome and an exact next command without beginning a session.

**Why this priority**: Expected readiness and operator-action states should not force a failed calibration or hide why no attempt was selected.

**Independent Test**: Supply current exact positive evidence, warm process evidence, and malformed target declarations in separate offline cases and verify each produces the specified guidance with zero effect-adapter calls.

**Acceptance Scenarios**:

1. **Given** current exact positive routing evidence, **When** the command evaluates the target, **Then** it starts no calibration and prints a paste-ready ordinary `fragcap deep-capture` command for the same stable target.
2. **Given** one declared target or launcher image is present, **When** the command evaluates the target without `--restart-warm`, **Then** it names the observed and required cold cases, states that fragcap did not control a process, and prints the exact close-and-retry command.
3. **Given** a warm target and explicit `--restart-warm`, **When** the operator confirms normal shutdown, **Then** the command reuses the existing bounded observation and fresh target-preparation workflow before any calibration plan is offered.
4. **Given** an ambiguous, unavailable, invalid, or unsupported proposal, **When** evaluation completes, **Then** the command reports every typed limitation and returns before any session effect.

---

### User Story 3 - Automate and Continue the Guided Path (Priority: P3)

An operator or local automation receives stable structured proposal, action, outcome, and next-command records while existing detailed Deep Capture events remain intact.

**Why this priority**: The guided surface must be scriptable without inventing a second execution or evidence contract.

**Independent Test**: Invoke no-effect and controlled execution cases in JSON mode, parse every line, and verify stable event names and fields identify the target, topology, readiness, selected case, reason, status, and next command.

**Acceptance Scenarios**:

1. **Given** JSON mode and a no-effect ready or warm outcome, **When** the command completes, **Then** one stable guided-calibration record contains all decision facts and no human prose appears on standard output.
2. **Given** JSON mode and a selected reachability attempt, **When** the caller does not use `--authorize-stdin`, **Then** execution is refused before effects with the existing structured-authorization requirement.
3. **Given** a completed reachability attempt, **When** the command reports its outcome, **Then** the next command is another guided evaluation so newly appended evidence is reassessed before ordinary Deep Capture is recommended.

### Edge Cases

- A selector that is a bare integer retains the existing row-number meaning and never becomes a platform application identifier.
- A stable identifier, handle, exact name, and current row selector must all resolve through the existing target authority and preserve its ambiguity and no-match diagnostics.
- A missing local store or unregistered target must not trigger discovery, registration, or a write.
- Process inventory failure remains an unavailable proposal and must not be treated as a cold target.
- A warm restart interruption, decline, timeout, process-enumeration failure, target drift, or non-cold refreshed state retains the existing distinct terminal outcome.
- A proposal containing zero steps with no current positive routing evidence is a refusal, not a readiness claim.
- A proposal containing a TLS step is outside this slice and must be refused rather than executed.
- Shell-sensitive target names never need quoting in the generated ordinary command because the durable stable identifier form is used.
- Existing low-level `deep-capture --calibrate` invocations and their argument validation remain unchanged.

## Requirements

### Functional Requirements

- **FR-001**: The CLI MUST expose a top-level `calibrate` command accepting one required registered-target selector positionally or through `--target`, or one durable stable identifier through `--id`.
- **FR-002**: The command MUST use the existing shared stored-target resolution authority and MUST NOT add discovery, implicit registration, a second selector interpretation, or a second storage shape.
- **FR-003**: The command MUST obtain a complete current process image inventory using only the permitted query-only snapshot technique and MUST treat inventory failure as unavailable rather than cold.
- **FR-004**: The command MUST load compatibility facts only for the resolved stored target and construct the S139 request with the exact native backend version, fragcap version, child-environment routing, IPv4, no protocol candidates, and the current target version when available.
- **FR-005**: The S139 proposal MUST remain the sole authority for topology, readiness, cold launch case, retest reason, and selected reachability case.
- **FR-006**: A missing, stale, legacy, mismatched, negative, or conflicting exact routing result MUST select at most the first proposed reachability step, and that step MUST use the routing protocol dimension.
- **FR-007**: The selected step MUST execute by adapting into the existing low-level Deep Capture calibration command path, including its complete authorization plan, final target-authority recheck, prepared session authority, managed launch, bounded capture, cleanup, artifact, and append-only fact behavior.
- **FR-008**: Reachability execution MUST never request certificate trust, HAR, TLS key-log, client-identity, TLS, or protocol calibration effects.
- **FR-009**: Current exact positive routing evidence MUST select no calibration effect and MUST produce a paste-ready ordinary Deep Capture command addressed by durable stable identifier.
- **FR-010**: A warm proposal without `--restart-warm` MUST identify the observed case, required cold case, declared images, absence of process control, and a paste-ready rerun using `--restart-warm`.
- **FR-011**: `--restart-warm` MUST reuse the existing operator-confirmed bounded normal-shutdown workflow, fresh target resolution, authority comparison, and cold-case verification; it MUST NOT close, signal, message, or terminate a process.
- **FR-012**: Every proposal limitation MUST be reported before bundle allocation, certificate preparation, proxy startup, trust, routing, launch, Capture, artifact, or fact effects.
- **FR-013**: A zero-step proposal MUST be classified as ready only when current exact positive routing evidence for the proposed case is present; every other zero-step state MUST refuse with a stable reason.
- **FR-014**: Human and JSON modes MUST expose stable proposal, action, outcome, and next-command facts without suppressing existing detailed Deep Capture events.
- **FR-015**: JSON execution MUST retain the existing same-process `--authorize-stdin` exact-plan-identifier requirement, and no blanket confirmation flag may be introduced.
- **FR-016**: A successful reachability attempt MUST recommend another `fragcap calibrate --id <ID>` evaluation so the newly appended fact is read before ordinary Deep Capture is recommended.
- **FR-017**: The existing low-level `deep-capture --calibrate`, ordinary Deep Capture, Capture, target storage, fact schema, and public Rust API contracts MUST remain backward compatible.
- **FR-018**: Offline and controlled tests MUST cover direct, Steam, publisher, current fact, every non-positive retest reason, warm guidance, warm retry, no-match, ambiguity, unavailable inventory, malformed topology, authorization decline, wrong identifier, and successful reachability without a live game, elevation, trust mutation, or prohibited process access.
- **FR-019**: S140 MUST NOT add automatic target registration, TLS or protocol attempt selection, a multi-attempt loop, workflow persistence, interruption resume, a storage migration, a dependency, or a Deep Capture completion claim, and MUST leave parent issue #380 open.

### Key Entities

- **Guided calibration request**: One stored-target selector plus the small set of execution bounds and authorization input that can be passed to the existing reachability path.
- **Guided calibration decision**: The S139 proposal projected into one stable CLI status, selected action, exact reason, and next command.
- **Guided calibration action**: One of run reachability, request operator-owned warm restart, report ready for ordinary Deep Capture, or refuse before effects.
- **Next command**: A paste-ready durable-identifier invocation that re-evaluates calibration or starts ordinary Deep Capture without embedding an unstable name or row index.

## Success Criteria

### Measurable Outcomes

- **SC-001**: A registered cold target with missing routing evidence reaches one complete reachability authorization plan from one target argument and no manual case, phase, routing, family, or protocol input.
- **SC-002**: Every no-effect state makes zero calls to session effect adapters and reports one deterministic action and reason.
- **SC-003**: Every generated next command parses successfully and selects the same durable target identity.
- **SC-004**: All supported target topologies and all S139 routing-evidence reasons produce deterministic human and structured outcomes in offline tests.
- **SC-005**: The complete repository verification gate passes with no new dependency, lockfile package, storage migration, prohibited process capability, or change to existing low-level calibration behavior.

## Assumptions

- S139 is the accepted proposal authority and its conservative child-environment plus IPv4 defaults remain correct for this bounded front door.
- The existing target resolver, process snapshot mechanism, S134 authorization plan, S121 fact persistence, session coordinator, and controlled harness remain the authorities for their respective responsibilities.
- Registered target handles are presentation data; generated commands use durable stable identifiers to avoid shell quoting and identity drift.
- Protocol discovery, TLS measurement, automatic target registration, multi-attempt progress, and resumable workflow state remain later work under #380.
