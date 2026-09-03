# Feature Specification: Complete Native Calibration Matrix

**Feature Branch**: `codex/121-native-calibration-matrix`

**Created**: 2026-09-03

**Status**: Draft

**Input**: User description: "Implement S121 as the complete native launch, routing, address-family, and protocol calibration slice for issue #317 under spec-kit autopilot."

## User Scenarios & Testing

### User Story 1 - Calibrate One Exact Native Case (Priority: P1)

An authorized operator selects one stored target, one supported launch case, one routing strategy, one loopback address family, and one protocol family, reviews the bounded plan, and runs a calibration that reports only what that exact case observed.

**Why this priority**: Calibration cannot safely authorize later inspection unless the planned case and the observed case have one exact identity.

**Independent Test**: A controlled target can exercise every supported protocol family over both address families through each implemented managed launch path, with each plan and terminal outcome naming the complete case and with unavailable combinations refused before effects.

**Acceptance Scenarios**:

1. **Given** a cold supported managed launch and an exact calibration case, **When** the operator authorizes the displayed plan, **Then** the run uses finite deadlines and reports the backend, routing strategy, address family, protocol, launch case, target version evidence when available, and outcome.
2. **Given** a requested combination that the selected launch or routing strategy cannot implement, **When** preflight evaluates it, **Then** calibration refuses before bundle creation, proxy start, trust mutation, launch, or fact mutation.
3. **Given** observations for several protocol families during one run, **When** the selected case names one protocol, **Then** only observations that prove that protocol can produce protocol-specific positive facts, while all retained observations and partial outcomes remain visible.

---

### User Story 2 - Preserve an Append-Only Evidence History (Priority: P1)

An operator can retest a case after fragcap, the native backend, or the target changes without erasing earlier positive, negative, partial, or conflicting observations.

**Why this priority**: Compatibility changes over time, and replacing old evidence would make both regressions and uncertainty invisible.

**Independent Test**: A pre-S121 store migrates without row loss, new rows retain their complete case identity and freshness inputs, and repeated conflicting runs remain separate chronological rows.

**Acceptance Scenarios**:

1. **Given** a store containing legacy compatibility rows, **When** it opens under S121, **Then** every row and field survives and legacy rows remain visible but cannot silently authorize a new exact case.
2. **Given** a current positive observation followed by a current negative or partial observation for the same exact case, **When** both are stored, **Then** both remain visible and the latest applicable row governs eligibility without rewriting the older row.
3. **Given** a retest after a context dimension changes, **When** the new result is appended, **Then** the old context remains independently queryable and is not promoted into the new context.

---

### User Story 3 - Consume Only Exact Current Evidence (Priority: P2)

An operator starting ordinary Deep Capture receives a deterministic decision based only on current facts applicable to the prepared native case, while target detail and calibration artifacts explain stale, mismatched, negative, and partial evidence without selecting an aggregate title verdict.

**Why this priority**: A precise calibration store is useful only if the eligibility gate and presentation surfaces preserve its dimensions and uncertainty.

**Independent Test**: Eligibility permutations prove that changing any applicable case dimension prevents a fact from authorizing the run, and human, JSON, bundle, and target-detail views agree on case identity and freshness.

**Acceptance Scenarios**:

1. **Given** a current reached-client row for the exact prepared route context, **When** ordinary Deep Capture evaluates eligibility, **Then** that row may satisfy only the routing prerequisite it directly proves.
2. **Given** only stale, legacy, mismatched, partial, or negative rows, **When** eligibility is evaluated, **Then** the run refuses with the missing or mismatched dimension and recommends an exact retest.
3. **Given** conflicting rows across protocols or address families, **When** the target is displayed or a bundle is finalized, **Then** every row remains distinct and no aggregate compatible or incompatible verdict is invented.

### Edge Cases

- A calibration records no relevant traffic, is interrupted, times out, or loses observations before classification.
- A target emits a protocol other than the selected protocol during the bounded window.
- A backend or target version is unavailable, malformed, or changes between planning and persistence.
- IPv4 and IPv6 results disagree for the same launch, routing, and protocol dimensions.
- A legacy row has none of the new case dimensions, or only some dimensions are present because migration was interrupted.
- Two rows have identical timestamps or conflicting outcomes; durable append order remains total.
- Persistence fails after observations were retained; the bundle reports the proposed row and failed append independently.

## Requirements

### Functional Requirements

- **FR-001**: The system MUST represent one calibration case with an exact launch case, backend name and version, routing strategy, loopback address family, selected protocol family, fragcap version, and target version evidence when available.
- **FR-002**: The system MUST provide an explicit bounded calibration path for every protocol family supported by the native proxy and MUST identify unsupported launch, routing, address-family, or protocol combinations before external effects.
- **FR-003**: The system MUST display the complete calibration case, effective deadlines, possible fact classes, trust action, and cleanup obligations before authorization in both human and structured output.
- **FR-004**: The system MUST keep calibration explicit, target-scoped, confirmation-gated, finite, reversible, and auditable, with no system-wide proxy fallback, silent trust mutation, pinning bypass, target process access, or target key extraction.
- **FR-005**: The system MUST derive positive protocol-specific facts only from retained classifications that exactly match the selected protocol case and satisfy the S120 fact-eligibility rules.
- **FR-006**: The system MUST retain partial, negative, mismatched, interrupted, timed-out, and failed outcomes without converting silence into routing, trust, protocol, or pinning claims.
- **FR-007**: The system MUST append every proposed compatibility fact as a distinct chronological row and MUST NOT update, replace, delete, or aggregate earlier conflicting facts during calibration or retest.
- **FR-008**: The local store MUST persist routing strategy, address family, and protocol dimensions alongside the existing launch, backend, version, target-version, provenance, and freshness fields.
- **FR-009**: Migration from every supported existing store version MUST preserve every target and compatibility row exactly; legacy rows MUST remain readable and visible but MUST be ineligible wherever an exact new dimension is required.
- **FR-010**: Freshness MUST be determined from explicit stale state plus exact applicability to the prepared case, not from an invented elapsed-time threshold.
- **FR-011**: Eligibility MUST select the latest current applicable row for each prerequisite and MUST refuse when the latest applicable row is negative, partial, stale, or missing.
- **FR-012**: Evidence MUST NOT be promoted across launch case, backend identity or version, routing strategy, address family, protocol family, or known target version without direct proof that the dimension is inapplicable to that fact class.
- **FR-013**: Protocol-independent routing facts MUST explicitly declare protocol as inapplicable; protocol behavior, inspectability, and trust facts MUST name one exact protocol family.
- **FR-014**: Calibration plans, lifecycle events, `compatibility.json`, manifests, terminal summaries, target detail, and machine-readable schemas MUST present the same case identity and append results.
- **FR-015**: Every discard, unpersisted proposal, and unlocalized observation loss MUST remain counted through existing artifact authorities and MUST NOT create a positive compatibility fact.
- **FR-016**: The controlled verification target MUST cover every supported protocol family and both loopback address families without a real game account, remote service, capture driver, or real trust-store mutation.
- **FR-017**: Ordinary Deep Capture MUST consume the exact current routing fact set for the prepared native backend, routing strategy, address family, launch case, and known target version.
- **FR-018**: S121 MUST NOT implement proxy bypass policy from issue #318 or claim Deep Capture feature completion before issue #334 closes.

### Key Entities

- **Calibration Case**: The immutable identity of one planned measurement, including target, launch case, backend, routing strategy, address family, selected protocol, versions, and bounded phase.
- **Compatibility Fact**: One append-only observed claim with a key, value, calibration-case dimensions applicable to that claim, provenance, freshness inputs, and durable row order.
- **Applicability Decision**: The deterministic comparison of one fact with one prepared case, including applicable, stale, legacy-incomplete, or a named mismatched dimension.
- **Calibration Outcome**: The terminal evidence result for the selected phase and protocol, kept separate from fact append status and cleanup truth.

## Success Criteria

### Measurable Outcomes

- **SC-001**: A closed controlled matrix covers 100 percent of shipped native protocol families over IPv4 and IPv6, with zero skipped required cases.
- **SC-002**: Changing any single applicable case dimension in eligibility tests prevents evidence from authorizing the changed case in 100 percent of permutations.
- **SC-003**: Migration tests preserve 100 percent of pre-S121 target and compatibility rows and field values, with legacy rows remaining displayable and ineligible for exact-context authorization.
- **SC-004**: For every calibration run, proposed facts equal successful appends plus failed appends, and retained classifications plus named classification loss reconcile without remainder.
- **SC-005**: Human output, structured events, compatibility artifacts, manifest fields, terminal summaries, and target detail agree on every applicable case dimension in all controlled scenarios.
- **SC-006**: All repository verification gates and the dedicated native conformance matrix pass with no new dependency or lockfile package.

## Clarifications

### Session 2026-09-03

- Q: How are migrated rows without new dimensions treated? -> A: Preserve and display them as legacy-incomplete evidence, but never use them to authorize an exact current case.
- Q: Does every fact require a protocol dimension? -> A: No. Routing facts explicitly mark protocol inapplicable, while protocol, inspectability, and trust facts require one exact protocol family.
- Q: What makes a fact current? -> A: Explicit non-stale state plus exact agreement on every dimension applicable to that fact class; no age threshold is invented.
- Q: How are conflicting retests resolved for eligibility? -> A: Preserve all rows and use the latest current applicable row for each prerequisite.
- Q: Does S121 include bypass-list correctness? -> A: No. Issue #318 remains a separate follow-up slice.

## Assumptions

- The only implemented routing strategy remains child-scoped environment routing; other declared strategies are refused and produce no calibration effects.
- The existing cold direct, owned cold Steam, and exact cold publisher-chain paths are the supported real launch cases; warm and unowned paths retain their explicit refusal behavior.
- Backend and fragcap versions are available from the native runtime. Target version remains optional because not every stored target supplies trustworthy build evidence.
- S120 protocol classification is the authority for protocol identity and fact eligibility.
- Existing artifact loss counters, cleanup journals, and confirmation boundaries remain authoritative and are extended rather than replaced.
