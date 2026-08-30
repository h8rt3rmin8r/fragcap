# Feature Specification: Library-First Deep Capture Sessions

**Feature Branch**: `codex/098-library-deep-capture`

**Created**: 2026-08-29

**Status**: Draft

**Input**: User description: "Kick off S098", implementing issue #252, expose a library-first Deep Capture session API and thin the CLI.

## User Scenarios & Testing

### User Story 1 - Run Deep Capture Through the Library (Priority: P1)

As a Rust library consumer, I can configure, preflight, start, observe, stop, and clean up a Deep Capture session without invoking the fragcap binary, so Deep Capture is a product capability rather than CLI-owned behavior.

**Why this priority**: Every later backend and launch-path change depends on a stable library-owned lifecycle. Without it, each addition deepens the existing command monolith.

**Independent Test**: A controlled consumer drives the complete session lifecycle through the public library surface and receives the same bundle, compatibility facts, outcomes, and cleanup truth as the shipped command, without linking to or invoking the CLI.

**Acceptance Scenarios**:

1. **Given** a stored controlled target and injected local adapters, **When** a consumer preflights and runs a session, **Then** it receives typed lifecycle events and a terminal report while the expected bundle and target-owned facts are written.
2. **Given** a configuration that fails preflight, **When** a consumer requests preflight, **Then** it receives a typed refusal and no proxy, trust, launch, bundle, or fact side effect occurs.
3. **Given** a session already started, **When** a consumer requests stop and cleanup, **Then** every owned resource receives a bounded cleanup attempt and the terminal report names every result.

---

### User Story 2 - Substitute Lifecycle Adapters (Priority: P1)

As a backend or integration author, I can supply narrow implementations for proxy execution, trust management, launch preparation, Capture execution, bundle storage, compatibility fact persistence, time, and event delivery, so the session policy can be tested and extended without editing the CLI or depending on a live game.

**Why this priority**: The planned native proxy spike and managed direct-executable launch must land behind library-owned seams rather than create new command-specific branches.

**Independent Test**: Controlled adapters exercise success, refusal, interruption, partial observation, backend failure, fact-write failure, bundle failure, and cleanup failure through the same public session coordinator.

**Acceptance Scenarios**:

1. **Given** an injected adapter set, **When** one adapter returns a typed failure, **Then** the coordinator preserves earlier observations, attempts every still-applicable finalization and cleanup action, and reports the failure without parsing text.
2. **Given** a replacement proxy adapter, **When** a session runs, **Then** no CLI source change or bundle-contract change is required to select and drive it.
3. **Given** a controlled environment with no capture driver, elevation, game account, remote service, or real trust mutation, **When** the public API tests run, **Then** the entire policy and artifact path remains verifiable.

---

### User Story 3 - Preserve Auditable Partial Results (Priority: P2)

As an authorized operator or automation consumer, I receive one terminal result that agrees with the event stream, bundle state, fact-write results, omissions, and cleanup report even when a session is interrupted or fails partway through.

**Why this priority**: Extracting orchestration must not weaken the evidence and cleanup guarantees established by the Deep Capture MVP and compatibility calibration.

**Independent Test**: Fault-injected runs compare the terminal result with every emitted artifact and event, and prove that observed data is retained while unobserved claims are never invented.

**Acceptance Scenarios**:

1. **Given** observations followed by an operational failure, **When** finalization completes, **Then** the observations remain available, the session is partial or failed as appropriate, and all authorities report the same terminal outcome.
2. **Given** a cleanup failure, **When** the session returns, **Then** the failed resource and remediation are visible and the overall result cannot be reported as complete.
3. **Given** no evidence for a compatibility claim, **When** fact persistence runs, **Then** no affirmative fact is written and the omission or inconclusive outcome remains explicit.

---

### User Story 4 - Keep the CLI as a Compatible Adapter (Priority: P2)

As a command-line user or script author, I can use the existing Deep Capture command, options, prompts, structured events, exit codes, and bundle formats without behavior drift while the CLI delegates all Deep Capture decisions to the public library.

**Why this priority**: The architecture correction is successful only if the shipped interface remains compatible and stops owning duplicate business rules.

**Independent Test**: The existing CLI contract suite passes unchanged in meaning, and targeted tests prove CLI code maps arguments, confirmation input, presentation, and terminal status without reimplementing session policy.

**Acceptance Scenarios**:

1. **Given** any previously supported Deep Capture invocation, **When** it runs after extraction, **Then** its validation class, side-effect ordering, events, artifacts, and exit status remain compatible.
2. **Given** a library event or terminal report, **When** the CLI presents it, **Then** human and structured output contain the same facts without the CLI reclassifying the result.
3. **Given** a new injected backend implementation, **When** it is selected through library configuration, **Then** the CLI needs only argument-to-configuration mapping and presentation wiring.

### Edge Cases

- Preflight succeeds but proxy startup fails before any observation.
- Trust succeeds but launch or Capture fails, requiring trust rollback and partial-bundle truth.
- The operator interrupts during launch, observation, shutdown, finalization, or cleanup.
- An adapter exceeds its advertised deadline or cannot confirm resource release.
- Fact persistence succeeds but bundle finalization fails, or bundle writing succeeds while a fact append fails.
- Event delivery fails or declines an event while session cleanup still must complete.
- A consumer calls lifecycle methods out of order or attempts to reuse a terminal session.
- A replacement adapter reports observations with missing or conflicting correlation anchors.
- No application observation is produced although Capture or proxy execution succeeded.

## Requirements

### Functional Requirements

- **FR-001**: The public library MUST let a consumer configure, preflight, start, observe, stop, finalize, and clean up a Deep Capture session without invoking or depending on the CLI.
- **FR-002**: Preflight MUST resolve every side-effect-free decision and return a prepared session value that is consumed without repeating target or launch resolution after side effects begin. Caller authorization MUST bind to that prepared plan's stable identifier; stale or mismatched authorization MUST be refused before effects.
- **FR-003**: The public surface MUST use typed configuration, lifecycle events, refusals, failures, observations, cleanup results, fact-write results, and terminal reports; consumers MUST NOT parse human output to recover session state.
- **FR-004**: Proxy execution, trust management, launch preparation, Capture execution, bundle writing, compatibility fact persistence, clock and identifier generation, and event delivery MUST each have a narrow substitutable boundary.
- **FR-005**: The library MUST provide the production implementations needed by the shipped Deep Capture path, while controlled tests MAY replace every external or privileged effect.
- **FR-006**: The coordinator MUST enforce the existing order: side-effect-free validation and preparation, plan emission and confirmation where calibration requires it, proxy startup, optional trust, managed launch and Capture, observation correlation, bounded proxy shutdown, compatibility fact-write attempts, every resource cleanup attempt, bundle finalization from one immutable terminal snapshot, and terminal reporting.
- **FR-007**: Every side effect MUST remain explicit, target-scoped, session-owned, bounded, reversible where applicable, and represented in events or the terminal report.
- **FR-008**: No path MAY use system-wide proxy mutation, silent certificate trust, certificate-pinning bypass, target memory access, injection, hooking, target key extraction, executable modification, Winsock modification, or an interception driver.
- **FR-009**: Capture packet acquisition and flow attribution MUST remain separate and MUST continue to use the existing ordinary Capture composition rather than a Deep Capture-specific packet pipeline.
- **FR-010**: `fragcap-core` MUST gain no platform dependency, I/O dependency, proxy concern, trust concern, or target-store concern.
- **FR-011**: The selected target entry MUST remain the only owner of compatibility facts, and repeated or conflicting observations MUST remain append-only evidence rather than an aggregate verdict.
- **FR-012**: The coordinator MUST retain observations collected before interruption or failure and MUST attempt every independent finalization and cleanup action that remains safe and applicable.
- **FR-013**: Every omitted artifact, skipped action, failed action, and cleanup result MUST be represented with a stable reason; silence MUST NOT become an affirmative compatibility claim.
- **FR-014**: The terminal report, terminal event, manifest, compatibility record, fact-write record, and cleanup record MUST agree on session state and outcome within their respective authority. Bundle finalization MUST consume one immutable terminal snapshot created only after compatibility fact-write and resource-cleanup attempts have been recorded.
- **FR-015**: Lifecycle methods MUST reject invalid ordering and session reuse with typed errors and no duplicated side effects.
- **FR-016**: Adapter failures and event-delivery failures MUST NOT prevent bounded cleanup attempts or erase previously collected evidence.
- **FR-017**: The existing Deep Capture command grammar, prompts, output streams, structured event names and fields, exit-code classes, bundle schema, compatibility facts, and safety refusals MUST remain compatible unless a separately recorded correctness defect requires a change.
- **FR-018**: The CLI MUST own only argument parsing, interactive input, mapping to public library values, presentation, and process exit mapping; it MUST NOT own Deep Capture classification, ordering, persistence, cleanup, or artifact policy.
- **FR-019**: Public API documentation MUST explain lifecycle ordering, ownership, deadlines, sensitive artifacts, failure semantics, adapter obligations, and which types are stable integration contracts.
- **FR-020**: Automated tests MUST exercise the public API directly and through the CLI, including success, preflight refusal, interruption, partial evidence, backend failure, fact-write failure, bundle failure, cleanup failure, and invalid lifecycle order.
- **FR-021**: The public API MUST remain usable with no capture driver, elevation, game account, remote service, or real trust-store mutation when controlled adapters are supplied.
- **FR-022**: This slice MUST NOT adopt a native proxy backend, add direct-executable managed launch, broaden supported protocols, change target storage, or add a second Deep Capture command.

### Key Entities

- **Session Configuration**: The immutable operator intent and bounded options required to prepare one Deep Capture session.
- **Prepared Session**: A side-effect-free validated session containing resolved target, launch case, destinations, deadlines, and adapter-ready execution inputs.
- **Session Coordinator**: The single owner of lifecycle ordering, state transitions, failure accumulation, finalization, cleanup, and terminal reporting.
- **Adapter Set**: Narrow capabilities for proxy, trust, launch, Capture, observation, facts, bundles, time, identifiers, and event delivery.
- **Lifecycle Event**: A typed observation of a plan, transition, resource action, application observation, cleanup result, or terminal result.
- **Terminal Report**: The authoritative in-memory result joining outcome, artifacts, fact writes, omissions, cleanup, and failures.
- **Compatibility Observation**: A proxy-side observation plus optional packet-flow and process correlation, preserved without an inferred aggregate verdict.

## Success Criteria

### Measurable Outcomes

- **SC-001**: A controlled consumer completes 100 percent of the Deep Capture lifecycle through the public library without invoking or depending on the CLI.
- **SC-002**: Every supported success and failure scenario produces a typed terminal report and zero scenario requires parsing human-readable output.
- **SC-003**: Automated fault-injection covers at least one failure at every side-effecting lifecycle stage and proves every applicable cleanup action is attempted.
- **SC-004**: All existing command and bundle contract tests retain their expected behavior, with zero unrecorded compatibility changes.
- **SC-005**: The CLI contains zero Deep Capture business-rule implementations for classification, lifecycle ordering, fact selection, bundle authority, or cleanup policy.
- **SC-006**: Direct public-API tests run successfully on an unelevated machine with no capture driver, game, remote service, or trust-store mutation.
- **SC-007**: Dependency, neutral-target, documentation, encoding, licensing, minimum-toolchain, and full repository gates all pass.
- **SC-008**: No prohibited instrumentation or system-wide networking change is introduced, as established by the repository’s mechanical lint and dependency gates plus targeted safety tests.

## Assumptions

- The existing v0.7.0 Deep Capture command, bundle, event, compatibility-fact, and cleanup contracts are the behavioral baseline.
- The `fragcap` facade is the public product boundary and may own orchestration that composes existing concrete crates; a new crate is justified only if it preserves dependency direction without sibling coupling.
- The current external `mitmdump` backend remains the production backend in this slice.
- Compatibility calibration and ordinary Deep Capture share one coordinator and differ through typed configuration, not separate orchestration paths.
- Interactive confirmation remains a caller-supplied decision because a reusable library does not own a terminal.
- The next native backend and direct-executable launch slices consume the boundaries established here but remain out of S098.
