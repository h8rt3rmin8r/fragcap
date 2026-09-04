# Feature Specification: Native Deep Capture Threat Model

**Feature Branch**: `codex/125-native-threat-model`

**Created**: 2026-09-04

**Status**: Draft

**Input**: User description: "Spec out and implement S125 under autopilot, closing the native Deep Capture threat-model and abuse-case-test boundary described by issue #323."

## User Scenarios & Testing

### User Story 1 - Audit Every Native Trust Boundary (Priority: P1)

A security reviewer can inspect one canonical model and find every native Deep Capture trust boundary, sensitive asset, threat, control owner, detection signal, containment boundary, evidence authority, and executable abuse-case test.

**Why this priority**: The controls already span routing, proxy protocols, launch ownership, certificates, artifacts, and recovery. A dispersed set of tests cannot prove that the whole shipped boundary was reviewed.

**Independent Test**: Validate the canonical model and confirm that every high-risk threat has complete prevention, detection, containment, evidence, and executable negative-test ownership.

**Acceptance Scenarios**:

1. **Given** a shipped native protocol or lifecycle path, **when** the model is inspected, **then** its trust boundaries, assets, threats, controls, evidence, and tests are present.
2. **Given** a high-risk threat with any missing control or test owner, **when** the repository gate runs, **then** it fails with the exact threat and missing field.
3. **Given** a referenced negative test is absent or ignored, **when** the repository gate runs, **then** it fails rather than accepting prose as proof.

---

### User Story 2 - Prove Abuse Fails Closed (Priority: P2)

An operator can rely on native Deep Capture to refuse unrelated clients, prohibited destinations, ambiguous framing, resource saturation, invalid certificate use, artifact escape, interrupted cleanup, and confused-deputy requests without silently opening a broader proxy path.

**Why this priority**: These failures cross the strongest security boundaries and can expose local network access, secrets, or unrelated traffic if they normalize into success.

**Independent Test**: Run the threat-owned negative tests and confirm each malicious or ambiguous input is refused, bounded, loss-accounted, or retained for exact recovery as specified.

**Acceptance Scenarios**:

1. **Given** an unrelated local client lacks the exact session capability, **when** it reaches a listener, **then** it is refused before payload forwarding and the refusal is counted.
2. **Given** a destination is private, listener-local, rebound, or otherwise outside policy, **when** connection is attempted, **then** no upstream connection occurs without an exact grant.
3. **Given** conflicting request framing or protocol ambiguity, **when** parsing occurs, **then** the request is refused without normalization or transparent fallback.
4. **Given** a finite resource limit is exhausted, **when** more work arrives, **then** existing ownership is preserved and the new work is visibly refused or dropped under its named counter.
5. **Given** cleanup is interrupted, **when** Doctor evaluates residue, **then** only exactly owned recoverable resources are offered for replay and unrelated state is preserved.

---

### User Story 3 - Force Review When the Attack Surface Changes (Priority: P3)

A maintainer cannot add a shipped native protocol family or change the direct native proxy dependency set without updating the threat model in the same change.

**Why this priority**: Protocol and dependency changes alter parser, transport, cryptographic, and resource-exhaustion assumptions. Review must be mechanically coupled to those changes.

**Independent Test**: Mutate controlled copies of the protocol and dependency inventories and confirm the gate reports drift until the model review identity is updated.

**Acceptance Scenarios**:

1. **Given** the shipped protocol-family inventory changes, **when** CI runs, **then** the threat-model gate fails until the reviewed inventory matches.
2. **Given** the direct `fragcap-proxy` dependency inventory changes, **when** CI runs, **then** the gate fails until the reviewed inventory matches.
3. **Given** the model and both inventories agree, **when** CI runs, **then** the gate prints the model version, covered threats, executable tests, protocol families, and dependencies.

### Edge Cases

- A threat identifier, boundary, asset, control, or test reference is duplicated.
- A high-risk row uses an empty string, unknown owner, missing evidence, or prose-only test claim.
- A test function exists only in a comment, is ignored, or is referenced ambiguously.
- A protocol family is renamed, added, or removed without model review.
- A direct dependency is optional, target-specific, promoted from dev, or removed.
- Malicious input uses mixed case, mapped addresses, dot segments, duplicate headers, conflicting lengths, unusual Unicode, or an oversized representation to hide its meaning.
- One threat spans several protocols but only one path has executable evidence.
- A residual risk is documented without explicit operator acceptance.

## Requirements

### Functional Requirements

- **FR-001**: The repository MUST carry one versioned canonical native Deep Capture threat registry and a human-readable threat-model document derived from the same reviewed scope.
- **FR-002**: The model MUST enumerate every trust boundary and sensitive asset for authorization, target ownership, routing, DNS and upstream access, protocol parsing, TLS and certificate authority, evidence retention, artifact export, lifecycle cleanup, and Doctor recovery.
- **FR-003**: The model MUST cover unrelated local clients, malicious targets and origins, DNS rebinding, SSRF, request smuggling and desynchronization, resource exhaustion, certificate abuse, artifact theft, cleanup interruption, and confused-deputy behavior.
- **FR-004**: Every threat MUST identify severity, applicable boundaries and assets, prevention, detection, containment, evidence authority, and test ownership.
- **FR-005**: Every high-risk threat MUST reference at least one executable negative test for each materially distinct shipped path it claims to cover. No high-risk row may pass through prose alone.
- **FR-006**: A residual-risk disposition MUST require an explicit recorded operator acceptance. S125 MUST infer none, so all high-risk rows in this slice MUST own executable negative evidence.
- **FR-007**: Test references MUST resolve to real, non-ignored Rust test functions in tracked source and MUST be unique within a threat row.
- **FR-008**: The gate MUST reject duplicate identifiers, unknown references, empty control ownership, unsupported vocabulary, and incomplete high-risk rows with deterministic diagnostics.
- **FR-009**: The model MUST bind to the exhaustive shipped native protocol-family inventory and fail when that inventory changes without a corresponding model review.
- **FR-010**: The model MUST bind to the direct runtime and target-specific dependency inventory of `fragcap-proxy` and fail when that inventory changes without a corresponding model review.
- **FR-011**: No listener or routing test may establish success without exact session authentication, target ownership, and upstream policy; no native path may behave as an open proxy.
- **FR-012**: Canonicalization and normalization MUST preserve malicious distinctions or refuse ambiguity; they MUST NOT turn invalid, conflicting, escaped, rebound, or oversized input into an allowed request.
- **FR-013**: Every refusal, saturation, discarded observation, and interrupted cleanup path MUST retain a named detection or evidence authority and MUST NOT silently disappear.
- **FR-014**: The P-1 prohibition MUST be reconfirmed for every routing and protocol family. S125 MUST add no injection, hook, target memory access, system-wide proxy mutation, executable modification, target key extraction, or target process handle.
- **FR-015**: The threat-model command MUST follow the repository 0/1/2 check contract and run inside `cargo xtask ci` without a game, account, Internet service, elevation, capture driver, or trust mutation.
- **FR-016**: Controlled validator tests MUST prove rejection of malformed registries, missing and ignored test references, protocol drift, and dependency drift.
- **FR-017**: Documentation MUST state when model review is required and distinguish completed native threat review from fuzzing, performance, Windows integration, packaging, supply-chain, and final acceptance work.
- **FR-018**: S125 MUST add no new third-party dependency or claim Deep Capture complete before issue #334.

### Key Entities

- **Threat Registry**: Versioned machine-readable inventory of reviewed threats and their owned mitigations and evidence.
- **Trust Boundary**: A transition where identity, authority, data, or lifecycle ownership must be re-established.
- **Sensitive Asset**: Capability, private key, decrypted content, routing authority, artifact, or recovery authority requiring protection.
- **Threat Row**: Stable risk identity linked to boundaries, assets, controls, evidence, and negative tests.
- **Review Inventory**: Exact shipped protocol families and direct native proxy dependencies covered by the current review.
- **Executable Evidence Reference**: A repository path and Rust test function that the gate proves exists and is not ignored.

## Success Criteria

### Measurable Outcomes

- **SC-001**: One hundred percent of enumerated high-risk threats carry prevention, detection, containment, evidence, and executable negative-test ownership.
- **SC-002**: One hundred percent of shipped native protocol families and direct `fragcap-proxy` dependencies match the reviewed inventories.
- **SC-003**: Every seeded incomplete row, missing or ignored test, protocol drift, and dependency drift causes a deterministic gate failure.
- **SC-004**: Controlled abuse cases establish zero unauthenticated forwarding, zero policy-bypassing upstream connections, zero ambiguous framing acceptance, and zero unrelated cleanup mutation.
- **SC-005**: The full repository verification gate passes with no new dependency package and no prohibited capability.

## Assumptions

- S103 through S124 own the native authorization, routing, protocol, artifact, lifecycle, classification, calibration, bypass, process-evidence, and Doctor authorities reviewed here.
- Existing negative tests remain valid evidence only when the registry names the exact test function and the gate proves it is present and enabled.
- Issues #324 through #329 own fuzzing, performance, Windows integration, packaging, supply-chain, and produced-artifact validation. Issue #334 owns final completion.

## Clarifications

### Session 2026-09-04

- High-risk residual risk is not implicitly acceptable. Every such row must bind to executable negative evidence in S125.
- The reviewed attack surface is the shipped native product through S124; later milestone work must update the model when it changes that surface.
- Protocol and direct proxy dependency review currency is executable CI state, not a prose reminder.

## Requirements Quality Checklist

- [x] Requirements describe security outcomes and review invariants rather than a chosen implementation layout.
- [x] Every mandatory abuse category from issue #323 is explicit.
- [x] High-risk evidence and residual-risk handling are unambiguous.
- [x] Scope exclusions preserve issues #324 through #334.
- [x] Success criteria are measurable and independently verifiable.
