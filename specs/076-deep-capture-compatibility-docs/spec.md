# Feature Specification: Deep Capture Compatibility Documentation

**Feature Branch**: `codex/220-deep-capture-compatibility-docs`

**Created**: 2026-08-26

**Status**: Implemented

**Input**: User description: "Issue #220. Publish supported traffic types and a
compatibility matrix generated from the local Deep Capture fact model."

## Clarifications

### Session 2026-08-26

- Q: Is the target compatibility matrix a checked-in list of games? A: No. It
  is a runtime projection of the selected target's local facts. Public
  documentation defines the traffic support table, matrix fields, and legend
  without publishing local title names.
- Q: Which command is the canonical target-specific matrix surface? A:
  `fragcap targets show`, because it already resolves one stored target and is
  read-only.
- Q: Does viewing compatibility refresh or probe the target? A: No. The view
  reports stored evidence only and explains how a later authorized Deep Capture
  run can refresh observations.
- Q: How are conflicting or repeated facts presented? A: Every stored fact is
  retained in deterministic order with its launch case, source, and freshness.
  The view does not collapse evidence into a guessed verdict.
- Q: What does `full` inspectability mean in this slice? A: The proxy observed
  HTTP semantics for that fact. It does not promise that every header, body,
  frame, or custom payload was retained in the session bundle.

## User Scenarios & Testing

### User Story 1 - Understand Traffic Coverage (Priority: P1)

An operator or game developer can read one authoritative table that explains
what Capture and Deep Capture currently provide for HTTP, HTTPS, WebSocket,
non-HTTP TLS, QUIC, UDP, and plaintext traffic.

**Why this priority**: Users cannot choose an inspection mode or interpret its
output correctly unless the product states its current protocol boundaries
without implying universal decryption.

**Independent Test**: Review the published reference and verify that all seven
traffic families have separate Capture and Deep Capture outcomes, limitations,
and analyzer guidance that agree with implemented behavior.

**Acceptance Scenarios**:

1. **Given** a user comparing Capture and Deep Capture, **When** they open the
   traffic support reference, **Then** they can distinguish packet visibility,
   HTTP-semantic inspection, metadata-only observations, and unsupported
   traffic.
2. **Given** HTTPS that rejects the local certificate authority, **When** the
   user reads the HTTPS row, **Then** the reference explains that pinning is not
   bypassed and decryption is unavailable.
3. **Given** QUIC, UDP, WebSocket, custom TLS, or plaintext traffic, **When** the
   user reads the corresponding row, **Then** the reference states the exact
   current boundary without extending HTTP support to unrelated protocols.

---

### User Story 2 - Inspect Local Target Evidence (Priority: P1)

An operator can inspect the compatibility facts stored for one selected target
through the existing target detail command. The output names what was observed,
under which launch case, where the evidence came from, and whether it is stale.

**Why this priority**: Target behavior depends on launch topology and local
observations. A static title list cannot accurately represent those conditions.

**Independent Test**: Populate a temporary target database with current,
stale, imported, user-confirmed, and observed facts, run the target detail
command, and verify every fact is rendered in deterministic order with no
invented result.

**Acceptance Scenarios**:

1. **Given** a target with current observed facts, **When** the user views the
   target, **Then** the compatibility section identifies each fact's key,
   value, launch case, evidence source, and current status.
2. **Given** a target with stale or explicitly stale evidence, **When** the user
   views the target, **Then** the output labels that evidence stale rather than
   presenting it as current advice.
3. **Given** a target with no facts, **When** the user views the target, **Then**
   the output says compatibility is unknown and does not infer support from the
   target's platform, engine, or title.
4. **Given** repeated or conflicting facts, **When** the user views the target,
   **Then** all evidence remains visible in deterministic order and no synthetic
   winner is selected.

---

### User Story 3 - Refresh and Contribute Facts Safely (Priority: P2)

An operator or contributor can understand how facts become current, which
sources are firsthand or imported, and which data must never appear in public
documentation, fixtures, or reports.

**Why this priority**: Compatibility guidance loses value if observations
cannot be refreshed or if collecting them risks publishing accounts, paths,
endpoints, or real local title names.

**Independent Test**: Follow the documented refresh and contribution guidance
using placeholder records and verify that it requires explicit measurement,
defines all evidence states, and contains no real title or personal data.

**Acceptance Scenarios**:

1. **Given** stale or unknown compatibility, **When** the user reads the refresh
   guidance, **Then** it explains that viewing is side-effect free and that a
   later authorized measurement can write new evidence.
2. **Given** facts from an observed run, user confirmation, imported catalog,
   or stale observation, **When** a contributor reads the legend, **Then** each
   source has a distinct meaning.
3. **Given** a contributor preparing a fixture or report, **When** they follow
   the contribution guidance, **Then** local paths, accounts, tokens, endpoints,
   host identifiers, and real local title names are excluded.

### Edge Cases

- A stored fact can be marked stale even when its evidence source was originally
  an observed run, user confirmation, or imported catalog.
- The explicit `stale-observation` source is also stale even if a malformed
  legacy row omitted the stale marker.
- Facts can share a key while differing by launch case, backend, version, or
  observation time; the matrix must preserve each row.
- Optional timestamps, versions, backend details, and notes can be absent
  without changing the fact to current or unknown.
- A scrubbed note can contain free text, but rendering must not transform it
  into a stronger compatibility claim.
- Capture can still contain packets for traffic that Deep Capture cannot inspect
  at the application layer.
- A WebSocket handshake can expose HTTP semantics while WebSocket data frames
  remain outside the current application-record contract.

## Requirements

### Functional Requirements

- **FR-001**: The user-facing reference MUST distinguish passive Capture from
  active Deep Capture before describing protocol-specific behavior.
- **FR-002**: The traffic support table MUST contain separate rows for HTTP,
  HTTPS, WebSocket, non-HTTP TLS, QUIC, UDP, and plaintext traffic.
- **FR-003**: Every traffic row MUST state the current Capture visibility, Deep
  Capture inspectability, applicable prerequisites or blockers, and expected
  output or analyzer use.
- **FR-004**: The reference MUST explain that Deep Capture observes only traffic
  routed through its scoped proxy and MUST NOT claim universal decryption,
  pinning bypass, QUIC decryption, custom protocol dissection, or target key
  extraction.
- **FR-005**: The reference MUST state that current HTTP-semantic records expose
  the fields actually emitted by the implementation and that `full`
  inspectability does not promise retention of every header, body, WebSocket
  frame, or custom payload.
- **FR-006**: The selected-target detail view MUST render a compatibility matrix
  generated exclusively from that target's stored compatibility facts.
- **FR-007**: Each rendered fact MUST include its key, value, launch case when
  present, evidence source, and freshness state.
- **FR-008**: The rendered facts MUST use deterministic ordering that does not
  depend on database row retrieval order.
- **FR-009**: The detail view MUST preserve repeated and conflicting evidence
  rather than calculating an unsupported aggregate verdict.
- **FR-010**: A target with no compatibility facts MUST be labeled unknown and
  MUST NOT receive inferred facts from platform, engine, executable, or title
  metadata.
- **FR-011**: Evidence-source documentation MUST distinguish observed run, user
  confirmation, imported catalog, stale observation, and absence of evidence.
- **FR-012**: Freshness documentation and rendering MUST distinguish current,
  explicitly stale, and unknown states. `stale-observation` MUST always render
  as stale.
- **FR-013**: Viewing compatibility MUST remain read-only and MUST NOT launch a
  target, start a proxy, mutate trust, contact a catalog, or silently refresh
  facts.
- **FR-014**: Refresh guidance MUST direct users to an explicit authorized
  measurement path and MUST state that newer evidence supplements rather than
  silently rewrites historical launch-specific observations.
- **FR-015**: Public documentation, committed tests, and fixtures MUST use
  placeholders and MUST NOT contain personal data, accounts, access tokens,
  local filesystem paths, private endpoints, host identifiers, or real local
  game titles gathered during compatibility work.
- **FR-016**: The master specification MUST record the compatibility projection,
  traffic taxonomy, and truthfulness rules introduced by this slice.
- **FR-017**: Existing target-detail fields and selector exit behavior MUST
  remain unchanged except for the additive compatibility section.

### Key Entities

- **Traffic Support Entry**: One documented traffic family with Capture
  visibility, Deep Capture inspectability, prerequisites, blockers, outputs,
  and analyzer guidance.
- **Compatibility Matrix**: A deterministic read-only projection of all stored
  compatibility facts for one selected target.
- **Compatibility Fact Row**: One fact key and value with optional launch case,
  evidence source, freshness, and optional observation metadata.
- **Evidence Source**: The provenance category for a fact: observed run, user
  confirmed, imported catalog, or stale observation.
- **Freshness State**: Current, stale, or unknown presentation state. Unknown is
  used when no fact exists, not as a guess about target behavior.

## Success Criteria

### Measurable Outcomes

- **SC-001**: The published traffic reference covers all seven required traffic
  families, with no blank Capture, Deep Capture, limitation, or output fields.
- **SC-002**: Automated tests render every evidence source and all three
  freshness states from placeholder-only facts.
- **SC-003**: A target containing repeated and conflicting facts renders every
  stored row in the same order across repeated runs.
- **SC-004**: A target with no facts is identified as unknown without performing
  any launch, proxy, trust, network, or database-write side effect.
- **SC-005**: Documentation and committed test data pass repository privacy,
  terminology, formatting, and link checks without a real local title, account,
  path, endpoint, or host identifier.
- **SC-006**: Existing target detail tests continue to pass with only the
  documented additive compatibility output.

## Assumptions

- The compatibility fact schema and local SQLite storage delivered by S072 are
  the source of truth for target-specific evidence.
- The Deep Capture MVP delivered by S075 is the source of truth for current
  traffic behavior and emitted application fields.
- A target-specific machine-readable export is not introduced by this slice;
  the existing human target-detail surface is the canonical matrix view.
- Public documentation describes support by traffic family and evidence state,
  not by named commercial title.
- Native proxy backends, new protocol dissectors, community synchronization,
  fact-editing commands, active compatibility probes, and universal HAR export
  remain outside this slice.
