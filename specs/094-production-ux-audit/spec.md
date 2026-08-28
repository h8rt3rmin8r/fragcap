# Feature Specification: Production UX And Accessibility Audit

**Feature Branch**: `codex/094-production-ux-audit`

**Created**: 2026-08-28

**Status**: Draft

**Input**: User description: "Kick off S094", implementing GitHub issue #249.

## User Scenarios & Testing

### User Story 1 - Trust The Published Route Set (Priority: P1)

As a maintainer, I need evidence that every route in the production documentation export can be reached and used, so the published site is assessed as a complete application rather than as a collection of source files.

**Why this priority**: A route omitted from the audit can hide a broken navigation path, stale public claim, or unusable page while the audit still appears successful.

**Independent Test**: Build the production export from the locked dependency graph, derive its route inventory, open every route from the built artifact, and reconcile every attempted route with a recorded outcome.

**Acceptance Scenarios**:

1. **Given** a clean locked dependency installation, **When** the production export is built, **Then** the build completes and the audit records the exact build command and result.
2. **Given** the built route inventory, **When** every route is exercised, **Then** each route has one recorded result and no route is silently omitted.
3. **Given** the built application, **When** an unknown route is opened, **Then** the audit records the not-found behavior and whether recovery navigation remains available.

---

### User Story 2 - Use The Site Across Access Modes (Priority: P1)

As a visitor using a keyboard, assistive semantics, a narrow screen, zoom, or a preferred theme, I need the documentation to preserve access to its navigation and content.

**Why this priority**: The audit exists to find barriers that source review and a successful build cannot establish.

**Independent Test**: Exercise the homepage and every documentation route at 320 px, 768 px, and desktop widths, complete the representative keyboard journey, inspect semantic structure and accessible names, and record any clipping, focus, contrast, or interaction defect with reproducible evidence.

**Acceptance Scenarios**:

1. **Given** any audited viewport, **When** long commands, tables, matrices, diagrams, and images are inspected, **Then** essential information remains reachable without silent clipping.
2. **Given** keyboard-only input, **When** the visitor traverses the skip link, top navigation, sidebar, search, table of contents, and footer, **Then** focus is visible, ordered, operable, and never trapped.
3. **Given** semantic inspection, **When** headings, landmarks, links, controls, images, and diagrams are examined, **Then** their hierarchy, purpose, and accessible names are recorded as passing or as a reproducible finding.
4. **Given** light and dark themes and 200 percent zoom, **When** representative routes are inspected, **Then** content and controls remain perceivable and operable.

---

### User Story 3 - Act On Audit Findings (Priority: P2)

As a maintainer reviewing the audit, I need each finding to have enough evidence and a narrow ownership boundary to decide and schedule a correction without repeating the audit.

**Why this priority**: A list of impressions cannot be triaged, reproduced, or used as release evidence.

**Independent Test**: Review the audit record and verify that every material defect names its route, viewport, reproduction, evidence, severity, affected requirement, and a linked non-duplicate issue, while every unavailable check is explicitly disclosed.

**Acceptance Scenarios**:

1. **Given** an observed defect, **When** overlap is searched, **Then** the audit links an existing issue or files one narrowly scoped follow-up issue.
2. **Given** a check that cannot run in the available environment, **When** the audit is finalized, **Then** the omission and its effect on confidence are recorded rather than represented as a pass.
3. **Given** the complete audit record, **When** a reviewer reconciles findings and follow-ups, **Then** every material finding has one disposition and no unrelated correction is included in S094.

### Edge Cases

- A route exists in generated output but is absent from visible navigation, or appears in navigation but is absent from generated output.
- A control is visible only at one breakpoint or theme, including mobile navigation and search controls.
- Content overflows intentionally inside a labeled scroll region versus being clipped with no access path.
- A Mermaid diagram has a text alternative but its visual labels become unreadable or unreachable at narrow widths.
- Search returns no result, stale commands, or a relevant current term below retired terminology.
- Browser automation cannot expose a native screen-reader announcement or operating-system contrast mode; the audit must distinguish semantic evidence from an unperformed assistive-technology check.
- An external link cannot be reached because of transient network state; the audit records that separately from an internal link defect.

## Requirements

### Functional Requirements

- **FR-001**: S094 MUST install site dependencies from the committed lockfile without modifying the resolved dependency graph.
- **FR-002**: S094 MUST build the production static export and record the command, environment, and outcome.
- **FR-003**: S094 MUST derive and record the complete public route inventory from the built artifact and site route sources.
- **FR-004**: S094 MUST open the homepage and every documentation route from the built artifact, including an unknown route for not-found behavior.
- **FR-005**: S094 MUST exercise keyboard-only access to the skip link, top navigation, mobile navigation, sidebar, search, table of contents, content links, theme control, and footer wherever each surface is present.
- **FR-006**: S094 MUST inspect headings, landmarks, focus order and visibility, control names and states, link purpose, image alternatives, diagram alternatives, and automated accessibility results.
- **FR-007**: S094 MUST inspect every route at 320 px, 768 px, and a desktop width, with representative routes additionally checked at 200 percent zoom.
- **FR-008**: S094 MUST inspect light and dark themes and assess text and essential non-text contrast against WCAG 2.2 AA thresholds: 4.5:1 for normal text, 3:1 for large text, and 3:1 for essential user-interface components and graphical objects.
- **FR-009**: S094 MUST inspect long commands, code blocks, tables, matrices, Mermaid diagrams, images, and footer placement for reachability and silent clipping.
- **FR-010**: S094 MUST query search for current Capture and Deep Capture vocabulary and record whether retired commands are foregrounded.
- **FR-011**: S094 MUST check internal links and anchors across the built site and use the repository's existing external-link surface for external checks.
- **FR-012**: Every audit observation MUST record route, viewport or access mode, reproduction, evidence, severity, and disposition.
- **FR-013**: Severity MUST use four levels: critical for a blocker affecting a primary journey with no workaround, high for a substantial accessibility or navigation barrier, medium for a localized barrier with a workaround, and low for a minor usability or conformance defect.
- **FR-014**: Before filing a follow-up, S094 MUST search open and closed issues for overlap and MUST link an existing issue when it already owns the defect.
- **FR-015**: Each newly filed follow-up issue MUST have one narrow defect boundary, reproduction evidence, acceptance criteria, appropriate labels, and the `Post-v0.7.0 documentation` milestone.
- **FR-016**: The audit MUST distinguish passed, failed, and not-run checks and MUST explain every not-run result and its confidence impact.
- **FR-017**: S094 MUST publish a durable audit report in the repository and MAY include compact non-sensitive screenshots when they materially improve reproducibility.
- **FR-018**: S094 MUST NOT bundle unrelated visual or content corrections, change product behavior, or claim checks that were not performed.
- **FR-019**: S094 MUST run the repository documentation, encoding, link, and full continuous-integration gates after the report is complete.
- **FR-020**: S094 MUST leave the documentation epic open until every audit finding and child issue required by its acceptance criteria has an explicit disposition.

### Key Entities

- **Audit Check**: One required examination with a subject, route set, access mode, method, result, evidence, and limitation.
- **Route Observation**: The outcome for one route at one viewport or interaction mode.
- **Finding**: A reproducible defect with severity, affected surface, evidence, requirement impact, and disposition.
- **Follow-up Issue**: The existing or newly filed narrow GitHub issue that owns correction of a material finding.
- **Audit Report**: The durable reconciliation of scope, environment, route coverage, checks, findings, limitations, and follow-ups.

## Success Criteria

### Measurable Outcomes

- **SC-001**: One hundred percent of routes in the reconciled production inventory have a recorded desktop result, and every documentation route has recorded 320 px and 768 px results.
- **SC-002**: The representative keyboard journey reaches every applicable named surface with no unrecorded focus stop, trap, or invisible focus state.
- **SC-003**: Every documentation route has a recorded automated accessibility result and semantic heading and landmark inspection.
- **SC-004**: Every page containing long commands, tables, matrices, Mermaid diagrams, or images has a recorded narrow-width reachability result.
- **SC-005**: Search outcomes are recorded for at least two current Capture terms, two current Deep Capture terms, and two retired command terms.
- **SC-006**: One hundred percent of material findings have a severity, reproduction, evidence, and exactly one linked disposition.
- **SC-007**: Every required check is marked passed, failed, or not run, with zero silent omissions.
- **SC-008**: The production export, documentation checks, encoding checks, link checks, and full repository gate complete successfully after the audit record is written.

## Assumptions

- S094 is an audit and evidence slice. Corrective UI work belongs in narrow follow-up issues unless a defect prevents the audit itself from running.
- Browser automation, DOM inspection, computed styles, screenshots, and automated accessibility rules provide valid evidence for the covered checks, but do not substitute for an unperformed native screen-reader session.
- The committed site route sources and production export together define the route inventory; discrepancies between them are findings.
- The desktop audit width is 1440 px, while the required narrow widths are exactly 320 px and 768 px.
- WCAG 2.2 Level AA is the conformance reference for accessibility classification.
- The production build is served locally so routing, search, scripts, and generated assets are exercised from the built artifact rather than a development server.
