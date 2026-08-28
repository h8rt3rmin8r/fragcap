# Feature Specification: Public Entry Point Reconciliation

**Feature Branch**: `codex/088-public-entry-points`

**Created**: 2026-08-28

**Status**: Draft

**Input**: User description: "Kick off S088 from issue #244 under the autopilot protocol"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Understand the Shipped Product (Priority: P1)

A prospective user arriving through the repository or documentation front door receives one accurate description of fragcap v0.7.0: Capture is passive process-attributed packet capture, and Deep Capture is shipped, explicit, scoped, reversible local proxy inspection for authorized sessions.

**Why this priority**: The current public entry points contradict the released product. That violates constitution principle P-11 and can cause users to dismiss a shipped capability as unavailable or misunderstand its security posture.

**Independent Test**: Read the repository landing page, public documentation index, and repository description without following deeper reference links. Each surface identifies both shipped modes, distinguishes their postures, and makes no universal inspection claim.

**Acceptance Scenarios**:

1. **Given** a reader opens the repository landing page, **When** they read the product summary and status, **Then** they learn that Capture and Deep Capture both ship in v0.7.0 and what each mode does.
2. **Given** a reader opens the public documentation index, **When** they read the product introduction, **Then** its definitions agree with the repository landing page.
3. **Given** a reader sees the repository description in GitHub, **When** they compare it with the landing page, **Then** it names both modes without promising universal attribution or application-layer inspection.

---

### User Story 2 - Contribute Against the Current Repository (Priority: P1)

A contributor receives current repository status, security boundaries, development workflow, and Npcap policy rather than pre-implementation instructions from before the Cargo workspace existed.

**Why this priority**: Stale contributor guidance sends valid work toward retired constraints and describes the governing security boundary incorrectly now that Deep Capture ships.

**Independent Test**: Read both contributor entry points and verify that they describe the existing workspace, current modes, current pull-request workflow, and the complete No Covert Target Instrumentation boundary.

**Acceptance Scenarios**:

1. **Given** a contributor opens either contributor guide, **When** they read the project status, **Then** neither surface says the repository is pre-implementation, lacks a Cargo workspace, or ends at the original S01 through S18 roadmap.
2. **Given** a contributor reviews prohibited techniques, **When** they compare the list with constitution P-1, **Then** the contributor guide distinguishes passive Capture from explicit Deep Capture while preserving the complete technique denylist.
3. **Given** a contributor reads the Npcap policy, **When** they compare it with the current release build and constitution, **Then** the guide distinguishes the shipped browser handoff from the optional user-confirmed vendor fetch and forbids bundling or redistribution.

---

### User Story 3 - File an Actionable Current Issue (Priority: P2)

A reporter filing a bug or feature request sees current commands, release expectations, Npcap requirements, and planning pointers.

**Why this priority**: Retired commands and requirements create unusable reproduction reports and make feature reporters search an obsolete roadmap.

**Independent Test**: Open each public issue form and verify every example, required field, scope confirmation, and planning link against v0.7.0 behavior and governance.

**Acceptance Scenarios**:

1. **Given** a reporter opens the bug form, **When** they inspect its reproduction example and environment fields, **Then** the form uses current commands and current Npcap requirements.
2. **Given** a reporter opens the feature form, **When** they read its scope checks and planning guidance, **Then** the form uses the current P-1 boundary and does not describe v0.2.0 or the original roadmap as future work.
3. **Given** either issue form is parsed by GitHub's issue-form rules, **When** validation runs, **Then** the form remains structurally valid.

### Edge Cases

- Capture must remain described as passive even while the product summary also names Deep Capture.
- Deep Capture wording must state explicit scope, reversibility, and authorization without implying certificate-pinning bypass, target key extraction, system-wide proxy fallback, or universal protocol support.
- Npcap remains separately licensed and never bundled or redistributed; the shipped `doctor --fix` opens the official download page after explicit confirmation, while a `net`-enabled source build may fetch and launch the vendor's own signed installer after the same confirmation.
- Historical release records and completed slice directories remain historical truth and are not rewritten as if v0.7.0 existed earlier.
- Deeper pages owned by issues #245 through #248 may remain stale during this slice, but entry points must link to current destinations and must not repeat their stale claims.
- Public examples must use synthetic targets and contain no account, local path, endpoint, or payload material.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Every in-scope public entry point MUST describe Capture and Deep Capture as shipped first-class modes in v0.7.0.
- **FR-002**: Every in-scope product definition MUST describe Capture as passive process-attributed packet capture.
- **FR-003**: Every in-scope product definition MUST describe Deep Capture as explicit, selected-target-scoped, reversible local proxy inspection for authorized sessions.
- **FR-004**: No in-scope surface MAY claim that Deep Capture is absent from released binaries or merely planned.
- **FR-005**: No in-scope surface MAY claim that the repository is pre-implementation, lacks a Cargo workspace, or stops at the S01 through S18 roadmap.
- **FR-006**: Npcap guidance MUST state that fragcap never bundles, hosts, caches as its own, or redistributes Npcap, while distinguishing the shipped `doctor --fix` browser handoff from the optional `net`-enabled vendor-fetch path and their explicit confirmation requirement.
- **FR-007**: The repository landing page MUST present current v0.7.0 status, current primary commands, and current Capture and Deep Capture capabilities without expanding into the walkthrough, CLI-reference, architecture-diagram, or bundle-reference work owned by later issues.
- **FR-008**: Both contributor entry points MUST agree with constitution P-1, P-8, P-10, and P-11 and with the current pull-request and Spec Kit workflow.
- **FR-009**: The bug issue form MUST use a current reproduction command, current version expectation, and the single still-required Npcap installation option.
- **FR-010**: The feature issue form MUST use the current No Covert Target Instrumentation boundary and current planning pointers.
- **FR-011**: The GitHub repository description MUST identify both Capture and Deep Capture without promising universal attribution or inspection.
- **FR-012**: All changed links and issue forms MUST remain structurally valid, and the production documentation site MUST build successfully.
- **FR-013**: All changed text MUST be UTF-8 without BOM, contain no mojibake, and satisfy repository punctuation and Markdown conventions.
- **FR-014**: The slice MUST NOT change runtime behavior, CLI grammar, capture behavior, output formats, dependency graph, workflow files, release configuration, or deeper documentation pages assigned to issues #245 through #249.

### Key Entities

- **Public entry point**: A first-contact surface in scope for this slice: `README.md`, `CONTRIBUTING.md`, the public documentation index, the public contributing page, a GitHub issue form, or the GitHub repository description.
- **Product definition**: The shared statement that distinguishes passive Capture from explicit, scoped, reversible Deep Capture.
- **Npcap policy**: The licensing and acquisition rule that prohibits bundling and redistribution, makes the shipped `doctor --fix` open the official download page, and permits a `net`-enabled source build to fetch the vendor installer after user confirmation.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All six in-scope file surfaces plus the GitHub repository description agree that Capture and Deep Capture ship in v0.7.0.
- **SC-002**: Searches of in-scope surfaces find zero claims that Deep Capture is planned or absent, zero claims that the repository is pre-implementation, and zero current-status claims that work stops at S18 or v0.2.0.
- **SC-003**: Both issue forms pass structural validation and contain only commands accepted by the v0.7.0 command surface.
- **SC-004**: Every changed local link resolves and the production documentation site builds successfully.
- **SC-005**: Repository documentation, glossary, encoding, punctuation, and full CI-parity gates pass.
- **SC-006**: The completed diff contains no runtime source, dependency, workflow, toolchain, release-configuration, or out-of-scope documentation changes.

## Assumptions

- v0.7.0 is the current released baseline for this slice.
- Issue #244 is the complete scope authority; issues #245 through #249 own the deeper documentation work it explicitly excludes.
- Existing glossary entries already define the terms used in the reconciled product definition.
- No due date or next release number is implied by completing this correction.
