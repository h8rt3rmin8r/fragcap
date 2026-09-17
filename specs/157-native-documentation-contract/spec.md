# Feature Specification: S157 complete native documentation contract

**Feature Branch**: `codex/s157-native-documentation-contract`

**Created**: 2026-09-17 UTC

**Status**: Implemented

**Input**: User selected the documentation slice after S156. Complete the agent-runnable native product contract under #427 and parent #331 without claiming independent security acceptance, installed QUIC retest, or final Deep Capture completion.

## User Scenarios & Testing

### User Story 1 - Follow one current product contract (Priority: P1)

An operator needs one coherent current guide that distinguishes passive Capture from explicit Deep Capture and accurately explains setup, supported traffic, refusals, artifacts, recovery, sensitivity, finite bounds, packaging, migration, and completion status.

**Why this priority**: The current pages contain the necessary material, but the reader-facing readiness page still describes S154 as a candidate and the contract is not yet closed against required sections and obsolete native-transition language.

**Independent Test**: Build and inspect the current documentation site from repository content, then verify every required product-contract topic and section is present, internally linked, searchable, accessible, and consistent with the published baseline and current source boundary.

**Acceptance Scenarios**:

1. **Given** a reader starting from installation or architecture, **When** they follow Capture or Deep Capture guidance, **Then** they reach current commands, exact support and refusal boundaries, artifact authority, recovery, privacy, packaging, and API guidance without obsolete external-backend instructions.
2. **Given** behavior merged after the published baseline, **When** current documentation names it, **Then** the prose distinguishes current source from published v0.10.1 instead of claiming the release contains unpublished behavior.
3. **Given** the independent review and final completion gates are still open, **When** any current status page is read, **Then** Deep Capture remains functional but incomplete and no universal decryption, pinning bypass, live-title compatibility, or final approval is implied.

### User Story 2 - Trust every worked example (Priority: P1)

An operator or integrator needs every current command and linked artifact specimen to be executable or readable by the same authority the product uses, without running the sensitive installed product during documentation verification.

**Why this priority**: #331 requires every command example to parse and every artifact example to validate. A prose-only review cannot prevent example drift.

**Independent Test**: Discover every current fragcap command example and linked artifact specimen, pass commands through the production command parser without dispatch, and validate artifacts through their exact versioned readers or projection contracts.

**Acceptance Scenarios**:

1. **Given** any current command example in the README or nonhistorical site, **When** documentation verification runs, **Then** the production parser accepts it without dispatch and any retired consent input causes failure.
2. **Given** a linked manifest, packet, application, correlation, cleanup, or API example, **When** documentation verification runs, **Then** an exact product reader or non-ignored executable contract owns it.
3. **Given** an absent, malformed, unsafe, duplicate, or unowned example reference, **When** verification runs, **Then** it fails with a bounded diagnostic that identifies the affected contract row.

### User Story 3 - Review complete traceability without false acceptance (Priority: P2)

A maintainer or independent reviewer needs a closed documentation inventory that maps each required topic and section to current authored guidance and executable authority while preserving the distinction between engineering traceability and independent whole-product acceptance.

**Why this priority**: S154 established topic-level readiness. S157 must turn it into the complete current contract baseline that later independent findings can amend rather than reconstruct.

**Independent Test**: Mutate each inventory authority class in isolation and verify missing topics, sections, current pages, executable tests, example ownership, status boundaries, and forbidden obsolete language fail deterministically.

**Acceptance Scenarios**:

1. **Given** the complete inventory, **When** verification runs, **Then** every required topic and section maps to one current page and at least one exact executable authority.
2. **Given** documentation engineering is complete, **When** its status is reported, **Then** #333, #413, parent #331, #334, and #278 remain open until their external evidence exists.
3. **Given** later independent findings, **When** they become available, **Then** they can be reconciled as bounded amendments to the contract rather than being inferred or fabricated now.

### Edge Cases

- A current page links historical material: retain the link only when it is explicitly labeled historical and not used as current executable authority.
- A command specimen appears in a plain-text fence or shell transcript: discover the fragcap invocation regardless of fence language while ignoring output and Mermaid diagrams.
- A placeholder such as a workflow identifier, inventory identifier, path, or target handle is intentionally not executable as written: replace only the placeholder token for parser validation without inventing a real local identity or dispatching the command.
- A linked artifact has no safe committed specimen: require an exact non-ignored reader or projection test and document that no shortened public specimen is authoritative.
- Current source documentation describes S154 through S156 engineering that is not in v0.10.1: label the source/release boundary without downgrading accurate current guidance.
- Independent-review findings do not yet exist: preserve the reconciliation hook and incomplete status rather than inventing a clean audit.

## Requirements

### Functional Requirements

- **FR-001**: Current documentation MUST present one consistent published v0.10.1 baseline and MUST distinguish later merged documentation, review tooling, and package-certification engineering from those published bytes.
- **FR-002**: Current guidance MUST cover architecture, first-run setup, Capture modes, CLI, protocols and routing, refusals, artifacts and correlation, Doctor and recovery, security and privacy, finite bounds, packaging and migration, and the stable library API.
- **FR-003**: Support and refusal guidance MUST preserve exact route, trust, protocol, launch, address-family, evidence, loss, and cleanup boundaries without universal decryption, pinning-bypass, compatibility, or completion claims.
- **FR-004**: Every current fragcap command example MUST be discovered and accepted by the production command parser without dispatch; retired Deep Capture consent inputs MUST remain rejected.
- **FR-005**: Every linked artifact specimen or example authority MUST be validated by the actual versioned product reader, exact projection contract, or exact non-ignored executable test appropriate to that artifact.
- **FR-006**: A versioned closed documentation inventory MUST bind each required topic to a current authored page, required sections, example authority, and exact executable evidence. Missing, duplicate, unknown, unsafe, historical-as-current, absent, ignored, or feature-inapplicable references MUST fail.
- **FR-007**: Current pages MUST remove stale S154 candidate wording and obsolete external proxy backend instructions while leaving historical specifications, changelogs, release handoffs, and slice records intact.
- **FR-008**: Documentation verification MUST cover the production static site, exported routes, internal destinations and anchors, accessibility, navigation, search, and documentation lint without unexpected skips.
- **FR-009**: Documentation status MUST keep independent review #333, installed QUIC retest #413, parent #331 reconciliation, final gate #334, and epic #278 explicitly outstanding until their actual evidence exists.
- **FR-010**: The slice MUST NOT change product runtime, dependencies, storage schemas, release version, tags, publication, owner policy, consent, routing, trust, cleanup, or evidence behavior, and MUST NOT run installed sensitive software, real games, or real trust mutation on the owner workstation.
- **FR-011**: Issue #427 MUST trace S157 as a child of #331, and the slice MUST leave a bounded reconciliation point for later independent findings rather than closing the parent prematurely.

### Key Entities

- **Documentation contract row**: Stable topic identity, current page, required section identities, command or artifact example authority, executable tests, applicable features, and completion-boundary classification.
- **Current command example**: A nonhistorical fragcap invocation that must parse through the product command tree without dispatch after documented placeholders are normalized.
- **Artifact example authority**: A committed specimen or exact contract reference owned by the product reader, projection, or non-ignored test appropriate to its schema and lifecycle.
- **Status boundary**: Published v0.10.1 identity, later current-source engineering, independent review state, installed retest state, and final completion authority kept as distinct facts.

## Success Criteria

### Measurable Outcomes

- **SC-001**: All required documentation topics and sections have exactly one valid current authority row, and mutation coverage demonstrates a specific failure for every authority class.
- **SC-002**: One hundred percent of current fragcap command examples parse through both applicable command trees without dispatch, and zero retired consent inputs appear in current guidance.
- **SC-003**: One hundred percent of linked artifact examples have an exact product reader, projection, or executable test authority, with no approximate validator presented as authoritative.
- **SC-004**: Production site build, route inventory, links, anchors, accessibility, navigation, search, documentation lint, and full repository CI complete with zero failures and zero unexpected skips.
- **SC-005**: Current documentation contains zero stale S154 candidate claims and zero current external-backend instructions, while every completion-status surface preserves the open #333/#413/#331/#334/#278 boundary.

## Assumptions

- Published v0.10.1 remains the current downloadable release during this slice; current merged S154 through S156 changes are documentation, review tooling, test, and package-certification engineering rather than a new product release.
- Existing product readers, command parser checks, site tests, and documentation lint are the primary verification authorities and should be extended rather than replaced.
- Actual independent-review findings are unavailable and cannot be authored by the implementation agent. S157 completes the baseline they will review and leaves later reconciliation explicit.
- Existing historical records remain immutable context. Only current guidance and current contract authorities are corrected.
