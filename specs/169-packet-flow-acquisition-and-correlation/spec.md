# Feature Specification: S169 packet-flow acquisition and correlation

**Feature Branch**: `codex/s169-packet-flow-acquisition-and-correlation`

**Created**: 2026-10-09

**Status**: Implemented; external PR verification and owner merge handoff pending.

**Input**: Owner-authorized S169 under Spec-Kit autopilot, correcting issue #468 end to end with automatic push, official PR, CI and at most two review rounds, then owner merge handoff.

## User Scenarios & Testing

### User Story 1 - Correlate a supported routed client (Priority: P1)

An operator calibrates an owned cold platform/client launch. Completed proxy exchanges must be joined to observed packet ownership for the exact client-to-listener connection when the supported capture path supplies that evidence.

**Why this priority**: The reported v0.10.5 session retained seven HTTP exchanges but no matching flow evidence. Success at the application layer cannot authorize a final-client compatibility fact by itself.

**Independent Test**: A synthetic owned platform/client tree and real loopback exchange exercise the production acquisition, attribution and correlation seams without a real game or real trust-store mutation.

**Acceptance Scenarios**:

1. **Given** an observed exact final-client TCP flow and complete ownership history, **When** the proxy exchange completes, **Then** packet annotations, process intervals, final application correlation, compatibility facts and manifest identify that same owner.
2. **Given** supported IPv4 or IPv6 loopback capture and an early client connection, **When** interfaces and filters refresh during acquisition, **Then** the flow remains obtainable without authorizing unrelated traffic.
3. **Given** an exact platform-owned connection, **When** correlation succeeds, **Then** it remains platform-owned and does not become final-client routing evidence.

### User Story 2 - Explain missing evidence (Priority: P1)

An operator sees completed proxy traffic but no usable final-client ownership. The terminal result names the missing evidence and a supported next action, distinguishing acquisition prerequisites from scope withholding, history loss, ambiguity and unavailable ownership.

**Why this priority**: Repeating gameplay is not a demonstrated remedy for an absent client/proxy flow.

**Independent Test**: Controlled complete exchanges with absent packet evidence produce an inconclusive outcome and the exact missing-evidence guidance.

**Acceptance Scenarios**:

1. **Given** no matching packet flow, **When** completed HTTP traffic is observed, **Then** the result names missing packet-flow correlation and cannot authorize calibration.
2. **Given** missing required loopback acquisition, **When** the supported attempt is prepared or run, **Then** a precise prerequisite failure is visible.
3. **Given** withheld or bounded-out history, **When** the flow is reconciled, **Then** its distinct omission remains visible and counted.

### Edge Cases

- IPv4, IPv6 and mapped-address identities, reversed packet directions and reused tuples.
- Early packets before target acquisition, dynamic filter refresh and finite acquisition/observation deadlines.
- Unrelated traffic, ambiguous or absent ownership, retained ownership, incomplete process history and final-client traffic bypassing the selected route.
- Packet buffer loss, watching/scope discard, per-flow history loss, global history loss and late owner-release traffic.
- Missing, retired or unreadable interfaces, unsupported link types and unavailable loopback prerequisites.

## Requirements

### Functional Requirements

- **FR-001**: Trace and document selected interfaces, loopback readiness, filters/refresh, flow identity, timestamps, socket ownership, acquisition gates and registry handoff for #468; distinguish demonstrated defects from unverified field causes.
- **FR-002**: Correct any demonstrated acquisition defect in its owning layer with a failing-before regression; obtain bounded evidence for supported exact routed target flows.
- **FR-003**: Preserve exact timestamped process ownership, target scope and honest platform/client/unrelated/ambiguous/unavailable outcomes. Accepted connections, environment inheritance and HTTP success do not establish ownership.
- **FR-004**: Cover both listener families, early traffic, interface/filter behavior, acquisition gates and deadlines where relevant to the demonstrated cause.
- **FR-005**: Reconcile packet flow identity and owner with process intervals, application correlation, compatibility facts and manifest completeness; artifact creation alone never authorizes facts.
- **FR-006**: Keep watch-discard, scope refusal and correlation-history loss distinct, bounded and conserved; no broad unrelated retention or fabricated ownership.
- **FR-007**: Report missing packet-flow evidence and a supported next action without claiming gameplay retry remedies the defect.
- **FR-008**: Complete controlled repository tests and the full repository gate, then push/publish one official S169 PR, address every review, trigger at most one second round and hand off with green exact-head CI. No merge or release is authorized.

### Key Entities

- **Packet flow**: Exact endpoint pair, transport, timestamps and observed owner, independent from proxy connection identity.
- **Proxy connection**: Exact accepted peer/listener identity and observation interval, reconciled against packet flow history.
- **Acquisition evidence**: Selected interface/filter/gate observations and separately counted losses or limitations.
- **Correlation result**: Correlated, ambiguous or unavailable owner with a stable reason and explicit completeness.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Every supported controlled owned-client case produces the expected final correlation and consistent downstream artifacts for both address families.
- **SC-002**: Every platform/unrelated/reused/ambiguous/withheld/unavailable case retains its expected distinct outcome and cannot falsely authorize final-client calibration.
- **SC-003**: Every scoped discarded observation reconciles to a named counter; no test permits unexplained packet loss or silent history truncation.
- **SC-004**: A focused regression demonstrates the confirmed correction, and missing-flow complete-exchange scenarios name the evidence boundary and supported next action.
- **SC-005**: All scoped issue criteria, automated gates and PR review findings are complete before owner handoff; no real-game rerun is required.

## Assumptions

- Scope is #468. Storage access/repair #464, garbage collection #458, IGDB #155, community sync #94 and separate Dependabot PR #470 are outside this slice.
- The supplied artifact summary establishes a concrete missing-evidence failure, not its machine-specific cause. Source and controlled reproduction determine the correction.
- The master specification sections 11, 12, 13.7, 15 and 17 govern existing ownership, packet, bundle and calibration contracts.
- Existing resource bounds, authority, native backend and public API compatibility remain binding; no driver installation, real capture against games, trust mutation or target instrumentation is needed for acceptance.

## Clarifications

### Session 2026-10-09

- Q: Can accepted proxy traffic substitute for packet ownership? A: No; only exact observed ownership authorizes final-client facts (FR-003).
- Q: Must the owner reproduce the incident again? A: No; controlled acquisition and endpoint/process evidence are the implementation acceptance boundary (SC-005).
- Q: May early scoped packet evidence survive acquisition? A: Use the smallest demonstrated correction with existing finite bounds and truthful accounting; preserve unrelated traffic exclusion (FR-002, FR-006).
