# Feature Specification: Native Protocol Conformance

**Feature Branch**: `codex/110-protocol-conformance`

**Created**: 2026-09-01

**Status**: Draft

**Input**: User description: "Implement S110 as the independent native HTTP and TLS interoperability and conformance gate for issue #305."

## User Scenarios & Testing

### User Story 1 - Trust a Complete Protocol Matrix (Priority: P1)

An operator or reviewer can inspect one committed matrix and determine which independent clients and origins exercised each required native proxy protocol, which standards and implementation versions apply, what result was expected, and what was observed.

**Why this priority**: A collection of passing unit tests cannot establish interoperability when rows can be absent, duplicated, silently skipped, or exercised only by the same implementation on both sides.

**Independent Test**: Validate the committed matrix without network access, external services, elevation, a capture driver, or a game, and prove that every required protocol has two distinct client implementations and two distinct origin implementations with passing positive and failure rows.

**Acceptance Scenarios**:

1. **Given** the required HTTP/1.1, HTTPS, HTTP/2, WebSocket, SSE, and gRPC families, **When** the matrix is validated, **Then** every family names standards, client, origin, version, expected result, observed result, evidence, and required CI tier.
2. **Given** a missing, duplicate, skipped, stale, or same-implementation row, **When** validation runs, **Then** the gate fails and names the exact row and reason.
3. **Given** TLS coverage, **When** the matrix is inspected, **Then** TLS 1.2, TLS 1.3, valid chains, wrong names, untrusted chains, client identity, and upstream failure are represented without weakening verification.

---

### User Story 2 - Reproduce Integrated Evidence (Priority: P2)

A contributor can run bounded loopback scenarios and reproduce synthetic evidence that reconciles protocol observations with application JSON Lines, HAR, TLS key logs, packet correlation, proxy lifecycle, cleanup lifecycle, cleanup summary, and manifest truth.

**Why this priority**: Protocol forwarding alone does not prove that the evidence product remains truthful across its independently produced artifacts.

**Independent Test**: Run the portable conformance harness from a clean checkout and compare its normalized report with the committed evidence while verifying every artifact with its production reader.

**Acceptance Scenarios**:

1. **Given** a successful required row, **When** its session finalizes, **Then** application, HAR, correlation, lifecycle, cleanup, and manifest facts reconcile exactly.
2. **Given** a required failure row, **When** its session finalizes, **Then** the expected refusal or partial result remains explicit and cannot be counted as a positive pass.
3. **Given** an evidence update, **When** the generator runs twice, **Then** the normalized committed output is byte-identical and contains no secret, private key, user path, account, or live endpoint.

---

### User Story 3 - Prove Analyzer Consumption in CI (Priority: P3)

A reviewer can see a supported CI tier invoke an unmodified TShark installation against committed pcapng and TLS key-log artifacts and reject unreadable or semantically empty evidence.

**Why this priority**: Internal parsers cannot prove the zero-modification analyzer interoperability claim.

**Independent Test**: Run the analyzer gate with TShark on a supported runner and require successful pcapng parsing, key-log loading, packet output, and expected protocol fields.

**Acceptance Scenarios**:

1. **Given** committed analyzer fixtures, **When** TShark opens the pcapng with the TLS key-log preference, **Then** it exits successfully and emits the expected packet and protocol facts.
2. **Given** missing TShark on the required analyzer tier, **When** the gate runs, **Then** the job fails rather than reporting a skip or pass.
3. **Given** a local machine without TShark, **When** the portable repository gate runs, **Then** it validates the committed analyzer result and clearly reports that live analyzer execution belongs to the dedicated CI tier.

### Edge Cases

- A required matrix row is present twice with different outcomes.
- Two aliases identify the same client or origin implementation.
- A version field is empty, unpinned, or disagrees with the lockfile or tool report.
- A failure case returns the expected error but accidentally records a positive protocol result.
- A complete application stream lacks its trailer or has unreconciled loss.
- HAR omits an unavailable fact instead of preserving its explicit omission reason.
- Packet correlation is unavailable or ambiguous for a controlled row expected to match.
- Cleanup completes but lifecycle, journal, summary, and manifest counts disagree.
- Analyzer input opens but contains no packets or no expected protocol field.
- A generated fixture contains a capability secret, certificate private key, absolute user path, or nondeterministic timestamp.
- A CI tier records skipped or ignored required tests as success.

## Requirements

### Functional Requirements

- **FR-001**: The repository MUST contain one versioned, machine-readable native conformance matrix for HTTP/1.1, HTTPS, HTTP/2, WebSocket, SSE, and gRPC.
- **FR-002**: Every matrix row MUST name a stable identifier, protocol, case, requirement status, standards references, client implementation and version, origin implementation and version, expected result, observed result, evidence references, artifact expectations, and CI tier.
- **FR-003**: Every standard protocol MUST have positive coverage from at least two genuinely distinct client harness implementations and two genuinely distinct origin harness implementations.
- **FR-004**: Implementation distinctness MUST be based on separately implemented protocol drivers or libraries, not aliases, configuration variants, or two names for the same helper.
- **FR-005**: Required coverage MUST include HTTP/1.1, HTTPS over TLS 1.2 and TLS 1.3, HTTP/2, WebSocket over HTTP/1.1 and RFC 8441, identity SSE, and gRPC envelope and status semantics.
- **FR-006**: Required failure coverage MUST include malformed framing, authentication refusal, wrong-name and untrusted certificate chains, origin disconnect, bounded timeout or cancellation, and cleanup or artifact failure.
- **FR-007**: A required row MUST pass only when its expected and observed results match and every named evidence assertion passes.
- **FR-008**: Missing, duplicate, skipped, ignored, not-run, unexpected-pass, or unexpected-failure required rows MUST fail validation and MUST NOT contribute to pass totals.
- **FR-009**: Matrix validation MUST reconcile declared protocol and implementation counts rather than accepting a hand-written total.
- **FR-010**: The portable harness MUST use bounded loopback origins, clients, deadlines, connections, streams, bodies, and evidence queues.
- **FR-011**: Harness traffic MUST remain synthetic and MUST require no Internet service, account, game, elevation, target process access, capture driver, or machine-wide proxy effect.
- **FR-012**: Integrated scenarios MUST validate application JSON Lines, HAR, TLS key logs, packet correlation, proxy lifecycle, cleanup lifecycle, cleanup summary, resource journal, and manifest version 2 with their production readers or schemas.
- **FR-013**: Integrated artifact counts, connection identities, stream identities, loss, correlation, completion, cleanup, and authority declarations MUST reconcile exactly.
- **FR-014**: Failure rows MUST preserve the exact refusal, partial, unavailable, ambiguous, loss, or cleanup state and MUST NOT synthesize missing success facts.
- **FR-015**: The repository MUST commit normalized synthetic conformance evidence and analyzer fixtures whose generation is deterministic.
- **FR-016**: Committed evidence MUST contain no capability secret, authentication value, private key, live user data, account identifier, absolute user path, or uncontrolled endpoint.
- **FR-017**: A drift test MUST fail when regenerated normalized evidence differs from the committed evidence.
- **FR-018**: A dedicated supported CI tier MUST run unmodified TShark against the committed pcapng and TLS key-log artifacts.
- **FR-019**: The analyzer tier MUST fail if TShark is unavailable, cannot read either configured input, emits zero packets, or omits the expected protocol facts.
- **FR-020**: Portable CI on Windows and Linux MUST validate the matrix, integrated evidence, committed result, and no-skip invariant.
- **FR-021**: Tool and library versions used as conformance identities MUST be exact, lock-resolved, or captured from the executing tool.
- **FR-022**: The conformance report MUST separate portable execution, Windows execution, and analyzer execution rather than promoting an unexecuted tier to pass.
- **FR-023**: Documentation MUST explain how to reproduce, inspect, update, and review the matrix and synthetic evidence.
- **FR-024**: S110 MUST close only issue #305, MUST correct stale prose that assigns generic transports to #305, and MUST preserve the incomplete Deep Capture claim.
- **FR-025**: S110 MUST NOT add product protocol support, generic transports, a process handle, traffic transmission outside loopback, certificate verification bypass, pinning bypass, or a feature-complete claim.

### Key Entities

- **Conformance Matrix**: The versioned set of required and informational rows plus derived coverage rules.
- **Conformance Row**: One exact client, proxy, origin, protocol, case, expected result, observed result, and evidence relationship.
- **Implementation Identity**: A stable name, kind, version source, and protocol-driver lineage used to establish independence.
- **Artifact Assertion**: One required fact checked against a production artifact reader or schema.
- **Analyzer Fixture**: Synthetic pcapng and key-log input intended for unmodified TShark consumption.
- **Conformance Report**: The normalized committed result with row outcomes, coverage totals, tier outcomes, tool versions, and omissions.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Every required protocol has at least two distinct client implementations and two distinct origin implementations, computed from passing required rows.
- **SC-002**: One hundred percent of required rows pass, with zero skipped, ignored, duplicate, missing, unexpected, or unexecuted required rows.
- **SC-003**: TLS 1.2, TLS 1.3, valid chain, wrong name, untrusted chain, client identity, and upstream failure cases have explicit expected and observed results.
- **SC-004**: One hundred percent of integrated artifact assertions reconcile across application JSON Lines, HAR, key log, correlation, lifecycle, cleanup, journal, and manifest evidence.
- **SC-005**: Regenerating normalized synthetic evidence twice produces byte-identical UTF-8 output with no mojibake or secret-bearing material.
- **SC-006**: TShark reads the committed pcapng with the committed key-log configuration, reports at least one packet, and emits every declared analyzer fact on its required CI tier.
- **SC-007**: Windows and Linux portable gates and the dedicated analyzer gate pass from a clean checkout with no skipped required row.
- **SC-008**: Full repository, MSRV, dependency, platform, encoding, documentation, and mojibake gates pass without adding a product dependency package or prohibited capability.

## Assumptions

- S102 through S109 provide the native runtime, protocol fidelity, artifacts, correlation, and crash-safe lifecycle contracts this slice validates.
- Existing product libraries remain exact-pinned; a test-only dependency is acceptable only if existing implementations cannot satisfy genuine independence and the usual license, MSRV, and capability review passes.
- TShark is an external CI analyzer, not a shipped dependency.
- Committed evidence records normalized observations and tool identities, not live secrets or private key material.
- Generic TCP, SOCKS, UDP, QUIC, HTTP/3, and other launch and transport coverage remain owned by milestone 3 issues #310 through #318 and are outside S110.
