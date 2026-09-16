# Feature Specification: S154 regression reliability and native documentation readiness

**Feature Branch**: `codex/s154-regression-documentation`\
**Created**: 2026-09-16 UTC\
**Status**: Draft\
**Input**: Approved S154, deterministic sink backpressure regression (#420) and bounded agent-runnable documentation readiness under #331. Push and PR creation are explicitly authorized; final merge belongs to the owner.

## User Scenarios & Testing

### User Story 1 - Trust stalled-consumer evidence (Priority: P1)

A contributor needs a repeatable regression proving that a stalled streaming consumer cannot block capture or damage an independent file sink.

**Why this priority**: #420 exposed a test whose required positive drop depends on operating-system TCP buffering and scheduling.

**Independent Test**: Exercise a deliberately blocked packet writer, bounded consumer queue and independent file output without a capture driver or installed product.

**Acceptance Scenarios**:

1. **Given** an acknowledged stalled packet writer, **When** its finite queue saturates, **Then** capture submission remains bounded, consumer-local drops are positive and the independent file contains every packet in order.
2. **Given** an accepting real TCP writer, **When** its kernel buffers accept all offered packets, **Then** reports remain honest without falsely requiring positive drops.
3. **Given** timeout or finish, **When** the consumer is terminated, **Then** every test-owned wait is finite and cleanup releases the writer.

### User Story 2 - Read one verified native contract (Priority: P2)

An operator needs current native documentation whose coverage, commands and references are mechanically checked against the published contract.

**Why this priority**: #331 includes bounded engineering work that can progress without claiming independent security acceptance.

**Independent Test**: Validate a closed topic inventory, existing command examples and production documentation site without running sensitive software on the owner's host.

**Acceptance Scenarios**:

1. **Given** the native documentation inventory, **When** it is checked, **Then** architecture, setup, CLI, protocols and routing, refusals, artifacts and correlation, diagnostics and recovery, security and privacy, bounds, packaging and migration, and public API each have an authored reference and executable authority.
2. **Given** a missing topic, duplicate topic or invalid reference, **When** validation runs, **Then** it fails with a specific diagnostic.
3. **Given** engineering readiness, **When** its status is reported, **Then** independent #333 review reconciliation and the final #334/#278 completion gate remain explicitly outstanding.

### Edge Cases

- Registration or blocked-write acknowledgement never arrives: fail within a finite deadline.
- The TCP kernel accepts the offered workload: this is not proof of an accounting bug.
- A consumer is terminated with unwritten work: preserve existing report semantics rather than inventing successful writes.
- A topic points to a historical record instead of a current contract: reject invalid authority and document historical distinctions.
- Candidate documentation changes are not yet released: do not claim they shipped in v0.10.1.

## Requirements

### Functional Requirements

- **FR-001**: Positive stalled-consumer loss MUST be exercised through deliberate writer backpressure, independent of TCP kernel capacity.
- **FR-002**: Controlled saturation MUST preserve the existing five-second capture-submission bound, complete ordered independent file output and positive consumer-local drops.
- **FR-003**: Offered, written, dropped and terminal outcomes MUST retain existing truthful semantics; registration, acknowledgement and cleanup MUST be bounded.
- **FR-004**: Real TCP join, trailer, multi-consumer and closed-consumer coverage MUST remain active. Positive-drop coverage MUST remain enforced in the controlled regression, without ignored tests, product timeout expansion or blind retries.
- **FR-005**: Documentation MUST have a closed traceable inventory of the topics in User Story 2, each with a current authored page and existing executable authority. Missing, duplicate and invalid references MUST fail.
- **FR-006**: Included command examples and applicable artifact examples MUST pass existing validators, and the production site MUST pass its existing tests without unexpected skips.
- **FR-007**: Published v0.10.1 source a7d24962999d38d7ff130722859d473543864862 MUST remain distinct from this unreleased PR. Owner administrative bypass, consent, trust, route scope and independent acceptance MUST remain unchanged.
- **FR-008**: #420 and a bounded #331 documentation child MUST individually trace the slice. The parent MUST remain open. Every received review MUST be addressed within the two-round maximum and final-head CI MUST be green before owner handoff.

### Key Entities

- **Consumer evidence**: offered packets, successful writes, local queue drops, terminal reason and independent file contents.
- **Documentation coverage row**: stable topic, current authored page and executable contract authority.
- **Release identity**: immutable published source separate from candidate documentation and engineering readiness.

## Success Criteria

### Measurable Outcomes

- **SC-001**: At least 20 fresh controlled isolation scenarios pass on each supported Linux and Windows CI platform, without retry, assertion relaxation or skip; each preserves ordered full file output and honest counters.
- **SC-002**: Every required documentation topic has valid authority; missing, duplicate and invalid rows produce specific failures.
- **SC-003**: Command and applicable artifact examples and production site tests pass with no unexpected skips or false independent-acceptance claims.
- **SC-004**: All final-head required checks are green and every actual review is reconciled with no more than one requested second round.

## Assumptions and Scope

The existing streaming test seam and documentation validators are reused. This slice changes no product runtime, dependency, storage schema, version, release tag or publication. Only controlled synthetic tests and static/build checks run on the owner host.

The historical #420 failure and successful same-source retry establish test nondeterminism, not exact kernel occupancy. Independent installed review/retest under #333/#413 remains separate. #331, #334 and #278 remain incomplete; optional owner field measurements under #372 and deferred #155/#94 are not slice gates.

Specification, clarification, checklist, plan, tasks and blocking analysis precede implementation. Pinned documentation edits receive a dated decisions fragment. Explicit push/PR authorization supersedes only the autopilot's default pre-push pause.
