# Feature Specification: Stable Public Rust API

**Feature Branch**: `codex/132-public-rust-api`

**Created**: 2026-09-05

**Status**: Complete

**Input**: Work slice S132 implementing issue #330 after all native protocol, routing, artifact, recovery, evidence, performance, integration, supply-chain, and packaging contracts have landed.

## User Scenarios & Testing

### User Story 1 - Run Every Shipped Deep Capture Capability as a Library Consumer (Priority: P1)

As a Rust library consumer, I need one documented public surface that reaches every shipped Deep Capture capability without invoking the fragcap command-line binary so I can embed the product in another authorized local tool.

**Why this priority**: The library is the product, and the completion gate cannot rely on command-private orchestration or backend implementation details.

**Independent Test**: Compile and run a standalone example against the public facade and production native backend in the controlled local lab, then verify that the same stable surface covers ordinary sessions, calibration, routing, supported protocols, artifacts, recovery, and terminal reporting.

**Acceptance Scenarios**:

1. **Given** the `deep-capture` feature and a controlled local target, **When** a consumer follows the published example, **Then** the production native backend completes local HTTP and TLS observations without invoking the CLI, reaching the Internet, changing host trust, or inspecting a target process.
2. **Given** any Deep Capture capability used by the shipped CLI, **When** the public API coverage contract is checked, **Then** that capability is reachable through the facade's declared stable surface.
3. **Given** a consumer that supplies custom effect adapters, **When** it prepares and drives a session, **Then** it uses documented facade traits and values rather than types owned by `fragcap-cli` or `fragcap-proxy`.

### User Story 2 - Depend on an Evolvable Compatibility Contract (Priority: P1)

As a crate author, I need explicit compatibility, ownership, concurrency, cancellation, and versioning guarantees so upgrades do not force me to infer which types or variants are stable.

**Why this priority**: Stabilization without an exact compatibility policy only converts accidental exports into accidental promises.

**Independent Test**: Build an external-consumer contract that imports only the declared stable surface, constructs supported inputs without exhaustive public-field literals, handles evolvable outputs without exhaustive matching, and checks the documented concurrency and cancellation properties.

**Acceptance Scenarios**:

1. **Given** a stable input configuration, **When** a compatible release adds an optional capability, **Then** existing consumers can continue constructing the input through supported constructors or builders.
2. **Given** an evolvable status or event enum, **When** a compatible release adds a variant, **Then** external consumers were already required to handle unknown future variants.
3. **Given** one coordinator and its adapter set, **When** ownership transfers or cancellation is requested, **Then** the contract states exactly which values may move between threads, which remain thread-confined, and where cooperative cancellation is observed.

### User Story 3 - Keep Backend Internals and CLI Policy Out of the Stable Surface (Priority: P2)

As a maintainer, I need the stable API to expose product concepts and narrow effect seams without committing backend caches, parser helpers, artifact writers, journal encoders, or command presentation types to semver compatibility.

**Why this priority**: The native backend and artifact formats must remain maintainable after the completion release without breaking consumers for internal refactors.

**Independent Test**: Compare the curated public contract with the facade exports and CLI imports, then verify that backend implementation modules are absent, the CLI consumes the stable facade path, and a repository gate rejects accidental stable-surface drift.

**Acceptance Scenarios**:

1. **Given** the stable module inventory, **When** backend implementation declarations are scanned, **Then** only explicitly supported production entry points appear and implementation-only types remain excluded.
2. **Given** the Deep Capture command implementation, **When** its imports are checked, **Then** all session policy values and traits come through the stable facade contract and no command-private business-rule type is needed.
3. **Given** a proposed public-surface change, **When** the contract test runs, **Then** unreviewed removal, rename, or accidental addition is reported before merge.

### Edge Cases

- A stable enum needs a new variant in a compatible release.
- A stable input needs a new optional field without breaking external construction.
- A consumer wants to move the coordinator across threads even though one injected adapter is thread-confined.
- Cancellation is requested while a blocking third-party adapter is inside one bounded call.
- A production native type necessarily appears at the facade boundary while its internal runtime lease, caches, and protocol machinery must remain hidden.
- A legacy top-level re-export exists but is not part of the curated stable inventory.
- The controlled example cannot bind IPv6 or another optional local capability on the current host.
- The public surface is available with `deep-capture` disabled or a required feature is omitted.

## Requirements

### Functional Requirements

- **FR-001**: The `fragcap` facade MUST publish one named, documented stable Deep Capture API surface for the first completion release.
- **FR-002**: Every shipped Deep Capture capability used by the CLI MUST be reachable through the stable public surface, including configuration, preparation, plan-bound authorization, lifecycle control, routing, native proxy selection, observations, classification, calibration, artifacts, recovery, readiness inputs, and terminal reports.
- **FR-003**: The stable surface MUST expose product-owned facade types and narrow adapter traits; it MUST NOT unnecessarily expose `fragcap-proxy` runtime leases, protocol engines, caches, parsers, writer internals, command presentation types, or command-private configuration.
- **FR-004**: The CLI MUST import Deep Capture product policy only through the stable facade surface, and executable checks MUST demonstrate that no CLI-only type is required to represent a shipped capability.
- **FR-005**: Stable input types MUST support external construction without requiring exhaustive public-field struct literals that prevent compatible additions.
- **FR-006**: Evolvable public enums and result shapes MUST be non-exhaustive or otherwise carry a documented compatibility mechanism; closed value sets MUST be identified explicitly when exhaustive matching is a compatibility promise.
- **FR-007**: The public contract MUST define semantic-version guarantees for the pre-1.0 completion release, including the stable surface, legacy exports, feature flags, schema versions, and documented exceptions for correctness or security defects.
- **FR-008**: The public contract MUST define `Send`, `Sync`, thread confinement, lease ownership, cleanup, and drop behavior for the coordinator, prepared plans, adapter traits, native backend entry points, reports, and cancellation handles.
- **FR-009**: Cancellation MUST be explicit, bounded, cooperative, and observable without unsafe preemption; a request received between adapter calls MUST prevent later non-cleanup effects, while an in-flight adapter remains obligated to honor its finite budget and cancellation contract.
- **FR-010**: A checked external-consumer contract MUST compile against the declared stable surface with the `deep-capture` feature and MUST fail when required stable exports, construction paths, trait guarantees, or non-exhaustive policies drift.
- **FR-011**: A documented example MUST compile and run the production native backend against the controlled local lab without invoking the CLI, requiring Internet access, changing host trust, using a game account, or retaining target data.
- **FR-012**: The example and contract tests MUST cover both success and cancellation or refusal behavior and MUST leave no listener, trust, route, process, or artifact residue.
- **FR-013**: Existing CLI behavior, bundle formats, protocol support, routing decisions, safety refusals, compatibility facts, and release package contents MUST remain compatible except for an explicitly recorded correctness fix.
- **FR-014**: The change MUST introduce no target instrumentation, system-wide proxy fallback, silent trust mutation, pinning bypass, hidden process, unbounded wait, uncounted loss, or new dependency package.
- **FR-015**: The master specification, outline, crate documentation, API contract, roadmap, and changelog fragment MUST agree on the stable boundary and MUST continue to state that Deep Capture remains incomplete until issue #334 closes.
- **FR-016**: S131's completed slice specification status MUST be corrected from Draft to Complete as merged housekeeping metadata.
- **FR-017**: The implementation MUST add no release tag, crate publication, scheduled task, soak run, or completion claim.

### Key Entities

- **Stable API Surface**: The reviewed facade module and exact export inventory that receives compatibility guarantees.
- **API Contract Version**: A machine-readable version for the curated surface, independent from artifact schema versions and package semantic version.
- **Session Builder**: The external construction path for immutable session intent and optional capabilities.
- **Cancellation Handle**: A cloneable cooperative signal that can be requested outside the coordinator and inspected at bounded lifecycle boundaries.
- **Adapter Set Builder**: The external construction path that gathers every required effect seam without exposing hidden fault-injection controls.
- **Stable Export Inventory**: The checked list of product types, traits, functions, constants, and production entry points intentionally covered by compatibility policy.
- **Controlled Native Example**: A no-CLI consumer of the production backend that runs only against bounded local test peers.

## Success Criteria

### Measurable Outcomes

- **SC-001**: One external-consumer contract reaches 100 percent of the Deep Capture capability groups used by the CLI through the stable facade surface.
- **SC-002**: Zero `fragcap-cli` Deep Capture imports bypass the declared stable facade path for product policy.
- **SC-003**: Zero backend lease, cache, parser, protocol-engine, artifact-writer, or command-presentation implementation types appear in the stable export inventory unless the contract records why the type is a necessary production entry point.
- **SC-004**: The controlled native example compiles and completes its local success path with zero Internet requests, zero real trust mutations, zero target-process inspection, and zero owned residue.
- **SC-005**: Contract tests prove all declared concurrency and cancellation guarantees, including at least one pre-effect cancellation and one cancellation observed between lifecycle stages.
- **SC-006**: Every stable enum is classified as evolvable or closed, a downstream compile-fail contract rejects exhaustive matching for the evolvable family, and every stable extensible input has a supported constructor or builder.
- **SC-007**: All focused tests and the full `cargo xtask ci` gate pass with zero ignored required checks.
- **SC-008**: The slice adds zero dependency or lockfile package and leaves the release and final Deep Capture completion actions untouched.

## Assumptions

- Issue #330 and master specification sections 2.1, 3.1, 8.2 through 8.7, 17.2.1, 24, 25, and 28.1 are the scope authorities.
- The curated stable module may coexist with legacy compatibility re-exports during the pre-1.0 transition, but only the curated inventory receives the explicit S132 compatibility guarantee.
- Thread confinement is an acceptable documented guarantee where forcing `Send` or `Sync` would exclude valid injected adapters or imply unsafe cancellation of blocking calls.
- Production native proxy entry types belong in the stable facade because consumers need them; their underlying `fragcap-proxy` implementation types do not.
- No dependency-based semantic-version checker is required if repository-owned compile contracts and an exact reviewed inventory make the promised boundary mechanical.
- Issue #331 owns final user documentation and issue #332 owns final CLI consent, progress, and recovery polish; S132 documents only what a Rust API consumer needs.
