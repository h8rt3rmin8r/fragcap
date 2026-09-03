# Feature Specification: Exhaustive Protocol Classification

**Feature Branch**: `codex/120-protocol-classification`

**Created**: 2026-09-03

**Status**: Complete

**Input**: User description: "S120: make Deep Capture protocol classification and omission reasons exhaustive under issue #316."

## Overview

S120 closes issue #316 by giving every published Deep Capture traffic family one versioned, stable classification outcome backed by explicit evidence. Classification distinguishes protocol identification from inspectability and from the reason an observation or artifact is incomplete. Unknown, unsupported, and failed are separate states. A parser failure remains an observed protocol-processing failure and can never become a target compatibility verdict.

The same outcome vocabulary reconciles detailed proxy observations, application records, artifact omissions, compatibility candidates, and human and machine-readable CLI summaries. Each authority retains its own facts: transport and parser records describe what occurred, artifact records describe what was produced or omitted, and compatibility policy promotes only outcomes whose required evidence is present. Summary layers derive from those records and never replace them.

This slice covers the traffic families already published by the native path: HTTP/1.1, HTTPS, HTTP/2, WebSocket, Server-Sent Events, gRPC, generic TCP, non-HTTP TLS, SOCKS5 TCP, SOCKS5 UDP, generic UDP, QUIC, HTTP/3, and packet-only unrouted traffic. It does not expand calibration persistence (#317), add proxy bypass policy (#318), complete process lifecycle evidence (#319), or claim Deep Capture completion (#334).

## Clarifications

### Session 2026-09-03

- Q: Where is classification authority located? -> A: The native proxy owns raw protocol evidence, while the facade owns the versioned public classification derived from it. Artifacts and CLI summaries consume that facade classification.
- Q: Can one outcome reason replace detailed failures? -> A: No. Stable outcome categories accompany, but never erase, raw transport, TLS, parser, retention, and writer evidence.
- Q: How are unknown, unsupported, and failed separated? -> A: Unknown means no supported determination was possible, unsupported means an identified family or version is deliberately not handled, and failed means an attempted supported operation failed with direct evidence.
- Q: What may become a compatibility fact? -> A: Only a classification with the evidence required by that fact. Parser, writer, truncation, and unknown outcomes remain observations and cannot be promoted into support or trust verdicts.
- Q: Does S120 change routing or calibration breadth? -> A: No. S120 supplies the classification contract consumed by #317 and #318 without implementing either dependent policy.

## User Scenarios & Testing

### User Story 1 - Understand Every Traffic Outcome (Priority: P1)

An operator can determine what traffic family was observed, whether it was inspectable, and why evidence is incomplete without interpreting ad hoc strings or silence.

**Why this priority**: Accurate classification is necessary for every later compatibility, artifact, and completion claim.

**Independent Test**: Exercise every cell in the published traffic matrix and confirm each produces one schema-versioned classification with an evidence-backed family, detection state, inspectability state, and reason.

**Acceptance Scenarios**:

1. **Given** supported HTTP/1.1, HTTP/2, WebSocket, SSE, gRPC, generic TCP, generic UDP, QUIC, or HTTP/3 evidence, **when** classification runs, **then** the exact family and observed inspectability are reported with schema version 1.
2. **Given** encrypted traffic without available application semantics, **when** classification runs, **then** the result is encrypted-opaque rather than full, unsupported, or failed.
3. **Given** an unknown protocol, an identified unsupported version, and a parser failure, **when** each is classified, **then** all three produce distinct stable states and reasons.

---

### User Story 2 - Preserve Failure And Omission Authority (Priority: P2)

An operator can trace an incomplete session from its summary to the exact detailed observation or artifact omission without one authority overwriting another.

**Why this priority**: Collapsing parser, trust, routing, retention, or writer failures into one label creates false compatibility conclusions and hides recoverable evidence.

**Independent Test**: Inject not-routed, not-reached, encrypted-opaque, certificate-pinned, client-auth-required, unsupported-version, parser-failed, truncated, and writer-failed cases, then reconcile detailed records, artifacts, manifest omissions, and summaries.

**Acceptance Scenarios**:

1. **Given** a parser error after traffic reached the proxy, **when** the session finalizes, **then** parser-failed remains a processing outcome and neither routing nor target support is inferred.
2. **Given** retained evidence is truncated while forwarding succeeds, **when** artifacts finalize, **then** the traffic outcome remains successful at its observed boundary and the retention omission is separately reported.
3. **Given** an artifact writer fails, **when** summaries are produced, **then** writer-failed describes artifact production and does not rewrite protocol or transport evidence.

---

### User Story 3 - Derive Compatibility Facts Without Guessing (Priority: P3)

A compatibility run publishes only facts that its classified evidence proves, while partial and negative observations remain visible.

**Why this priority**: Calibration and ordinary Deep Capture eligibility depend on facts that must not be inferred from ambiguous failures.

**Independent Test**: Feed all classification states through compatibility selection and verify only explicitly eligible outcomes create fact candidates.

**Acceptance Scenarios**:

1. **Given** a full supported observation correlated to the selected client, **when** compatibility candidates are selected, **then** the proven protocol and inspectability facts may be proposed.
2. **Given** unknown, parser-failed, writer-failed, truncated, or not-reached evidence, **when** candidates are selected, **then** no support or trust verdict is fabricated.
3. **Given** contradictory observations, **when** the append-only compatibility result is rendered, **then** each observation remains separate and the summary reports the conflict without choosing an unproved winner.

---

### User Story 4 - Reconcile Human And Machine Summaries (Priority: P4)

An operator receives the same classification counts and omission meanings in human output, JSON output, application records, and the bundle manifest.

**Why this priority**: Divergent summaries force users to choose which surface to trust.

**Independent Test**: Produce a mixed controlled session and compare counts and reason identities across detailed records, final artifact metadata, and both CLI renderers.

**Acceptance Scenarios**:

1. **Given** a mixed session, **when** it finalizes, **then** every summary count is derivable from detailed versioned records and conservation holds.
2. **Given** a reason unavailable to an older reader, **when** a record is read, **then** the schema version and raw detailed evidence remain available without guessing a replacement meaning.

### Edge Cases

- Traffic reaches the listener but authentication fails before a protocol family is known.
- A supported protocol version is identified and its parser fails after partial metadata was retained.
- TLS succeeds but ALPN is absent or unknown, leaving encrypted or decrypted protocol-unknown evidence.
- Certificate pinning and upstream client-certificate requirements occur at different TLS boundaries.
- Forwarding completes while body or datagram retention truncates, queue admission fails, or the application writer retires.
- A manifest writer fails after complete application records exist.
- More than one detailed reason applies to one session, including transport success plus artifact failure.
- A record carries a future classification schema version.
- Conflicting classifications arrive for distinct connections in one compatibility run.
- Packet-only unrouted traffic has no proxy observation and must not be described as proxy loss.

## Requirements

### Functional Requirements

- **FR-001**: Every published traffic matrix cell MUST map to one versioned protocol classification containing a traffic family, detection state, inspectability state, and evidence reason where required.
- **FR-002**: The traffic family vocabulary MUST cover HTTP/1.1, HTTPS, HTTP/2, WebSocket, SSE, gRPC, generic TCP, non-HTTP TLS, SOCKS5 TCP, SOCKS5 UDP, generic UDP, QUIC, HTTP/3, and packet-only unrouted traffic.
- **FR-003**: Detection MUST distinguish identified, unknown, unsupported family or version, and failed processing. These states MUST NOT be inferred from one another.
- **FR-004**: Inspectability MUST distinguish full application semantics, metadata-only evidence, decrypted protocol-unknown bytes, encrypted-opaque bytes, packet-only evidence, and unavailable application evidence.
- **FR-005**: Stable outcome reasons MUST include not-routed, not-reached, encrypted-opaque, certificate-pinned, client-auth-required, unsupported-version, parser-failed, truncated, and writer-failed.
- **FR-006**: Detailed transport, TLS, parser, retention, queue, and writer records MUST remain available alongside their classification and MUST NOT be rewritten into a different authority's verdict.
- **FR-007**: A parser failure MUST remain distinct from a compatibility verdict, target support result, routing result, TLS trust result, and artifact writer result.
- **FR-008**: Unknown MUST remain distinct from unsupported and failed on every serialized and displayed surface.
- **FR-009**: Classification MUST reject or explicitly preserve invalid combinations rather than silently normalize them into a valid outcome.
- **FR-010**: Application JSON Lines MUST declare the classification schema version and carry enough classification identity for every applicable detailed record and reconciling trailer.
- **FR-011**: Manifest artifact omissions MUST use the stable reason vocabulary where applicable and MUST retain artifact authority, severity, completeness, and loss independently from protocol classification.
- **FR-012**: Human and JSON CLI summaries MUST report counts by stable classification state and omission reason, derived from detailed records.
- **FR-013**: Summary counts MUST reconcile exactly with all retained observations plus explicit bounded-loss counts. Missing records MUST NOT be counted as an observed protocol outcome.
- **FR-014**: Compatibility fact selection MUST define the exact classification evidence eligible for routing, propagation, TLS trust, protocol behavior, and inspectability facts.
- **FR-015**: Unknown, parser-failed, writer-failed, truncated, not-routed, and not-reached outcomes MUST NOT by themselves create positive support, routing, propagation, or trust facts.
- **FR-016**: Conflicting observations MUST remain append-only and separately visible; classification MUST NOT select a preferred observation without an existing evidence rule.
- **FR-017**: Readers MUST reject unsupported future classification schema versions while preserving the underlying detailed record as readable evidence where the enclosing artifact version permits it.
- **FR-018**: The controlled conformance matrix MUST exercise every valid classification and every required reason transition without Internet, elevation, a game, target secrets, or live capture.
- **FR-019**: Existing packet truth, forwarding behavior, authentication, routing, TLS verification, body and datagram retention, and cleanup ownership MUST remain unchanged.
- **FR-020**: S120 MUST add no interception driver, target process access, target key extraction, pinning bypass, system-wide proxy mutation, or Deep Capture completion claim.
- **FR-021**: Architecture, glossary, plan status, proxy documentation, AGENTS, and changelog MUST record S120 as closing #316 while leaving #317, #318, and #334 open.

### Key Entities

- **Classification Schema**: The version that assigns stable meaning to protocol families, detection states, inspectability states, and outcome reasons.
- **Traffic Family**: The bounded published protocol or transport family supported by the Deep Capture matrix.
- **Detection State**: Whether a family was identified, remained unknown, was identified as unsupported, or failed during supported processing.
- **Inspectability State**: The highest evidence boundary actually observed, independent from forwarding success.
- **Outcome Reason**: A stable evidence-backed category explaining why classification or artifact completeness is limited.
- **Detailed Evidence**: Raw transport, TLS, parser, retention, loss, writer, and correlation facts retained by their owning streams.
- **Compatibility Eligibility**: The exact evidence predicate that permits one classified observation to propose one compatibility fact.
- **Classification Summary**: A derived count projection that reconciles with detailed observations and bounded loss.

## Success Criteria

### Measurable Outcomes

- **SC-001**: One hundred percent of published traffic matrix cells produce exactly one valid schema-versioned classification in the controlled conformance suite.
- **SC-002**: Every required reason has at least one positive and one invalid-transition test, and zero invalid combinations serialize as valid classifications.
- **SC-003**: Unknown, unsupported, and failed remain pairwise distinct across proxy-to-facade mapping, application records, compatibility policy, manifest omissions, and CLI output.
- **SC-004**: Parser-failed, truncated, and writer-failed cases produce zero fabricated positive compatibility facts in exhaustive policy tests.
- **SC-005**: Human and JSON summary counts match detailed retained records plus explicit bounded-loss totals for every controlled mixed session.
- **SC-006**: Existing supported protocol conformance rows preserve forwarding and evidence behavior while gaining additive classification truth.
- **SC-007**: The complete repository verification suite passes with dependency, license, analyzer, formatting, lint, encoding, and text-hygiene policy satisfied.

## Assumptions

- S104 through S119 supply the complete current native protocol, routing, evidence, artifact, and IPv6 implementations classified by this slice.
- The native proxy remains the authority for raw protocol and failure observations; the facade remains the stable public policy and artifact authority.
- Application JSON Lines and manifest version 2 permit additive fields. Their existing record authority and loss contracts remain unchanged.
- Calibration expansion and stale-evidence policy remain #317. S120 only makes current evidence eligible or ineligible through explicit classification.
- Proxy bypass and local-destination correctness remain #318. Packet-only or bypassed outcomes are classified without adding bypass parsing or policy.
