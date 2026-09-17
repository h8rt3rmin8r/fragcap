# Feature Specification: S156 deterministic package certification

**Feature Branch**: `codex/s156-deterministic-package-certification`\
**Created**: 2026-09-17 UTC\
**Status**: Draft\
**Input**: Correct the nondeterministic package-certification loopback assertion that failed post-S155 main runs 35196728506 and 35196728671. Preserve firewall containment, process ownership, non-loopback rejection and cleanup, replace transient socket presence as positive authority with structured proxy-start and reached-client evidence, version the report, add exhaustive mutation coverage and bounded predicate diagnostics, and never run the installed sensitive product locally.

## Clarifications

### Session 2026-09-17

- No critical ambiguity required operator input. The supplied directive fixes the positive authority, preserved negative authorities, report versioning, diagnostic boundary, hosted execution boundary and completion gate.

## User Scenarios & Testing

### User Story 1 - Certify deterministic controlled reachability (Priority: P1)

A release maintainer needs package certification to pass when the controlled client demonstrably reached the native loopback proxy, even if bounded socket-table polling did not sample the short-lived connection.

**Why this priority**: The current polling requirement produces false failures after merge and undermines both release confidence and independent-review replay.

**Independent Test**: Validate a synthetic certification report whose firewall, process, structured proxy-start, reached-client and cleanup authorities are complete while its diagnostic socket sample is empty.

**Acceptance Scenarios**:

1. **Given** complete firewall containment, exact product process ownership, an exact loopback native proxy-start event, a matching reached-client terminal event and reconciled cleanup, **When** no transient loopback socket is sampled, **Then** controlled smoke certification succeeds and reports the sampling result as diagnostic only.
2. **Given** any observed non-loopback endpoint or unexpected process owner, **When** certification evaluates the smoke, **Then** it fails regardless of structured positive evidence.
3. **Given** missing, malformed, duplicate, contradictory or mismatched proxy-start or reached-client evidence, **When** certification evaluates the smoke, **Then** it fails without falling back to socket presence.

### User Story 2 - Diagnose failed predicates safely (Priority: P1)

A maintainer needs a failed hosted campaign to name the exact violated certification predicate without exposing machine-local or captured values.

**Why this priority**: Runs 35196728506 and 35196728671 reported only one compound failure, so maintainers could not distinguish transient sampling from an actual containment breach.

**Independent Test**: Mutate every authority independently and assert that each failure yields its own bounded stable diagnostic code and no raw path, address, payload or credential.

**Acceptance Scenarios**:

1. **Given** one failed authority, **When** certification stops, **Then** the diagnostic names that predicate using a closed bounded vocabulary.
2. **Given** several failed authorities, **When** certification stops, **Then** every failed predicate is reported once in deterministic order within the report and message bounds.
3. **Given** arbitrary host values in observations, **When** a failure is rendered, **Then** those values are absent from public diagnostics.

### User Story 3 - Consume a versioned certification report (Priority: P2)

A release or review workflow needs a strict current report contract that distinguishes deterministic success authority from optional socket diagnostics while retaining explicit compatibility with historical reports.

**Why this priority**: Changing an authority without changing and testing its report contract would leave stale readers and ambiguous evidence.

**Independent Test**: Validate current schema reports and every independent authority mutation, then confirm the previous schema remains readable under its historical rules but cannot be emitted as current evidence.

**Acceptance Scenarios**:

1. **Given** a current report, **When** validation runs, **Then** every required field, closed token, surface identity and cross-field invariant is enforced.
2. **Given** a historical schema-3 report, **When** compatibility validation runs, **Then** it remains readable under schema-3 semantics and is identified as legacy evidence.
3. **Given** an unknown schema or an attempt to mix schema-3 and schema-4 fields, **When** validation runs, **Then** it fails specifically.

### Edge Cases

- Structured startup names a wildcard, non-loopback, wrong port, wrong backend or wrong address family.
- Several proxy-start or terminal calibration events make the authority ambiguous.
- Reached-client precedes proxy-start or belongs to a different plan, phase or surface.
- Socket polling observes no endpoint, only wildcards, or a loopback endpoint after the deterministic evidence is already complete.
- A sampled endpoint belongs to a process outside the certified product descendant set.
- Firewall installation or removal is partial, cleanup reconciliation is incomplete, or a child exceeds its deadline.
- A legacy report claims socket-observed success while a current report omits deterministic evidence.
- Diagnostic construction reaches its count or byte bound.

## Requirements

### Functional Requirements

- **FR-001**: Positive controlled-smoke authority MUST require exact structured native proxy-start evidence and exact terminal reached-client evidence from the certified execution.
- **FR-002**: Transient socket-table presence MUST NOT be required for positive success. Bounded socket sampling MUST remain diagnostic and MUST continue to fail on any observed non-loopback endpoint or unexpected process owner.
- **FR-003**: Existing firewall containment, certified descendant ownership, non-loopback rejection, hidden non-interactive child execution, finite deadlines and exhaustive cleanup reconciliation MUST remain mandatory.
- **FR-004**: Proxy-start evidence MUST be unique, native, exact-loopback, address-family consistent and bound to the controlled session. Reached-client evidence MUST be unique, terminal, successful and bound to the same session.
- **FR-005**: Certification MUST publish a closed versioned current report contract that records the deterministic authorities, diagnostic socket observation and cleanup outcome separately.
- **FR-006**: Historical schema-3 reports MUST retain read-only compatibility under their historical validation rules. New reports MUST use schema 4, and mixed or unknown schemas MUST fail.
- **FR-007**: Mutation tests MUST independently invalidate every required field, closed token, uniqueness rule, cross-field invariant, positive authority, containment authority and cleanup authority.
- **FR-008**: Each failed predicate MUST produce one stable diagnostic identifier from a closed vocabulary, in deterministic order and within explicit count and byte bounds.
- **FR-009**: Public diagnostics and reports MUST NOT contain raw local paths, arbitrary endpoint values, payloads, credentials, capability material, private keys, host identifiers or unbounded stderr.
- **FR-010**: Portable and installed smoke surfaces MUST remain separate, unique and independently complete, and both executable digests MUST match the certified package entry.
- **FR-011**: Pinned script changes MUST have a dated decision record. Documentation MUST identify structured evidence as positive authority and socket polling as negative and diagnostic evidence.
- **FR-012**: Product execution MUST occur only on disposable hosted Windows infrastructure. Local verification is limited to source, parsing, unit, contract and static checks.
- **FR-013**: Both the published-review-candidate and Windows package-certification workflows MUST pass from the final pull-request head without retries, waivers or relaxed assertions.
- **FR-014**: Every received review comment MUST be answered and resolved within no more than two review rounds before owner handoff.

### Key Entities

- **Controlled smoke authority**: The conjunction of firewall containment, product process ownership, structured proxy-start, structured reached-client and cleanup reconciliation.
- **Structured evidence**: Parsed bounded lifecycle events belonging to the certified session, never free-text substring inference.
- **Socket diagnostic**: Bounded samples of descendant endpoints retained for unexpected-owner and non-loopback rejection, not positive success.
- **Package certification report schema 4**: Current closed report with two smoke surfaces and separated authority, diagnostics and cleanup facts.
- **Predicate diagnostic**: Stable bounded identifier for one failed certification invariant.

## Success Criteria

### Measurable Outcomes

- **SC-001**: A complete controlled smoke with zero sampled endpoints passes when every deterministic authority is valid.
- **SC-002**: Independent mutations cover 100 percent of required positive, containment, ownership, schema, uniqueness, chronology and cleanup predicates, and every mutation fails with its expected predicate identifier.
- **SC-003**: Any observed non-loopback endpoint or unexpected owner fails in 100 percent of mutation cases, regardless of valid structured positive evidence.
- **SC-004**: Failure output contains at most the documented predicate count and byte limit and contains none of the prohibited raw-value classes.
- **SC-005**: Current schema-4 positive and negative tests pass, schema-3 compatibility tests pass, and unknown or mixed schema tests fail.
- **SC-006**: Both affected hosted workflows complete green from the same final pull-request head without a rerun.
- **SC-007**: No installed product, real game, live capture or real trust mutation runs on the owner workstation during S156.

## Assumptions and Scope

The controlled native product already emits structured proxy-start and calibration-phase records with enough identity to bind startup and reached-client state to one execution. S156 corrects certification authority and evidence contracts; it does not change product proxy behavior, routing, trust, protocol handling, release bytes or independent-review acceptance.

Historical schema 3 is preserved only as a reader compatibility contract. Current package certification emits schema 4. Socket sampling continues because it provides valuable negative evidence, but absence of a sampled transient endpoint is not an observed failure.

Specification, clarification, requirements checklist, plan, tasks and blocking analysis precede implementation. Explicit push and pull-request authorization satisfies the autopilot pre-push pause; the operator remains the merger.
