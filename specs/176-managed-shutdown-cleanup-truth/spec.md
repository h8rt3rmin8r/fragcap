# Feature Specification: Managed Shutdown and Cleanup Truth

**Feature Branch**: `codex/s176-managed-shutdown-cleanup-truth`

**Created**: 2026-10-21

**Status**: Draft

**Input**: Owner-approved S176 bundle of GitHub issues #498 and #499.

## Scope

**Scope in**: Exact session-owned cold direct, Steam, and ordered publisher Deep Capture launches; one pre-effect authorization for automatic end-of-observation close, bounded graceful and force steps, route-owner exit verification, structured lifecycle evidence, and Doctor's independent historical cleanup outcome.

**Scope out**: Passive Capture, warm or unowned applications, general process manager behavior, global application shutdown, unrelated Doctor elevation readiness (#487), release publication, and rewriting retained session evidence.

**Done when**: All repository-controlled #498 and #499 acceptance cases pass, the Spec-Kit analyze gate has no unresolved findings, and the full repository gate passes. An authorized real-title calibration is attempted when the target and environment are available and its result is reported separately from controlled verification.

## User Scenarios & Testing

### User Story 1 - Owned game closes after observation (Priority: P1)

An operator who authorizes a cold managed Deep Capture run sees the exact applications that may be closed, the grace and force deadlines, and the risk to unsaved progress before any session effect. At the actual observation end, fragcap closes only applications it can prove that session started. It keeps the proxy available until every route owner is verified gone or the release deadline expires.

**Why this priority**: #498 removes the routine manual shutdown race that caused partial cleanup in an active release.

**Independent Test**: For controlled cold direct, Steam, and publisher launches, authorize a displayed plan, end observation, and verify exact owner exit, evidence order, and proxy lifetime.

**Acceptance Scenarios**:

1. **Given** an exact cold managed root and authorized shutdown plan, **when** observation ends, **then** fragcap sends a process-bound graceful request where supported, waits a finite interval, and claims route release only after verified exit of all route owners.
2. **Given** an exact owner that ignores grace, **when** the force deadline is reached, **then** only the still-owned process receives bounded termination and route release requires subsequent verified exit.
3. **Given** an owned application with no supported graceful path, **when** observation ends, **then** the evidence names that limitation and the authorized bounded force path is applied only to its exact process object.
4. **Given** launch or observation failure after an owned route was created, **when** cleanup runs, **then** the same finite shutdown and evidence contract applies before proxy stop.

### User Story 2 - Uncertain ownership never controls another application (Priority: P1)

An operator's warm, unrelated same-name, escaped, or ambiguously identified process remains untouched. The session reports partial cleanup and a concrete normal-exit recovery action if it cannot prove and release every routed owner.

**Why this priority**: Process identity is the authority for a destructive action; a name or PID alone cannot carry it.

**Independent Test**: Inject controlled same-name, PID-reuse, path/account/session mismatch, ancestry loss, unrelated Steam title, and control failure cases while asserting that only exact owned process objects receive close or terminate requests.

**Acceptance Scenarios**:

1. **Given** a same-name process outside the launch receipt, **when** shutdown evaluates it, **then** it receives no control request.
2. **Given** PID reuse, changed creation time, path, account, session, or causal ancestry, **when** shutdown evaluates the candidate, **then** it refuses control and records the exact identity mismatch.
3. **Given** incomplete process-tree evidence or an escaped child, **when** shutdown cannot prove ownership, **then** it leaves that process untouched and reports unresolved route release.
4. **Given** a graceful or force request failure, **when** exit cannot be verified, **then** the proxy is not reported as safely released and cleanup remains partial.

### User Story 3 - Doctor preserves a failed session's history (Priority: P1)

An operator can inspect a retained session and see its original failed route-owner cleanup even after all applications have exited. Doctor independently reports whether anything currently blocks a new session.

**Why this priority**: #499 shows that a readable journal can erase a real failed historical outcome from human and JSON views.

**Independent Test**: Use synthetic complete and partial manifests, failed cleanup projections, readable and missing journals, active and exited owners, and compare human, JSON, and history-details output.

**Acceptance Scenarios**:

1. **Given** a readable journal and terminal manifest with failed route-owner cleanup, **when** Doctor reads the session after the owner exits, **then** it reports failed historical cleanup and the resource reason while current readiness may be true.
2. **Given** an active routed owner or recoverable resource, **when** Doctor reads the session, **then** it reports the actual present blocker and available recovery action separately from history.
3. **Given** a complete manifest or a missing journal, **when** Doctor classifies history, **then** its human and JSON facts agree with the available authorities without inventing a fix or changing the stored manifest.

### Edge Cases

- Observation can finish early for terminal process or acquisition failure; shutdown cannot use a packet epoch as its clock.
- A root may exit before a child, and a publisher or Steam chain may require reverse-order close; an unrelated Steam title must not be closed.
- A GUI process may expose no eligible window, ignore close, or exit between validation and request.
- Terminate returning success does not prove exit, and a reused PID must not be treated as the original owner.
- Interruption or evidence-write failure leaves an auditable partial state and does not silently claim route release.
- A failed historical session can coexist with current Deep Capture readiness and no recoverable resource.

## Requirements

### Functional Requirements

- **FR-001**: A managed Deep Capture authorization plan MUST show the exact application and launcher scope, finite graceful and force deadlines, and unsaved-progress consequence before proxy, trust, routing, or launch effects. The same accepted plan MUST authorize automatic shutdown at observation end without another prompt.
- **FR-002**: Process-control authority MUST bind a retained session launch receipt to process-object identity, creation time, canonical executable path, account, session, and proven causal ancestry; PID and image name alone MUST never authorize control.
- **FR-003**: A graceful close MUST be addressed to the verified process when supported. A bounded force operation MUST target only an exact still-owned process using named minimal Windows rights and no memory rights. Every operation MUST disable interactive prompts and hidden console launches.
- **FR-004**: The proxy and session trust MUST remain available until verified route-owner exit or a finite release deadline. Proxy cleanup MUST follow route-release outcome; control call success alone MUST never establish release.
- **FR-004a**: The process watcher MUST remain active from managed launch through route-owner release, including after packet Capture ends or fails to acquire. Shutdown MUST consume starts and exits observed during graceful close and reconcile queued events after watcher closure before claiming release. A late or unproven descendant leaves release unresolved.
- **FR-005**: Lifecycle evidence MUST record each close attempt, grace expiry, force result, identity refusal, unresolved owner, verified exit, and proxy/cleanup ordering without retaining unnecessary process secrets.
- **FR-006**: Warm, pre-existing, same-name, escaped, unrelated, reused-PID, and ambiguous processes MUST receive no close or terminate request. Failure or ambiguity MUST retain partial cleanup and concrete recovery guidance.
- **FR-007**: Doctor MUST derive historical completion from terminal manifest and cleanup evidence, alongside current journal/resource state. Readable journal alone MUST NOT turn a partial or failed past session into healthy completed history.
- **FR-008**: Doctor MUST keep current readiness, historical outcome, and presently recoverable action distinct in human, JSON, and history-details views. It MUST NOT rewrite failed history or suggest a fix for an already exited process.
- **FR-009**: Controlled Windows tests MUST cover direct, Steam, publisher, same-name/unrelated titles, PID reuse, identity mismatch, ancestry loss, grace ignored, no window, force failure, interruption, and route/proxy ordering. Synthetic Doctor tests MUST cover readable/missing journals, complete/partial/failed manifests, active owner, and post-timeout owner exit.

### Key Entities

- **Managed launch receipt**: Session-specific proof of the created root and the declared owner chain.
- **Owned process identity**: Process object plus creation instant, canonical executable path, account, session, and causal ancestry proof.
- **Shutdown authorization**: Immutable pre-effect plan naming exact scope, deadlines, and unsaved-progress warning.
- **Route release outcome**: Verified exit, identity refusal, control failure, or deadline expiry with ordered lifecycle evidence.
- **Historical session outcome**: Terminal manifest and cleanup facts independent of current resource readiness.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Every controlled cold direct, Steam, and ordered publisher scenario reaches its exact expected route-release outcome and orders proxy stop after verified exit or the documented finite deadline.
- **SC-002**: Across all controlled mismatch and unrelated-process cases, zero unowned processes receive graceful or force requests.
- **SC-003**: Every controlled failed shutdown or uncertain ownership case retains partial cleanup with a named unresolved owner and actionable normal-exit guidance; zero cases claim release from a terminate return value alone.
- **SC-004**: For every Doctor history matrix case, human, JSON, and detailed views agree on historical state, current readiness, failed resource, and present recovery action.
- **SC-005**: The included regressions and repository verification gates pass at the exact PR head; any real-title result is recorded as separate field evidence.

## Assumptions

- The owner-approved S176 plan includes constitution 1.6.0's explicit Deep Capture process-control exception; passive Capture remains unchanged.
- Session-owned cold managed launches already carry one immutable target, route, and authorization context. This slice extends that authority rather than introducing a general process manager.
- Existing journal, manifest, and cleanup artifacts remain the authorities for current resources and terminal historical outcome; old bundles are read without migration or mutation.

## Clarifications

### Session 2026-10-21

- **Q: When is automatic control authorized?** A: In the existing complete pre-effect Deep Capture plan, before any route, trust, proxy, or launch effect. No second prompt is required after approval.
- **Q: What if Steam is also serving an unrelated title?** A: The unrelated title and shared Steam root receive no control request. The session reports unresolved route release if it cannot prove that the inherited route is gone.
- **Q: What is the finite shutdown schedule?** A: Preserve the existing 120-second route-owner ceiling. Give eligible owners up to 60 seconds for graceful exit, apply force to exact still-owned owners by 90 seconds, and use the remaining time only for exit verification and evidence. No phase resets the ceiling.
- **Q: What does Doctor's historical failure do to readiness?** A: It remains visible in historical findings; it blocks new-session readiness only when a current routed owner or recoverable resource actually remains.
