# Feature Specification: Native Proxy Backend Spike

**Feature Branch**: `codex/099-native-proxy-spike`

**Created**: 2026-08-29

**Status**: Complete

**Input**: User description: "Kick off S099", implementing issue #253, run the non-shipping native proxy backend spike and make the post-MVP backend decision from measured evidence.

## User Scenarios & Testing

### User Story 1 - Measure the Native Candidate (Priority: P1)

As a fragcap maintainer, I can run one controlled Windows experiment against the native proxy candidate and observe its lifecycle, protocol, certificate, cache, and analyzer behavior, so the backend decision rests on reproduced evidence rather than documentation claims.

**Why this priority**: The candidate cannot be considered for adoption until its core behavior is demonstrated on fragcap's target platform under fragcap-owned scope and cancellation.

**Independent Test**: Run the controlled candidate harness on Windows and verify that it binds only to loopback, accepts explicit proxy traffic, records the required protocol observations without silent loss, separates certificate authority material from trust, reports cache behavior, and shuts down through an owned cancellation signal.

**Acceptance Scenarios**:

1. **Given** an authorized local test service and no system proxy change, **When** the native candidate handles the controlled traffic matrix, **Then** the evidence records lifecycle results and every observed or missing request, response, handshake, and message class.
2. **Given** an active candidate run, **When** the harness requests cancellation, **Then** the listener and every owned resource stop within the declared bound and the result records any residue.
3. **Given** candidate-owned certificate material, **When** HTTPS traffic is tested, **Then** generation or import remains distinct from operating-system trust and no silent trust mutation occurs.

---

### User Story 2 - Compare Against the Shipped Baseline (Priority: P1)

As a fragcap maintainer, I can send equivalent controlled traffic through the native candidate and the shipped external baseline, so protocol visibility, HAR-source evidence, key-log behavior, and lifecycle differences are concrete.

**Why this priority**: A candidate result in isolation does not show whether replacing the current backend preserves the observations Deep Capture already relies on.

**Independent Test**: Execute the same HTTP, HTTPS, HTTP/2, and WebSocket scenarios through both backends and produce a normalized comparison that distinguishes observed, unsupported, failed, and not measured outcomes without treating absence as success.

**Acceptance Scenarios**:

1. **Given** the same local endpoints and request corpus, **When** both backend runs complete, **Then** their evidence is comparable by scenario and names body, protocol, message, HAR-source, key-log, shutdown, and cleanup outcomes separately.
2. **Given** a baseline tool is unavailable or a scenario cannot be exercised, **When** the comparison is finalized, **Then** the limitation is explicit and cannot be reported as parity.

---

### User Story 3 - Audit Repository Compatibility (Priority: P1)

As a maintainer responsible for releases, I can inspect the candidate's exact dependency, license, minimum-toolchain, target-conditional, and build-time impact without adding it to the shipped workspace graph.

**Why this priority**: A technically capable proxy is unusable if its dependency graph violates the license allowlist, silently breaks the Rust 1.82 policy, or creates an unreviewed release obligation.

**Independent Test**: Resolve the exact candidate feature set in an isolated spike, inspect normal and target-conditional dependency paths, run the repository license policy, build on the pinned and minimum toolchains, and record reproducible counts and timings.

**Acceptance Scenarios**:

1. **Given** the isolated candidate manifest, **When** dependency and license audits run, **Then** every resolved package and target-conditional root-store path is accounted for with exact commands and retained outputs or summaries.
2. **Given** Rust 1.82 and the pinned toolchain, **When** the spike is checked and built, **Then** each result is recorded independently and no missing gate is interpreted as compatibility.
3. **Given** the completed spike, **When** the released workspace metadata is inspected, **Then** no candidate package or spike-only dependency appears in the product graph or release artifacts.

---

### User Story 4 - Record One Backend Decision (Priority: P2)

As a project owner, I receive a dated evidence-backed decision that selects exactly one follow-up boundary: adopt the candidate, patch or fork it, evaluate the smaller native fallback, or retain the external baseline.

**Why this priority**: The spike is useful only if its findings close the open backend question and prevent multiple speculative adoption paths from proceeding at once.

**Independent Test**: Review the evidence table against every acceptance criterion and verify that the decision names one outcome, its rationale, remaining risks, and the exact scope of one follow-up issue without changing the shipping backend.

**Acceptance Scenarios**:

1. **Given** complete measured evidence, **When** the decision record is reviewed, **Then** it traces every conclusion to a result and identifies one follow-up path.
2. **Given** a hard blocker or inconclusive proof point, **When** the decision is recorded, **Then** the uncertainty remains visible and the record does not recommend product adoption without the missing proof.

### Edge Cases

- The native candidate builds on the pinned toolchain but fails on Rust 1.82.
- A target-conditional package appears in metadata but not in the active Windows dependency tree.
- HTTP/2 is negotiated differently between direct TLS and CONNECT traffic.
- WebSocket upgrade is visible while message frames are absent or transformed.
- A request or response body is streamed, decoded, compressed, empty, truncated, or larger than an imposed observation bound.
- Certificate cache entries outlive the listener or cannot be enumerated and removed deterministically.
- Key logging is possible for upstream proxy TLS but not for the client-facing proxy-owned TLS session.
- Cancellation stops accepting connections while existing connections remain alive past the deadline.
- The external baseline is not installed or behaves differently across versions.
- A controlled scenario fails before reaching either backend, making comparison invalid.

## Requirements

### Functional Requirements

- **FR-001**: The spike MUST evaluate the exact native candidate and feature set selected by the accepted Deep Capture backend research and MUST record any necessary deviation.
- **FR-002**: The spike MUST remain outside the released workspace dependency graph and MUST NOT change the shipping backend, product feature defaults, installer, or release artifacts.
- **FR-003**: The candidate harness MUST bind only to a loopback address, accept only explicitly proxied controlled traffic, and perform no system-wide proxy mutation.
- **FR-004**: The candidate harness MUST support fragcap-owned bounded startup, cancellation, connection draining, shutdown, and residue reporting.
- **FR-005**: Equivalent controlled scenarios MUST measure HTTP/1.1 requests and responses, HTTPS through CONNECT, HTTP/2 through CONNECT, and WebSocket handshake and message visibility for the candidate and external baseline.
- **FR-006**: Body evidence MUST distinguish complete, empty, bounded, truncated, decoded, unsupported, failed, and not measured outcomes; no loss or transformation MAY be silent.
- **FR-007**: The spike MUST establish whether application observations contain sufficient information to generate fragcap-owned HAR records without delegating HAR authority to the backend.
- **FR-008**: Certificate authority generation or import MUST remain separate from operating-system trust installation, and the spike MUST NOT install trust silently or bypass certificate pinning.
- **FR-009**: Certificate cache ownership, location, bounds, logging, lifetime, and cleanup behavior MUST be measured and any unowned or unbounded state MUST be reported.
- **FR-010**: Proxy-owned TLS key-log feasibility MUST be proven through public candidate interfaces, proven to require a maintained patch or fork, or explicitly deferred with evidence that names which TLS side is inaccessible.
- **FR-011**: The audit MUST record the exact resolved dependency delta, normal and target-conditional paths, direct and transitive licenses, root-store packages, source provenance, and repository allowlist result.
- **FR-012**: The audit MUST measure candidate behavior on Rust 1.82 and the pinned toolchain and treat any minimum-toolchain exception as a proposed explicit policy decision.
- **FR-013**: The audit MUST record reproducible clean and warm build timings and the resulting artifact or build-cache sizes needed to evaluate release cost.
- **FR-014**: Evidence MUST name the operating system, architecture, tool versions, candidate and baseline versions, commands, scenario inputs, expected observations, actual observations, and limitations.
- **FR-015**: The comparison MUST use one normalized result vocabulary and MUST NOT infer parity from missing, skipped, or inconclusive observations.
- **FR-016**: The final research record MUST choose exactly one of four outcomes: adopt, patch or fork, evaluate the smaller native fallback, or retain the external baseline.
- **FR-017**: The decision MUST define one bounded follow-up issue and MUST NOT file or implement multiple speculative backend paths.
- **FR-018**: No path MAY add injection, hooks, target memory reads, executable modification, Winsock changes, interception drivers, target TLS key extraction, certificate-pinning bypass, or silent system-wide proxying.
- **FR-019**: All committed spike artifacts MUST exclude captured credentials, session tokens, private keys, raw operator traffic, machine-specific paths, and addresses attributable to the operator.
- **FR-020**: Repository verification MUST prove the shipped workspace and release graph remain unchanged apart from research documents, isolated spike material, tests for the isolated harness, and an unreleased changelog record.

### Key Entities

- **Controlled Scenario**: One locally generated protocol interaction with declared inputs, expected observations, time bounds, and cleanup expectations.
- **Backend Run**: One candidate or baseline execution over the controlled scenario matrix, including version, environment, lifecycle events, observations, failures, and cleanup.
- **Observation Result**: A normalized status and evidence set for one protocol or lifecycle proof point, preserving unsupported and unknown states.
- **Dependency Audit**: The exact package, feature, target, license, toolchain, build-time, and size evidence for the isolated candidate graph.
- **Backend Decision**: The dated conclusion that selects one follow-up path and traces it to the complete evidence set.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Every one of the 12 issue acceptance criteria has a recorded pass, fail, unsupported, or not-measured result with evidence; zero criteria are left implicit.
- **SC-002**: The same controlled HTTP, HTTPS, HTTP/2, and WebSocket scenario set runs against both backends, and every scenario has a normalized comparison row.
- **SC-003**: Every observed HTTP/1.1 request and response body is accounted for as complete or with an explicit non-complete reason; zero silent truncations or omissions remain.
- **SC-004**: Candidate startup and shutdown are each exercised at least 10 times, including active-connection cancellation, with every deadline miss or residue recorded.
- **SC-005**: One hundred percent of resolved direct, transitive, and target-conditional packages are included in the dependency and license audit.
- **SC-006**: Rust 1.82 and pinned-toolchain results, clean and warm build timings, package-count delta, and size impact are all reproducible from recorded commands.
- **SC-007**: The released workspace resolves with zero native-candidate packages before and after the spike.
- **SC-008**: The final record selects exactly one follow-up outcome and cites evidence for every deciding factor.
- **SC-009**: No test changes system proxy state, silently installs trust, contacts an uncontrolled remote service, or retains private key material in committed output.

## Assumptions

- S098's public proxy adapter is the future product integration boundary, but S099 does not integrate the candidate with the shipping coordinator.
- Version-pinned candidate and baseline behavior is more useful than testing an unbounded latest version.
- Controlled local services can represent the protocol and lifecycle properties required for the backend decision without using a game account or third-party endpoint.
- Private temporary certificate material and raw run logs may be generated locally during measurement, then summarized into sanitized committed evidence and removed.
- If a proof point cannot be completed on this machine, the result remains not measured and blocks an adoption decision unless another outcome is clearly supported.
