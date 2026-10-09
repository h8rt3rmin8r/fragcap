# Feature Specification: Initiating-user bundle access and historical repair

**Feature Branch**: `codex/s170-initiating-user-bundle-access`

**Created**: 2026-10-09

**Status**: Implemented and locally verified; hosted review checkpoint pending

**Input**: Owner-authorized S170 autopilot, implementing issue #464 in full, followed by an official PR, exact-head CI and at most two external review rounds. Human merge and release remain separate.

## Scope and authority

S170 follows human-merged S169 at `edc8afd6c74f5f8a64ea01fcd1e2f4aeea044894`. It establishes ordinary-user enumeration and read access to retained session output produced under elevation, while preserving restricted sensitive material and recovering historical inaccessible bundles. It traces to master specification sections 13.7, 19, 25 and 26.3. Storage collection, empty-container purge (#458), target enrichment (#155), community sync (#94), dependency PR #470 and release publication are outside this slice.

## User Scenarios & Testing

### User Story 1 - Read newly retained output from the normal desktop (Priority: P1)

An operator runs capture with administrative rights and then opens the retained directory and its evidence in Explorer or an analyzer from their ordinary Windows session. The recipient is the intended individual Windows account, independently of the creating token's default owner or Administrators membership.

**Why this priority**: Retained evidence is unusable when inspection itself requires elevation, and mixed access to lifecycle files prevents reliable diagnosis.

**Independent Test**: Synthetic bundle production under an elevated or equivalent controlled Windows context is followed by actual enumeration and reads under the intended unelevated context, including inherited, temporary and final artifacts; an unrelated principal cannot read them.

**Acceptance Scenarios**:

1. **Given** same-account elevation, **When** a session publishes retained output, **Then** that account's unelevated context enumerates the container and reads every intended artifact, including lifecycle journals.
2. **Given** different-account elevation, **When** an exact intended recipient and destination can be established through a supported explicit workflow, **Then** output belongs to that recipient's storage contract; otherwise a named refusal precedes output or session effects.
3. **Given** a denied permission application or access verification, **When** publication is attempted, **Then** the report distinguishes written, verified-accessible and unresolved output and offers the exact recovery action without claiming accessible success.

### User Story 2 - Recover an existing inaccessible bundle (Priority: P1)

An operator inspects and repairs one exact historical fragcap bundle for one validated recipient, retaining all contents and the existing lifecycle and retention obligations.

**Why this priority**: Correct permissions on future sessions alone leave the reported historical outputs inaccessible.

**Independent Test**: Controlled old-policy directories and independently protected sidecars reproduce denied unelevated reads; exact confirmed repair enables those reads without altering file contents, unrelated files or retention state.

**Acceptance Scenarios**:

1. **Given** a recognized historical bundle with recoverable inaccessible artifacts, **When** the operator approves an exact previewed repair, **Then** all intended retained files become readable by the intended unelevated recipient and every changed permission is reported.
2. **Given** an unrelated directory, ambiguous ownership, a reparse-point escape, unsupported state or changed preview, **When** repair is requested, **Then** a bounded refusal leaves unrelated data untouched.
3. **Given** interrupted or partially failed permission repair, **When** inspection or retry occurs, **Then** current results remain truthful and retry is idempotent without deleting files or claiming unresolved access is repaired.

### Edge Cases

- Same-account full, split, filtered and ordinary Windows tokens; a different elevated producer and intended recipient; missing, expired or mismatched recipient evidence.
- Inaccessible parent traversal, protected child ACLs, zero-length sidecars, newly created and atomically renamed artifacts, custom destinations, and concurrent artifact creation.
- Legacy bundles with Administrators ownership, unreadable manifest or journal, unknown files, active bundles, malformed identity or unsupported record version.
- Junctions, symlinks, path aliases, ambiguous roots, path replacement after preview, denied security-descriptor updates and failed effective-access verification.
- Repair does not imply trust/proxy recovery, evidence cleanup, retention collection or empty-container purge.

## Requirements

### Functional Requirements

- **FR-001**: Establish and validate one exact intended recipient SID and an unelevated access context before creating or protecting Windows session output. Administrator-group ownership and arbitrary desktop-session discovery MUST NOT substitute for recipient identity.
- **FR-002**: Same-account elevation MUST preserve the initiating user's ordinary enumeration and read access to the output directory and every retained artifact.
- **FR-003**: Different-account elevation MUST have explicit supported recipient and destination semantics. Unknown, mismatched or unprovable recipients MUST cause a truthful refusal before session output or external effects, rather than silently selecting the elevated account's storage.
- **FR-004**: One access contract MUST cover necessary parent traversal, directory inheritance, captures, manifests, application records, proxy and process evidence, cleanup summaries, lifecycle/resource/sensitive-action journals, temporary files and atomic publication. Every missing output-container intermediate MUST receive the exact recipient contract at creation and pass actual recipient enumeration before descendants or session effects; existing accessible ancestors MUST retain their descriptors.
- **FR-004a**: Historical repair MUST include necessary traversal/enumeration corrections for exact proven fragcap-owned output containers above the bundle. These parent objects MUST be part of the confirmed bounded inspection; their correction MUST NOT propagate into sibling bundles or rewrite arbitrary profile/custom ancestors. An obstructing unowned ancestor MUST produce an explicit unresolved refusal.
- **FR-005**: Restricted artifacts MUST grant the exact intended principals, without Everyone, all Users or unrelated account grants. Producer and SYSTEM requirements MUST be explicit and must not be mistaken for recipient access.
- **FR-006**: Effective access MUST be checked in the intended unelevated context, covering actual enumeration and file reads as well as required creation/inheritance paths. The elevated producer's successful read is insufficient.
- **FR-007**: Provide read-only inspection of one exact retained fragcap bundle, including recipient identity, path ownership, inaccessible artifacts, limits and proposed permission changes. Ordinary Doctor MUST remain read-only.
- **FR-008**: Provide exact user-directed preview-and-confirm repair for already-created inaccessible bundles, including independently protected lifecycle sidecars. Repair MUST preserve all content bytes, names, retention choices and recovery obligations.
- **FR-009**: Inspection/repair MUST validate whole-bundle provenance and path containment, reject unrelated or ambiguously owned paths, and refuse reparse-point escapes and stale preview identity. Bounds and unknown/unreadable/unsupported state MUST remain visible.
- **FR-010**: Permission or verification failure MUST precede any successful accessible-publication claim. Human and structured output and analyzer guidance MUST distinguish artifact written, recipient access verified, and unresolved repair.
- **FR-011**: Repair MUST report per-path actual effects and failures, support idempotent retry after interruption or partial completion, and MUST NOT collect evidence, remove containers or change trust/proxy resources.
- **FR-012**: Controlled Windows tests MUST prove new and historical bundle enumeration/read in unelevated contexts, inherited and atomic-created access, rejected unrelated-principal access, exact repair content conservation, identity/path refusals and partial failures. No game, live packet capture or real sensitive owner bundle is needed.
- **FR-013**: Master specification, public output/recovery help and documentation MUST describe shipped recipient/access/repair semantics and controlled verification limits. Existing Capture/Deep Capture consent, cleanup and retention authority MUST remain intact.
- **FR-014**: Spec-Kit consistency analysis and repository CI-parity gates MUST pass before publication; all returned external findings MUST be addressed within at most two review rounds before owner merge handoff.

### Key Entities

- **Output recipient**: Exact Windows account SID, proven unelevated access context and applicable destination; distinct from elevated producer and object owner.
- **Bundle access contract**: Intended principals, retained artifact population, inheritance/traversal obligations and verified access outcomes.
- **Access inspection**: Exact bundle and recipient identity, bounded inventory, ownership evidence, current access and repair proposal.
- **Access repair**: Confirmed current inspection, per-path permission effects and final recipient-context verification; content and retention remain unchanged.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Every required new-output scenario permits normal recipient enumeration and complete reads for 100% of its retained artifact population, while unrelated-principal scenarios deny those reads.
- **SC-002**: Every accepted historical repair preserves 100% of fixture content bytes and permits normal recipient access to all intended artifacts; every rejected unrelated, ambiguous, escaped or stale case changes zero unrelated objects.
- **SC-003**: Every failed or partial permission/verification scenario reports its exact unresolved result without an accessible-success claim; repeated repair is idempotent.
- **SC-004**: Every #464 acceptance criterion maps to controlled repository evidence and documentation, and the full automated gate passes without new security-test waivers.

## Assumptions

The repository's existing bundle, resource registry, path-containment and consent mechanisms remain the authority for retained content and external effects. Windows-specific token and ACL adapters belong above platform-neutral core. A privileged producer may have access to sensitive output as an explicitly identified principal; unrelated local users do not. The slice implements access correction, not universal game compatibility, an independent whole-product audit or a new release.

## Clarifications

### Session 2026-10-09 (autopilot decisions)

- Recipient selection: use exact individual identity and proven unelevated context; reject guessed desktop accounts and group-relative ownership. Different-account execution requires explicit proof and destination semantics, with an early truthful refusal when proof is unavailable.
- Historical repair authority: use an exact bounded bundle inspection and explicit confirmation. New-output correction alone is insufficient; retained inaccessible lifecycle sidecars are included.
- Repair versus retention: content and empty containers remain intact. Neither permission repair nor a read-only inspection authorizes #458 collection or purge.
- Completion evidence: controlled synthetic Windows scenarios satisfy implementation acceptance; no mutation of the owner's historical sensitive outputs or future field rerun is required.
