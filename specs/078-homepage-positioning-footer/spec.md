# Feature Specification: Homepage Positioning And Next-Command Footer

**Feature Branch**: `codex/078-homepage-positioning-footer`

**Created**: 2026-08-26

**Status**: Implemented

**Input**: User description: "Consolidate issues #232 and #208: rewrite the homepage first-viewport positioning for Capture and Deep Capture, correct prerequisite claims, use synthetic verified CLI output, and label the targets next-command footer."

## Clarifications

### Session 2026-08-26

- Q: What exact label should distinguish the populated target listing's suggestion from data output? A: Render `Next command:  fragcap capture <row>` after one blank line. The label is literal and the row remains selected by the existing readiness rules.
- Q: Should S078 freeze replacement homepage prose verbatim? A: No. The specification fixes factual and voice requirements plus prohibited claims. Exact prose may improve during implementation while those requirements remain true.
- Q: Should the homepage continue using real game titles in its specimen? A: No. The specimen uses synthetic target handles and current CLI columns so public documentation carries no local-title implication and cannot be mistaken for a compatibility claim.
- Q: Does S078 change homepage layout or navigation? A: No. Preserve the masthead, restrained instrument layout, dependency diagram, capability list, links, and single primary action.
- Q: Does S078 also fix finding fidelity in the ENGINE and SENSITIVITIES columns? A: No. That behavior is tracked independently by #211. S078 must not make a stronger claim than the current output can support.

## User Scenarios & Testing

### User Story 1 - Understand Fragcap In One Viewport (Priority: P1)

A technically competent visitor reaches the homepage and immediately understands the result fragcap provides, why ordinary packet capture does not carry process ownership, and how Capture differs from Deep Capture.

**Why this priority**: The live homepage currently makes technically inaccurate and incomplete claims on the project's highest-visibility surface.

**Independent Test**: Build the documentation site and inspect the first viewport. It leads with process-attributed game traffic, describes correlation rather than destroyed operating-system data, distinguishes both modes, and contains none of the retired claims.

**Acceptance Scenarios**:

1. **Given** a visitor who knows packet capture but not fragcap, **When** they read the opening block, **Then** they learn that packet records do not ordinarily preserve process ownership and fragcap correlates flows with Windows socket and process-lifecycle observations.
2. **Given** the two product modes, **When** the visitor reads the positioning block, **Then** Capture is described as passive observation and Deep Capture as explicit, target-scoped local proxy inspection for compatible targets.
3. **Given** a flow that cannot be attributed or traffic that cannot be inspected, **When** the product is described, **Then** the page makes no universal attribution or decryption claim.

---

### User Story 2 - Recognize The Next Command (Priority: P1)

An operator runs `fragcap targets` after machine findings have been shown and can distinguish the suggested capture command from table rows and machine observations.

**Why this priority**: The current bare command is valid but visually reads as stray output or another machine finding, which makes the primary next action unclear.

**Independent Test**: Render populated target output with and without machine findings. The output contains one blank-line-separated `Next command:` line, while a bare `fragcap` still differs from explicit `fragcap targets` by exactly the existing help footer.

**Acceptance Scenarios**:

1. **Given** a populated listing, **When** fragcap selects the best row using existing readiness and install-presence rules, **Then** it prints `Next command:  fragcap capture <row>` after one blank line.
2. **Given** a populated listing followed by a `Machine:` section, **When** the next command is rendered, **Then** its label and blank-line boundary prevent it from being read as a machine finding.
3. **Given** the same listing rendered through bare `fragcap` and explicit `fragcap targets`, **When** their bytes are compared, **Then** the only difference remains the bare invocation's existing help footer.

---

### User Story 3 - Trust The Homepage Specimen And Prerequisites (Priority: P2)

A visitor sees a compact `fragcap targets` specimen that uses synthetic targets, matches the current column and footer contract, and is followed by mode-accurate dependency guidance.

**Why this priority**: A hand-maintained specimen that shows stale columns, real titles, or an unlabeled footer undercuts the accuracy promised by the surrounding prose.

**Independent Test**: Compare the homepage specimen against the current human listing contract and build the static site. The specimen uses synthetic handles, current columns, and the exact labelled next-command form; the callout identifies Npcap as the live-capture prerequisite, Wireshark as a recommended analyzer, and `doctor` as the readiness authority.

**Acceptance Scenarios**:

1. **Given** the homepage specimen, **When** it is compared with the current CLI, **Then** its header and next-command label use the same vocabulary and ordering.
2. **Given** public documentation, **When** target names are inspected, **Then** the specimen contains only synthetic placeholders and does not imply compatibility for an actual title.
3. **Given** the prerequisite callout, **When** a reader distinguishes requirements from recommendations, **Then** Npcap is required for live packet capture, Wireshark is recommended for analysis, and `fragcap doctor` is the source of Capture and Deep Capture readiness findings.

### Edge Cases

- A target listing with no rows keeps its existing labelled `Add one` and `Scan a folder` suggestions; the populated-only `Next command` line is not added.
- A listing where every install root is missing keeps the existing deterministic fallback to row one; this slice changes presentation, not selection.
- An unattributed flow remains a valid Capture outcome and must not make the homepage wording false.
- Deep Capture can produce metadata-only, unsupported, or unknown outcomes and must not be described as universally decrypting traffic.
- Wireshark may be absent while fragcap still writes analyzer-compatible output; the page must not call Wireshark a prerequisite.
- The Npcap installer may be obtained through explicit `doctor --fix` confirmation, but fragcap still never bundles, hosts, embeds, or redistributes it.

## Requirements

### Functional Requirements

- **FR-001**: The homepage opening MUST lead with the user outcome of process-attributed game traffic rather than a packet-count thought experiment.
- **FR-002**: The homepage MUST explain that ordinary packet records do not preserve process ownership and that fragcap correlates captured flows with separate Windows socket and process-lifecycle observations.
- **FR-003**: The homepage MUST NOT claim that the operating system destroyed the relevant information, that a launch chain has a fixed hop count, or that fragcap resolves every flow.
- **FR-004**: The homepage MUST describe Capture as passive observation and Deep Capture as explicit, target-scoped local proxy inspection for compatible targets.
- **FR-005**: The homepage MUST NOT claim universal Deep Capture routing, certificate acceptance, protocol support, inspection, or decryption.
- **FR-006**: The homepage MUST state that Npcap in WinPcap-compatible mode is required for live packet capture and that fragcap never bundles, hosts, embeds, or redistributes Npcap.
- **FR-007**: The homepage MUST describe Wireshark as a recommended analyzer rather than a prerequisite.
- **FR-008**: The homepage MUST direct readiness questions for Capture and Deep Capture to `fragcap doctor` without claiming that every reported Deep Capture warning blocks Capture.
- **FR-009**: The populated target listing MUST render the exact label `Next command:  fragcap capture <row>` after one blank line.
- **FR-010**: The row selected for the next command MUST remain governed by the existing readiness and install-presence precedence.
- **FR-011**: Bare `fragcap` and explicit `fragcap targets` MUST continue to differ by exactly the existing help footer line and no other output.
- **FR-012**: Empty target output MUST retain its current labelled population suggestions and MUST NOT add the populated-listing next-command label.
- **FR-013**: The homepage command specimen MUST use current `TARGET`, `CAPTURE`, `ENGINE`, and `SENSITIVITIES` columns plus the exact next-command label.
- **FR-014**: The homepage command specimen MUST use synthetic target handles, products, and findings only.
- **FR-015**: The master specification sections governing target-listing output, site structure, and brand voice MUST be reconciled with S078 in the same change.
- **FR-016**: Historical S057 artifacts MUST remain unchanged; S078 records that its verbatim-copy requirement is superseded.
- **FR-017**: The homepage masthead, dependency diagram, capability-list structure, navigation, restrained visual language, and single primary action MUST remain intact.
- **FR-018**: Homepage capability statements MUST distinguish mode-specific safety and MUST NOT describe all of fragcap as passive-only.
- **FR-019**: Tests MUST prove the exact populated footer, machine-section separation, empty-state preservation, and bare-versus-explicit output invariant.
- **FR-020**: Documentation verification MUST prove the retired packet-count, destroyed-information, fixed-hop, passive-only, and two-prerequisite wording is absent from the authored homepage and generated site.

## Success Criteria

### Measurable Outcomes

- **SC-001**: The built homepage's first positioning block names process-attributed game traffic, Capture, and Deep Capture within the first viewport.
- **SC-002**: All five retired homepage claims named by FR-020 have zero matches in the authored homepage and generated static export.
- **SC-003**: Populated target output contains exactly one `Next command:` label, and its row equals the row selected before S078 for the same target set.
- **SC-004**: Empty target output contains zero `Next command:` labels and retains both existing population suggestions.
- **SC-005**: The homepage specimen contains zero real game titles and exactly the current four listing column names.
- **SC-006**: The full repository CI gate and production documentation build pass with no unrelated output, command, layout, or navigation regression.

## Assumptions

- The issue #232 candidate copy is directional rather than verbatim; implementation may tighten wording while preserving every requirement above.
- The current narrow Deep Capture MVP and its compatibility limits remain as specified by S075 and S076.
- Issue #211 owns fidelity presentation inside ENGINE and SENSITIVITIES cells and is outside this slice.
- No new glossary term is expected. Existing entries for Capture, Deep Capture, flow attribution, Npcap, and Wireshark remain authoritative.
- The site remains a static Fumadocs/Next.js application and no new frontend dependency is needed.
