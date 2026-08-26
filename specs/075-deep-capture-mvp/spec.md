# Feature Specification: Deep Capture MVP

**Feature Branch**: `codex/219-deep-capture-mvp`

**Created**: 2026-08-25

**Status**: Implemented

**Input**: User description: "Issue #219. Define and build the Deep Capture MVP path after the architecture, compatibility facts, session bundle, and doctor readiness slices have landed."

## Clarifications

### Session 2026-08-25

- Q: Should the MVP try to support every known launch topology? A: No. The MVP proves one coherent Deep Capture path for a known-compatible target and refuses unsupported targets explicitly.
- Q: Can the MVP fall back to system-wide proxy configuration when launch-scoped proxy settings do not reach the target? A: No. System-wide proxy mutation remains outside the MVP and must not happen silently.
- Q: Should the MVP use the external `mitmdump` backend first? A: Yes. Doctor already detects it, and it gives the shortest auditable path to a working local proxy while native Rust backends remain a separate dependency and architecture decision.
- Q: Does a synthetic controlled target satisfy the verification requirement? A: Yes. The MVP must include a controlled local target path that exercises proxy routing, trust behavior, application records, and bundle correlation without a third-party game account.
- Q: Should Deep Capture feel like a separate product flow? A: No. It should use the existing target resolution, capture status, session identity, output, compatibility fact, and doctor cleanup surfaces.

### Session 2026-08-26

- Q: Which real managed launch path does the MVP implement? A: A Steam protocol launch from a cold Steam state, and only when current facts prove client routing and proxy propagation for that exact case. Warm Steam cannot inherit environment changes made in fragcap, and the existing Capture launcher does not launch direct executables, so both are preflight refusals.
- Q: When is the effective Capture launch validated? A: Before the proxy starts, session CA material is created, or current-user trust changes. Deep Capture consumes that prepared configuration for the run rather than resolving it again after side effects.

## User Scenarios & Testing

### User Story 1 - Run one coherent Deep Capture session (Priority: P1)

An authorized operator selects a registered target whose stored facts show a compatible launch-scoped proxy path. `fragcap deep-capture` starts a session-owned proxy, launches the target with scoped proxy configuration, captures packets, records application-layer observations when available, writes a bundle, updates compatibility facts, and reports cleanup.

**Why this priority**: This is the product proof. Without one end-to-end session, the prior design slices remain disconnected contracts.

**Independent Test**: Run Deep Capture against a controlled local target with no game account, verify the proxy observes HTTP and HTTPS traffic, and validate the resulting bundle manifest, application JSONL, `.fcapng`, status events, compatibility facts, and cleanup report.

**Acceptance Scenarios**:

1. **Given** a known-compatible target and an available proxy backend, **When** the user runs Deep Capture, **Then** the command produces one session bundle containing `.fcapng`, `manifest.json`, application JSONL, proxy/process sidecars, compatibility update metadata, and cleanup status.
2. **Given** inspectable HTTP or HTTPS traffic reaches the session proxy, **When** the bundle is written, **Then** application records name the same `session_id` and `flow_id` family used by the packet and manifest artifacts.
3. **Given** cleanup completes, partially completes, fails, or is deferred, **When** the command exits, **Then** human and JSON status output names the cleanup result and manifest path without dumping sensitive contents.

### User Story 2 - Refuse unsupported Deep Capture paths clearly (Priority: P1)

An operator chooses a target with no known scoped proxy-compatible launch path or a missing proxy backend. The command refuses before mutating trust, launching the target, or attempting a system-wide fallback.

**Why this priority**: Compatibility outranks richness. A failed Deep Capture attempt must teach the user what was missing and preserve the machine.

**Independent Test**: Invoke Deep Capture with injected facts for missing backend, unknown proxy propagation, metadata-only inspectability, and unsupported launch path, and assert deterministic refusal messages and exit codes.

**Acceptance Scenarios**:

1. **Given** `mitmdump` is unavailable, **When** Deep Capture starts, **Then** the command exits with a blocking readiness error before launch.
2. **Given** the selected target has no fact confirming scoped proxy routing can reach the client, **When** Deep Capture starts, **Then** the command refuses and tells the user which fact is missing.
3. **Given** the target routes traffic but speaks an unsupported protocol or rejects the local CA, **When** the session ends, **Then** the output records metadata-only or unsupported inspectability instead of claiming decryption.

### User Story 3 - Make trust explicit and reversible (Priority: P1)

An operator runs Deep Capture on a machine where the fragcap Deep Capture CA is absent. The command explains that a purpose-specific local CA is required, asks for explicit confirmation before trust changes, scopes trust to fragcap-owned material, and reports cleanup or retained trust state.

**Why this priority**: Deep Capture is intentionally active. Trust mutation is the most sensitive user-visible operation and must be explicit, auditable, and reversible.

**Independent Test**: Exercise CA lifecycle planning and cleanup with a fake trust store adapter so tests prove no silent trust installation, no wrong-store mutation, and no fabricated cleanup success.

**Acceptance Scenarios**:

1. **Given** no fragcap Deep Capture CA is trusted, **When** Deep Capture needs HTTPS inspection, **Then** the command requires explicit confirmation before installing trust.
2. **Given** the user declines trust installation, **When** Deep Capture continues or exits, **Then** the command does not install trust and records HTTPS as uninspectable or refuses according to the selected mode.
3. **Given** a session-owned CA, trust entry, proxy process, key log, or sensitive sidecar requires cleanup, **When** cleanup runs or is deferred, **Then** the cleanup report names each resource and outcome.

## Requirements

### Functional Requirements

- **FR-001**: The CLI MUST expose a Deep Capture command or subcommand that resolves one selected target through the same stored-target selector rules used by `capture`.
- **FR-002**: Deep Capture MUST require a stored target for the MVP. Raw `--process` Deep Capture is out of scope because it cannot supply target-scoped launch facts or compatibility fact updates.
- **FR-003**: Deep Capture MUST run a blocking preflight over proxy backend availability, session storage, CA/trust state, target compatibility facts, and the effective Capture launch configuration before starting the proxy, creating session CA material, mutating trust, or launching the target.
- **FR-004**: The MVP proxy backend MUST be an owned child process using the external `mitmdump` executable detected by doctor. The backend interface MUST be narrow enough to replace with a native Rust backend later without changing the CLI contract.
- **FR-005**: The command MUST configure proxy settings only for the managed launch environment or equivalent target-scoped launch surface. The real-target MVP supports a fact-backed cold Steam protocol launch; it MUST refuse warm Steam and direct-executable launch cases. It MUST NOT mutate system-wide proxy settings.
- **FR-006**: The command MUST refuse launch paths whose stored facts do not show scoped proxy routing to the final client or whose behavior is unknown and not being explicitly measured by a controlled harness.
- **FR-007**: The command MUST create, reuse, trust, and clean up only fragcap-owned Deep Capture CA material, with explicit user confirmation before any trust mutation.
- **FR-008**: Silent trust changes are prohibited. In non-interactive mode, a missing required trust confirmation MUST be a refusal unless an explicit pre-confirmed flag is present.
- **FR-009**: The command MUST run the existing packet capture path in tandem with the proxy so `.fcapng` remains the packet truth artifact.
- **FR-010**: The command MUST write a session bundle conforming to the #216 manifest model, including artifact declarations, omissions, sensitivity, correlation anchors, and cleanup status.
- **FR-011**: Application JSONL MUST record HTTP and HTTPS observations when the proxy can inspect them, and MUST record metadata-only or unsupported observations when traffic reaches the proxy but cannot be decoded into HTTP semantics.
- **FR-012**: HAR MAY be emitted only when HTTP semantics are observable. HAR support exposed by this slice MUST be shaped as utility-wide output capability, even if the first producer is Deep Capture.
- **FR-013**: Status output MUST use the existing emitter/event model and add Deep Capture lifecycle events for preflight, proxy start, trust state, launch, application observation, bundle write, and cleanup.
- **FR-014**: The local compatibility fact store MUST be updated after a session with observed proxy routing, propagation, trust behavior, protocol behavior, inspectability, final socket owner role, launch case, proxy backend, backend version, and scrubbed notes.
- **FR-015**: Compatibility fact updates MUST avoid PII, local filesystem paths, account identifiers, access tokens, hostnames, IP addresses, and real local title names in any committed fixture or public documentation.
- **FR-016**: The command MUST classify every observed traffic family as inspectable, metadata-only, unsupported, or unknown and surface that classification in the manifest and status output.
- **FR-017**: Cleanup MUST report proxy process, occupied port, CA material, trust entry, key log, bundle sidecars, and manifest state. It MUST NOT claim success for resources it did not observe or attempt.
- **FR-018**: The MVP MUST include a controlled local target verification path that exercises proxy routing and HTTPS trust without requiring a third-party game account, real title, or external service.
- **FR-019**: The MVP MUST NOT introduce code injection, hooks, memory reads, target TLS key extraction, executable modification, Winsock/LSP changes, packet interception drivers, or system-wide proxy fallback.
- **FR-020**: The MVP MUST preserve ordinary Capture behavior and help output except where the new Deep Capture command is intentionally added.

### Key Entities

- **DeepCaptureCommand**: The CLI entry point for one selected target, preflight options, bundle destination, trust confirmation, analyzer aids, and controlled-test support.
- **ProxyBackend**: A narrow orchestration interface for starting, monitoring, and stopping an owned local inspection proxy. The MVP implementation wraps `mitmdump`.
- **TrustManager**: A platform-aware adapter for fragcap-owned CA material and current-user trust state. Tests use a fake adapter; Windows implementation must avoid silent mutation.
- **DeepCaptureSession**: The coordinator that ties target resolution, proxy lifecycle, launch, packet capture, application observation, bundle writing, compatibility facts, status events, and cleanup into one session id.
- **ApplicationObservation**: A structured record for HTTP/HTTPS events, metadata-only proxy observations, decode failures, unsupported protocol notes, and proxy errors.
- **CompatibilityObservation**: The scrubbed set of facts written back to `deep_capture_facts` after the run.
- **ControlledTargetHarness**: A local verification target used by tests and maintainer demonstrations. It uses placeholder names and local endpoints only.

## Success Criteria

- **SC-001**: A controlled local Deep Capture run produces a valid session bundle with `.fcapng`, application JSONL, manifest, proxy/process sidecars, compatibility update metadata, and cleanup report.
- **SC-002**: The same run emits human status and `--json` events for Deep Capture phases without escape sequences in non-terminal output.
- **SC-003**: Missing backend, missing trust confirmation, unknown launch compatibility, and unsupported protocol cases are refused or reported with distinct messages and machine-readable event fields.
- **SC-004**: Compatibility facts written by the run can be read through the existing store APIs and contain only scrubbed placeholder-safe values in committed tests.
- **SC-005**: Existing Capture and doctor behavior remains stable except for the intentional Deep Capture command/help additions and the additive packet `flow_id` required for cross-artifact correlation.

## Assumptions

- The MVP can depend on an installed external `mitmdump` executable for runtime Deep Capture and can skip live backend demonstration tests when it is unavailable, provided deterministic fake-backend tests cover CI.
- Native Rust proxy backend research remains important but is not the first MVP implementation step.
- The exact command spelling may be adjusted during implementation to match clap help ergonomics, but it must remain a first-class Deep Capture surface rather than a hidden capture flag.
- The controlled target harness may live in test support or an internal maintainer command, but it must not require real accounts, real title names, or remote services.
