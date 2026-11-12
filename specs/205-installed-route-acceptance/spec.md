# Feature Specification: Installed Direct Route Acceptance

**Feature Branch**: codex/s205-installed-route-acceptance
**Created**: 2026-11-12
**Status**: Draft
**Input**: Owner-authorized S205 autopilot continuation of merged S204, completing installed-route, direct-calibration, and debug-report acceptance for #593, #594, and #595.

## Scope

**Scope in**: Finish the exact remaining acceptance of #593, #594, and #595 through a cold controlled client, candidate MSI and portable ZIP entry points, observed passive-packet ordering, and native session diagnostics. Add reproducible regressions, correct demonstrated defects, and reconcile issue records with observed results.

**Scope out**: Private-title compatibility claims, release publication, launcher-chain packet-scope issue #596, launcher TLS issue #597, deferred key-export research #580, target-process instrumentation, and ordinary Capture changes.

**Done when**: Each included issue has a passing controlled or package acceptance record for every scoped criterion, every required local and hosted gate passes, review findings are closed, and an official pull request is ready for the owner's final merge decision. Future private-title testing and release observation are not issue gates.

## Clarifications

### Session 2026-11-12

- **Installed default-path evidence**: Use the installed CLI's documented target store and generated session bundle without storage overrides. A package harness with redirected roots is separate evidence and cannot prove default placement, retention, or cleanup.
- **Driver-order proof**: Read passive original-packet presence from an actual Npcap artifact and compare it with the selected route receipt. A route event or synthetic fixture cannot stand in for the passive packet.
- **Existing installation**: Preserve the current published v0.11.0 installation identity and native evidence before any candidate MSI operation; restore its exact installed state afterward. Refuse an operation if the restoration bytes or ownership cannot be verified.
- **Route diagnostics**: The public installed route always retains its bounded route stream. Diagnostics-off is a separate lower-level matrix case, not an installed command option or an omitted artifact.
- **Cloudbase**: Keep one finite unprivileged peer, create no remote files or service, and verify its ports are closed after each run.

## User Scenarios & Testing

### User Story 1 - Exchange bytes through an installed direct route (Priority: P1, #593)

An operator deliberately starts a cold controlled raw-TCP client through an installed Fragcap route. The client reaches a controlled non-loopback peer and exchanges exact bytes in both directions. Unrelated workstation traffic continues, and a failed route reports the last proven stage.

**Independent Test**: Run candidate MSI and ZIP entry points with cold clients and ephemeral external IPv4 and IPv6 peers, their required native route stream, active sentinels, refusal, cancellation, repeat, and teardown. Separately run the signed-driver library route with diagnostics enabled and disabled. Compare published v0.11.0 as a regression baseline without assuming that a failure reproduces on every host.

**Acceptance Scenarios**:

1. **Given** a cold exact client and responding peer, **when** the candidate route is authorized, **then** the client observes the full reply and both ends account for equal nonzero bytes in each direction.
2. **Given** an upstream CONNECT with no client-visible exchange, **when** the tunnel closes, **then** the session reports a zero-byte non-success and the exact last proven stage.
3. **Given** refusal, cancellation, or route teardown, **when** the session ends, **then** unrelated sentinels continue, loss is accounted, and no session-owned route or remote listener remains.

### User Story 2 - Qualify direct calibration with either passive packet order (Priority: P1, #594)

An operator can trust a positive direct-route fact when exact final-client ownership and client-visible transfer are proved, whether passive capture retained the original packet or diversion consumed it first. Passive packet availability remains an independent fact.

**Independent Test**: Observe original-present and original-absent Npcap/WinDivert ordering with packaged components, then compare TCP and supported UDP positive and negative decisions across compatibility, terminal verdict, target fact, and ordinary admission.

**Acceptance Scenarios**:

1. **Given** exact one-use receipt, final-client generation, client response, and two-way proxy transfer, **when** the original passive packet is present or absent, **then** reachability agrees while packet availability reports the observed difference.
2. **Given** incomplete delivery, no payload, competing or reused owner, truncated history, helper identity, or refusal, **when** calibration evaluates evidence, **then** it reports the precise non-positive or independent packet-history result without inventing a passive packet.
3. **Given** a positive exact fact, **when** ordinary Deep Capture assesses the same target and route identity, **then** admission uses that fact; changed identity or incomplete proof is refused before effects.

### User Story 3 - Export a complete and honest debug report (Priority: P1, #595)

An operator exports a debug report from a complete or partial native session and can trace selected route, process, proxy, packet, calibration, and cleanup milestones. Missing or damaged evidence stays visible, and export does not alter the source.

**Independent Test**: Cover complete, zero-byte, refused, queue-gap, writer-failure, crash-prefix, missing-artifact, incomplete-cleanup, and optional-capture cases; compare private source, shareable projection, terminal human and JSON summaries, manifest, Doctor, retention, and bundle export.

**Acceptance Scenarios**:

1. **Given** a complete session, **when** the report is exported, **then** stable local identities and timestamps join the route and supporting evidence, state completeness, and distinguish CONNECT from client exchange.
2. **Given** a partial source, **when** report export runs, **then** unavailable fields and loss are explicit, source bytes remain unchanged, and no complete route or cleanup is fabricated.
3. **Given** an ordinary share recipient, **when** report sharing runs, **then** private route identity and unrelated payload are omitted through a declared transformation and existing access rules.

### Edge Cases

- Candidate and published MSI identities may differ; the comparison must not silently replace or corrupt the operator's existing installation.
- A host may expose only one passive driver ordering. The test plan must establish an observed second order or report the exact missing qualification; a fixture alone does not satisfy it.
- Pre-receive packet loss remains unknown rather than zero.
- The controlled peer may be unreachable or collide with an existing shared-host workload. Use finite waits, distinct ports, and prove cleanup without changing unrelated cloudbase state.
- A packaged host may redirect native outputs. The installed handoff names effective canonical paths and actual retained artifacts or marks default placement unverified.

## Requirements

### Functional Requirements

- **FR-001 (#593)**: Candidate MSI and ZIP entry points MUST prove exact client-visible non-loopback IPv4 and IPv6 TCP exchange with nonzero equal two-way byte counts, the required route stream, and active unrelated sentinels. The signed-driver library path MUST separately pass diagnostics enabled and disabled.
- **FR-002 (#593)**: Selected-flow evidence MUST retain first-packet and checksum flags, captured and chosen interface, synthetic reply generation and send, observed client response, proxy leg, transfer counts, terminal reason, and precise gaps without route-event payload.
- **FR-003 (#593)**: Upstream CONNECT without client-visible exchange MUST remain non-positive; refusal, cancellation, repeat, and cleanup MUST have distinct finite outcomes and explicit post-receive loss while pre-receive loss stays unknown.
- **FR-004 (#594)**: Positive direct calibration MUST bind exact final-client generation, one-use receipt, original tuple, client delivery, and two-way proxy exchange independently of passive original-packet retention for TCP and supported UDP.
- **FR-005 (#594)**: Packaged Npcap and WinDivert acceptance MUST observe both original-present and original-absent selected flows. Their route decisions MUST agree under equal delivery proof while packet availability remains distinct.
- **FR-006 (#594)**: Competing or reused ownership, helper identity, incomplete socket events, refusal, no-payload exchange, and passive history gaps MUST retain precise outcomes. Compatibility, verdict, target store, and later admission MUST consume the same exact decision.
- **FR-007 (#595)**: The native route stream MUST be versioned, bounded, crash-readable, queue and writer loss-accounted, and integrated with manifest, Doctor, recovery, retention, and bundle access authority for complete and partial sessions.
- **FR-008 (#595)**: Debug-report export MUST leave native source unchanged, join available route, process, proxy, packet, calibration, and cleanup evidence, and declare missing, partial, and uncertain facts.
- **FR-009 (#595)**: Human and JSON terminal summaries MUST agree with report authority; a shareable report MUST omit private route identity and unrelated payload through a declared transformation.
- **FR-010**: Ordinary Capture MUST remain passive. No test may require private-title traffic, target instrumentation, a permanent external service, or an unapproved custom native storage path.

### Key Entities

- **Installed test journey**: Package identity, effective executable and native paths, cold client generation, route plan, bundle, retained artifacts, and cleanup state.
- **Selected flow proof**: One-use receipt, process generation, original tuple, packet disposition, client response, proxy leg, and two-way transfer.
- **Packet evidence state**: Original passive packet retained, absent, or history incomplete, independent of route delivery.
- **Debug report**: Source-preserving derived projection with declared completeness, loss, transformations, and unavailable facts.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Candidate installed and portable routes complete exact IPv4 and IPv6 two-way exchanges under active sentinels with no unaccounted post-receive loss; required route-stream, library diagnostics on/off, and failure rows run without skips.
- **SC-002**: Physically observed passive-packet order cases have equal positive route decisions under equal proof, distinct packet-availability states, and zero false positives across specified negative cases.
- **SC-003**: Every report fixture reconciles stream, manifest, Doctor, access, source preservation, summaries, and cleanup; no private receipt or unrelated payload appears in shareable output.
- **SC-004**: The default-path handoff records actual canonical target-store, session-registry, and bundle locations plus actual retained artifact names, or explicitly marks any unproven location unverified.
- **SC-005**: Local repository gates, required signed-driver and package rows, hosted CI, and bot review closeout pass on the exact PR head.

## Assumptions

- S203 and S204 are the implementation baseline. S205 changes code only where acceptance evidence identifies a gap.
- The workstation's installed v0.11.0 is the published comparison baseline. Candidate installation requires an exact preservation and restoration plan; hosted package evidence alone does not establish workstation default paths.
- Cloudbase is shared. The owner authorized temporary use if no persistent file or service remains and existing workloads are not disturbed.
- A physical driver-order observation may require a disposable Windows environment or controlled capture order. A fixture cannot substitute for it.
- No new dependency or release is expected.
