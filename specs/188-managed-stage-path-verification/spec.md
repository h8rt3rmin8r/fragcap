# Feature Specification: S188 Managed stage path verification

**Feature Branch**: `codex/s188-managed-stage-path-verification`
**Created**: 2026-11-02
**Status**: Draft
**Input**: Owner-approved correction of active-release regression #547 after S187 and v0.10.12.

## User Scenarios & Testing

### User Story 1 - Acquire the exact managed client (Priority: P1)

An operator starts guided Deep Capture for a cold stored Steam target. The selected client starts beneath the owned platform within the authorized wait, but its creation event supplies only an executable name. Fragcap establishes the client's exact executable path from evidence bound to that process instance, matches the declared terminal stage, begins scoped packet capture, and retains target-owned frames when the controlled client sends traffic.

**Why this priority**: The active-release regression blocks the ordinary single-client Steam path and leaves a successful launch with no captured frames.

**Independent Test**: A controlled Windows production-path launch emits a basename-only client start under the owned Steam instance, sends target-owned traffic, and produces a pcapng with frames readable by an unmodified analyzer before the finite acquisition deadline.

**Acceptance Scenarios**:

1. **Given** an owned Steam root, a declared absolute client path, and a basename-only client start with independently verified path and creation identity, **when** the event is acquired, **then** the terminal client binds before the wait expires and scoped packet capture begins.
2. **Given** a client whose observed image and verified path disagree with the declared path, **when** it starts under the owned Steam root, **then** no client stage binds and the mismatch remains visible.
3. **Given** a PID whose current process object belongs to a later process instance, **when** the older creation event is evaluated, **then** its path cannot be used to bind the older event.

---

### User Story 2 - Keep ordered publisher stages exact (Priority: P1)

An operator's cold Steam target declares an ordered publisher chain before the final client. Each stage binds only when its own exact executable path and creation-event ancestry are established. A missing path, ambiguous process instance, or escaped parent remains an explicit acquisition limitation rather than a claimed match.

**Why this priority**: The same S187 exact-path synthesis applies to every declared intermediary, so correcting only the terminal client would leave publisher targets blocked.

**Independent Test**: A controlled Steam-to-publisher-to-client chain binds in order from basename-only starts, while wrong-path, stale-instance, escaped-parent, and missing-path variants bind no affected stage.

**Acceptance Scenarios**:

1. **Given** a declared ordered publisher chain with verified paths and creation-event ancestry, **when** its stages start with basename-only images, **then** each declared stage and the final client bind in order.
2. **Given** a declared stage whose path or ancestry cannot be established, **when** acquisition runs, **then** that stage does not bind and the result names an unverified or ambiguous identity.
3. **Given** a matching executable name outside the owned ancestry, **when** it starts, **then** it cannot satisfy a declared publisher or client role.

---

### User Story 3 - Explain failed acquisition precisely (Priority: P1)

When the final client is not acquired, the operator sees the actual acquisition stop reason in the top-level CLI result and diagnostics. The partial packet artifact, proxy observations, manifest, and cleanup report retain their own observed states without being overwritten by that reason.

**Why this priority**: The installed failure's outer error obscured `acquisitiontimeout` and made a zero-frame packet artifact appear to explain proxy exchanges it did not contain.

**Independent Test**: A controlled missing-path acquisition runs to its finite timeout and asserts the top-level reason, diagnostic limitation, partial manifest, zero-frame packet truth, independently retained proxy observations, and cleanup status.

**Acceptance Scenarios**:

1. **Given** no verified terminal stage by the finite deadline, **when** Deep Capture exits, **then** the top-level result and diagnostics retain `acquisitiontimeout` rather than only a generic child exit code.
2. **Given** a failed acquisition with proxy observations but no scoped frames, **when** artifacts finalize, **then** the packet, proxy, manifest, and cleanup records each report their own observed state.

### Edge Cases

- An ETW event supplies a basename while its command line contains an absolute path; the command line alone never establishes executable-path identity.
- A process exits before its path can be queried, access is denied, a PID is reused, or the creation time cannot be reconciled; the stage remains unverified.
- A process with the right path has the wrong parent, or a parent instance is ambiguous; ancestry never follows from name or PID alone.
- Startup snapshots and ordinary Capture keep their existing evidence semantics; a snapshot does not acquire a managed creation-event stage.
- Process evidence may be retained after a stage failure; a stage transition is recorded only for a real binding.

## Requirements

### Functional Requirements

- **FR-001 (#547)**: A basename-only managed start MUST be eligible for an exact path predicate only after Fragcap obtains a canonical executable path independently verified against that same process instance and creation event.
- **FR-002 (#547)**: Exact path and ordered ancestry requirements introduced by S187 MUST remain in force for the simple Steam client and every declared publisher stage; image name or command line alone MUST NOT satisfy a path predicate.
- **FR-003 (#547)**: A stale or reused PID, wrong canonical path, escaped parent, missing path, or ambiguous instance MUST NOT bind the affected stage and MUST produce a specific unverified or ambiguous acquisition diagnostic.
- **FR-004 (#547)**: The original event image and command line MUST remain distinguishable from subsequently verified process-path evidence in retained observations.
- **FR-005 (#547)**: A controlled Windows production-path regression MUST demonstrate basename-only exact-client acquisition before the finite deadline, active scoped packet capture, and a valid pcapng containing target-owned frames under an unmodified analyzer.
- **FR-006 (#547)**: The controlled regression MUST include wrong-path, stale PID or creation-time, escaped-parent, missing-path, and ordered publisher-chain countercases, and MUST exercise the synthesized platform profile through real stage matching and acquisition rather than only checking predicate presence.
- **FR-007 (#547)**: The top-level CLI result and diagnostics MUST retain the actual `acquisitiontimeout` stop reason when acquisition times out, with the same human and structured-result truth.
- **FR-008 (#547)**: Failed acquisition MUST preserve independent packet, proxy application, partial manifest, and cleanup facts without inventing packets, stage matches, or positive calibration.
- **FR-009**: The correction MUST remain bounded to existing managed launch and capture authority, with no target memory access, new process-control rights, private title evidence in public artifacts, or release publication in this slice.

### Key Entities

- **Observed start**: The PID, creation event time, parent, image text, and command-line availability reported by the event source.
- **Verified process path**: A canonical executable path with a provenance and process-instance join that can be evaluated against a declared exact path.
- **Managed stage binding**: One declared role matched to a process instance only after path and ordered ancestry checks succeed.
- **Acquisition limitation**: A named reason a candidate stage could not be verified, separate from a stage match and from the terminal stop reason.
- **Capture outcome**: The terminal stop reason and independent packet, proxy, manifest, and cleanup results.

## Success Criteria

### Measurable Outcomes

- **SC-001**: The controlled basename-only simple Steam case binds the terminal client within the authorized wait and records at least one target-owned frame in analyzer-readable pcapng.
- **SC-002**: All five adverse identity cases (wrong path, stale PID or creation time, escaped parent, missing path, and ambiguous identity) produce zero false stage bindings and a named limitation.
- **SC-003**: The controlled ordered publisher chain binds each declared stage and the final client in order; an invalid intermediary prevents terminal acquisition.
- **SC-004**: A controlled acquisition timeout surfaces `acquisitiontimeout` in human and structured top-level results while packet, proxy, manifest, and cleanup observations reconcile independently.
- **SC-005**: Repository gates and the applicable isolated Windows production-path regression pass at the candidate head. A later private installed-title retest or release observation is not an implementation issue closure gate.

## Assumptions

- The field observations in #547 accurately describe v0.10.12, but the private native bundle is not copied into source, tests, public issues, or the PR.
- A controlled Windows fixture and isolated CI runner can establish production-path behavior without launching the private title or taking active workstation focus.
- S187's declared exact paths, ordered stage ancestry, and bounded lifecycle remain the governing contracts; this slice repairs their evidence input and result reporting.
- No release version, tag, registry package, or public asset is changed by this implementation slice.
