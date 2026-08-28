# Feature Specification: Production Accessibility Remediation

**Feature Branch**: `codex/095-production-accessibility-remediation`

**Created**: 2026-08-28

**Status**: Draft

**Input**: User description: "Kick off S095", grouping GitHub issues #263, #264, #265, and #268 into one production accessibility remediation slice.

## User Scenarios & Testing

### User Story 1 - Reach The Primary Content Directly (Priority: P1)

As a keyboard or assistive-technology user, I need every public page to expose one unambiguous primary content region and an early way to bypass repeated navigation, so I can reach the page content without traversing the site chrome on every route.

**Why this priority**: The production audit found that every documentation route lacks a primary landmark and skip target, while the homepage exposes two nested primary landmarks. This blocks reliable landmark navigation across the whole public site.

**Independent Test**: Open the homepage and every documentation route from the production export, inspect their landmark structure, and activate the first bypass control using keyboard input.

**Acceptance Scenarios**:

1. **Given** any public route, **When** its semantic structure is inspected, **Then** exactly one primary content landmark contains the page-specific content.
2. **Given** any public route at any supported viewport, **When** keyboard focus begins traversing the page, **Then** an early bypass control becomes visible and identifies the primary content as its destination.
3. **Given** the bypass control has focus, **When** it is activated, **Then** focus or navigation reaches the primary content without traversing persistent navigation.

---

### User Story 2 - Read Normal Text In The Light Theme (Priority: P1)

As a visitor using the light theme, I need ordinary navigation, summary, code, and syntax text to remain distinguishable from its background, so shared low-contrast styling does not make the documentation difficult to read.

**Why this priority**: Shared light-theme tokens produce repeated normal-text contrast ratios below the 4.5:1 threshold across navigation and content surfaces.

**Independent Test**: Select the light theme on representative documentation routes and measure every affected foreground token against each shipped background on which it appears.

**Acceptance Scenarios**:

1. **Given** light-theme normal text using the shared muted style, **When** its contrast is measured against every shipped background on which it appears, **Then** every ratio is at least 4.5:1.
2. **Given** light-theme normal syntax text using the affected red style, **When** its contrast is measured against the code background, **Then** the ratio is at least 4.5:1.
3. **Given** the corrected light theme, **When** the same routes are viewed at desktop, tablet, and narrow widths, **Then** the corrected styles remain in effect without changing the information or interaction model.

---

### User Story 3 - Navigate A Truthful Content Outline (Priority: P2)

As a visitor who navigates by headings, I need generated changelog pages to descend through a coherent heading hierarchy, so the page outline represents the content structure rather than skipping two levels immediately after the title.

**Why this priority**: Seven generated changelog pages currently move directly from the page heading to a fourth-level heading, which presents a misleading hierarchy to heading navigation.

**Independent Test**: Build the generated changelog routes and verify the sequence of visible content headings on all affected pages.

**Acceptance Scenarios**:

1. **Given** any generated changelog page, **When** its visible content headings are traversed in document order, **Then** no heading descends by more than one level.
2. **Given** the seven routes identified by the production audit, **When** each page is regenerated, **Then** its first content section descends coherently from the page title.
3. **Given** the corrected hierarchy, **When** release content is compared with its source, **Then** no changelog prose or release history has been rewritten or omitted.

---

### User Story 4 - Identify Architecture Diagrams (Priority: P2)

As a screen-reader user, I need each architecture diagram to expose a concise purpose-specific name, so I can identify the graphic before deciding whether to examine its surrounding explanation.

**Why this priority**: Both architecture diagrams expose a graphics-document role without a programmatic name.

**Independent Test**: Open the production architecture route and inspect the accessible name of each rendered diagram at every audited viewport.

**Acceptance Scenarios**:

1. **Given** the architecture page, **When** either rendered diagram is inspected through accessibility semantics, **Then** it exposes a concise name specific to that diagram's purpose.
2. **Given** the named diagrams at 320, 768, and 1440 pixels, **When** they are rendered, **Then** their names remain available and their existing visual design and surrounding prose remain unchanged.

### Edge Cases

- A layout component wraps content already represented by a primary landmark, which must not create nested primary landmarks.
- A skip destination is not normally focusable, or a route change leaves focus on persistent navigation rather than the destination.
- The bypass control is visually hidden while idle but must become visible without clipping when focused at 320 pixels.
- A shared text token appears on more than one light-theme background, so passing against one background must not conceal a failure against another.
- Generated changelog content begins with different source heading depths across releases, so correction must normalize the rendered relationship rather than assume one fixed source depth.
- Multiple diagrams share one page, so one generic name reused for both would not identify either diagram's purpose.

## Requirements

### Functional Requirements

- **FR-001**: S095 MUST make the homepage and every documentation route expose exactly one primary content landmark.
- **FR-002**: S095 MUST provide every public route with an early keyboard-reachable bypass control whose destination is that route's primary content.
- **FR-003**: The bypass control MUST be visually apparent when focused, identify its destination, and work at 320, 768, and 1440 pixel viewports.
- **FR-004**: Activating the bypass control MUST move navigation or focus to the primary content without requiring traversal of persistent site navigation.
- **FR-005**: S095 MUST preserve the existing navigation, page chrome, content order, and route inventory while correcting landmark structure.
- **FR-006**: S095 MUST ensure that visible headings on every generated changelog route never descend by more than one level at a time.
- **FR-007**: S095 MUST preserve all changelog prose, links, anchors, and release history while correcting the generated heading hierarchy.
- **FR-008**: S095 MUST ensure that the light-theme muted normal-text style currently observed as `rgb(115,115,115)` meets or exceeds a 4.5:1 contrast ratio against both observed shipped backgrounds, `rgb(241,241,241)` and `rgb(245,245,245)`, and against any other shipped background on which it appears.
- **FR-009**: S095 MUST ensure that the affected light-theme red syntax normal-text style currently observed as `rgb(215,58,73)` meets or exceeds a 4.5:1 contrast ratio against its observed `rgb(241,241,241)` code background and any other shipped background on which it appears.
- **FR-010**: S095 MUST preserve the existing theme's visual hierarchy and interaction model rather than redesigning the theme.
- **FR-011**: S095 MUST give each architecture diagram a concise, purpose-specific programmatic name.
- **FR-012**: S095 MUST preserve the architecture diagrams' visual content and surrounding architecture prose.
- **FR-013**: Automated regression coverage MUST verify the primary landmark count, bypass destination, heading hierarchy, affected contrast ratios, and diagram names from production-equivalent rendered output.
- **FR-014**: Verification MUST cover the homepage, all documentation routes for shared landmark behavior, all generated changelog routes for heading hierarchy, every affected light-theme foreground/background pairing, and both architecture diagrams.
- **FR-015**: S095 MUST update durable audit evidence so findings F01, F02, F03, and F06 have explicit correction status without changing the dispositions of F04 and F05.
- **FR-016**: S095 MUST NOT change search ranking, not-found recovery navigation, product behavior, capture behavior, or release history.
- **FR-017**: S095 MUST satisfy the repository's documentation, static-export, encoding, and continuous-integration gates.

### Key Entities

- **Primary Content Region**: The single semantic region containing route-specific content and serving as the bypass destination.
- **Bypass Control**: The early keyboard-reachable control that becomes visible on focus and leads to the primary content region.
- **Rendered Heading Sequence**: The ordered heading levels exposed by one generated page after its page title.
- **Contrast Pair**: One normal-text foreground color and the shipped background color against which it must be measured.
- **Diagram Name**: The concise programmatic label that identifies one rendered architecture graphic's purpose.
- **Regression Observation**: A production-equivalent rendered assertion tied to one of the four corrected audit findings.

## Success Criteria

### Measurable Outcomes

- **SC-001**: One hundred percent of the homepage and documentation routes expose exactly one primary content landmark.
- **SC-002**: One hundred percent of public routes expose a working bypass control before persistent navigation, and it remains visible when focused at 320, 768, and 1440 pixels.
- **SC-003**: Zero generated changelog routes contain a visible heading descent that skips one or more intermediate levels.
- **SC-004**: Every affected light-theme normal-text foreground/background pair measures at least 4.5:1 contrast.
- **SC-005**: Both architecture diagrams expose distinct, purpose-specific accessible names at every audited viewport.
- **SC-006**: Automated regression checks fail when any corrected landmark, bypass, heading, contrast, or diagram-name condition is reintroduced.
- **SC-007**: All repository documentation, static-export, encoding, and continuous-integration gates complete successfully with no change to the public route inventory or release content.

## Assumptions

- WCAG 2.2 Level AA remains the conformance reference established by S094.
- The production export and the S094 route inventory define the public route set for regression coverage.
- Existing visual hierarchy may receive the minimum token adjustments required to meet contrast thresholds, but visual redesign is outside S095.
- A diagram's programmatic name identifies purpose; the surrounding prose continues to provide its detailed explanation.
- Hardware-equivalent keyboard traversal and native screen-reader output remain manual evidence limits unless the implementation environment provides them. Automated semantic checks do not claim those checks were performed.
- Issues #266 and #267 remain outside S095 and are grouped into S096 for site discovery and recovery.
