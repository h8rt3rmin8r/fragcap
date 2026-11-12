# Feature Specification: Direct Route Acceptance

**Feature Branch**: `codex/s204-direct-route-acceptance`
**Created**: 2026-11-12
**Status**: Draft
**Input**: Owner-authorized S204 autopilot continuation of the S203 direct-route recovery, covering the remaining controlled acceptance for #593, #594, #595, and #598. Push and official PR are authorized; final review and merge remain with the owner.

## Scope

**Scope in**: Qualify the S203 external selected TCP route, direct-route calibration, native route diagnostics and debug report, and corrected auto inspection help against their issue criteria. Add missing regressions, fix defects demonstrated by them, and reconcile the issue and slice records with observed results.

**Scope out**: Launcher-chain packet-scope defect #596, client-facing TLS refusals #597, key-export research #580, private-title compatibility claims, a new release, target instrumentation, and changes to ordinary Capture.

**Done when**: Each included issue has controlled evidence for its scoped behavior, remaining limitations are stated precisely, all required repository and Windows/package gates pass, and the official PR is reviewed and green for the owner's merge decision. A future field observation or new release is not a standing issue gate.

## Clarifications

### Session 2026-11-12

- **Issue closure**: Repository-controlled and controlled-peer evidence is the implementation gate. A packaged default-path claim additionally requires exact effective native locations and retained artifact inventory. S204 will not treat the private v0.11.0 bundle or a new release as an unbounded prerequisite.
- **Remote peer**: Cloudbase is a shared host. Use a temporary, bounded, unprivileged listener only after checking port availability, create no remote files or service, and confirm the listener and socket are gone after the run.
- **Evidence authority**: Client-observed bytes and exact selected route proof establish reachability; passive Npcap original-packet presence is recorded independently. The debug report never promotes a missing source to success.

## User Scenarios & Testing

### User Story 1 - Use a selected external TCP route (Priority: P1, #593)

An authorized controlled client connects to an owned non-loopback peer through the signed direct route and exchanges bytes in both directions. If routing stops before usable exchange, the session identifies the stage and does not present an upstream connection as client success.

**Independent Test**: Run the packaged Windows route with controlled IPv4 and IPv6 peers, exact byte assertions, active unrelated sentinels, diagnostic recording enabled and disabled, and clean teardown.

**Acceptance Scenarios**:

1. **Given** an exact cold selected client and responsive external peer, **when** the client connects and sends data, **then** client-visible response and exact two-way bytes are observed.
2. **Given** an upstream tunnel that transfers no client data, **when** it closes, **then** route and calibration summaries identify zero-byte non-success separately from upstream connection.
3. **Given** refusal or cancellation, **when** the route ends, **then** selected-flow disposition, unrelated sentinel continuity, and cleanup are explicit.

### User Story 2 - Calibrate with consumed original packets (Priority: P1, #594)

An authorized calibration can qualify an exact working final-client route whether Npcap retained its original packet or WinDivert consumed it first. Passive packet evidence remains a separate fact, and ambiguous ownership or incomplete route delivery remains non-positive.

**Independent Test**: Compare original-present and original-absent TCP and supported UDP cases, plus competing owner, reuse, passive history gap, incomplete route proof, refusal, and no-payload cases, across compatibility, verdict, target fact, and later admission.

**Acceptance Scenarios**:

1. **Given** exact selected ownership, client delivery, and proxy exchange, **when** the original passive packet is absent, **then** the route may qualify while packet availability stays unavailable.
2. **Given** an upstream connection without client delivery, **when** calibration completes, **then** no positive route fact is written.
3. **Given** conflicting ownership or incomplete route proof, **when** evidence is reconciled, **then** the exact reason prevents a positive fact and never attributes a helper as the client. A passive history gap alone remains visible but does not negate independent exact route proof.

### User Story 3 - Read an honest native debug report (Priority: P1, #595)

The operator exports a debug report from a complete or partial native bundle. It names each selected route's milestones, transfer and evidence gaps, and cleanup state without changing the source or exposing private routing identifiers in the shareable projection.

**Independent Test**: Exercise complete, zero-byte, refused, queue-loss, writer-failure, crash-prefix, missing-artifact, partial-cleanup, and optional-capture cases. Compare terminal human and JSON facts with the report, source digests, and artifact access.

**Acceptance Scenarios**:

1. **Given** a complete session, **when** the report is exported, **then** route, process, proxy, packet, calibration, and cleanup facts reconcile by stable local identity.
2. **Given** a partial or damaged source, **when** the report is exported, **then** unavailable and incomplete fields are explicit and no complete outcome is fabricated.
3. **Given** active unrelated traffic, **when** route diagnostics are enabled, **then** finite selected-flow metadata is retained without unrelated payload capture or synchronous hot-path formatting or file output.

### User Story 4 - Read current inspection-scope help (Priority: P2, #598)

An operator sees that auto selects launch-chain inspection for every supported topology and that explicit game inspects only the final client. Installed help, guided plan, and current documentation agree.

**Independent Test**: Compare the actual resolver for launcher-mediated and direct cases with command help, guided planning, and current documentation; inspect packaged command help.

**Acceptance Scenarios**:

1. **Given** either supported topology, **when** auto is selected, **then** launch-chain is the effective scope and active help says so.
2. **Given** explicit game, **when** the plan is prepared, **then** game-only remains distinct and is described accurately.

### Edge Cases

- A selected route may connect upstream but never observe a client ACK or data.
- Driver ordering may retain the original packet, later packets only, or no passive packet.
- Tuple reuse, competing owners, a helper socket, or incomplete route ownership proof must not qualify a final-client route. Passive Npcap history loss remains an independent packet-completeness fact.
- The event queue or writer may lose records; a crash may leave only a valid prefix.
- A partial bundle may lack packet, process, proxy, calibration, or cleanup records.
- An exported report destination may alias its source or be inaccessible to the authorized recipient.
- Controlled external tests must leave the shared remote peer with no service or files created by the test.

## Requirements

### Functional Requirements

- **FR-001 (#593)**: Selected non-loopback IPv4 and IPv6 TCP MUST transfer exact client-to-origin and origin-to-client bytes or report the precise failed stage; upstream CONNECT alone MUST NOT qualify.
- **FR-002 (#593)**: Selected route evidence MUST distinguish original packet, chosen interface and checksum handling, generated/sent synthetic reply, client ACK, directional transfer, zero-byte closure, refusal, and cancellation while keeping pre-receive loss unknown.
- **FR-003 (#594)**: Positive direct calibration MUST require exact final-client generation, one-use route receipt, client delivery, and working proxy exchange independently of passive original-packet retention.
- **FR-004 (#594)**: Original-present and original-absent cases MUST preserve separate packet evidence status; ambiguous ownership, reuse, helper identity, incomplete route proof, refusal, and no-payload cases MUST remain non-positive with named reasons. Passive packet-history truncation alone MUST NOT erase independently proven exact route reachability.
- **FR-005 (#594)**: Compatibility output, terminal verdict, stored fact applicability, and ordinary admission MUST derive from the same exact route decision.
- **FR-006 (#595)**: The selected-flow event stream MUST be versioned, bounded, crash-readable, loss-accounted, protected, and registered in manifest, retention, Doctor, cleanup, and bundle sharing authority.
- **FR-007 (#595)**: The debug report MUST derive from native complete or partial evidence without source mutation; it MUST show unavailable facts, route and proxy stages, correlation, transfer, loss, and cleanup, and keep sensitive source identifiers out of shareable output.
- **FR-008 (#595)**: Human and JSON terminal summaries MUST reconcile with report authority, and diagnostics on and off MUST satisfy the existing controlled route and unrelated-sentinel bounds without broad hot-path payload work.
- **FR-009 (#598)**: Installed command help, guided plan, resolver-linked test, and current documentation MUST agree that auto resolves to launch-chain for supported topologies and explicit game remains game-only.
- **FR-010**: Ordinary Capture MUST remain passive. No test or diagnostic may require target-process instrumentation or a system-wide capture to establish direct-route behavior.

### Key Entities

- **Selected route flow**: Exact client generation, original tuple, one-use route receipt, proxy leg, packet disposition, and terminal state.
- **Packet evidence state**: Passive packet availability, separate from route delivery and client reachability.
- **Native route event**: Bounded selected-flow transition with sequence, timestamp, and explicit loss.
- **Debug report**: Derived projection of native bundle evidence with declared unavailable facts and transformation.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Controlled non-loopback IPv4 and IPv6 selected TCP cases each exchange exact bytes in both directions with active unrelated sentinels and no unexplained post-receive loss.
- **SC-002**: Original-present and original-absent positive cases agree on route result while reporting distinct passive packet availability; every specified negative case remains non-positive.
- **SC-003**: All report fixtures reconcile selected flows, source completeness, queue and writer loss, terminal summary, and cleanup without changing source bytes.
- **SC-004**: Required Windows route and MSI/ZIP package matrix rows run without skips, including diagnostics enabled and disabled, refusal, cancellation, and cleanup.
- **SC-005**: Installed help and active documentation match the resolver for auto and game in both topology cases.

## Assumptions

- S203 provides the implementation baseline; S204 extends or corrects it only where controlled evidence finds a gap.
- The owner's cloudbase permission covers a temporary controlled peer with no persistent service or file changes; its existing workloads take precedence.
- The repo-pinned Npcap SDK is available locally, while Npcap itself and signed WinDivert retain their existing separate installation and package rules.
- Repository-controlled evidence closes implementation issues. A later active-release failure warrants a new version-specific defect rather than an open-ended validation issue.
