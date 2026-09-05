# Feature Specification: Native Supply-Chain and Compatibility Gate

**Feature Branch**: `codex/130-native-supply-chain`

**Created**: 2026-09-05

**Status**: Complete

**Input**: Work slice S130 implementing issue #328 after the native dependency graph selected by #280 became final.

## User Scenarios & Testing

### User Story 1 - Block Unsafe Dependency Drift (Priority: P1)

As a release maintainer, I need one mandatory gate over the complete locked dependency graph so a dependency, feature, source, target, license, advisory, toolchain, duplicate-major, or unsafe-code policy regression cannot reach a release unnoticed.

**Why this priority**: The native proxy graph handles TLS, HTTP, cryptography, asynchronous I/O, and Windows effects. Release speed is useful only if the shipped graph remains inside the reviewed security and compatibility boundary.

**Independent Test**: Mutate one governed graph fact at a time and verify the ordinary pull-request gate identifies and rejects each unapproved change while the unchanged locked graph passes on supported targets.

**Acceptance Scenarios**:

1. **Given** the reviewed locked graph and policy, **when** the supply-chain gate evaluates every workspace feature and supported target edge, **then** it reports one complete passing result with no unclassified dependency or edge.
2. **Given** a dependency with a disallowed license, advisory, source, yanked or abandoned status, excessive Rust requirement, undeclared feature, prohibited duplicate major, or unreviewed unsafe posture, **when** the gate runs, **then** it fails and names the exact package, version, target or feature context, policy rule, and remediation class.
3. **Given** a Windows-only or optional dependency edge, **when** a non-Windows host runs the static gate, **then** that edge remains present in the reviewed inventory and cannot disappear from evaluation because it is inactive on the current host.

---

### User Story 2 - Govern Dependency Maintenance and Exceptions (Priority: P2)

As a maintainer responding to routine updates or an urgent security advisory, I need documented, mechanically checked procedures and finite exceptions so dependency maintenance is fast, reviewable, and cannot silently weaken policy.

**Why this priority**: A strict gate without a usable update and emergency process encourages either stale dependencies or ad hoc bypasses at the moment of highest risk.

**Independent Test**: Exercise the routine-update and emergency-patch procedures against controlled policy fixtures, including valid and expired exceptions, and verify each path produces a deterministic pass or actionable refusal without changing release authority.

**Acceptance Scenarios**:

1. **Given** a policy exception, **when** it is evaluated, **then** it is accepted only when it names an owner, rationale, exact scope, creation date, expiry date, and removal condition.
2. **Given** an expired, malformed, unused, or broader-than-needed exception, **when** the gate runs, **then** it fails rather than warning or silently retaining the exception.
3. **Given** a routine dependency update or urgent advisory, **when** a maintainer follows the documented procedure, **then** the graph, policies, release evidence, compatibility floor, and required verification are re-evaluated before merge.

---

### User Story 3 - Ship Auditable Dependency Evidence (Priority: P3)

As a release consumer or reviewer, I need an accurate machine-readable software bill of materials and concise third-party dependency notices in the official release outputs so I can identify what the release contains and its distribution obligations.

**Why this priority**: The evidence makes the policy boundary externally inspectable and fulfills release-distribution obligations, but it depends on the graph authority established by User Story 1.

**Independent Test**: Generate evidence from the locked release graph, independently validate every component against that graph, assemble the portable archive and installer inputs, and verify both outputs contain the validated files while refusing stale or incomplete evidence.

**Acceptance Scenarios**:

1. **Given** the locked release graph, **when** release evidence is generated, **then** every shipped third-party component appears exactly once with identity, version, source, license expression, dependency role, and applicable target or feature context.
2. **Given** generated evidence whose graph digest, component set, version, or required field is stale or altered, **when** release validation runs, **then** packaging fails before any release is published.
3. **Given** a valid release build, **when** the official portable archive and installer payload are assembled, **then** both contain the validated software bill of materials and third-party notices alongside the existing license and notice files.

### Edge Cases

- Advisory data is unavailable, invalid, or older than the permitted freshness window.
- A package is yanked or newly classified as abandoned without changing `Cargo.lock`.
- A dependency is reachable only through a Windows target clause, build dependency, development dependency, or optional feature.
- The same package name appears in multiple major versions for a justified and an unjustified reason.
- A package omits `rust-version`, declares a floor above the workspace MSRV, or resolves differently under the pinned development toolchain and claimed MSRV.
- A crate contains or transitively exposes unsafe code required for a platform binding or cryptographic implementation, while a newly introduced unsafe dependency has no review.
- A registry, Git, or path source is unknown, mutable, missing a checksum, or outside the workspace.
- A license expression offers both permitted and disallowed alternatives, uses a deprecated identifier, or lacks sufficient metadata for notice generation.
- A policy exception is expired, duplicated, unused, non-specific, or has a malformed date.
- Generated evidence is truncated, duplicated, nondeterministic, stale relative to `Cargo.lock`, or includes a package that is not part of the shipped release feature set.
- A security-critical exact pin changes without its cadence record and compatibility review changing with it.
- The release workflow can package or publish before supply-chain evidence validation succeeds.

## Requirements

### Functional Requirements

- **FR-001**: S130 MUST define one versioned, closed supply-chain policy that covers every package and dependency edge reachable from the complete workspace graph, including all features, Windows-only edges, build dependencies, development dependencies, and optional dependencies.
- **FR-002**: The ordinary pull-request gate MUST fail on disallowed or indeterminate licenses, known actionable advisories, yanked packages, disallowed abandoned packages, unknown or unapproved registries and Git sources, mutable external sources, and prohibited interception dependencies.
- **FR-003**: The gate MUST validate the workspace MSRV and pinned toolchain contract, reject a dependency that declares a higher Rust requirement than the supported floor without an approved isolation rule, and exercise published metadata and lockfile compatibility through the existing MSRV authority.
- **FR-004**: The gate MUST compare the exact resolved feature and target-edge inventory with reviewed policy and fail when a direct dependency enables an undeclared feature, changes default-feature posture, or introduces an unclassified platform edge.
- **FR-005**: The gate MUST identify simultaneous major-version lineages and reject each lineage not covered by an exact, finite policy record; patch and minor duplication MUST remain visible without being mislabeled as a major-lineage violation.
- **FR-006**: The gate MUST maintain an exhaustive unsafe-code posture for third-party packages, distinguish packages whose unsafe implementation is required and reviewed from packages expected to forbid unsafe code, and fail on an unreviewed package or posture change without claiming source-code proof the gate did not perform.
- **FR-007**: Security-critical direct dependencies MUST be exact-pinned or constrained by an equally strict reviewed rule and MUST carry a documented owner, review cadence, compatibility boundary, and emergency-update expectation.
- **FR-008**: Every policy exception MUST carry a unique identifier, owner, rationale, exact package and rule scope, creation date, expiry date, and removal condition; malformed, expired, duplicate, unused, or over-broad exceptions MUST fail the gate.
- **FR-009**: Advisory evaluation MUST fail closed when advisory data cannot be obtained or validated within the declared freshness policy, while distinguishing infrastructure failure from an advisory finding.
- **FR-010**: The repository MUST provide tested routine-update and emergency-security-patch procedures that preserve pull-request review, required CI, release authorization, compatibility validation, evidence regeneration, and rollback guidance.
- **FR-011**: The release process MUST generate a deterministic machine-readable software bill of materials from the exact locked release feature and target graph and MUST independently validate its component identity, version, source, license, dependency role, target context, feature context, and graph binding.
- **FR-012**: The release process MUST generate deterministic third-party notices from the same validated graph, including each distributed component's identity, version, declared license expression, source, and applicable attribution or notice reference without reproducing project-authored release notes.
- **FR-013**: Generated supply-chain evidence MUST carry schema version, fragcap version, source revision, lockfile digest, policy digest, generation time, tool identity, target identity, feature-set identity, completeness result, and deterministic component ordering.
- **FR-014**: The portable archive and installer payload MUST contain the validated software bill of materials and third-party notices, and the release workflow MUST prevent artifact publication and crate publication when generation or validation fails.
- **FR-015**: A static ordinary-CI authority MUST validate policy schema, closed vocabularies, package and edge coverage, exception governance, workflow ownership, release ordering, and required artifact contents without requiring network access or executing product, trust, routing, capture, or proxy effects.
- **FR-016**: A network-backed advisory authority MUST run on pull requests, `main`, manual dispatch, and a bounded recurring cadence; it MUST not create a desktop scheduled task, local recurring automation, or persistent background process.
- **FR-017**: The gate MUST emit bounded, actionable diagnostics and a public-safe summary that contains no credentials, private key material, payload content, certificate bytes, account identifiers, host names, or absolute operator paths.
- **FR-018**: The implementation MUST reuse the existing task runner, license rules, dependency audit workflow, release workflow, and MSRV authority rather than introduce a product runtime subsystem or dependency.
- **FR-019**: Changes to pinned workflows, dependency policy, toolchain enforcement, or release packaging MUST include a dated S130 decision fragment and trace to issue #328.
- **FR-020**: S130 MUST add no product runtime behavior, target process access, packet or proxy behavior, recurring local task, final packaging-completion claim, or Deep Capture feature-completion claim.

### Key Entities

- **Supply-Chain Policy**: The versioned closed authority for sources, licenses, advisories, compatibility, features, target edges, duplicate majors, unsafe posture, critical pins, cadence, and exceptions.
- **Resolved Graph Inventory**: The normalized package, edge, target, feature, role, checksum, license, Rust requirement, and source facts derived from the locked workspace graph.
- **Policy Exception**: A finite approval for one exact rule and package scope with owner, dates, rationale, and removal condition.
- **Critical Dependency Record**: Maintenance authority for a security-sensitive direct package or coordinated package family.
- **Supply-Chain Report**: The reconciled result of evaluating one graph inventory against one policy and advisory snapshot.
- **Software Bill of Materials**: Machine-readable release component evidence bound to the release graph and artifact identity.
- **Third-Party Notices**: Human-readable dependency identity, licensing, source, and attribution evidence generated from the same release graph.

## Success Criteria

### Measurable Outcomes

- **SC-001**: One successful static run classifies 100 percent of locked packages and dependency edges across the complete feature set and supported Windows target, with zero unknown packages, unknown sources, unclassified target edges, or unused exceptions.
- **SC-002**: Controlled mutations for every governed rule class produce a non-zero result naming the exact violating package, rule, and remediation class, with zero warning-only bypasses for release-blocking findings.
- **SC-003**: Every accepted exception has all seven required governance fields and the gate rejects it on the first run after expiry.
- **SC-004**: The software bill of materials and third-party notices reconcile to 100 percent of components in the locked shipped graph with zero missing, duplicate, unexpected, or stale entries.
- **SC-005**: Both portable archive and installer payload validation find exactly one current software bill of materials and one current third-party notices file, and release publication cannot start when either is absent or invalid.
- **SC-006**: Routine-update and emergency-patch procedure tests cover valid update, advisory finding, stale advisory data, expired exception, feature drift, target-only drift, and rollback paths.
- **SC-007**: Static supply-chain validation completes within 60 seconds on a normal developer machine, and the network-backed audit completes within 15 minutes in CI.
- **SC-008**: S130 adds zero product runtime dependencies and performs zero target, capture, proxy, trust-store, routing, or process effects.

## Assumptions

- Issue #280's selected native dependency graph, Rust 1.88 MSRV, and exact security-critical direct pins are the baseline S130 governs rather than redesigns.
- The complete policy inventory includes development, build, and test dependencies for repository risk, while release evidence includes only the normal runtime packages reachable from the shipped Windows feature set and excludes generator-only, build-only, and development-only components.
- Existing `cargo-deny` policy remains the network-backed license, advisory, ban, and source authority; S130 closes its trigger and configuration gaps and adds deterministic repository-owned validation around facts it does not model completely.
- Abandoned-package policy may use explicit reviewed classification because ecosystem abandonment signals are advisory metadata rather than an infallible fact; silence or unknown status is never converted into a claim of active maintenance.
- Packaging installation, upgrade, repair, uninstall, signatures, and final content certification remain S131 under issue #329; S130 supplies and wires the dependency evidence those checks consume.
- Final Deep Capture completion remains issue #334.
