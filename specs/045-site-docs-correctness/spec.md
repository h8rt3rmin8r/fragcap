# Feature Specification: site/docs correctness (profile JSON, placeholders, Mermaid layout, docs footer)

**Feature Branch**: `045-site-docs-correctness`

**Created**: 2026-08-14

**Status**: Draft

**Input**: GitHub issues #115, #116, #120, #112. Four site-only documentation
and layout defects on the fragcap documentation website, batched into one slice
because they touch the same page family and are verified by the same site build.
No Rust, schema, or CLI change; every edit is under `site/`.

## Clarifications

### Session 2026-08-14

Resolved from the issue reports, the approved plan, and the current site source
(no operator escalation needed):

- Q: Which JSON shape do the rewritten profile examples follow? -> A: The
  current published schema `docs/schema/target-schema.v1.json` and the committed
  fixture `crates/fragcap-profile/tests/fixtures/schema/profile-valid.json`,
  including the top-level `kind` field the current doc omits. The doc's key
  tables are reconciled against that schema while converting.
- Q: Where does the single retained concrete game slug live? -> A: In the
  `--profile <REF>` flag reference on the CLI page, as the one example of what a
  real value looks like. Every other occurrence becomes a typed placeholder.
- Q: Vertical Mermaid direction keyword? -> A: `TD`, matching the third diagram
  already present on the architecture page, rather than the issue's `TB` spelling
  (both render top-down; `TD` keeps the page consistent).
- Q: How is the docs footer made to sit in flow? -> A: The footer is rendered
  inside the docs content column (not as a body-level sibling of the
  full-viewport docs layout) and kept on the home page as it renders correctly
  today, with exactly one footer per page.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A profile author reads accurate format docs (Priority: P1)

An author opening the profile-schema reference or the writing-a-profile guide
sees the profile format documented as JSON, with examples that parse against the
shipped schema, and command examples that name a JSON profile path. Today both
pages describe and demonstrate TOML, a format the tool no longer accepts since
the profile format migrated to JSON; the pages are wrong, not merely stale.

**Why this priority**: A reader who copies a documented TOML profile produces a
file the tool rejects. Wrong instructions are the most damaging defect in the
set. This is the P1 (`bug` labeled) issue #115.

**Independent Test**: Open the two pages in a built site; confirm no page
describes the format as TOML, every profile example is valid JSON matching the
current schema, and no command example references a `.toml` path.

**Acceptance Scenarios**:

1. **Given** the profile-schema reference page, **When** it is read, **Then**
   its description and body name the format as JSON and its example is a JSON
   object carrying the schema's real keys (including the top-level `kind`).
2. **Given** the writing-a-profile guide, **When** it is read, **Then** every
   profile example is JSON and both command examples name a `.json` profile.
3. **Given** any converted example, **When** validated against
   `docs/schema/target-schema.v1.json`, **Then** it conforms.

### User Story 2 - No reader mistakes an example slug for a shipped profile (Priority: P2)

A reader encountering profile examples sees typed placeholders (a game id, an
executable name, a profile path) everywhere except one clearly-labeled concrete
example, and no apologetic sentence claiming an example is "illustrative, not a
shipped profile." Today the verbatim slug `eso` appears in roughly ten places
plus a hedging sentence, which reads as if `eso` were a real shipped profile.

**Why this priority**: It is a clarity and honesty fix (issue #116) that does not
block a reader the way a wrong format does, but it removes a standing source of
confusion and overlaps the same two files as User Story 1.

**Independent Test**: Grep the built docs for the literal `eso`; confirm exactly
one intentional occurrence remains (the concrete `--profile` example) and the
hedging sentence is gone.

**Acceptance Scenarios**:

1. **Given** the CLI reference `--profile <REF>` flag, **When** read, **Then**
   exactly one generic slug remains there as the concrete example value.
2. **Given** every other former `eso` occurrence, **When** read, **Then** it is
   an angle-bracket placeholder appropriate to its position (`<game-id>`,
   `<client>.exe`, `<profile>.json`).
3. **Given** the getting-started page, **When** read, **Then** the "illustrative,
   not a shipped profile" sentence is absent.

### User Story 3 - The architecture diagrams are readable at normal widths (Priority: P2)

A reader on the architecture page sees each dependency and pipeline diagram laid
out so it fits the documentation content column and is legible without zooming or
horizontal scrolling, in both light and dark themes and at desktop and mobile
widths. Today two diagrams are declared left-to-right and render far wider than
the column.

**Why this priority**: A diagram that cannot be read at normal widths fails its
only purpose (issue #120). It is isolated to one page.

**Independent Test**: Open the architecture page at desktop and mobile widths in
both themes; confirm each diagram fits the content column and its labels are
legible.

**Acceptance Scenarios**:

1. **Given** the two previously-horizontal diagrams, **When** the page renders,
   **Then** each is laid out vertically and fits the content column width.
2. **Given** the already-vertical third diagram, **When** the page renders,
   **Then** it is unchanged.
3. **Given** either converted diagram at mobile width in dark theme, **When**
   viewed, **Then** node and edge labels are legible without horizontal scroll.

### User Story 4 - The docs footer sits directly under the content (Priority: P2)

A reader on any docs page sees the site footer at the bottom of the content, with
no empty full-viewport gap before it and without the reader having to scroll past
a blank region. The home page footer is unchanged. Today the footer on docs pages
is detached and parked a full viewport below the content.

**Why this priority**: It is a visible layout defect on every docs page (issue
#112), though it does not block reading.

**Independent Test**: Open any docs page in a built site; confirm the footer
appears immediately after the content with no empty viewport-height gap, the
sidebar sticky behavior is unchanged, the home footer is unchanged, and exactly
one footer renders.

**Acceptance Scenarios**:

1. **Given** any docs page, **When** scrolled to the end of the content, **Then**
   the footer is directly below the content with no full-viewport gap.
2. **Given** any docs page, **When** rendered, **Then** exactly one footer is
   present (no double render).
3. **Given** the home page, **When** rendered, **Then** its footer is visually
   unchanged from today.

### Edge Cases

- A converted vertical Mermaid graph reads awkwardly tall: acceptable fallback is
  to split it into smaller focused sub-diagrams rather than force one tall graph.
- A JSON example must stay in sync with the schema: if the doc's key table and
  the schema disagree, the schema wins and the doc is corrected to match.
- The single retained `eso` slug must remain unmistakably an example, not read as
  a shipped, resolvable profile reference.
- The footer's visual treatment (typeface, casing, low emphasis, endorsement and
  links) must be preserved; this is a positioning fix, not a restyle.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The profile-schema reference and the writing-a-profile guide MUST
  describe the profile format as JSON, with no page describing it as TOML.
- **FR-002**: Every profile example on those pages MUST be valid JSON conforming
  to `docs/schema/target-schema.v1.json`, carrying the schema's real keys
  (including the top-level `kind`), with the doc key tables reconciled to the
  schema.
- **FR-003**: Every command example MUST reference a `.json` profile path, not
  `.toml`.
- **FR-004**: Exactly one generic game slug MUST remain in the docs, as the
  concrete example value under the `--profile <REF>` flag reference; every other
  former `eso` occurrence MUST be a typed placeholder appropriate to its position.
- **FR-005**: The hedging sentence stating the example is illustrative and not a
  shipped profile MUST be removed.
- **FR-006**: The two horizontally-laid-out architecture diagrams MUST be laid
  out vertically so they fit the content column; the already-vertical diagram
  MUST be left unchanged.
- **FR-007**: The docs-page footer MUST render inside the docs content flow so it
  sits directly below the content with no full-viewport gap; exactly one footer
  MUST render per page; the home-page footer MUST remain correct and visually
  unchanged; the sidebar and table-of-contents sticky behavior MUST be unchanged.
- **FR-008**: All edits MUST be confined to the `site/` directory. No Rust,
  schema, CLI, or fixture file is changed by this slice.
- **FR-009**: All added or edited text MUST be UTF-8, LF, and free of em and en
  dashes; a changelog fragment MUST be added.

### Key Entities

- **Profile example**: a documented JSON object demonstrating the profile schema
  (game identity, capture defaults, stages), replacing the former TOML examples.
- **Typed placeholder**: an angle-bracket token standing for a real value a
  reader supplies (`<game-id>`, `<client>.exe`, `<profile>.json`).
- **Architecture diagram**: a Mermaid flowchart on the architecture page; its
  layout direction is the property this slice changes.
- **Footer**: the single site-wide footer component; its placement relative to
  the docs layout is the property this slice changes.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Zero documentation pages describe the profile format as TOML and
  zero command examples reference a `.toml` path (grep-verifiable).
- **SC-002**: Every profile example on the two pages validates against the
  published schema.
- **SC-003**: Exactly one `eso` slug remains in the docs and the hedging sentence
  is absent (grep-verifiable).
- **SC-004**: On the architecture page each diagram fits the content column and
  is legible in light and dark themes at desktop and mobile widths.
- **SC-005**: On every docs page the footer sits directly below the content with
  no full-viewport gap, exactly one footer renders, and the home footer is
  unchanged.
- **SC-006**: The site production build succeeds with all changes applied.

## Assumptions

- The current published schema and the committed profile fixture are the
  authoritative reference for the JSON examples; where the existing doc key table
  disagrees, the schema is correct.
- Retaining a single concrete slug (rather than zero) is preferred so the
  `--profile` reference shows what a real value looks like.
- The documentation website toolchain (its layout components and Mermaid
  rendering) is unchanged; this slice edits content and layout placement only.
- The footer's existing visual design is correct and is preserved; only its
  position on docs pages changes.
