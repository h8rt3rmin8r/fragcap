# Feature Specification: Direct Route Recovery and Debug Report

**Feature Branch**: codex/s203-direct-route-recovery

**Created**: 2026-11-12

**Status**: Implementation ready for hosted review; external non-loopback qualification pending

**Input**: Owner-authorized S203 autopilot bundle of GitHub issues #593, #594, #595, and #598. The owner authorized automatic push, an official pull request, CI, the initial external Codex and security review, and at most one explicitly triggered second review round. Final review and merge remain with the owner.

## Scope

**Scope in**: Diagnose and repair the selected external TCP forwarding defect; make direct-route calibration work when diversion prevents Npcap from retaining the original packet; retain a bounded typed direct-route event stream and export a first-class debug report from native bundles; correct the active auto inspection-scope help and verify it against actual resolution.

**Scope out**: Launch-chain packet-scope defect #596, client-facing TLS refusals #597, target-supported TLS key export research #580, private-title compatibility claims, a new release, target instrumentation, a system-wide proxy or capture prerequisite, and changing ordinary Capture behavior.

**Done when**: Each included issue's acceptance criteria is independently traced to controlled evidence; the spec-kit analyze gate has no unresolved critical or high finding; repository-controlled local, Windows route, packaging, and hosted CI gates pass; every received review comment is resolved with at most one explicit second review request; and the official PR is ready for the owner's final review and merge. A later installed private-title observation is not an issue-completion prerequisite.

## Clarifications

- **2026-11-12, positive direct-route fact**: Require an exact one-use final-client receipt and client-side delivery evidence plus successful two-way application transfer for TCP. For UDP require an exact authenticated association, selected client datagram, contacted-peer response, and observed client-facing delivery. An upstream CONNECT, consumed packet, or successful synthetic send alone is insufficient. This follows #593 and #594's explicit distinction between connected upstream and reachable client.
- **2026-11-12, passive packet absence**: Keep route success and passive packet completeness as independent dimensions. A route may prove reachability when WinDivert consumed the first packet before Npcap observed it; the packet field remains unavailable with its driver-order cause, never fabricated as a capture observation.
- **2026-11-12, debug report access**: The detailed route stream stays a sensitive native artifact. The exported debug report uses stable opaque flow IDs and a manifest-like transformation record for omitted exact endpoints or paths, while retaining precise stages and loss. Private source records remain available under existing bundle access for optional exact maintainer capture matching. The source bundle is never modified.
- **2026-11-12, diagnosis**: Record candidate interface and checksum faults, reproduce the failing external handshake, and fix only the demonstrated cause or causes. A successful WinDivertSend call is not proof that Windows delivered the reply to the client.

## User Scenarios & Testing

### User Story 1 - Selected external TCP works end to end (Priority: P1, #593)

A user starts an authorized exact direct route for a cold controlled client. The client completes a non-loopback connection and exchanges application bytes with its selected peer. If it cannot, the session reports the failing stage instead of presenting an upstream CONNECT as completed client reachability.

**Why this priority**: The active release opened upstream tunnels but transferred zero bytes and blocked the final client.

**Independent Test**: Repeat the failure with the v0.11.0 source behavior, then run a packaged controlled IPv4 and IPv6 cold client against a non-loopback origin. Assert client-visible handshake and exact bidirectional bytes, unrelated active sentinels, teardown, and distinct empty-tunnel outcomes.

**Acceptance Scenarios**:

1. **Given** a selected external TCP flow and a responsive origin, **when** the client sends and receives bytes, **then** the route records the exact client handshake and directional byte equality.
2. **Given** an upstream CONNECT with no client ACK or transferred data, **when** the tunnel closes, **then** the result identifies an upstream-connected zero-byte route and does not claim client reachability.
3. **Given** a route failure or cancellation, **when** cleanup runs, **then** packet disposition, selected flow, unrelated sentinels, and resource release have explicit outcomes.

### User Story 2 - Calibration trusts exact route evidence (Priority: P1, #594)

A user calibrates direct-route reachability with either Npcap/WinDivert driver order. Fragcap uses the exact selected-flow receipt, process generation, route delivery, and proxy evidence to decide reachability while separately reporting whether passive packets were retained.

**Why this priority**: The shipped requirement for a retained original packet prevents legitimate consumed flows from yielding a usable fact.

**Independent Test**: Exercise identical controlled TCP and UDP route cases with original packet retained and absent; include reuse, competing owners, incomplete history, refusal, and no-payload cases. Compare compatibility output, verdict, persisted fact, and later admission.

**Acceptance Scenarios**:

1. **Given** an exact final-client generation and consumed original packet, **when** client-side response and usable proxy traffic are proven but Npcap lacks the original, **then** a route-positive fact may be written with packet evidence explicitly unavailable.
2. **Given** an upstream CONNECT without proven client delivery, **when** calibration assesses the flow, **then** no positive reachability fact is written.
3. **Given** ambiguous ownership, reused identity, a route gap, or incomplete receipt, **when** correlation runs, **then** it reports the exact non-positive reason and never attributes the helper to the game.

### User Story 3 - Export a native debug report (Priority: P1, #595)

After a complete or partial session, the user exports a debug report from the native bundle. The report joins route, process, proxy, packet, calibration, manifest, and cleanup evidence by stable session-local identity; states unavailable facts and gaps; and does not mutate the source bundle.

**Why this priority**: Aggregate route counts cannot locate the client handshake or delivery failure seen in the active release.

**Independent Test**: Controlled fixtures cover a healthy flow, an upstream-connected zero-byte flow, a refused flow, queue loss, writer failure, crash prefix, incomplete cleanup, missing passive packet, and optional external capture correlation. Verify export access and source immutability.

**Acceptance Scenarios**:

1. **Given** a complete native bundle, **when** the user exports a debug report, **then** every selected flow has stage, transfer, correlation, loss, and cleanup outcomes with explicit source authority.
2. **Given** a partial or missing route stream, **when** the report is exported, **then** it identifies precisely which facts are unavailable or incomplete and does not invent completion.
3. **Given** route diagnostics enabled under broad pass-through traffic, **when** selected and unrelated traffic run together, **then** selected metadata remains bounded and unrelated payload is neither retained nor formatted on the hot path.

### User Story 4 - Auto inspection help agrees with behavior (Priority: P2, #598)

A user reading the active command help sees that auto selects full launch-chain inspection for supported topologies, while explicit game selects game-only inspection. The same result appears in planning and guided calibration surfaces.

**Independent Test**: Parse help and resolve auto/game for direct and launcher-mediated cases, then compare active user documentation and installed command help.

**Acceptance Scenarios**:

1. **Given** a launcher-mediated or direct target, **when** auto scope is resolved, **then** effective scope is launch-chain and the current help states that behavior.
2. **Given** explicit game scope, **when** the plan is built, **then** effective scope is game-only and the help states the distinction.

### Edge Cases

- A route receipt may exist before a proxy leg, after a helper fails, or without client ACK; each state remains distinct.
- Npcap may retain the original packet, retain later packets only, truncate its flow history, or observe none; missing passive evidence remains visible.
- A flow tuple may be reused or overlap another owner; route and packet identity must not be joined by tuple alone.
- The route event queue may fill or its writer may fail; losses must be bounded, counted, and present in artifact and terminal completeness.
- A crash may leave a valid header and prefix without a trailer; the report must read it conservatively.
- A partial session may have missing process, proxy, packet, calibration, or cleanup artifacts; the report reports those as unavailable.
- Report export must reject unsafe output/source aliasing, preserve native access rules, and avoid leaking secrets into an unprotected share copy.

## Requirements

### Functional Requirements

- **FR-001 (#593)**: The selected external TCP route MUST complete a client-visible handshake and preserve exact bidirectional bytes for a controlled non-loopback peer on IPv4 and IPv6, or identify the precise failed stage.
- **FR-002 (#593)**: Synthetic reply generation, chosen direction/interface metadata, checksum status, send outcome, observed client ACK, and terminal transfer outcome MUST be distinguishable; unverified suspected causes MUST not be treated as established fixes.
- **FR-003 (#593/#595)**: Upstream-connected zero-byte tunnels MUST have a distinct non-success route outcome in the proxy, calibration, debug report, and terminal summaries; post-receive loss MUST remain separate from unknown pre-receive loss.
- **FR-004 (#594)**: A direct-route correlation MUST join a one-use selected-flow receipt, exact process generation/ancestry, original tuple/destination, packet disposition, client-side delivery evidence, and proxy leg; it MUST NOT require Npcap to retain a legitimately consumed original packet.
- **FR-005 (#594)**: Positive route reachability MUST require proven final-client generation and working client-to-origin and origin-to-client route evidence. A successful upstream CONNECT, packet-consumed count, or missing passive observation alone MUST NOT qualify.
- **FR-006 (#594)**: Packet evidence state MUST independently distinguish retained, absent due to driver order, history incomplete, and unobserved/unknown; ambiguity, reuse, competing ownership, failed delivery, route refusal, and no-payload flows MUST remain non-positive with exact reasons.
- **FR-007 (#594)**: Compatibility records, calibration verdict, target-store fact, and ordinary Deep Capture admission MUST agree on the same exact route authority and applicability.
- **FR-008 (#595)**: One typed versioned route stream MUST record selected TCP and supported UDP lifecycles, including refusal and no-byte paths, with stable session-local flow and proxy identifiers, sequence, timestamps, clock uncertainty, header/trailer, finite size and queue bounds, crash-prefix reading, and explicit loss.
- **FR-009 (#595)**: Broad pass-through processing MUST perform no synchronous file write, JSON formatting, unbounded allocation, or unrelated payload retention; diagnostics enabled and disabled MUST meet the existing latency, loss, and unrelated-sentinel bounds.
- **FR-010 (#595)**: The route artifact MUST be registered with manifest authority, sensitivity, completeness, finalization, and loss, and integrated with native retention, Doctor inventory/recovery, cleanup, and share-copy behavior.
- **FR-011 (#595)**: A CLI debug-report export MUST derive a reviewable report from a complete or partial native bundle without changing it, joining route, process, proxy, packet, calibration, and cleanup facts while marking missing, partial, uncertain, and optional external-capture facts explicitly.
- **FR-012 (#595)**: Human and JSON terminal summaries MUST derive from the same route evidence and distinguish upstream connection, client data exchange, empty closure, and diagnostic gaps.
- **FR-013 (#598)**: Active CLI long help and current documentation MUST state that auto selects launch-chain inspection for all supported topologies and explicit game selects game-only; a resolver-bound conformance test MUST cover direct and launcher-mediated cases.
- **FR-014**: Ordinary Capture MUST remain passive. Deep Capture diagnostics MUST be session-scoped, protected under existing output-recipient rules, and must never add target-process instrumentation or make system-wide capture mandatory.
- **FR-015**: Every included issue MUST retain independent acceptance traceability; any incomplete issue MUST be reported explicitly and must not be closed by inclusion alone.

### Key Entities

- **Selected route flow**: One exact process-generation and original-destination route attempt with a session-local identity, receipt, packet disposition, and terminal state.
- **Route event**: A typed stage transition or counted gap associated with a selected flow, source clock, sequence, and bounded metadata.
- **Route evidence state**: Independent delivery, proxy, packet, and ownership facts whose combination determines a positive, failed, or unavailable route result.
- **Debug report**: A derived, access-controlled, immutable-source projection of native session evidence with explicit authority and completeness for each joined fact.

## Success Criteria

### Measurable Outcomes

- **SC-001**: A controlled non-loopback IPv4 and IPv6 client each completes a selected TCP handshake and transfers exact bidirectional bytes; active unrelated sentinels remain continuous with zero unexplained post-receive loss.
- **SC-002**: Retained-original and absent-original driver-order cases produce the same positive route verdict when exact client delivery and two-way transfer are proven, while absent packet truth remains explicitly unavailable.
- **SC-003**: Every negative route fixture, including zero-byte CONNECT, ambiguity, reuse, refusal, lost reply, no-payload, and incomplete history, produces its exact non-positive result in compatibility, calibration, store, and later admission.
- **SC-004**: Complete and partial debug-report fixtures reconcile all selected-flow records, route queue and writer loss, source-artifact completeness, and cleanup state without changing source bytes.
- **SC-005**: Controlled diagnostic load and packaged MSI/ZIP journeys pass existing latency, loss, sentinels, and cleanup gates with diagnostics both enabled and disabled; required rows are not skipped.
- **SC-006**: Active help, guided plan, and installed command wording agree with the scope resolver for auto and game cases.

## Assumptions

- The private installed v0.11.0 bundles described in the issues are not available in this checkout. Controlled reproduction and repository-controlled evidence are the completion gate; no private target identity or traffic is required.
- Existing signed WinDivert, Npcap, native session artifact, output-recipient, and compatibility machinery remain the authority. This slice introduces no new driver, target memory access, or protocol decryption technique.
- A package-level selected-route test can use an owned controlled client and non-loopback peer. Any actual installed-product claim follows the repository's default-path and retention rules.
