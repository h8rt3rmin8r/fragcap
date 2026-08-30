# Feature Specification: Native Deep Capture Proxy Foundation

**Feature Branch**: `codex/102-native-proxy-foundation`

**Created**: 2026-08-30

**Status**: Complete

**Input**: Establish the native Rust Deep Capture completion contract, dependency policy, library-owned proxy backend, bounded runtime ownership, and honest public roadmap for issues #279, #280, #281, #282, and #291.

## Clarifications

### Session 2026-08-30

- Q: How much compatible roadmap work belongs in this slice? -> A: Resolve the five ordered foundation issues #279, #280, #281, #282, and #291 together; defer protocol, certificate, trust, event-parity, and CLI-cutover work that depends on this foundation.
- Q: May the foundation accept traffic before forwarding and inspection exist? -> A: Yes, solely to prove bounded ownership; each accepted socket is closed without parsing, forwarding, decryption, or an inspectability claim.
- Q: Does the selected dependency graph participate in the workspace MSRV claim? -> A: Yes; S102 will not create a second, weaker feature-specific toolchain promise.

## User Scenarios & Testing

### User Story 1 - Trust the completion boundary (Priority: P1)

As an operator or contributor, I can identify exactly which Deep Capture capabilities are native, which still use the external backend, which are unsupported by policy, and which tracked work owns every remaining gap.

**Why this priority**: Deep Capture cannot be evaluated honestly while shipped behavior and native completion are conflated.

**Independent Test**: Read the public and architecture documentation, then trace every supported protocol, launch path, artifact, lifecycle obligation, and limitation to either a native owner, a permanent refusal, or an open issue under #278.

**Acceptance Scenarios**:

1. **Given** the current release still uses an external proxy, **When** a user reads the README or Deep Capture references, **Then** each page states that Deep Capture is functional but not native or feature-complete and links to #278.
2. **Given** a maintainer evaluates the native roadmap, **When** they inspect the master specification, **Then** the ordered milestones, component ownership, protocol and launch matrix, exit criteria, and permanent constitutional refusals are explicit.
3. **Given** a future change claims native completion, **When** it is compared with the completion contract, **Then** any unowned behavior or unmet exit criterion prevents that claim.

---

### User Story 2 - Embed the native backend boundary (Priority: P1)

As a Rust consumer, I can instantiate and control the library-owned native proxy backend without depending on the command-line binary or an external process.

**Why this priority**: A native product path requires an embeddable owner for effects before protocol inspection and trust behavior can be added safely.

**Independent Test**: Build a consumer against the public library feature, instantiate the native backend, start it on an explicit loopback endpoint, observe typed state, then stop and clean it up without invoking the CLI or another executable.

**Acceptance Scenarios**:

1. **Given** a valid explicit loopback configuration, **When** a consumer starts the native backend, **Then** the backend owns a loopback listener and returns a typed lease with a stable identity and version.
2. **Given** a running lease, **When** a consumer observes, stops, and cleans it up, **Then** every operation returns typed results and repeated stop or cleanup calls are safe.
3. **Given** the native foundation has no protocol inspection implementation yet, **When** a client connects, **Then** the connection is bounded, counted, and closed without claiming that traffic was inspected.

---

### User Story 3 - Bound every native runtime resource (Priority: P1)

As an operator, I can trust that starting and stopping the native foundation will not leave listeners, tasks, connections, or unreported failures behind.

**Why this priority**: Unbounded or detached work would violate the session coordinator's ownership and cleanup guarantees before any protocol feature is added.

**Independent Test**: Exercise saturation, cancellation, task failure, repeated stop and cleanup, forced shutdown, and ten consecutive start-stop cycles while asserting finite limits and complete terminal accounting.

**Acceptance Scenarios**:

1. **Given** the configured connection limit is reached, **When** another connection arrives, **Then** it is refused or closed, saturation advances exactly once, and no unbounded task is created.
2. **Given** active connections at shutdown, **When** the drain budget expires, **Then** remaining work is forced down and every incomplete join or failure appears in the cleanup report.
3. **Given** any stop, cleanup, cancellation, or worker failure path, **When** the lease terminates, **Then** no task outlives the lease unless the terminal report names that cleanup failure.
4. **Given** ten sequential start-stop cycles on the same endpoint, **When** each cleanup completes, **Then** the next cycle binds successfully and runtime accounting returns to zero.

---

### User Story 4 - Maintain one enforceable dependency policy (Priority: P2)

As a maintainer, I can build, audit, publish, and upgrade the native proxy graph under one explicit toolchain and dependency policy.

**Why this priority**: The earlier candidates failed because their exact graphs crossed the repository's Cargo, advisory, license, or ownership boundaries.

**Independent Test**: Run the repository's MSRV, dependency, license, advisory, packaging, and Windows build gates with the native feature enabled and verify that published metadata is readable by the claimed toolchain.

**Acceptance Scenarios**:

1. **Given** the selected async, HTTP, TLS, certificate, and cryptography stack, **When** repository gates run, **Then** the lockfile is advisory-clean and every license and crate edge is permitted.
2. **Given** the declared workspace MSRV, **When** the full claimed feature graph builds with that Cargo version, **Then** manifests and lockfiles parse and compile successfully.
3. **Given** a Windows release build, **When** release artifacts are packaged, **Then** the native proxy library and its selected graph are included without Python, mitmdump, certutil, OpenSSL commands, or a hidden system proxy mutation.

### Edge Cases

- A non-loopback bind address is rejected before any listener exists.
- Zero connection capacity, zero task capacity, or a zero shutdown budget is rejected as invalid configuration.
- A bind race reports the operating-system failure and leaves no partial lease.
- A connection arrives exactly while stop begins; it is either owned and joined or refused and counted, never detached.
- A worker panics or its completion channel closes unexpectedly; the terminal result records the failure.
- Stop or cleanup is called more than once, or cleanup is called without an earlier stop.
- Observation occurs before start completion, during drain, and after cleanup.
- The native feature is disabled; existing Capture and external-backed Deep Capture behavior remain buildable and historically accurate.

## Requirements

### Functional Requirements

- **FR-001**: The master specification MUST require a native Rust backend for eventual Deep Capture completion and supersede the external-backend end state with an ordered completion plan.
- **FR-002**: The completion contract MUST assign every current artifact, event, compatibility fact, cleanup obligation, CLI behavior, supported protocol, and launch path to a native component, permanent refusal, or open issue.
- **FR-003**: The completion contract MUST define measurable exit criteria for all four milestones under #278.
- **FR-004**: The protocol and launch matrix MUST distinguish implemented behavior, planned behavior, observed compatibility, and permanent constitutional refusals.
- **FR-005**: Public documentation MUST state prominently and consistently that Deep Capture is functional but is not yet native, self-contained, or feature-complete.
- **FR-006**: Each documented remaining limitation MUST link to its owning issue or parent epic, while historical shipped behavior remains accurate.
- **FR-007**: The workspace MUST contain a publishable native proxy library with no dependency on the CLI crate.
- **FR-008**: The facade MUST expose the native backend behind an explicit feature boundary without silently replacing the shipped external backend.
- **FR-009**: A Rust consumer MUST be able to configure, start, observe, stop, and clean up the native backend through typed library APIs.
- **FR-010**: Backend identity and version MUST be stable typed values, not presentation strings assembled by the CLI.
- **FR-011**: The native backend MUST bind only an explicitly configured loopback endpoint and MUST reject ambient, wildcard, or non-loopback routing.
- **FR-012**: The runtime MUST enforce finite limits for accepted connections, worker tasks, per-connection buffered data, and shutdown duration.
- **FR-013**: The runtime MUST never detach a listener or connection task; every task MUST be joined or named as an incomplete cleanup failure.
- **FR-014**: Saturated connections MUST be refused or closed without unbounded work and MUST advance typed accounting.
- **FR-015**: Stop and cleanup MUST be idempotent and MUST honor the coordinator-provided budget.
- **FR-016**: Panic, cancellation, accept failure, forced shutdown, and incomplete drain outcomes MUST be preserved in terminal observations or cleanup results.
- **FR-017**: The foundation MUST NOT parse, forward, decrypt, or claim inspectability for application traffic in this slice.
- **FR-018**: The selected async, HTTP, TLS, certificate, and cryptography dependencies MUST be explicit, minimal-feature, license-approved, advisory-clean, and recorded with maintenance policy.
- **FR-019**: The workspace MSRV claim MUST apply to the selected native graph and MUST be mechanically tested; all published manifests and the lockfile MUST be readable by that toolchain.
- **FR-020**: Windows release builds and package verification MUST compile the selected native graph.
- **FR-021**: Existing controlled tests MUST remain able to substitute proxy backends without constructing the production runtime.
- **FR-022**: The implementation MUST NOT use Python, mitmdump, certutil, OpenSSL commands, injection, hooks, target memory access, target TLS key extraction, interception drivers, pinning bypass, or silent system-wide routing.
- **FR-023**: The repository's architecture gate MUST enforce the approved dependency direction for the native proxy library.
- **FR-024**: The decision to resume native implementation after S100 MUST be recorded as an explicit, justified product-direction change.

### Key Entities

- **Native proxy configuration**: Explicit loopback endpoint, finite connection and task capacities, finite per-connection buffer bound, and shutdown budget.
- **Native proxy backend**: Library-owned factory with stable identity and version that creates one runtime lease.
- **Native proxy lease**: Sole owner of the listener, runtime, cancellation state, connection tasks, counters, and terminal failures.
- **Runtime observation**: Typed point-in-time state and monotonic counters that never imply protocol inspection.
- **Cleanup report**: Idempotent terminal outcome describing listener closure, joined work, forced work, failures, and residue.
- **Completion contract**: Architecture ownership map, support matrix, milestone exit criteria, and permanent refusals governing #278.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Every Deep Capture behavior and remaining gap in the completion contract has exactly one native owner, permanent-refusal reason, or open issue.
- **SC-002**: A library integration test starts and fully cleans a native loopback lease without launching the CLI or any external executable.
- **SC-003**: Ten consecutive start-stop cycles complete with zero live listeners, zero live connection tasks, and zero unreported failures after each cycle.
- **SC-004**: Saturation tests exceed every configured runtime limit while observed live work never exceeds that limit and each refusal is counted.
- **SC-005**: Cancellation, panic, and forced-timeout tests account for 100 percent of started tasks as joined, failed, forced, or explicitly incomplete.
- **SC-006**: The complete repository CI gate, MSRV gate, dependency gate, license/advisory audit, package verification, and Windows feature build pass with the native graph.
- **SC-007**: README, architecture, CLI, compatibility, and output documentation contain no claim that Deep Capture is native, self-contained, or feature-complete before #278 closes.

## Assumptions

- This slice resolves #279, #280, #281, #282, and #291 as one ordered foundation batch.
- Existing external-backed Deep Capture remains the shipped CLI path until the production integration issue #290 changes it.
- HTTP forwarding and observation, TLS interception, CA lifecycle, trust mutation, key logging, richer application protocols, and compatibility-lab parity remain owned by later issues under #278.
- The native foundation may accept and immediately close bounded loopback connections, but it reports no application observation and makes no inspectability claim.
- Raising the workspace MSRV is acceptable only when the exact selected graph requires it and all repository metadata, automation, and public claims change together.
