# Feature Specification: Documentation site

**Feature Branch**: `023-docsite`

**Created**: 2026-08-11

**Status**: Draft

**Input**: Roadmap slice S18 sub-slice C (specification sections 22, 23, and
4.6). Build the fragcap documentation website: a Fumadocs on Next.js application,
statically exported, serving a minimal landing page and the documentation set at
fragcap.com on GitHub Pages, with Cloudflare providing DNS only. Split the
interim monolithic glossary into one page per category plus a generated
alphabetical index, and ship the documentation linter that turns constitution
P-6 from hand-kept into mechanically enforced.

## Clarifications

### Session 2026-08-11

Resolved under autopilot from the constitution, the architecture of record
(specification sections 4.2 through 4.6, 22, and 23), the vendored brand kit, and
the house scripting standards, with three items escalated to and answered by the
operator.

- Q: The domain is registered on Cloudflare; should hosting move to Cloudflare
  Pages, or stay on GitHub Pages as specification sections 22.1 and 23.2 state?
  -> A (operator, 2026-08-11): stay on GitHub Pages. The site is a static export
  built and deployed by continuous integration to GitHub Pages; Cloudflare serves
  DNS only (apex address records plus a `www` alias) with HTTPS enforced. No
  vendor hosting account, no Cloudflare deploy token in continuous integration,
  and no `wrangler` dependency (it manages Workers and Pages, not DNS). This
  keeps specification sections 22.1 and 23.2 as written.
- Q: How is the Cloudflare-side configuration delivered, given continuous
  integration holds no Cloudflare credential? -> A (operator, 2026-08-11):
  documented steps only. The exact DNS records and the GitHub Pages settings are
  relayed to the operator as a runbook and applied by hand once after merge.
  Enabling Pages and editing DNS are operator actions, out of scope for the code
  slice, which ends at the pre-push halt.
- Q: The glossary source has grown an eighth category, "Command Line and
  Diagnostics" (eight authored entries), but specification section 4.4 names
  seven and section 22.4 binds the split to "one page per category from section
  4.4." How is the count reconciled? -> A (operator, 2026-08-11): amend section
  4.4 to add "Command Line and Diagnostics" as a legitimate eighth category, and
  reconcile section 22.4's count to eight. The split then produces eight category
  pages plus the generated index. The amendment is recorded as a dated decision.
- Q: Where does the generated alphabetical index live, and is it hand-editable?
  -> A: the per-category pages are the authored single source; the alphabetical
  index at the glossary root is generated from them by the linter's fix mode and
  by the site build, never hand-edited. A committed index that diverges from its
  category sources is a linter failure.
- Q: Do the guide, reference, architecture, and contributing pages reproduce the
  master specification, or link to it? -> A: the site is authored documentation
  written for readers, not a mirror of `docs/fragcap-specification.md`. It states
  what a reader needs and links the specification for depth. The glossary is the
  one body the site and the linter share as a single source.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A newcomer reaches a first successful capture (Priority: P1)

Someone who has just found fragcap opens fragcap.com. The landing page states in
one sentence what the tool is, shows one worked invocation with its output, names
the capture-driver prerequisite plainly, and links to getting started, the
repository, and the glossary. Following getting started, they install the capture
driver with the required options, verify the install with the diagnostics
command, run their first capture, and open the result, in that order. Before the
first instruction they are told that payloads are encrypted, that the launcher to
client handoff is not in the capture, and that a loopback conversation is usually
a process talking to itself, so the capture they get matches the capture they
expect.

**Why this priority**: The site's primary job is to take a newcomer from landing
to a correct first capture without surprises. A site that omits the prerequisite,
or orders the first run wrong, produces a run that exits zero and contains nothing
the reader wanted, which is the exact failure the getting-started ordering exists
to prevent.

**Independent Test**: Build the static export and load the landing page and the
getting-started page; confirm the landing page carries one sentence, one worked
invocation with output, the named prerequisite, and the three links and nothing
else (no testimonials, feature grid, or call to action); confirm getting started
fronts the prerequisite ahead of the first usage instruction and sets the three
expectations before the first capture step.

**Acceptance Scenarios**:

1. **Given** the built site, **When** the landing page renders, **Then** it
   states what fragcap is in one sentence, shows one worked invocation with its
   output, names the prerequisite, and links to getting started, the repository,
   and the glossary, and carries no testimonials, feature grid, or call to
   action.
2. **Given** the getting-started page, **When** it renders, **Then** the
   capture-driver prerequisite appears ahead of the first usage instruction and
   the first-run steps are ordered install, verify, capture, open.
3. **Given** the getting-started page, **When** it renders, **Then** the three
   expectations (encrypted payloads, the uncaptured launcher handoff, loopback as
   self-talk) are stated before the first capture instruction.
4. **Given** any documentation page carrying a usage instruction, **When** it
   renders, **Then** the capture-driver prerequisite is stated ahead of it.

---

### User Story 2 - The glossary is one page per category plus a generated index (Priority: P1)

A reader who meets an unfamiliar term follows its link into the glossary. The
glossary presents one page per category and an alphabetical index at its root
that lists every term across all categories. Search finds a term as an
independent result at the heading level, and a compound identifier is findable by
its parts because the search tokenizer splits on whitespace, underscores, and
hyphens. The "why it matters here" note on an entry renders as a distinct visual
element, not as ordinary body text. Every cross-link between entries resolves,
within a page and across pages.

**Why this priority**: The glossary is the single body the site and the linter
share, and the constitution's Undefined Term Rule points every unfamiliar term in
the documentation at it. A split that drops a cross-link, or an index that
diverges from its sources, breaks the one navigation aid the whole site relies
on.

**Independent Test**: Split the interim `docs/glossary.md` into the eight
category source pages, generate the index, and build the site; confirm the index
lists every term alphabetically and matches the category sources; confirm a
sample of within-page and cross-page cross-links resolve; confirm a compound
identifier is found by a part through the tokenizer; confirm the "why it matters
here" note renders as its own element.

**Acceptance Scenarios**:

1. **Given** the interim glossary, **When** it is split, **Then** there is one
   source page per category from specification section 4.4 (eight after the
   amendment) and a generated alphabetical index at the glossary root.
2. **Given** the generated index, **When** the linter's fix mode regenerates it,
   **Then** the committed index is byte-for-byte identical to the regenerated one
   (no drift).
3. **Given** a term with a cross-link to another category's entry, **When** the
   site is built, **Then** the link resolves to that entry's anchor.
4. **Given** a compound identifier such as a snake_case or hyphenated term,
   **When** a reader searches for one of its parts, **Then** the term appears as a
   result, because the tokenizer splits on whitespace, underscores, and hyphens.
5. **Given** an entry that influenced a design decision, **When** it renders,
   **Then** its "why it matters here" note is a distinct visual element.

---

### User Story 3 - The documentation linter enforces the glossary discipline (Priority: P1)

A contributor adds a term to the documentation or changes a glossary entry.
`scripts/lint-docs.sh` in check mode confirms that every entry carries a prose
blurb or detail (not merely metadata markers) and that a references section or
matters callout, where present, is not empty, that every internal cross-link
resolves, and that every glossary reference in the canonical documents names a
defined term. In fix mode it regenerates the alphabetical index in place. In link
mode, on a weekly schedule, it verifies that every external reference URL
responds. Continuous integration runs the check mode on every push, so P-6 is
enforced mechanically rather than by hand.

**Why this priority**: Specification section 4.6 and the constitution's P-6 both
require this linter, and until it exists P-6 is satisfiable but unenforced: a term
can enter the documentation with no entry, or a cross-link can rot, with nothing
catching it. The linter is what turns the rule from aspiration into a gate.

**Independent Test**: Run `bash scripts/lint-docs.sh check` against the split
glossary and confirm it passes; introduce an entry with no prose blurb or detail,
an empty references section, a dangling cross-link, and a glossary reference in a
canonical document naming an undefined term, and confirm check mode fails naming
each; run fix mode and confirm it regenerates the index with no other change;
confirm `cargo xtask docs check` and the `ci` aggregate run the check mode.

**Acceptance Scenarios**:

1. **Given** the split glossary, **When** `bash scripts/lint-docs.sh check` runs,
   **Then** it validates entry completeness, cross-link resolution, and the
   glossary-reference check, and exits 0.
2. **Given** an entry with no prose blurb or detail, an empty references section,
   a dangling cross-link, or a glossary reference to an undefined term, **When**
   check mode runs, **Then** it exits non-zero and names each failure.
3. **Given** a stale committed index, **When** fix mode runs, **Then** it
   regenerates the index in place and changes nothing else.
4. **Given** the linter, **When** the compliance checker runs against it, **Then**
   it reports the script compliant with the ShruggieTech Bash standard.
5. **Given** the `ci` aggregate and `ci.yml`, **When** they run, **Then** the
   documentation check is among the gates, so P-6 is enforced on every push.

---

### User Story 4 - One entry point builds and serves the site (Priority: P2)

A contributor runs `cargo xtask docs` and the documentation site starts locally
with hot reload. `cargo xtask docs build` produces the static export that
continuous integration deploys, and `cargo xtask docs check` runs the
documentation linter. The same entry point serves local development and the
continuous-integration build, so what a contributor sees locally is what ships.

**Why this priority**: Specification section 22.6 makes `cargo xtask docs` the
one entry point for the site, local and in continuous integration. A build that
works in continuous integration but not from the documented command, or the
reverse, splits the contributor's mental model from what deploys.

**Independent Test**: Run `cargo xtask docs build` and confirm it produces a
static export whose root contains a `.nojekyll` marker and a `CNAME` file naming
fragcap.com, with no base path configured and image optimization disabled; run
`cargo xtask docs check` and confirm it runs the linter; confirm the command
returns the 0/1/2 contract.

**Acceptance Scenarios**:

1. **Given** the site app, **When** `cargo xtask docs build` runs, **Then** it
   produces a static export whose root contains `.nojekyll` and `CNAME`
   (fragcap.com), with no base path and image optimization disabled.
2. **Given** the site app, **When** `cargo xtask docs check` runs, **Then** it
   runs `scripts/lint-docs.sh check` and returns its result under the 0/1/2
   contract.
3. **Given** the site app, **When** `cargo xtask docs` runs, **Then** it starts
   the site locally with hot reload.

---

### User Story 5 - Continuous integration builds and deploys the site (Priority: P2)

On a push to the default branch, continuous integration builds the static export
and deploys it to GitHub Pages; on a pull request it builds the export without
deploying, so a broken build fails the pull request rather than the deploy. A
weekly job runs the linter's link mode to catch external-reference rot. Both
workflows were failing skeletons; this slice replaces them with real ones.

**Why this priority**: The site is worthless undeployed, and a deploy that only a
human can trigger rots. Wiring the build and deploy into continuous integration
is what makes the site a living artifact. The skeletons deliberately failed until
this slice; leaving them would either report false red forever or, if removed,
lose the reservation.

**Independent Test**: Confirm `docs.yml` builds the export, asserts the
`.nojekyll` marker and `CNAME` are present, uploads the Pages artifact and
deploys with the Pages permissions and environment on the default branch, and
builds without deploying on a pull request; confirm `links.yml` carries a weekly
schedule running the linter's link mode. The first green run is watched to
completion before being reported as passing (as `platform.yml` was).

**Acceptance Scenarios**:

1. **Given** a push to the default branch, **When** `docs.yml` runs, **Then** it
   builds the static export, confirms `.nojekyll` and `CNAME` are present, and
   deploys to GitHub Pages with `pages: write` and `id-token: write` permissions
   and a `github-pages` environment.
2. **Given** a pull request, **When** `docs.yml` runs, **Then** it builds the
   export and does not deploy.
3. **Given** the weekly schedule, **When** `links.yml` runs, **Then** it runs the
   linter's link mode against the documentation's external references.

---

### Edge Cases

- The static export omits the `.nojekyll` marker: the static host's legacy
  processing strips the framework asset directory and the site renders unstyled
  and non-interactive. The build asserts the marker's presence, so this fails the
  build rather than shipping.
- A base path is configured: the site serves from a repository subpath and every
  root-relative link breaks at the apex domain. The configuration forbids a base
  path, and the build asserts none is set.
- A glossary cross-link points at an anchor that moved when a term re-leveled
  during the split: the linter's cross-link check fails naming the dangling link,
  so the split cannot ship a broken navigation aid.
- The committed alphabetical index drifts from its category sources: fix mode
  regenerates it and check mode fails on any difference, so the index cannot go
  stale silently.
- A canonical document references a glossary term that has no entry: the linter's
  glossary-reference check fails naming the undefined term (constitution P-6, the
  Undefined Term Rule).
- An external reference URL rots: the weekly link mode reports it; it is not a
  per-commit failure, because link liveness depends on third parties, not on the
  commit.
- The brand orange is used as a general call-to-action color or to signal capture
  status alone: this violates the brand discipline (orange scarce, status never by
  color alone) and the "instrument, not weapon" acceptance test.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: A documentation site MUST be built with Fumadocs on Next.js,
  statically exported, and MUST live in a self-contained application directory
  managed with the project's chosen Node package manager, isolated from the Rust
  workspace.
- **FR-002**: The static export MUST enable export mode, disable image
  optimization, configure no base path, and emit a `.nojekyll` marker and a
  `CNAME` file naming fragcap.com into the export root (specification section
  22.2).
- **FR-003**: The site MUST carry the information architecture of specification
  section 22.3: a landing page, and documentation sections for getting started,
  guides, reference, architecture, glossary, and contributing.
- **FR-004**: The landing page MUST state what fragcap is in one sentence, show
  one worked invocation with its output, name the capture-driver prerequisite, and
  link to getting started, the repository, and the glossary, and MUST carry no
  testimonials, feature grid, or call to action (specification section 23.1).
- **FR-005**: The getting-started page MUST order the first run as install the
  capture driver with the required options, verify with the diagnostics command,
  capture, then open the result, and MUST set the three expectations (encrypted
  payloads, the uncaptured launcher handoff, loopback as self-talk) before the
  first capture instruction.
- **FR-006**: Every documentation page carrying a usage instruction MUST state the
  capture-driver prerequisite ahead of it (the npcap-constraint requirement S18
  carries).
- **FR-007**: The interim `docs/glossary.md` MUST be split into one authored
  source page per category from specification section 4.4, plus a generated
  alphabetical index at the glossary root.
- **FR-008**: Specification section 4.4 MUST be amended to add "Command Line and
  Diagnostics" as an eighth category, and section 22.4's category count MUST be
  reconciled accordingly; the amendment MUST be recorded as a dated decision.
- **FR-009**: The glossary MUST index at the heading level so each term is an
  independent search result, and the search tokenizer MUST split on whitespace,
  underscores, and hyphens so a compound identifier is findable by its parts.
- **FR-010**: A glossary entry's "why it matters here" note MUST render as a
  distinct visual element, and every cross-link between entries MUST resolve
  within and across category pages.
- **FR-011**: `scripts/lint-docs.sh` MUST exist, be built to the ShruggieTech Bash
  standard, and pass the repository's Bash compliance checker; it MUST provide
  three modes: check (validate and report, non-zero on failure), fix (regenerate
  the alphabetical index in place), and link (external reference verification).
- **FR-012**: Check mode MUST enforce the specification section 4.6 checks as they
  apply to the authored corpus: entry completeness (a prose blurb or detail on
  every entry, and a non-empty references section or matters callout where
  present), cross-link resolution, the glossary-reference check (every glossary
  reference in the canonical documents of section 4.2 names a defined term, the
  Undefined Term Rule), and, in link mode, external URL liveness. A references
  section is validated where present but not mandated on every entry: much of the
  glossary is fragcap's own internal vocabulary for which no primary source
  exists, and fabricating one would violate P-9 (recorded in the decisions
  fragment). A bare prose word that is not referenced as a glossary term is not
  scanned, because no sound rule distinguishes it from ordinary English.
- **FR-013**: The generated alphabetical index MUST be reproducible: fix mode
  regenerates it from the category sources, and check mode fails if the committed
  index differs, so the index cannot drift.
- **FR-014**: `cargo xtask docs` MUST replace the current stub: `docs` starts the
  site locally with hot reload, `docs build` produces the static export (the same
  entry point continuous integration uses, specification section 22.6), and `docs
  check` runs `scripts/lint-docs.sh check`, all under the 0/1/2 exit contract.
- **FR-015**: A documentation check MUST be added to the `cargo xtask ci` aggregate
  and as a named step in `ci.yml`, so P-6 is enforced on every push.
- **FR-016**: `.github/workflows/docs.yml` MUST be replaced with a real build and
  deploy: build the static export, confirm the `.nojekyll` marker and `CNAME` are
  present, upload the Pages artifact and deploy on the default branch with `pages:
  write` and `id-token: write` permissions and a `github-pages` environment, and
  build without deploying on a pull request.
- **FR-017**: `.github/workflows/links.yml` MUST be replaced with a real weekly
  job running the linter's link mode against the documentation's external
  references.
- **FR-018**: The site MUST apply the vendored `brand/` kit: Space Grotesk for
  display, Geist for body, and Geist Mono for code, packet data, and interface
  labels (all Open Font License 1.1, with the license texts shipped); the color
  tokens with the roughly 80 percent neutral, 15 percent cyan, at most 5 percent
  orange ratio, dark first; the favicons and web manifest; and the 1280 by 640
  social preview.
- **FR-019**: The site MUST carry the "A ShruggieTech project" endorsement in Geist
  Mono, uppercase, subordinate, outside the logo's clear space (footer or about
  only), and MUST NOT create a combined parent and product logo (Q-8).
- **FR-020**: The site's visual and verbal choices MUST satisfy the "instrument,
  not weapon" acceptance test: no saturated multi-color palette, no weapon,
  crosshair, skull, shield, hooded-figure, or circuit-board imagery, no exploit
  vocabulary, orange kept scarce and never the sole carrier of status, and the
  dry, precise voice that links unfamiliar terms to the glossary rather than
  simplifying (specification section 23.3).
- **FR-021**: Any term this slice introduces MUST receive a glossary entry in the
  same change (constitution P-6).
- **FR-022**: The changes to `.github/workflows/**` and `scripts/**` (pinned
  artifacts), and the specification section 4.4 amendment, MUST be recorded as
  dated decisions in the changelog.

### Key Entities *(include if data involved)*

- **Site application**: the Fumadocs on Next.js app in its own directory,
  statically exported, carrying the landing page and the documentation set.
- **Category page**: one authored glossary source page per specification section
  4.4 category (eight after the amendment).
- **Generated index**: the alphabetical glossary index at the glossary root,
  generated from the category pages and never hand-edited.
- **Documentation linter**: `scripts/lint-docs.sh`, three modes (check, fix,
  link), enforcing the specification section 4.6 checks and regenerating the
  index.
- **Docs task**: `cargo xtask docs` with its `docs`, `docs build`, and `docs
  check` subcommands, the single entry point for local development and continuous
  integration.
- **Docs workflows**: `docs.yml` (build and deploy to GitHub Pages) and
  `links.yml` (weekly link-mode verification), replacing the failing skeletons.
- **Brand kit**: the vendored `brand/` assets (fonts, logos, favicons, color
  tokens, social preview, endorsement) the site applies.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `cargo xtask docs build` produces a static export whose root
  contains a `.nojekyll` marker and a `CNAME` file naming fragcap.com, with no
  base path configured and image optimization disabled.
- **SC-002**: The glossary is split into one source page per specification section
  4.4 category (eight after the amendment) plus a generated alphabetical index,
  and every glossary cross-link resolves.
- **SC-003**: `bash scripts/lint-docs.sh check` exits 0 on the compliant glossary
  and exits non-zero naming each of an entry with no prose blurb or detail, an
  empty references section, a dangling cross-link, and a glossary reference to an
  undefined term; fix mode regenerates the index with no other change.
- **SC-004**: The alphabetical index is reproducible: regenerating it with fix
  mode leaves the committed index byte-for-byte unchanged.
- **SC-005**: `cargo xtask ci` passes with the documentation check included, and
  `scripts/lint-docs.sh` passes the repository's Bash compliance checker and
  carries the required encoding (UTF-8 without BOM, LF, no em or en dashes) per the
  conventions linter.
- **SC-006**: The landing page carries exactly one sentence of definition, one
  worked invocation with output, the named prerequisite, and the three links, and
  no testimonials, feature grid, or call to action; the getting-started page
  fronts the prerequisite and orders the first run install, verify, capture, open.
- **SC-007**: The site applies the brand fonts, color tokens, favicons, social
  preview, and the "A ShruggieTech project" endorsement, and satisfies the
  "instrument, not weapon" acceptance test with no excluded imagery or vocabulary.
- **SC-008**: `docs.yml` builds and (on the default branch) deploys the export to
  GitHub Pages with the Pages permissions and environment, and builds without
  deploying on a pull request; `links.yml` runs the linter's link mode weekly.

## Assumptions

- Hosting is GitHub Pages per specification sections 22.1 and 23.2; Cloudflare
  serves DNS only, configured by the operator from a documented runbook after
  merge (operator decisions, 2026-08-11). No Cloudflare credential enters
  continuous integration and `wrangler` is not a dependency.
- Enabling GitHub Pages, setting the custom domain, and editing Cloudflare DNS are
  operator actions performed once after merge; they are out of scope for the code
  slice, which ends at the pre-push halt. The site's live behavior at the apex
  domain (styling proving the `.nojekyll` marker worked, HTTPS, deep-link routing)
  is verified by the operator post-merge, tier 2, as live capture has been since
  S09.
- Node and a package manager are available on the continuous-integration leg that
  builds the site; the exact versions are pinned in `docs.yml` so the build is
  reproducible, and the pin is recorded as a dated decision because the workflow is
  a pinned artifact.
- The authored documentation is written for readers and links the master
  specification for depth rather than reproducing it; the glossary is the one body
  the site and the linter share as a single source.
- The vendored `brand/` kit (version 1.0.0) is complete and correct as of the
  brand session (2026-08-10); this slice applies it and does not re-derive it.
- `docs/glossary.md` at the time of the split is the authoritative interim source;
  the split re-levels each term's heading and rewrites in-file anchors to
  cross-page links, and the linter's cross-link check is the guard on that
  mechanical step.
