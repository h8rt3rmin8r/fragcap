# Feature Specification: Site Discovery And Recovery

**Feature Branch**: `codex/096-site-discovery-recovery`

**Created**: 2026-08-28

**Status**: Draft

**Input**: User description: "Kick off S096", grouping GitHub issues #266 and #267 into one production-site discovery and recovery slice.

## User Scenarios & Testing

### User Story 1 - Find Current Command Guidance First (Priority: P1)

As a visitor searching for a command name from an older fragcap release, I need current command and migration guidance to appear before historical release notes, so I do not mistake a retired interface for the command I should run today.

**Why this priority**: The production audit showed that exact searches for `fragcap run` and `fragcap tap` lead with v0.5.0 history even though the current command surface has replaced both names. Following that first result gives a user obsolete instructions.

**Independent Test**: Search the hydrated production export for both retired command names, inspect the ordered results, and confirm that current guidance appears first while matching changelog history remains available later in the result set.

**Acceptance Scenarios**:

1. **Given** the production search dialog, **When** a visitor searches for `fragcap run`, **Then** the first result leads to current guidance that identifies the supported replacement.
2. **Given** the production search dialog, **When** a visitor searches for `fragcap tap`, **Then** the first result leads to current guidance that identifies the supported replacement.
3. **Given** either retired-command query, **When** all results are inspected, **Then** matching historical changelog material remains searchable but does not precede current guidance.
4. **Given** the existing current-guidance query set, **When** those searches are repeated after the correction, **Then** each still returns current relevant documentation before release history.

---

### User Story 2 - Recover From A Missing Page (Priority: P2)

As a visitor who follows a stale or mistyped link, I need the not-found page to identify the site and offer clear routes back to useful content, so a missing page is a recoverable stop rather than a dead end.

**Why this priority**: The exported not-found response correctly uses HTTP 404 but currently contains no links and no primary landmark. The visitor has no in-page path back to fragcap or its documentation.

**Independent Test**: Request an absent path from the production export at narrow and desktop widths, confirm the response remains HTTP 404, and follow each recovery link to its intended public destination.

**Acceptance Scenarios**:

1. **Given** any absent path, **When** the static host serves the not-found page, **Then** the response status is 404 and the page identifies fragcap, states that the page was not found, and exposes one primary content landmark.
2. **Given** the not-found page, **When** a visitor chooses the homepage recovery link, **Then** the link leads to `/`.
3. **Given** the not-found page, **When** a visitor chooses the documentation recovery link, **Then** the link leads to the current getting-started route.
4. **Given** the not-found page at 320 or 1440 pixels, **When** its content and controls are inspected, **Then** all recovery content remains visible, keyboard reachable, and free of root horizontal overflow.

### Edge Cases

- A retired command appears in historical prose many times, so adding one current mention must produce a stable current-first result rather than depend on incidental source order.
- A retired-command query uses different letter casing or surrounding whitespace, so the search correction must follow the search engine's normal query handling.
- A historical result remains relevant context, so the correction must not delete changelog content or remove changelog pages from the search index.
- A current query overlaps changelog wording, so the correction must not globally suppress or demote all historical results ahead of more relevant current content.
- A missing path is nested several segments deep or resembles a generated route, so it must still return the exported not-found page with HTTP 404 rather than a successful fallback response.
- The shared skip link targets `main-content`, so the custom not-found page must provide that unique destination without adding a second primary landmark.
- Client hydration or prefetching on the not-found page must not change the response status or produce browser and console errors.

## Requirements

### Functional Requirements

- **FR-001**: S096 MUST make the first production search result for `fragcap run` lead to current guidance that names the supported replacement.
- **FR-002**: S096 MUST make the first production search result for `fragcap tap` lead to current guidance that names the supported replacement.
- **FR-003**: Current guidance for each retired command MUST state the present command or workflow plainly enough that the visitor can continue without consulting release history.
- **FR-004**: Matching historical changelog results MUST remain searchable for both retired-command queries but MUST appear after current guidance.
- **FR-005**: Search behavior for `packet attribution`, `capture scope`, `Deep Capture`, and `proxy-owned TLS key` MUST continue to lead with current relevant documentation.
- **FR-006**: Search verification MUST exercise the production-equivalent client search experience and assert a nonempty result population before checking order.
- **FR-007**: S096 MUST provide a branded exported not-found page that identifies fragcap and explains that the requested page was not found.
- **FR-008**: The not-found page MUST expose exactly one primary content landmark with the unique destination `main-content`.
- **FR-009**: The not-found page MUST provide keyboard-reachable recovery links to `/` and `/docs/getting-started`, with labels that identify their destinations.
- **FR-010**: An absent path MUST retain HTTP status 404 while serving the custom recovery page.
- **FR-011**: The not-found page MUST remain fully visible without root horizontal overflow at 320 and 1440 pixel viewports.
- **FR-012**: Production-equivalent regression coverage MUST verify the not-found status, identity, primary landmark, recovery destinations, narrow-width reachability, hydration, and absence of browser or console errors.
- **FR-013**: S096 MUST preserve all changelog content, public routes, current documentation navigation, Capture behavior, and Deep Capture behavior.
- **FR-014**: S096 MUST update the S094 audit record with explicit correction evidence for findings F04 and F05 without rewriting the original observations or S095 evidence.
- **FR-015**: S096 MUST satisfy the repository's documentation, static-export, encoding, and continuous-integration gates.

### Key Entities

- **Search Query**: A visitor-entered phrase evaluated by the production client search experience.
- **Search Result**: An ordered link and excerpt from current documentation or preserved release history.
- **Current Guidance**: Documentation for the command surface and workflow shipped in the current release.
- **Historical Result**: Searchable changelog material describing an earlier release without serving as current instruction.
- **Not-Found Response**: The static host's HTTP 404 response containing the exported recovery page.
- **Recovery Link**: A labelled destination from the not-found page to the homepage or current getting-started guide.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Both retired-command queries return current replacement guidance as their first search result in the hydrated production export.
- **SC-002**: Both retired-command queries retain at least one matching historical changelog result after the current result.
- **SC-003**: All four established current-guidance queries continue to lead with current relevant documentation.
- **SC-004**: Every tested absent path returns HTTP 404 and a page with one `main-content` primary landmark plus recovery links to exactly the homepage and current getting-started destinations.
- **SC-005**: The not-found recovery page has zero root horizontal overflow at both 320 and 1440 pixels, and every recovery link remains visible and keyboard reachable.
- **SC-006**: Automated regressions fail if retired guidance returns to first place, historical context disappears, the not-found response becomes successful, either recovery link disappears, or the not-found primary landmark becomes invalid.
- **SC-007**: All repository documentation, static-export, encoding, and continuous-integration gates complete successfully with the public route inventory unchanged at 54.

## Assumptions

- The S094 query set and production audit remain the baseline evidence for search relevance.
- Current command guidance may add concise migration wording for retired names, but release history remains unchanged and searchable.
- The current getting-started route is the most useful documentation recovery destination because it begins the supported first-run journey.
- The existing brand assets, theme tokens, shared skip link, and typography are reused; S096 does not redesign the site.
- The exported `404.html` and `_not-found.html` files remain implementation artifacts outside the 54-route public inventory.
- Hardware-equivalent assistive-technology testing remains outside the automated evidence boundary recorded by S094 and S095.
