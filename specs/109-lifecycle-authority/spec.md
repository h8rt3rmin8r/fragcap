# Feature Specification: Crash-Safe Lifecycle Authority

**Feature Branch**: `codex/109-lifecycle-authority`

**Created**: 2026-09-01

**Status**: Draft

**Input**: User description: "Implement S109 as the crash-safe lifecycle authority slice, closing target-scoped routing strategies, general resource journaling and recovery, and complete proxy and cleanup sidecars while carrying forward the S108 bounded-loss correction."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Authorize One Exact Route (Priority: P1)

An authorized operator can review one immutable plan that declares every target-scoped routing effect before Deep Capture starts. The selected strategy reports whether configuration reached the socket-owning process and never promotes an unsupported target to a hidden machine-wide route.

**Why this priority**: Recovery cannot identify or reverse a routing effect that was never represented explicitly. The route plan is the prerequisite for every later lifecycle obligation in this slice.

**Independent Test**: Prepare direct-child, command-argument, target-owned configuration, unsupported, and future transport route plans without starting a proxy, then prove that authorization binds the exact plan and that only the implemented child-environment strategy can apply an effect.

**Acceptance Scenarios**:

1. **Given** a supported direct child, **When** its route is prepared, **Then** the complete child-only environment effect, verification rule, and cleanup obligation are declared before authorization.
2. **Given** an unsupported target or strategy, **When** preparation runs, **Then** the session refuses before proxy, trust, routing, or launch effects.
3. **Given** a route was applied, **When** the socket owner is observed, **Then** the result states reached, not reached, escaped, ambiguous, or unavailable from evidence without inference.
4. **Given** a target-owned configuration strategy, **When** a change is planned, **Then** its exact original bytes, replacement bytes, ownership proof, and restoration action are part of the immutable plan.

---

### User Story 2 - Recover Every Owned Effect (Priority: P2)

An authorized operator can terminate fragcap at any lifecycle transition, restart the machine or process, and recover exactly the resources the interrupted session owned. Unrelated listeners, files, trust entries, routes, launches, and evidence remain untouched.

**Why this priority**: In-memory leases and final reports cannot protect the operator after abrupt termination. Durable obligation-before-effect ordering is the safety boundary for active Deep Capture.

**Independent Test**: Interrupt a controlled session before and after every external-effect transition, replay its journal through startup and doctor recovery, and prove exact idempotent cleanup, retained evidence, and explicit unresolved residue.

**Acceptance Scenarios**:

1. **Given** an external effect is about to occur, **When** the effect begins, **Then** a durable pending obligation already identifies its owner, exact target, and safe recovery action.
2. **Given** a complete or partial journal, **When** recovery runs repeatedly, **Then** owned cleanup is idempotent and no unrelated resource is removed.
3. **Given** corrupt, truncated, unsupported, or contradictory journal input, **When** recovery inspects it, **Then** it fails safely, preserves evidence, and names the unresolved obligation without guessing.
4. **Given** cleanup cannot finish, **When** the session finalizes or recovery ends, **Then** the retained obligation remains durable and visible to doctor and the bundle.
5. **Given** all obligations reached terminal states, **When** compaction occurs, **Then** the audit chronology remains reconstructable and the compacted journal retains the final disposition of every resource.

---

### User Story 3 - Audit Lifecycle Evidence as It Happens (Priority: P3)

An operator or machine consumer can read crash-safe proxy and cleanup event streams while a session runs and reconcile every application connection and acquired resource without parsing human logs.

**Why this priority**: The current proxy record contains only coarse start and stop context, while cleanup is a final summary. Neither provides a complete chronology or a trustworthy crash prefix.

**Independent Test**: Run controlled successful, partial, overloaded, writer-failed, and interrupted sessions; parse both streams incrementally; and reconcile their headers, events, gaps, trailers, manifest declarations, journal records, application connections, and terminal report.

**Acceptance Scenarios**:

1. **Given** a proxy session is running, **When** listener, admission, connection, DNS, TLS, protocol, error, loss, stop, or drain activity occurs, **Then** the proxy stream appends a versioned machine-readable event carrying the available connection identity.
2. **Given** a resource obligation changes state, **When** it is created, attempted, retried, released, retained, or fails, **Then** the cleanup stream appends the transition and its journal linkage.
3. **Given** orderly finalization, **When** each stream closes, **Then** exactly one trailer reconciles accepted, written, dropped, failed, connection, resource, and terminal counts.
4. **Given** interruption or writer failure, **When** a consumer reads the surviving prefix, **Then** it is explicitly incomplete and loss remains counted.
5. **Given** application events contain a proxy connection identifier, **When** the completed bundle is reconciled, **Then** every retained identifier resolves to proxy lifecycle evidence or an explicit counted gap.

### Edge Cases

- Authorization is attempted against a route plan that changed after review.
- A target-owned file changes between preparation and application or between application and restoration.
- The process terminates after an obligation is flushed but before the effect, or after the effect but before its completion record.
- The journal ends inside a record, contains a repeated sequence, or names a path outside the owned session boundary.
- A trust entry or listener address has been reused by an unrelated owner before recovery.
- Recovery itself is interrupted and resumes from its own durable progress.
- The proxy or cleanup sidecar fails while forwarding and cleanup must continue.
- More distinct body-loss identities occur than the bounded localized-loss index can retain.
- A connection event is lost while application observations for that connection survive.
- The final cleanup summary is written while the cleanup chronology is incomplete.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Every routing strategy MUST expose one immutable plan covering preparation, authorization, application, verification, and cleanup.
- **FR-002**: A route plan MUST declare every external effect and cleanup obligation before authorization.
- **FR-003**: The implemented child-environment strategy MUST preserve existing launch behavior while moving all secret-bearing route values behind the route plan boundary.
- **FR-004**: The strategy model MUST represent child environment, explicit command argument, reversible target-owned configuration, HTTP proxy, SOCKS, and future protocol-specific routing without claiming unimplemented strategies are available.
- **FR-005**: Unsupported routing MUST fail before proxy, certificate trust, routing, or launch effects.
- **FR-006**: Routing verification MUST distinguish reached socket owner, not reached, escaped, ambiguous, unavailable, and not attempted from observed evidence.
- **FR-007**: No strategy MAY silently apply a system-wide proxy, trust, Winsock catalog, interception driver, executable modification, hook, injection, or process-memory effect.
- **FR-008**: Reversible target-owned configuration plans MUST bind exact original content, replacement content, ownership, conflict detection, and restoration evidence.
- **FR-009**: A durable resource journal MUST record a pending obligation before each external effect begins.
- **FR-010**: Journal records MUST cover proxy listener and task ownership, certificate authority files, trust thumbprints, routing changes, launch state, bundle artifacts, cleanup attempts, recovery attempts, and terminal disposition.
- **FR-011**: Every journal record MUST carry stable session, plan, resource, sequence, operation, state, and ownership evidence sufficient for exact recovery.
- **FR-012**: Journal publication MUST make completed records durable against ordinary process termination and machine restart before the corresponding effect proceeds.
- **FR-013**: Recovery MUST be bounded, idempotent, restartable, and unable to remove a resource whose current identity no longer matches the recorded owner.
- **FR-014**: Corrupt, truncated, unsupported, duplicated, reordered, or contradictory journal records MUST fail safely without executing an uncertain cleanup action.
- **FR-015**: Completed journals MAY compact only when every original obligation and transition remains auditable from the retained representation.
- **FR-016**: Startup and doctor MUST use the same read-only journal inspection and exact recovery implementation; the complete doctor readiness presentation remains outside this slice.
- **FR-017**: Recovery MUST retain declared evidence unless an existing explicit cleanup or retention policy authorizes its removal.
- **FR-018**: The proxy event stream MUST begin before listener acquisition with one versioned header and append listener, admission, connection, DNS, upstream, TLS, protocol, error, loss, stop, and drain records as observed.
- **FR-019**: The cleanup event stream MUST begin before the first external effect with one versioned header and append obligation, attempt, retry, result, retained, recovery, and journal-link records as observed.
- **FR-020**: Orderly completion of each event stream MUST write exactly one reconciling trailer; a prefix without that trailer MUST remain readable and explicitly incomplete.
- **FR-021**: Every retained application proxy connection identifier MUST resolve to proxy lifecycle evidence or an explicit counted gap.
- **FR-022**: Every acquired resource MUST resolve to one released, retained, failed, timed-out, or not-needed terminal cleanup state.
- **FR-023**: Proxy and cleanup stream counts MUST reconcile with application accounting, journal state, manifest version 2 declarations, final cleanup summary, and terminal report.
- **FR-024**: A sidecar writer failure MUST NOT stop forwarding or skip cleanup, and all subsequent unavailable records MUST contribute to named loss accounting.
- **FR-025**: `cleanup.jsonl` MUST be the authoritative cleanup chronology; the existing `cleanup.json` MUST remain a derived final summary for compatibility and MUST declare the chronology as its source.
- **FR-026**: Manifest version 2 MUST declare proxy chronology, cleanup chronology, and cleanup summary as distinct roles with one authority each and truthful completion or omission states.
- **FR-027**: Per-connection, per-stream, and per-resource indexes MUST have finite configured bounds independent of session duration or traffic volume.
- **FR-028**: When localized body-loss identity capacity is exhausted, total loss MUST remain exact and overflow MUST be reported as unlocalized rather than growing memory without bound.
- **FR-029**: Tests MUST cover deterministic plans, authorization mismatch, file conflicts, kill-at-every-transition recovery, restartable recovery, ownership reuse, corrupt journals, writer failure, bounded loss overflow, count reconciliation, and Windows restart behavior.
- **FR-030**: Documentation, glossary, specification, issue closure, and user-visible status MUST preserve the incomplete Deep Capture claim and defer final HTTP/TLS conformance to issue #305.

### Key Entities

- **Routing Strategy**: A typed capability that prepares, applies, verifies, and reverses one target-scoped routing mechanism.
- **Route Plan**: An immutable, authorized declaration of route effects, secret-bearing values, ownership proofs, verification rules, and cleanup actions.
- **Resource Obligation**: One external effect that must reach a recorded terminal cleanup or retention state.
- **Resource Journal**: The durable ordered record of obligations and lifecycle transitions for one session.
- **Recovery Decision**: The ownership-checked action or refusal produced for one unresolved obligation.
- **Proxy Lifecycle Record**: One versioned listener, connection, DNS, TLS, protocol, error, loss, stop, or drain observation.
- **Cleanup Lifecycle Record**: One versioned obligation, attempt, retry, result, retained, or recovery observation.
- **Cleanup Summary**: The compatibility projection of final resource dispositions derived from the cleanup chronology.
- **Loss Localization**: A bounded attribution of discarded body evidence to connection, stream, direction, and representation, with exact unlocalized overflow totals.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: One hundred percent of attempted routing effects are present in the authorized route plan before any external effect begins.
- **SC-002**: All tested unsupported, changed-plan, escaped, ambiguous, and unavailable routing cases refuse or report their exact state with zero inferred reachability.
- **SC-003**: Kill-at-every-transition tests recover or retain one hundred percent of owned obligations and remove zero unrelated resources.
- **SC-004**: Repeating recovery any number of times produces the same terminal resource dispositions and no additional external mutation.
- **SC-005**: Every corrupt or partial journal case produces a readable safe refusal with zero uncertain cleanup actions.
- **SC-006**: One hundred percent of retained application connection identifiers resolve to lifecycle evidence or a counted gap.
- **SC-007**: One hundred percent of acquired resources resolve to exactly one terminal cleanup disposition.
- **SC-008**: Successful event streams have exactly one header and trailer, while every interrupted prefix remains readable and is never reported complete.
- **SC-009**: Sidecar, journal, manifest, summary, and terminal counts reconcile exactly across all controlled success, pressure, interruption, and writer-failure scenarios.
- **SC-010**: Memory used for localized loss, connection, and resource identity tracking remains within fixed configured bounds under unbounded synthetic identity churn.
- **SC-011**: Existing Deep Capture direct-child sessions and manifest version 1 and version 2 readers remain compatible.
- **SC-012**: Complete repository, MSRV, dependency, Windows platform, encoding, and documentation gates pass without adding a dependency package or a prohibited capability.

## Assumptions

- S109 closes issues #306, #320, and #336 in dependency order and leaves #305 for S110.
- Existing native proxy, trust, application JSON Lines, correlation, HAR, manifest version 2, sensitive-artifact, and cleanup summary contracts are reused.
- The current launch-scoped child environment is the only routing strategy implemented by this slice; later issues implement publisher, SOCKS, UDP, QUIC, and other route handlers.
- `cleanup.jsonl` becomes primary operational evidence, while `cleanup.json` remains a compatibility projection and not a competing chronology authority.
- Recovery integration exposes shared startup and doctor seams; issue #321 retains full doctor findings, presentation, and fix UX.
- Process lifecycle completion remains owned by #319, and final conformance remains owned by #305.
- No new third-party package is expected. Any proposed package requires a separate license, MSRV, capability, and supply-chain decision.
