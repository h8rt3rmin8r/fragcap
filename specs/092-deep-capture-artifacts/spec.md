# Feature Specification: Deep Capture Bundle and Artifact Reference

**Feature Branch**: `codex/092-deep-capture-artifacts`

**Created**: 2026-08-28

**Status**: Draft

**Input**: User description: "Kick off S092", implementing issue #248.

## User Scenarios & Testing

### User Story 1 - Identify the Authority of Every Output (Priority: P1)

As an operator or reviewer, I need one public reference that distinguishes ordinary Capture outputs from every Deep Capture bundle artifact so I know which file answers each analysis question without treating all outputs as equivalent.

**Why this priority**: The current output page says two formats carry the same facts, which is false for v0.7.0 and can lead readers to draw conclusions from an artifact that does not own them.

**Independent Test**: Read the reference from its introduction through the artifact matrix and identify the authoritative source for packet bytes, application observations, HTTP projection, proxy lifecycle, process chronology, compatibility evidence, and cleanup results.

**Acceptance Scenarios**:

1. **Given** a reader comparing Capture and Deep Capture, **When** they open the output reference, **Then** ordinary pcapng and packet JSON Lines remain clearly documented as Capture outputs while the Deep Capture bundle is introduced as a manifest-indexed artifact set.
2. **Given** a Deep Capture bundle, **When** the reader consults the artifact matrix, **Then** every artifact produced or declared by v0.7.0 has one explicit authority, sensitivity, lifetime, and production or omission condition.
3. **Given** an unmodified packet analyzer, **When** the reader needs packet truth, **Then** the reference identifies `capture.fcapng` as ordinary pcapng and does not imply that decrypted application objects were folded into it.

---

### User Story 2 - Handle Sensitive Artifacts Deliberately (Priority: P1)

As an operator, I need the reference to explain the sensitivity and lifecycle of packet payloads, application records, HAR, proxy and process sidecars, and proxy-owned TLS key logs so I can retain, inspect, share, or remove them deliberately.

**Why this priority**: These artifacts can carry URLs, identifiers, payloads, process details, or session secrets. Ambiguous handling guidance creates a privacy and security risk.

**Independent Test**: Use only the reference to determine which artifacts are ordinary, sensitive, or secret-adjacent, when a requested key log exists, when it is usable live, and what cleanup does or does not remove.

**Acceptance Scenarios**:

1. **Given** an operator considering `--key-log`, **When** they read the reference, **Then** they learn that the final-path file is created before proxy traffic, populated by the proxy for live analyzer use, retained only when nonempty, and is never described as target key extraction.
2. **Given** a completed bundle, **When** the operator considers sharing it, **Then** the reference says to keep the original intact, work from a scrubbed copy, and treat sensitive sidecars at least as carefully as packet payloads.
3. **Given** `fragcap doctor --fix`, **When** the operator reads the lifecycle guidance, **Then** the reference distinguishes session cleanup from later confirmation-gated stale-residue cleanup and does not promise automatic deletion of a completed bundle.

---

### User Story 3 - Interpret State, Omissions, and Correlation (Priority: P2)

As an analyst, I need exact manifest state, omission, and correlation guidance so I can tell the difference between an unavailable artifact, an unobserved protocol, a partial run, and evidence that can be joined across files.

**Why this priority**: A missing artifact or correlation anchor must not be mistaken for proof that traffic or activity did not occur.

**Independent Test**: Read a synthetic manifest example and correctly explain its session state, each omission reason, the available joins, and the limits of those joins without needing local or account data.

**Acceptance Scenarios**:

1. **Given** a manifest state of `complete`, `partial`, or `failed`, **When** the reader uses the reference, **Then** they can distinguish operational completion from inspection coverage and understand whether packet truth may exist.
2. **Given** an omission entry, **When** the reader looks up its exact v0.7.0 reason token, **Then** they can identify the omitted role and why fragcap did not declare that artifact as produced.
3. **Given** records across the manifest, application JSON Lines, packet comments, proxy log, and process trace, **When** the reader follows the correlation guidance, **Then** they use only anchors actually present and treat absent anchors as unavailable correlation rather than a negative observation.
4. **Given** a reader arriving from the CLI or compatibility reference, **When** they follow the bundle link, **Then** they reach the authoritative output contract without duplicated or conflicting artifact guidance.

### Edge Cases

- A session can be `complete` while producing no HAR or key log because those artifacts are optional and depend on request and observation conditions.
- A session can be `partial` with useful `capture.fcapng` and sidecar evidence after a later operation fails.
- A session can be `failed` without packet truth; the absence must remain explicit rather than represented by a fabricated capture.
- `application.jsonl` can contain metadata-only or unsupported observations and can carry no usable `flow_id`; neither case proves that no packet traffic existed.
- A requested HAR can be omitted when no observation contains the HTTP method and URL needed by the projection.
- A requested TLS key log can be omitted when the proxy produces no nonempty key material; an empty placeholder is removed before finalization.
- `process-trace.jsonl` can contain an explicit unavailable record when no stage lifecycle event was observed; packet attribution remains the process authority in that case.
- Cleanup can fail for one resource while evidence is retained. The cleanup report, not the session state alone, owns that result.
- An early initialization failure can leave `cleanup.json` without a final `manifest.json`; the reference must distinguish this recovery record from a finalized bundle.

## Requirements

### Functional Requirements

- **FR-001**: The public output reference MUST remove the claim that fragcap has only two equivalent output formats and MUST distinguish ordinary Capture sinks from the Deep Capture session bundle.
- **FR-002**: The reference MUST preserve the existing pcapng and packet JSON Lines contracts, including packet attribution, payload behavior, finalization, and loss accounting.
- **FR-003**: The reference MUST define `manifest.json` as the durable bundle index and MUST explain manifest versions, target and session identity, launch and proxy context, trust state, artifacts, omissions, correlation, and cleanup summary at the level exposed by v0.7.0.
- **FR-004**: The reference MUST define `complete`, `partial`, and `failed` as operation states and MUST state that `complete` is not a claim of universal inspection or production of every optional artifact.
- **FR-005**: The reference MUST document every current artifact role: `manifest`, `pcapng`, `application-jsonl`, `har`, `tls-key-log`, `proxy-log`, `process-trace`, `compatibility`, and `cleanup`.
- **FR-006**: For every artifact role, the reference MUST state its current path, authority, sensitivity label, retention or cleanup lifetime, whether it is required or optional, and its production or omission conditions.
- **FR-007**: The reference MUST keep `capture.fcapng` authoritative for packet bytes, timestamps, interfaces, process attribution comments, correlation annotations, and loss accounting, while remaining readable by unmodified pcapng analyzers.
- **FR-008**: The reference MUST define `application.jsonl` as the canonical machine-readable proxy-observation stream and MUST document its header, observation, and trailer record families without implying that every record contains HTTP semantics or packet correlation.
- **FR-009**: The reference MUST describe `http.har` as an optional projection of the HTTP method, URL, response status, and observation time currently retained by v0.7.0, with empty header, cookie, query, and body structures rather than invented values.
- **FR-010**: The reference MUST describe `tls-keylog.log` as an optional, secret-adjacent analyzer aid containing proxy-owned TLS session material. It MUST explain final-path creation before traffic, incremental live use, nonempty-only retention, and the absence of target-process key extraction.
- **FR-011**: The reference MUST distinguish `proxy.jsonl`, `process-trace.jsonl`, `compatibility.json`, and `cleanup.json` by the facts each sidecar owns and by the limits of those facts.
- **FR-012**: The reference MUST document every omission reason currently emitted in a finalized v0.7.0 manifest: `writer-failed`, `no-http-semantics`, `not-requested`, and `not-produced`, including the artifact roles and severities with which they occur.
- **FR-013**: The reference MUST distinguish manifest omissions from application observation reasons and from cleanup resource statuses so similarly worded values are not treated as one shared vocabulary.
- **FR-014**: The reference MUST document correlation anchors where currently present, including `session_id`, target identity, `flow_id`, proxy connection id, event time bounds, process id and image, role, and attribution state, and MUST say which artifacts do not carry all of them.
- **FR-015**: The reference MUST explain that absent correlation anchors mean the join is unavailable, not that the corresponding packet, process, or application activity did not occur.
- **FR-016**: The reference MUST include a synthetic manifest example containing no local paths, real title names, account identifiers, access tokens, private endpoints, host identifiers, or real TLS secrets.
- **FR-017**: The reference MUST provide handling guidance that preserves original observations, recommends sharing only a reviewed scrubbed copy, and accurately describes confirmation-gated stale-residue cleanup.
- **FR-018**: The Deep Capture compatibility and CLI reference pages MUST link to the output reference as the authoritative bundle and artifact contract without duplicating its complete matrix.
- **FR-019**: Public terminology and links MUST use existing glossary definitions for session bundle, HAR, local development certificate authority, and proxy-owned TLS key-log export.
- **FR-020**: Documentation validation and the production static export MUST pass, and focused source checks MUST prove coverage of all artifact roles, state tokens, omission reasons, and required cross-links.
- **FR-021**: The slice MUST remain documentation-only and MUST NOT change runtime behavior, dependencies, workflows, toolchains, release machinery, or the master specification.

### Key Entities

- **Session Bundle**: The directory of one Deep Capture run, rooted at a final manifest when finalization succeeds and containing distinct packet, application, diagnostic, compatibility, and cleanup artifacts.
- **Manifest**: The durable index that names session identity and state, artifact declarations, omissions, correlation summary, and aggregate cleanup result.
- **Artifact Declaration**: A produced artifact's role, relative path, authority, sensitivity, content type, and required flag.
- **Omission**: A declaration that one expected role was not produced, carrying a role, exact reason token, and severity.
- **Correlation Anchor**: A structured identifier or time bound that can join observations across packet, application, proxy, process, compatibility, and manifest contexts.
- **Cleanup Resource Result**: One resource's cleanup status and reason, owned by `cleanup.json` and summarized by the manifest.

## Success Criteria

### Measurable Outcomes

- **SC-001**: One artifact matrix covers 9 of 9 current Deep Capture roles with an authority, sensitivity, lifetime, required status, and production or omission condition for each.
- **SC-002**: Readers can distinguish all 3 session states and all 4 finalized-manifest omission reason tokens from the reference without consulting implementation source.
- **SC-003**: The reference contains zero claims that Capture pcapng, Capture packet JSON Lines, and Deep Capture sidecars carry equivalent facts.
- **SC-004**: Every correlation field named by the reference appears in at least one current v0.7.0 artifact, and no artifact is claimed to carry an anchor it does not emit.
- **SC-005**: The synthetic example contains zero local or account-derived values and zero usable TLS secrets.
- **SC-006**: The compatibility and CLI references each contain a direct link to the bundle contract, with zero duplicated full artifact matrices.
- **SC-007**: Documentation checks, production site export, repository lint, and the complete CI-parity gate finish with zero failures.

## Assumptions

- S092 implements GitHub issue #248 after issues #244, #245, and #247 in the `Post-v0.7.0 documentation` milestone.
- The current Rust implementation and its controlled Deep Capture tests are the authority for the exact v0.7.0 artifact paths, values, and omission tokens when older forward-looking specification prose is broader.
- This slice corrects and expands `site/content/docs/reference/output-formats.mdx` in place rather than adding a competing page, so existing links remain stable.
- Issue #246 owns the mechanical website-to-clap command-tree gate, and issue #249 owns the rendered production UX and accessibility audit.
- The reference documents operator-visible retention and cleanup behavior. It does not introduce a retention policy, automatic expiry, or new scrub command.
