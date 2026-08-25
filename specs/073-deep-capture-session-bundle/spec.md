# Feature Specification: Deep Capture session bundle

**Feature Branch**: `073-deep-capture-session-bundle`

**Created**: 2026-08-25

**Status**: Draft

**Input**: User description: "Issue #216. Design the Deep Capture session bundle and output correlation model before implementing writers or proxy orchestration. Define pcapng, JSON, HAR, key-log, proxy log, process trace, compatibility metadata, manifest, status output, and correlation anchors."

## Clarifications

### Session 2026-08-25

- Q: Is `.fcapng` the only Deep Capture artifact? A: No. `.fcapng` remains packet truth, while decrypted application records and analyzer aids live in sidecars indexed by a manifest.
- Q: Does HAR belong only to Deep Capture? A: No. HAR is a utility-wide output shape when HTTP semantics are observable. Capture can produce HAR for plaintext or already-decrypted HTTP; Deep Capture is the mode expected to make HTTPS HAR useful.
- Q: Should TLS key logs be treated as decrypted output? A: No. A key log is an analyzer aid for proxy-owned TLS tunnels. It is sensitive session material, named and linked from the manifest, and never produced silently.
- Q: Is the manifest optional? A: No. A multi-artifact session without a manifest is not a bundle. The manifest is the durable index and the cleanup/status handoff contract.
- Q: Should pcapng carry every decrypted application object? A: No. pcapng carries packet attribution and enough correlation anchors for sidecars. Application records belong in JSONL and HAR sidecars.

## User Scenarios & Testing

### User Story 1 - Receive one coherent Deep Capture bundle (Priority: P1)

An authorized operator runs a future Deep Capture command for a known-compatible target. At the end, the output directory contains a manifest plus packet, application, analyzer, proxy, process, and compatibility artifacts that can be understood as one session rather than a loose collection of files.

**Why this priority**: Deep Capture creates multiple artifacts by design. Without a bundle model, doctor cleanup, analyzer setup, status output, and later MVP implementation will disagree about what exists.

**Independent Test**: Validate a sample manifest and bundle layout against the contract in this slice, checking every required artifact role and sensitivity marker.

**Acceptance Scenarios**:

1. **Given** a Deep Capture session writes multiple artifacts, **When** the session completes, **Then** the manifest names every artifact, its role, path, sensitivity, and authority.
2. **Given** a user opens the bundle later, **When** they inspect the manifest, **Then** they can identify the target, mode, proxy backend, CA thumbprint state, cleanup report, aggregate cleanup status, and compatibility fact updates without reading every sidecar.
3. **Given** an artifact is optional for a session, **When** it is not produced, **Then** the manifest records the omission reason rather than leaving ambiguity.

### User Story 2 - Correlate packets, processes, flows, and application records (Priority: P1)

A developer or researcher needs to tie a decrypted HTTP transaction back to the process, role, flow, packet window, and capture session that produced it. The bundle defines stable anchors for that join before any writer implementation begins.

**Why this priority**: Decrypted traffic without attribution loses the core value of fragcap. The packet capture and application records must join deterministically.

**Independent Test**: Inspect the sample records and verify that every application record includes session and flow anchors and can reference process/role context without parsing pcapng comments.

**Acceptance Scenarios**:

1. **Given** an application event is produced by the proxy, **When** it is written to JSONL or HAR-derived records, **Then** it carries `session_id`, `target_id`, `flow_id`, proxy connection id, process id when known, role when known, and timing fields.
2. **Given** packets exist for the same flow, **When** an analyzer joins sidecars, **Then** it can use the shared packet and sidecar `flow_id` plus time bounds to locate the relevant packet window.
3. **Given** attribution is unavailable for a record, **When** the record is written, **Then** the missing process/role state is explicit rather than omitted silently.

### User Story 3 - Protect sensitive Deep Capture material (Priority: P1)

An operator wants to understand and clean up sensitive session state, including proxy-owned TLS key logs, decrypted application records, and certificate/trust metadata. The bundle model marks sensitivity and cleanup state consistently.

**Why this priority**: Deep Capture intentionally creates higher-sensitivity artifacts than ordinary Capture. The design must make doctor and future cleanup implementation straightforward.

**Independent Test**: Validate the example manifest includes sensitivity classifications and the cleanup report reference for key logs, proxy state, local CA trust, and output artifacts.

**Acceptance Scenarios**:

1. **Given** a TLS key log is produced, **When** the manifest is written, **Then** it is marked sensitive, session-scoped, proxy-owned, and analyzer-aid only.
2. **Given** cleanup succeeds, partially succeeds, or fails, **When** the cleanup report is written, **Then** it names each trust, proxy, port, process, key-log, and artifact state relevant to the session.
3. **Given** human or machine-readable status output references the bundle, **When** it reports completion, **Then** it points to the manifest and names sensitive artifacts without dumping their contents.

## Requirements

### Functional Requirements

- **FR-001**: A Deep Capture session bundle MUST contain a manifest that indexes every produced artifact and every expected-but-omitted artifact.
- **FR-002**: `.fcapng` MUST remain the packet truth artifact and MUST NOT be overloaded with full decrypted application objects.
- **FR-003**: Application JSONL MUST be the canonical machine-readable application event stream for proxy observations.
- **FR-004**: HAR MUST be available only when HTTP semantics are observable. Capture MAY produce HAR for plaintext or otherwise observable HTTP; Deep Capture MAY produce HAR for HTTPS that the proxy can inspect.
- **FR-005**: TLS key-log files MUST be treated as sensitive analyzer aids for proxy-owned TLS tunnels, not as decrypted output, and MUST be linked from the manifest only when explicitly requested by an output profile or analyzer integration setting.
- **FR-006**: The manifest MUST identify session id, target id or stable target handle, capture mode, start and stop times, proxy backend identity, proxy backend version, proxy mode, CA thumbprint state, artifact paths, artifact sensitivity, cleanup report reference, aggregate cleanup status, and compatibility fact update references.
- **FR-007**: Every application record MUST carry enough correlation fields to join against packet flows, process/role context, and the session manifest without parsing human text.
- **FR-008**: Proxy logs and process traces MUST be sidecars with structured records suitable for status output and later doctor cleanup.
- **FR-009**: The design MUST define human-readable and machine-readable status outputs at the level of facts, not visual formatting.
- **FR-010**: Packet annotations MUST carry the same `flow_id` used by application sidecars whenever a packet has a flow key.
- **FR-011**: Status output MUST report session identity, mode, phase, completion state, artifact inventory, omission summary, proxy state, trust state, cleanup summary, and manifest path without dumping sensitive contents.
- **FR-012**: This design slice MUST add no runtime dependency and MUST NOT implement writers, proxy orchestration, or CLI flags.

### Key Entities

- **Session bundle**: A directory or logical artifact set rooted at one manifest. It groups packet truth, application events, optional HAR, optional TLS key log, proxy log, process trace, compatibility update records, and cleanup report for one Capture or Deep Capture session.
- **Session manifest**: The authoritative bundle index. It names the target, mode, artifact roles, sensitivity, paths, proxy/trust state, correlation anchors, cleanup report reference, and aggregate cleanup status.
- **Correlation anchor**: A stable field used to join artifacts. Required anchors are `session_id`, `target_id` or stable target handle, `flow_id`, proxy connection id where applicable, process id where known, role where known, and time bounds.
- **Artifact authority**: The rule identifying which artifact owns a fact. Packet bytes, flow ids, and loss accounting are owned by `.fcapng`; application transactions are owned by application JSONL and HAR; bundle membership is owned by the manifest; per-resource cleanup state is owned by the cleanup report.

## Success Criteria

### Measurable Outcomes

- **SC-001**: The slice contains a complete example bundle layout and manifest.
- **SC-002**: The manifest contract names every artifact role, authority, sensitivity, and omission rule.
- **SC-003**: The correlation model allows joining application records to packet flows and process/role attribution using structured fields present on both packet and sidecar records.
- **SC-004**: The design explicitly states when HAR can be produced from Capture and when it requires Deep Capture.
- **SC-005**: The design gives issue #218 enough stable cleanup targets to implement doctor readiness and cleanup without inventing artifact names, status fields, or sensitivity rules.

## Assumptions

- This slice designs the output model only. It does not implement HAR serialization, proxy event writers, key-log generation, or CLI commands.
- The exact `session_id` generation algorithm can be implemented later, but the id must be unique enough within local output storage and stable across all artifacts in one session.
- Paths in examples are relative to the bundle root to avoid committing local filesystem details.
