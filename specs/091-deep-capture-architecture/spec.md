# Feature Specification: Deep Capture Architecture and Trust Boundaries

**Feature Branch**: `codex/091-deep-capture-architecture`

**Created**: 2026-08-28

**Status**: Ready for planning

**Input**: User description: "Kick off S091", implementing issue #247.

## User Scenarios & Testing

### User Story 1 - Understand the Two Capture Modes (Priority: P1)

A reader can distinguish passive Capture from explicit Deep Capture and follow a separate execution diagram for each without mistaking one mode's capabilities or side effects for the other's.

**Why this priority**: The current architecture page presents a passive-only product even though Deep Capture ships in v0.7.0. That framing hides a security-relevant mode and makes the public architecture disagree with the product.

**Independent Test**: Read only the mode overview and the two execution diagrams, then identify each mode's activation, data path, outputs, side effects, and capability limits without consulting source code.

**Acceptance Scenarios**:

1. **Given** a reader evaluating ordinary Capture, **When** they follow its diagram, **Then** they see passive packet acquisition, external process attribution, scoped retention, and unmodified-analyzer output with no proxy or trust change.
2. **Given** a reader evaluating Deep Capture, **When** they follow its diagram, **Then** they see explicit target selection, compatibility preflight, prepared managed launch, the local proxy, packet capture, application observations, correlation, the session bundle, and cleanup.
3. **Given** both diagrams, **When** the reader compares them, **Then** Capture remains the packet-truth foundation and Deep Capture adds only observations from traffic that reaches the proxy.

### User Story 2 - Evaluate Trust and Security Boundaries (Priority: P1)

An operator or reviewer can identify what Deep Capture changes, which component owns each change, how consent is expressed, what remains target-scoped, and how cleanup is audited.

**Why this priority**: Proxy routing and certificate trust are active security boundaries. Hiding ownership, scope, or cleanup would make the public explanation unsafe even if the data-flow diagram were technically accurate.

**Independent Test**: Trace the trust-boundary explanation from `--trust-ca` authorization through current-user Root trust, proxy operation, target launch, and cleanup, then verify that every forbidden fallback or covert technique is explicitly excluded.

**Acceptance Scenarios**:

1. **Given** a planned Deep Capture session, **When** the reader reviews trust behavior, **Then** they learn that `--trust-ca` is the explicit authorization, there is no second interactive trust prompt, trust is limited to a fragcap-owned CA in the current-user store, and cleanup records the result.
2. **Given** traffic that bypasses the proxy, rejects the CA, uses certificate pinning, or uses an unsupported protocol, **When** the reader evaluates the architecture, **Then** no application-inspection claim is made and packet Capture remains independently described.
3. **Given** a security reviewer, **When** they inspect the mode boundaries, **Then** system-wide proxy fallback, silent trust, target key extraction, process injection, hooks, target memory reads, and executable modification are clearly excluded.

### User Story 3 - Interpret Outputs and Operational Dependencies (Priority: P2)

A reader can distinguish packet truth, proxy observations, proxy-owned analyzer material, correlation metadata, and cleanup evidence, while understanding how Npcap and mitmdump enter the architecture.

**Why this priority**: Different artifacts have different authority and sensitivity. Treating every output as equivalent, or presenting third-party dependencies inaccurately, creates confident misunderstandings about what a session proved.

**Independent Test**: Use the architecture page alone to classify each named output by producer and authority, identify the two external runtime dependencies, and follow links to the detailed compatibility, output, glossary, and getting-started references.

**Acceptance Scenarios**:

1. **Given** a Deep Capture bundle, **When** the reader examines its high-level architecture, **Then** `.fcapng` is identified as packet truth, application JSONL as proxy observations, TLS key logs as optional proxy-owned analyzer material, and manifest plus cleanup records as session audit evidence.
2. **Given** an absent Npcap installation, **When** the reader examines acquisition guidance, **Then** they learn that fragcap never bundles, hosts, caches as its own, or redistributes Npcap and that `doctor --fix` can perform only the current explicitly confirmed vendor-acquisition behavior.
3. **Given** an absent mitmdump backend, **When** the reader compares modes, **Then** only Deep Capture is unavailable and ordinary Capture remains architecturally independent.

### Edge Cases

- A target has unknown, stale, conflicting, or wrong-launch compatibility evidence, so Deep Capture stops before session side effects.
- Steam is already running or the stored target requires an unsupported direct-executable managed launch.
- Traffic reaches packet Capture but never reaches the local proxy.
- HTTPS reaches the proxy but rejects the fragcap-owned CA or uses certificate pinning.
- A requested sidecar is omitted, empty, or unavailable while packet truth remains useful.
- Cleanup cannot remove a session-owned resource and must preserve auditable ownership evidence.
- Npcap or mitmdump is unavailable, but the unaffected portions of diagnostics and architecture remain usable.
- Mermaid diagrams are viewed at mobile width and must remain readable without silently clipping critical nodes or labels.

## Requirements

### Functional Requirements

- **FR-001**: The architecture page MUST describe Capture and Deep Capture as distinct shipped modes with separate, accurate execution diagrams.
- **FR-002**: The Capture view MUST show passive packet acquisition through Npcap, external socket and process evidence, target-scope retention, named loss accounting, `.fcapng` packet truth, and unmodified analyzer compatibility.
- **FR-003**: The Deep Capture view MUST include stored-target selection, a read-only compatibility preflight, prepared target-scoped managed launch, a fragcap-owned loopback mitmdump child, ordinary packet Capture, application observations, correlation, a manifest-indexed session bundle, and cleanup.
- **FR-004**: The page MUST state that the current real-target Deep Capture path requires current `proxy-routing = reached-client` and `proxy-propagation = confirmed` evidence for a cold Steam protocol launch and refuses unknown, stale, conflicting, wrong-launch, warm-Steam, and direct-executable cases.
- **FR-005**: The page MUST define `--trust-ca` as the explicit authorization for a fragcap-owned current-user Root trust change, state that no second interactive trust prompt occurs, and describe trust cleanup as session-owned and auditable.
- **FR-006**: The page MUST distinguish `.fcapng` packet truth, application JSONL proxy observations, optional HAR, optional proxy-owned TLS key-log material, proxy and process traces, compatibility updates, the manifest, and cleanup evidence without claiming equivalent authority.
- **FR-007**: The page MUST explain that application-to-packet correlation uses structured session and flow anchors when available and MUST NOT claim that absent correlation is fabricated.
- **FR-008**: The page MUST limit Deep Capture claims by actual routing, CA acceptance, certificate pinning, and current protocol support, while preserving ordinary Capture availability for traffic outside the proxy path.
- **FR-009**: The page MUST state that Deep Capture never falls back to system-wide proxy settings and MUST exclude silent trust, target TLS key extraction, injection, hooks, target memory reads, layered service providers, packet interception drivers, and executable modification.
- **FR-010**: Npcap guidance MUST state that fragcap never bundles, hosts, caches as its own, or redistributes Npcap or its installer and MUST describe only the currently shipped, explicitly confirmed `doctor --fix` acquisition behavior.
- **FR-011**: The dependency explanation MUST identify Npcap as the Capture packet backend, mitmdump as the shipped Deep Capture proxy backend, and Wireshark or another unmodified pcapng analyzer as a consumer rather than part of fragcap's capture engine.
- **FR-012**: The page MUST link to the master specification, getting-started guide, Deep Capture compatibility reference, output-format reference, CLI help reference, and relevant glossary entries without absorbing issue #248's full artifact matrix.
- **FR-013**: Every diagram MUST use concise labels and a direction that remains understandable at mobile and desktop widths, with prose carrying details that would make a node unreadably dense.
- **FR-014**: All examples, paths, identifiers, and diagram labels MUST be synthetic and disclose no local title, account, endpoint, host, or captured material.
- **FR-015**: The change MUST remain documentation-only except for deterministic documentation validation needed to prove links, diagrams, and production buildability.
- **FR-016**: Documentation, glossary, link, encoding, punctuation, production static-export, and repository CI-parity checks MUST pass.

## Success Criteria

### Measurable Outcomes

- **SC-001**: A reader can correctly classify every node in both execution diagrams as Capture-only, Deep-Capture-only, shared, external dependency, or output evidence.
- **SC-002**: One hundred percent of security-sensitive actions shown for Deep Capture identify their trigger, scope, owner, and cleanup or refusal behavior.
- **SC-003**: Every v0.7.0 output family named by issue #247 appears with a distinct producer and authority, with zero claims that all artifacts contain equivalent facts.
- **SC-004**: A phrase audit finds zero passive-only product framing, system-wide fallback claims, silent trust implications, pinning-bypass claims, or target-key-extraction claims.
- **SC-005**: Both Mermaid diagrams contain no more than twelve primary nodes apiece and the production site export builds successfully.
- **SC-006**: Documentation checks, internal links, text-hygiene checks, and the full repository CI-parity gate complete successfully.

## Assumptions

- S091 implements GitHub issue #247 after completed issues #244 and #245 in the `Post-v0.7.0 documentation` milestone.
- v0.7.0 and the current merged command behavior are the shipped baseline.
- The architecture page is explanatory rather than a second master specification; detailed tables remain in the linked reference pages.
- Issue #248 owns the exhaustive Deep Capture artifact, omission, sensitivity, and lifetime reference. S091 names enough output authority to make its diagrams truthful without duplicating that future work.
- Issue #246 owns a mechanical website-to-clap command-tree gate, and issue #249 owns the rendered production UX and accessibility audit.
- Existing glossary entries cover the architecture vocabulary introduced by this slice; any genuinely new term will receive an entry before use.
