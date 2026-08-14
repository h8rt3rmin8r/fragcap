# Feature Specification: Dependency-model docs, Mermaid diagrams, and install tutorial

**Feature Branch**: `042-dependency-model-docs`

**Created**: 2026-08-14

**Status**: Draft

**Input**: Resolves #108, the docs and onboarding parts of #107, and #61. Slice
042 of the post-v0.3.0 fix-up stream. It documents the external-dependency model
once and links to it, corrects the stale README install-option table, renders
Mermaid diagrams on the documentation site, and adds an annotated install
walkthrough built from real screenshots. No Rust or behavior change.

## Clarifications

### Session 2026-08-14

Resolved under autopilot from the constitution, the architecture of record, and
the slice scope (no options were materially irreversible or ambiguous enough to
require an operator decision):

- Q: How should the site render Mermaid? -> A: A theme-aware client `<Mermaid>`
  component wired into `site/mdx-components.tsx`, using the `mermaid` package and
  rendered in the browser. Chosen over a build-time `rehype-mermaid` plugin
  because that plugin rasterizes through Playwright at build, which adds a
  browser dependency to the static export and CI; a client component keeps the
  build browser-free and follows both themes.
- Q: Where do the three diagrams live on the site? -> A: The existing
  `site/content/docs/architecture.mdx` page carries all three; Getting Started
  links to it rather than duplicating them.
- Q: What is the canonical home of the dependency model? -> A: The authored
  `docs/glossary/platform-and-distribution.md`, with the npcap, Wireshark, and
  extcap entries stating the required, recommended, and optional tiers; the
  generated site glossary is produced from it and never edited by hand.
- Q: How is the `fragcap doctor` verification step shown? -> A: As a fenced code
  block of real doctor output, not a terminal screenshot: copyable, searchable,
  theme-aware, and not tied to one machine's console rendering.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A newcomer installs the prerequisites and verifies the setup (Priority: P1)

A first-time reader lands on the Getting Started page. They learn, in plain
terms, that npcap is required (the capture driver), Wireshark is recommended (the
analyzer, whose own installer provides npcap), and the Wireshark extcap
integration is optional. They follow a numbered walkthrough whose steps are
illustrated by real screenshots of the Wireshark and Npcap installers, then run
`fragcap doctor` and match its output against a shown reference to confirm the
prerequisites are in place.

**Why this priority**: The most common failure for a new user is an unclear or
wrong prerequisite story, which produces a run that exits zero and captures
nothing. A correct, illustrated, verifiable walkthrough is the single highest
value in this slice and the reason #107, #108, and #61 were filed.

**Independent Test**: Build the site and read the Getting Started page: the three
dependency tiers are stated with what each provides, the five install steps each
carry a labeled screenshot, and the closing step shows real `fragcap doctor`
output as the success check. Delivers a complete install-to-verify path on its
own.

**Acceptance Scenarios**:

1. **Given** the built site, **When** a reader opens Getting Started, **Then**
   they can name which dependency is required, which is recommended, and which is
   optional, and what each one provides, without leaving the page.
2. **Given** the walkthrough, **When** a reader follows it in order, **Then**
   each step that shows an installer screen is accompanied by the matching
   screenshot with descriptive alt text, ending in a `fragcap doctor`
   verification step.
3. **Given** modern Npcap, **When** a reader reaches the loopback step, **Then**
   the docs do not instruct them to enable a "Support loopback traffic capture"
   option, because current Npcap installs loopback automatically.

### User Story 2 - A reader understands the pieces and the data flow from diagrams (Priority: P2)

A reader who wants the mental model sees three diagrams: what the pieces are
(fragcap, npcap, Wireshark, extcap), how captured data flows at runtime (from the
interface through npcap and fragcap's capture and attribution to the pcapng and
JSON Lines outputs, and the extcap path into Wireshark), and how npcap is
acquired (detection only, never bundled by fragcap). The same diagrams render on
the documentation site and in the master specification on GitHub.

**Why this priority**: The dependency relationships are exactly the thing prose
explains poorly and a diagram explains at a glance. It depends on a rendering
mechanism the site does not yet have, so it follows the prose story.

**Independent Test**: Build the site and open the page carrying the diagrams: all
three render as diagrams rather than code blocks. Open the master specification on
GitHub: the same three render there. Passes on its own once the rendering
mechanism and the three sources exist.

**Acceptance Scenarios**:

1. **Given** the Mermaid rendering mechanism, **When** the site builds, **Then**
   a `mermaid` fenced block renders as a diagram in the static export.
2. **Given** the three seed diagrams, **When** a reader views them on the site
   and on GitHub, **Then** both surfaces show the same three diagrams.

### User Story 3 - The dependency model is stated once and cannot drift (Priority: P2)

A contributor updating the dependency story edits one canonical source. The
README and the Getting Started page summarize and link to it rather than
restating it, and the wording matches the `fragcap doctor` severities and the
taxonomy decision already recorded in slice 040.

**Why this priority**: Single-sourcing is what keeps the tool and the docs from
disagreeing over time, which is the durable form of the bug this slice fixes. It
is P2 because the immediate reader value is carried by Story 1.

**Independent Test**: Grep the repository: the required/recommended/optional
definitions live in one authored location under the glossary; the README and
Getting Started reference it rather than duplicating the definitions. The wording
matches the slice 040 taxonomy decision and the doctor severities.

**Acceptance Scenarios**:

1. **Given** the canonical glossary source, **When** the README and Getting
   Started mention the dependency model, **Then** they summarize and link rather
   than restate the tier definitions.
2. **Given** the `fragcap doctor` severities (required, recommended, optional),
   **When** a reader compares them to the docs, **Then** the tier language
   matches.

### Edge Cases

- A reader on a light or dark site theme must see legible diagrams in both;
  Mermaid rendering must not assume a single theme.
- The master specification is read on GitHub, which renders `mermaid` fences
  natively; the diagram sources must stay valid on both GitHub and the site
  renderer rather than depending on one engine's extensions.
- A new glossary term introduced by this slice must gain a glossary entry in the
  same change (constitution P-6); a term used but never defined is a lint
  failure in the documentation check.
- Screenshot alt text and captions are prose and are linted: no em or en dashes
  anywhere, including alt text.
- The generated site glossary tree (`site/content/docs/glossary/*.mdx`) is
  produced by `site/scripts/prebuild.mjs` and gitignored; editing it directly
  would be overwritten and is out of bounds. Only the authored
  `docs/glossary/*.md` sources are edited.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The external-dependency model MUST be defined once, in an authored
  glossary source under `docs/glossary/`, stating three tiers: npcap required
  (the capture driver), Wireshark recommended (the analyzer whose installer also
  provides npcap, by the Nmap Project), and the Wireshark extcap integration
  optional (ships with Wireshark, needs only `fragcap extcap install`).
- **FR-002**: The README and the site Getting Started page MUST summarize the
  dependency model and link to the canonical source rather than restating the
  tier definitions.
- **FR-003**: The tier language MUST match the `fragcap doctor` severities and
  the `changelog.d/dependency-taxonomy.decisions.md` fragment from slice 040.
- **FR-004**: The README install-option table MUST be corrected: it MUST NOT
  present "Support loopback traffic capture" as a user action, because current
  Npcap installs loopback automatically and dropped that checkbox.
- **FR-005**: The docs MUST state that Npcap is by the Nmap Project and that the
  Wireshark installer bundles Npcap.
- **FR-006**: The documentation site MUST render `mermaid` fenced code blocks as
  diagrams in its static export, in both light and dark themes.
- **FR-007**: Three seed diagrams MUST be authored as `mermaid` fences: (a) what
  the pieces are, (b) the runtime data flow including the extcap path into
  Wireshark, and (c) npcap acquisition and bundling, detection only.
- **FR-008**: The same three diagrams MUST also appear as `mermaid` fences in
  `docs/fragcap-specification.md`, which GitHub renders.
- **FR-009**: The Getting Started page MUST carry an annotated install
  walkthrough built from the five provided screenshots, served from a new
  `site/public/screenshots/` directory and referenced as `/screenshots/*.png`.
- **FR-010**: Each install screenshot MUST have descriptive, dash-free alt text
  and a caption tying it to the step it illustrates.
- **FR-011**: The walkthrough MUST close with a `fragcap doctor` verification
  step that shows real doctor output.
- **FR-012**: Any new term the docs introduce MUST gain a glossary entry in this
  same change (constitution P-6).
- **FR-013**: All added or edited text MUST be UTF-8 without BOM, LF line
  endings, and free of em and en dashes, including image alt text.
- **FR-014**: A `changelog.d/` fragment MUST record this documentation change.
- **FR-015**: The change MUST NOT alter any Rust crate, CLI surface, or runtime
  behavior.

### Key Entities *(include if feature involves data)*

- **Dependency tier**: one of required, recommended, optional, each naming the
  component, what it provides, and how it is acquired.
- **Seed diagram**: a Mermaid source rendered on both the site and GitHub;
  three exist (pieces, runtime data flow, acquisition and bundling).
- **Install screenshot**: a labeled image of one installer step, with alt text
  and a caption, served under `/screenshots/`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A reader can determine all three dependency tiers and what each
  provides from one canonical location, reached by a link from both the README
  and Getting Started.
- **SC-002**: The install walkthrough covers all five captured installer steps,
  each with a labeled screenshot, and ends with a `fragcap doctor` verification
  step.
- **SC-003**: The site static export builds successfully and renders all three
  diagrams as diagrams and all five screenshots as images.
- **SC-004**: The three diagrams also render on GitHub from
  `docs/fragcap-specification.md`.
- **SC-005**: The README no longer lists "Support loopback traffic capture" as a
  user action, and states the Nmap Project and Wireshark-bundles-Npcap framing.
- **SC-006**: `cargo xtask ci` is green, including the documentation linter (no
  em or en dashes anywhere including alt text, a glossary entry for every term,
  UTF-8 and LF).
- **SC-007**: The dependency-tier wording in the docs matches the `fragcap
  doctor` severities with no contradiction.

## Assumptions

- The five install screenshots are provided and final (the minified set:
  `01_wireshark_choose-components_extcap`, `02_wireshark_choose-install-location`,
  `03_wireshark_packet-capture_install-npcap`, `04_wireshark_npcap-setup`,
  `05_wireshark_npcap-setup_options`). No separate `fragcap doctor` screenshot was
  provided; the doctor verification step is shown as real command output in a code
  block, which is more maintainable, accessible, and theme-aware than a terminal
  capture. This is recorded as a plan decision.
- The Mermaid rendering mechanism for the fumadocs site is selected in the plan
  from the viable options (a rehype-mermaid build plugin versus a client
  component wired into `site/mdx-components.tsx`); either satisfies FR-006.
- The canonical dependency-model source is a glossary page under
  `docs/glossary/`; the generated `site/content/docs/glossary/*.mdx` are never
  edited by hand.
- The master specification already renders `mermaid` on GitHub, so FR-008 needs
  only the three fences added, not a new mechanism.
- The installer MSI extcap work and any re-decision of the dependency taxonomy
  are out of scope (separate slice; taxonomy decided in slice 040).
