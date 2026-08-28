# Feature Specification: Verified First Capture and Deep Capture Journeys

**Feature Branch**: `codex/090-getting-started-journeys`

**Created**: 2026-08-28

**Status**: Ready for planning

**Input**: User description: "Kick off S090", implementing issue #245.

## User Scenarios & Testing

### User Story 1 - Complete a First Capture (Priority: P1)

A new Windows operator can follow the getting-started guide from prerequisite verification through target selection, a bounded Capture run, and opening the resulting packet capture in an unmodified analyzer.

**Why this priority**: Capture is the baseline product path. A first-run guide that uses retired commands or overstates what Capture observes prevents a new operator from reaching the shipped capability safely.

**Independent Test**: Read the Capture path from its first prerequisite through its final result, run every command through the current command grammar or an existing no-side-effect equivalent, and compare every shown result with current synthetic output contracts.

**Acceptance Scenarios**:

1. **Given** a new operator with fragcap, Npcap, and an analyzer installed, **When** they follow the Capture journey, **Then** every command is accepted by the current CLI and they produce a bounded `.fcapng` file.
2. **Given** an operator reading what Capture provides, **When** they reach the expectation and result sections, **Then** the guide distinguishes packet bytes, process attribution, encrypted payloads, and packet comments without claiming application-layer inspection.
3. **Given** a successful synthetic target listing, **When** the operator chooses a row, **Then** the specimen uses the current listing columns and repeats the labelled next command emitted by the CLI.

### User Story 2 - Complete a Known-Compatible Deep Capture (Priority: P1)

An operator who has a stored target with current compatibility evidence can continue from Capture into a known-compatible Deep Capture session, understand every prerequisite and confirmation, and find the resulting bundle and cleanup evidence.

**Why this priority**: Deep Capture ships in v0.7.0, but the current first-run guide omits it. Operators need an exact, consent-forward path that does not imply broader compatibility or inspection than the product has observed.

**Independent Test**: Follow the Deep Capture continuation with a synthetic known-compatible stored target, validate its invocation against the current command grammar, and verify that the described preflight, trust, traffic, bundle, and cleanup states agree with the shipped contracts.

**Acceptance Scenarios**:

1. **Given** a stored target with current cold Steam launch compatibility evidence, **When** the operator follows the Deep Capture path, **Then** the guide requires managed launch, the current supported launch case, a supported proxy backend, explicit CA trust authorization through `--trust-ca`, and a bounded run.
2. **Given** a Deep Capture session that completes, partially completes, or fails, **When** the operator reads the result guidance, **Then** the guide identifies the manifest as the session index, distinguishes packet truth from application observations, and explains the corresponding cleanup record.
3. **Given** application and TLS artifacts in a bundle, **When** the operator reads the handling guidance, **Then** the guide identifies their sensitivity and links to the detailed compatibility and output references.

### User Story 3 - Recognize Unsupported or Unknown Paths (Priority: P2)

An operator whose target lacks current compatibility evidence can recognize the refusal before attempting side effects and can distinguish unsupported traffic from a broken installation.

**Why this priority**: Honest refusal preserves the safety boundary and prevents the guide from turning missing compatibility evidence into a guessed title claim.

**Independent Test**: Read the guide with a synthetic target that has no compatibility facts and verify that it directs the operator to the read-only detail view and compatibility reference without presenting an unsupported calibration command.

**Acceptance Scenarios**:

1. **Given** a target with no stored compatibility evidence, **When** the operator checks it before Deep Capture, **Then** the guide identifies `unknown` as a preflight refusal and does not claim that the current release can calibrate it automatically.
2. **Given** traffic outside the current proxy path, **When** the operator compares Capture and Deep Capture outcomes, **Then** the guide preserves packet-capture availability while limiting application inspection claims to observed supported traffic.
3. **Given** an unavailable proxy, missing elevation, or incomplete cleanup, **When** the operator consults the guide, **Then** the next diagnostic action is explicit and does not silently broaden scope or mutate system-wide proxy settings.

### Edge Cases

- The default target-store paths are absent because the operator uses an override or has not initialized a store.
- Target discovery returns no ready rows, or the desired target is not discovered automatically.
- The target detail view reports no compatibility facts, stale facts, or facts for a different launch case.
- Deep Capture creates a partial or failed bundle and cleanup is incomplete.
- HTTPS traffic reaches the proxy but rejects the local CA, or the traffic uses QUIC, UDP, certificate pinning, or another unsupported application protocol.
- The operator selected `--no-payload`, so Capture does not retain payload bytes by explicit scope choice.
- Examples are copied on a machine whose local paths, interface names, or installed versions differ from the synthetic specimens.

## Requirements

### Functional Requirements

- **FR-001**: The guide MUST present two connected, sequential journeys: first Capture, then a known-compatible Deep Capture session.
- **FR-002**: Every executable example MUST be accepted by the v0.7.0 command grammar or be clearly marked as illustrative output rather than an invocation.
- **FR-003**: Doctor and target-listing specimens MUST match the current human-output structure, including v0.7.0, Deep Capture readiness, current target columns, and the labelled next command.
- **FR-004**: The guide MUST describe catalog and local database paths as optional overrides rather than installation requirements.
- **FR-005**: The Capture journey MUST state exactly that Capture records packet observations and process attribution, retains payload bytes unless the operator selects `--no-payload`, and does not by itself provide HTTP semantics or decrypted encrypted traffic.
- **FR-006**: The Deep Capture journey MUST require a stored target, managed launch, current launch-specific compatibility evidence, the shipped mitmdump backend, explicit authorization through `--trust-ca` before current-user CA trust changes, and a bounded session.
- **FR-007**: The guide MUST identify the currently supported real-target launch path as a cold Steam protocol launch and MUST identify warm Steam and direct-executable cases as refused in v0.7.0.
- **FR-008**: The guide MUST explain complete, partial, and failed bundle states, the manifest's role as the session index, and the existence and meaning of cleanup evidence.
- **FR-009**: The guide MUST distinguish `.fcapng` packet truth, application JSONL observations, optional HAR, optional proxy-owned TLS key logs, proxy logs, process traces, compatibility updates, and cleanup reports without claiming that all artifacts carry equivalent facts.
- **FR-010**: The guide MUST identify application observations and TLS key-log material as sensitive and MUST tell operators to retain, inspect, and share them with the same care as packet payloads.
- **FR-011**: The guide MUST state the current HTTP, HTTPS, WebSocket, non-HTTP TLS, QUIC, UDP, plaintext, certificate-pinning, and header/body limitations without generalizing one observed traffic family to all target traffic.
- **FR-012**: The guide MUST identify unknown or stale compatibility evidence as a refusal for the shipped Deep Capture path and MUST NOT imply that v0.7.0 can automatically calibrate an unknown target.
- **FR-013**: All target handles, executable names, paths, addresses, payloads, and output specimens MUST be synthetic and MUST contain no real title, account, host, endpoint, or local operator material.
- **FR-014**: The guide MUST link to the current Deep Capture compatibility and relevant glossary references and MUST point to command-specific `--help` for options. It MUST NOT rely on the stale output-format page for Deep Capture bundle handling before issue #248 corrects that page.
- **FR-015**: The guide MUST preserve the explicit Deep Capture safety boundary: no system-wide proxy fallback, silent trust change, certificate-pinning bypass, target TLS key extraction, process injection, hooks, or target memory reads.
- **FR-016**: The change MUST remain documentation-only except for deterministic documentation validation that is necessary to prove the guide's examples and specimens.
- **FR-017**: Documentation, link, encoding, punctuation, and production static-export checks MUST pass.

## Success Criteria

### Measurable Outcomes

- **SC-001**: One top-to-bottom read gives a new operator exactly one accepted command sequence for a first Capture and one accepted sequence for a known-compatible Deep Capture session.
- **SC-002**: One hundred percent of executable examples in the guide are accepted by the current CLI grammar or a no-side-effect validation seam.
- **SC-003**: Every shown doctor and target specimen agrees with current synthetic output structure and contains zero retired `KNOWN` columns, required database-path claims, or pre-v0.7.0 version labels.
- **SC-004**: Every Deep Capture prerequisite, confirmation, traffic limit, bundle state, and cleanup outcome named in issue #245 appears at least once in the guide or through a direct nearby reference link.
- **SC-005**: A review of all examples finds zero real titles, account identifiers, local paths, private endpoints, host identifiers, or captured payloads.
- **SC-006**: The production documentation export, internal documentation checks, and repository CI-parity gate complete successfully.

## Assumptions

- S090 implements GitHub issue #245 and follows the documentation epic sequence after completed issue #244.
- v0.7.0 remains the shipped behavior baseline throughout this slice.
- The guide uses a synthetic stored target whose current facts prove the supported cold Steam launch case; it does not create or publish real-title compatibility evidence.
- Detailed bundle and architecture references remain owned by issues #248 and #247. S090 supplies enough first-run guidance to use the product safely and links to the current references without absorbing those later issues.
- The current CLI reference remains hand-maintained until issue #246 adds its broader command-tree gate. S090 may add narrow validation for its own copied commands without implementing that future gate.
- Production visual and accessibility auditing remains owned by issue #249. This slice builds the site and verifies source contracts but does not claim that later audit is complete.
