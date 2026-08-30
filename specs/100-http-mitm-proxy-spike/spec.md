# Feature Specification: Smaller Native Proxy Fallback Spike

**Feature Branch**: `codex/100-http-mitm-proxy-spike`

**Created**: 2026-08-30

**Status**: Draft

**Input**: User description: "Kick off S100 as defined", implementing issue #274 by evaluating the smaller native proxy fallback against the S099 research contract.

## User Scenarios & Testing

### User Story 1 - Measure the Smaller Native Candidate (Priority: P1)

As a fragcap maintainer, I can run one controlled Windows experiment against the smaller native proxy candidate and observe its protocol, certificate, lifecycle, analyzer, and cleanup behavior, so the fallback decision rests on reproduced evidence.

**Why this priority**: S099 found a hard dependency and toolchain blocker in the first native candidate. The selected follow-up is valuable only if the smaller fallback is measured against the same standard.

**Independent Test**: Run the controlled candidate harness on Windows and verify that it binds only to loopback, accepts explicit proxy traffic, records every required observation or absence, separates certificate authority material from trust, and stops within declared bounds.

**Acceptance Scenarios**:

1. **Given** authorized local services and no system proxy change, **When** the smaller native candidate handles the controlled matrix, **Then** every required request, response, handshake, message, HAR-source, key-log, certificate, and lifecycle proof point has an explicit result.
2. **Given** an active candidate run, **When** cancellation is requested, **Then** the listener and owned resources stop within the declared bound and any residue is reported.
3. **Given** candidate-owned certificate material, **When** HTTPS traffic is tested, **Then** generation and import remain distinct from operating-system trust and no validation bypass or silent trust mutation occurs.

---

### User Story 2 - Compare All Measured Backends (Priority: P1)

As a fragcap maintainer, I can compare the smaller fallback with the S099 native candidate and the external baseline using one evidence vocabulary, so capability and fidelity differences remain concrete.

**Why this priority**: A fallback result in isolation cannot show whether it improves the blocker without losing behavior already demonstrated by another backend.

**Independent Test**: Execute the S099 controlled scenarios through the smaller fallback, normalize its results against the committed S099 evidence, and verify that missing, unsupported, failed, partial, and complete outcomes never collapse into parity.

**Acceptance Scenarios**:

1. **Given** equivalent local inputs, **When** fallback evidence is compared with S099 candidate and baseline evidence, **Then** protocol, body, WebSocket, HAR-source, key-log, lifecycle, and cleanup results are aligned by required proof-point key.
2. **Given** a scenario unsupported by the fallback, **When** comparison is finalized, **Then** the unsupported result remains visible and cannot be reported as parity.
3. **Given** two complete observations with different protocol, length, or digest, **When** comparison is finalized, **Then** parity is false.

---

### User Story 3 - Audit Repository Compatibility (Priority: P1)

As a maintainer responsible for releases, I can inspect the fallback's exact dependency, license, advisory, minimum-toolchain, target-conditional, and build impact without adding it to the shipped workspace graph.

**Why this priority**: The fallback exists to answer the blocker found in S099. It is not viable if the smaller graph still violates licensing, advisories, Rust 1.82 parsing, or release boundaries.

**Independent Test**: Resolve the exact fallback feature set in an isolated spike, inspect normal and target-conditional paths, run the repository-equivalent policy audit, build on minimum and pinned toolchains, and record reproducible counts, timings, and sizes.

**Acceptance Scenarios**:

1. **Given** the isolated candidate manifest, **When** dependency, source, advisory, and license audits run, **Then** every resolved package and target-conditional root-store path is accounted for.
2. **Given** Rust 1.82 and the pinned toolchain, **When** the spike is parsed, checked, and built, **Then** each result is recorded independently and no missing gate is interpreted as compatibility.
3. **Given** the completed spike, **When** released workspace metadata is inspected, **Then** no fallback package or spike-only dependency appears in the product graph or release artifacts.

---

### User Story 4 - Close the Backend Decision (Priority: P2)

As a project owner, I receive one dated evidence-backed backend outcome and no speculative follow-up tree.

**Why this priority**: S100 is the single fallback path selected by S099. Its result must close the comparison rather than creating another unbounded research branch.

**Independent Test**: Review every issue criterion against the evidence and verify that the decision selects exactly one backend outcome, states its rationale and remaining obligations, leaves the shipped backend unchanged, and files no additional speculative backend issue.

**Acceptance Scenarios**:

1. **Given** complete measured and audit evidence, **When** the decision is recorded, **Then** every deciding claim traces to a result and exactly one backend outcome is selected.
2. **Given** a hard blocker or inconclusive proof point, **When** the decision is recorded, **Then** uncertainty remains visible and adoption is not recommended without the missing proof.

### Edge Cases

- The fallback parses on Rust 1.82 but one transitive package does not compile there.
- The fallback supports HTTPS interception but cannot expose both request and response bodies.
- HTTP/2 works client-facing but downgrades upstream, or is unsupported on one side.
- WebSocket upgrade succeeds while no public frame hook exists.
- The certificate signer has unbounded or unobservable state.
- Proxy-owned TLS key logging is unavailable through public interfaces.
- A target-conditional package introduces a disallowed license or bundled root store.
- A current advisory affects an active dependency path, while another advisory affects only an inactive target path.
- Cancellation stops accepting connections while an existing connection exceeds the deadline.
- The external baseline is unavailable, leaving only committed S099 evidence for comparison.
- A controlled scenario fails before reaching the fallback, making the result not measured rather than failed backend behavior.

## Requirements

### Functional Requirements

- **FR-001**: The spike MUST evaluate `http-mitm-proxy` 0.18.0 and MUST record any necessary deviation from that exact candidate or version.
- **FR-002**: The spike MUST remain outside the released workspace graph and MUST NOT change the shipped `mitmdump` backend, product defaults, installer, or release artifacts.
- **FR-003**: The fallback harness MUST bind only to loopback, accept only explicitly proxied controlled traffic, contact no uncontrolled remote service, and perform no system-wide proxy mutation.
- **FR-004**: The fallback harness MUST provide bounded startup, cancellation, connection shutdown, and residue reporting under fragcap ownership.
- **FR-005**: The S099 matrix MUST be reused or adapted to measure HTTP/1.1 bodies, HTTPS through CONNECT, HTTP/2 through CONNECT, WebSocket handshake and messages, HAR-source fields, certificate authority and trust separation, bounded certificate state, proxy-owned TLS key logging, and cleanup.
- **FR-006**: Every required proof point MUST produce exactly one normalized complete, partial, empty, bounded, truncated, unsupported, failed, or not-measured result; missing rows MUST NOT disappear.
- **FR-007**: Complete observations MAY have parity only when protocol, byte length, and digest agree.
- **FR-008**: The spike MUST establish whether public fallback observations contain enough information for fragcap-owned HAR generation.
- **FR-009**: Certificate generation or import MUST remain separate from operating-system trust installation, and the spike MUST NOT silently install trust or disable certificate validation.
- **FR-010**: Certificate state ownership, bounds, lifetime, logging, and cleanup MUST be measured, and any unowned or unbounded state MUST be reported.
- **FR-011**: Proxy-owned client-facing TLS key-log feasibility MUST be proven through public fallback interfaces or recorded as unsupported or not measured with the inaccessible boundary named.
- **FR-012**: The audit MUST record exact normal and target-conditional dependency paths, direct and transitive licenses, sources, root-store packages, advisories, and repository allowlist results.
- **FR-013**: The audit MUST independently measure manifest parsing, checking, and building on Rust 1.82 and the pinned toolchain.
- **FR-014**: The audit MUST record reproducible clean and warm build timings, package counts, artifact or cache size, and released-workspace graph isolation.
- **FR-015**: Evidence MUST name the operating system, architecture, tool versions, candidate and baseline versions, commands, fixed inputs, expected observations, actual observations, and limitations.
- **FR-016**: The final record MUST compare the fallback with the committed S099 `hudsucker` evidence and external `mitmdump` baseline.
- **FR-017**: The final record MUST select exactly one backend outcome and MUST NOT file or implement additional speculative backend paths.
- **FR-018**: No path MAY add injection, hooks, target memory reads, executable modification, Winsock changes, interception drivers, target TLS key extraction, certificate-pinning bypass, or silent system-wide proxying.
- **FR-019**: Committed artifacts MUST exclude captured credentials, tokens, private keys, raw operator traffic, machine-specific paths, ephemeral ports, and operator-attributable addresses.
- **FR-020**: Repository verification MUST prove the shipped workspace and release graph remain unchanged apart from research documents, isolated spike material, isolated tests, and an unreleased decision fragment.

### Key Entities

- **Controlled Scenario**: One locally generated protocol interaction with fixed inputs, expected observations, time bounds, and cleanup expectations.
- **Fallback Run**: One execution of the smaller candidate over the controlled matrix, including version, environment, observations, failures, and cleanup.
- **Observation Result**: One required proof point with a normalized status and fidelity evidence.
- **Three-Way Comparison**: Required proof points aligned across the fallback, S099 native candidate, and external baseline without inferring absent values.
- **Dependency Audit**: Exact package, feature, target, source, license, advisory, toolchain, timing, and size evidence for the isolated fallback graph.
- **Backend Outcome**: The dated conclusion selecting one backend path without creating additional speculative research branches.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Every issue acceptance criterion has an explicit result and supporting evidence; zero criteria are implicit.
- **SC-002**: Every S099 required proof point appears in the fallback comparison even when all compared backends omit it.
- **SC-003**: Every observed request, response, and WebSocket payload is accounted for by length and digest or an explicit non-complete reason; zero silent transformations remain.
- **SC-004**: Candidate startup and shutdown are exercised at least 10 times, including active-connection cancellation, with every deadline miss or residue recorded.
- **SC-005**: One hundred percent of resolved direct, transitive, and target-conditional packages appear in the dependency, license, source, and advisory audit.
- **SC-006**: Rust 1.82 and pinned-toolchain parse, check, and build results, clean and warm timings, package counts, and size impact are reproducible from recorded commands.
- **SC-007**: The released workspace resolves with zero fallback packages before and after the spike.
- **SC-008**: The final record compares all three backends and selects exactly one backend outcome with evidence for every deciding factor.
- **SC-009**: No test changes system proxy state, disables certificate validation, silently installs trust, contacts an uncontrolled remote service, or retains private material in committed output.

## Assumptions

- S099's normalized evidence contract is the comparison authority and may be extended only to represent the third backend without weakening its completeness or fidelity rules.
- Version-pinned fallback behavior is more useful than testing an unbounded latest release.
- Controlled loopback services represent the protocol and lifecycle properties needed for this decision without a game account or third-party endpoint.
- Private temporary certificate material and raw run logs may be generated locally, summarized into sanitized evidence, and removed before completion.
- If a proof point cannot be completed on this machine, it remains not measured and cannot support parity or adoption.
