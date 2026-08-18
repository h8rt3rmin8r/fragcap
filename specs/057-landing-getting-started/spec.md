# Feature Specification: Landing page and getting-started rewrite

**Feature Branch**: `057-landing-getting-started`

**Created**: 2026-08-18

**Status**: Draft

**Input**: Slice S057, issue #144, milestone v0.5.0 (the last v0.5.0 slice; depends on S056).

## Context

fragcap's public site and first-run documentation still describe the tool that
existed before slice S054. S054 collapsed `run`/`tap`/`watch` into one `capture`
verb, removed the entire profile-file surface (the `profile` command, the AppData
profile directory, the `--profile` selector, `steam profile`, `profile validate`),
and made target registration and discovery the way a user names what to capture.
S055 added the `fragcap targets` hero listing and `fragcap capture <n>`. None of
that reached `site/content/docs` or the landing page.

The result is a first-run path that routes a new user into hand-authoring a
schema-validated JSON profile file that the tool no longer reads, a landing page
that reads as reference documentation rather than reaching a visitor who has not
yet thought about attribution, and a reference set that documents commands that no
longer exist. This slice converges the site and the first-run narrative onto what
shipped, and closes the getting-started QA batch (#130, #131, #132, #133, #134,
#135) as it lands.

This is a documentation slice with one small, necessary companion code change (see
User Story 4): `fragcap doctor` still prints a `profile dir` row and a `Profiles`
section referencing the retired profile directory, a leftover S054 did not remove.
That leftover must be removed for the getting-started `doctor` sample to be both
faithful to the binary and free of the retired directory.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A first-time reader reaches a completed capture (Priority: P1)

A reader who has never used fragcap opens the getting-started guide and follows it
top to bottom. Nothing in the guide asks them to run a command before the tool
that provides it is installed, to click a download with no link, or to author a
file. Prerequisites are acquired before the numbered steps begin. They install
fragcap, verify the environment (told plainly that capture needs an elevated
terminal), let `fragcap targets` show them what is capturable on their machine,
run `fragcap capture <n>` against a listed row, and open the resulting `.fcapng`
file in Wireshark. The guide ends with a capture file on disk.

**Why this priority**: This is the wall the whole v0.5.0 release exists to remove.
A getting-started guide that dead-ends (issues #130, #131, #135) or teaches a
retired file format is the single highest-value thing to fix, and it is the
slice's headline acceptance criterion.

**Independent Test**: Read the rewritten `getting-started.mdx` as a literal
first-time reader. Confirm every instruction is executable at the point it appears,
no step references a retired verb/command/directory/slug, and the final step
produces a capture file. The documentation linter and the site build pass.

**Acceptance Scenarios**:

1. **Given** a reader with no prior knowledge, **When** they follow the guide top
   to bottom, **Then** every command is runnable at the point it is given (the
   binary exists, the link is present, the privilege is stated) and the last step
   yields a `.fcapng` file.
2. **Given** the "Before you begin" section, **When** the reader reaches the
   numbered steps, **Then** the npcap/Wireshark prerequisites have already been
   acquired (conditionally: skip if present), not deferred to a later numbered
   step (#132).
3. **Given** the "Get a target" step, **When** the reader looks for what to
   capture, **Then** discovery is the happy path (`fragcap targets`), a Steam App
   ID is defined inline with where to find it, and no placeholder is left
   undefined (#135).
4. **Given** the "Verify the install" step, **When** the reader runs `fragcap
   doctor`, **Then** the guide has told them to open the terminal as Administrator
   and this is where the optional `fragcap extcap install` is introduced (#134,
   #130).

### User Story 2 - A technically competent visitor understands the gap in one screen (Priority: P1)

A visitor who is competent with packet capture but has never thought about
process attribution lands on `fragcap.com`. The opening line states the problem
plainly ("Your capture recorded 40,000 packets. It cannot tell you which one your
game sent."), the following two paragraphs explain why (capture happens below the
socket layer; the process is three launcher hops from what you launched; fragcap
reconstructs the link and writes it into the file). A rendered `fragcap targets`
output block shows the tool working, a dependency diagram answers "what is npcap
and why do I need it," and a single call to action points at getting started. The
page carries no testimonials, feature grids, badges, pricing, or sponsorship
solicitation, and reads as instrument documentation with a strong opening.

**Why this priority**: The landing page is the first surface a visitor meets and
the spec (section 23.1, as amended) makes the strong opening a requirement. It is
independently valuable and independently testable from the getting-started rewrite.

**Independent Test**: View the landing page. Confirm the settled opening copy is
present verbatim, the persuasive asset is a `fragcap targets` output block in the
monospace face, the dependency diagram is present, there is exactly one primary
call to action, and none of the section-23.1 prohibitions appear. No retired verb
or slug appears anywhere on the page.

**Acceptance Scenarios**:

1. **Given** the landing page, **When** it is viewed, **Then** the settled opening
   sentence and the two explanatory paragraphs from issue #144 appear as written.
2. **Given** the landing page, **When** the worked example is shown, **Then** it is
   a `fragcap targets` listing in the monospace face, not the retired `fragcap run
   --profile eso` block.
3. **Given** the landing page and the brand page, **When** either is searched for
   retired verbs or slugs, **Then** none are found.

### User Story 3 - The reference set matches the shipped command surface (Priority: P2)

A reader consulting the reference and guides finds only commands the binary
actually has. The CLI reference documents `capture`, `replay`, `targets`,
`technologies`, `steam`, `doctor`, `extcap`, `catalog`, and `schema` with their
real flags; the capture-modes guide uses `fragcap capture`; the two pages that
taught authoring profile files (`writing-a-profile`, `profile-schema`) are gone,
with their navigation and inbound links updated; nothing points at a page that no
longer exists.

**Why this priority**: The acceptance criterion "no documentation page references
the retired verbs/directory" is site-wide, so the reference convergence is
required for the slice to pass its own acceptance test, but it is lower user-facing
priority than the first-run and landing surfaces a new visitor meets first.

**Independent Test**: Grep the whole `site/` tree for the retired verbs, commands,
selector, and slugs; find none. Build the site; no broken internal links. The CLI
reference matches the `clap` grammar in `crates/fragcap-cli/src/cli.rs`.

**Acceptance Scenarios**:

1. **Given** the CLI reference, **When** it is read against `cli.rs`, **Then** every
   documented command and its flags exist in the grammar, and no retired command
   appears.
2. **Given** the site navigation and all inbound links, **When** the two retired
   pages are removed, **Then** no link or nav entry points at them.
3. **Given** the capture-modes guide, **When** its examples are read, **Then** they
   use `fragcap capture --target <selector>`, not `fragcap run --profile`.

### User Story 4 - The doctor sample is faithful and free of the retired directory (Priority: P1)

The `fragcap doctor` output shown in getting started matches what the binary
prints. Because the binary still emits a `profile dir` identity row and a
`Profiles` section pointing at the retired profile directory, the binary is
corrected to stop emitting them, and the sample reflects the corrected output.

**Why this priority**: The acceptance criteria require both a faithful doctor
sample and no documentation reference to the retired profile directory. These two
cannot both hold while the binary emits the retired rows, so the code change is a
gating dependency of the getting-started rewrite, not optional polish.

**Independent Test**: Run `fragcap doctor` (or its unit tests) and confirm the
report carries no `profile dir` row and no `Profiles` section; confirm the identity
section is `version`, `binary`, `catalog db`, `local db`. Confirm the
getting-started sample matches. `cargo xtask ci` is green.

**Acceptance Scenarios**:

1. **Given** the corrected binary, **When** `doctor` runs, **Then** the report
   contains no `profile dir` row and no `Profiles` section, and the exit status and
   all other rows are unchanged.
2. **Given** the getting-started guide, **When** its doctor sample is compared to
   the binary output, **Then** they agree.

### Edge Cases

- A reader who already has npcap and Wireshark installed must be able to skip the
  prerequisite walkthrough without wading through it (conditional framing, #132).
- A reader on a non-Steam or manually-installed game must have a path that does not
  assume Steam (the `fragcap targets add`/`scan` route), since discovery and Steam
  registration are not the only case (#135 sub-point 2).
- The `doctor` JSON output (`--json`) must also lose the retired rows, not only the
  human report, so no machine consumer sees the retired directory.
- Removing the two reference pages must not break the glossary index reproducibility
  check or leave a dangling cross-link in a canonical document.

## Requirements *(mandatory)*

### Functional Requirements

**Landing and brand (US2)**

- **FR-001**: The landing page MUST open with the settled sentence "Your capture
  recorded 40,000 packets. It cannot tell you which one your game sent." followed
  by the two explanatory paragraphs from issue #144, implemented as written.
- **FR-002**: The landing page's primary worked example MUST be a rendered `fragcap
  targets` output block in the monospace face on the dark ground, and MUST NOT be
  the retired `fragcap run --profile eso` block.
- **FR-003**: The landing page MUST carry the dependency-model diagram (npcap
  required, Wireshark recommended, extcap optional) and exactly one primary call to
  action (getting started).
- **FR-004**: The landing page MUST NOT carry testimonials, feature grids, badges,
  pricing, or sponsorship solicitation, and MUST hold to section 23.3 voice.
- **FR-005**: The brand page's typographic specimen line MUST NOT use the retired
  `fragcap run --profile eso` command; it MUST use a current invocation.

**Getting started (US1)**

- **FR-006**: The getting-started guide, followed literally by a reader with no
  prior knowledge, MUST end with a capture file on disk, using `fragcap targets`
  then `fragcap capture <n>`.
- **FR-007**: "Before you begin" MUST acquire the npcap/Wireshark prerequisites
  before the numbered steps, framed conditionally (skip if already present) (#132),
  and MUST keep the extcap mention descriptive rather than issuing a command the
  reader cannot yet run (#130).
- **FR-008**: The "Install fragcap" step MUST provide a download affordance: a link
  to the GitHub releases page and the names of the `.msi`, `.zip`, and `.sha256`
  assets (#131).
- **FR-009**: The "Verify the install" step MUST tell the reader to open the
  terminal as Administrator and explain that elevation is what turns the privilege
  and capture rows green, and MUST be the single home for the optional `fragcap
  extcap install` guidance (#134, #130).
- **FR-010**: The "Get a target" step MUST present discovery (`fragcap targets`) as
  the happy path, define a Steam App ID inline with where to find it, and reconcile
  the capture step so no placeholder is left undefined; it MUST NOT reference
  authoring a profile file or the `--profile` selector (#135).
- **FR-011**: The guide MUST carry the dependency-model diagram so a confused
  installer meets the "what is npcap and why do I need it" answer on the first-run
  page rather than only on Architecture.
- **FR-012**: The npcap narrative across the guide MUST be internally coherent with
  the installer's exit-dialog npcap prompt and the S056 `doctor --fix` npcap action:
  npcap is a separate, detection-only prerequisite; fragcap may fetch and launch the
  vendor's own signed installer only under explicit interactive confirmation (as in
  `doctor --fix`); nothing fragcap ships bundles or hosts npcap (#133 docs half).

**Reference convergence (US3)**

- **FR-013**: The CLI reference MUST document the real command surface (`capture`,
  `replay`, `targets`, `technologies`, `steam`, `doctor`, `extcap`, `catalog`,
  `schema`) and their flags as declared in `crates/fragcap-cli/src/cli.rs`, and MUST
  NOT document `run`, `tap`, `watch`, `profile`, or `steam profile`.
- **FR-014**: The capture-modes guide MUST use `fragcap capture --target <selector>`
  in its examples and MUST NOT use `fragcap run --profile`.
- **FR-015**: The pages `guides/writing-a-profile.mdx` and
  `reference/profile-schema.mdx` MUST be removed, and every inbound link and
  navigation entry (in `index.mdx`, `architecture.mdx`, `meta.json`, and the landing
  page) MUST be updated so nothing points at a removed page.
- **FR-016**: No page under `site/` MUST reference the retired profile directory,
  the retired verbs (`run`/`tap`/`watch`), the retired commands (`steam profile`,
  `profile validate`), the `--profile` selector, or a profile slug that does not
  exist.

**Doctor companion change (US4)**

- **FR-017**: `fragcap doctor` MUST NOT emit a `profile dir` identity row or a
  `Profiles` section, in either its human report or its `--json` output, and its
  identity section MUST be `version`, `binary`, `catalog db`, `local db`.
- **FR-018**: The doctor change MUST NOT alter the exit status or any other check,
  and the now-dead `profile_dir`/`bundled_count`/`user_count` inputs and their probe
  plumbing MUST be removed rather than left computing unused values.
- **FR-019**: The getting-started `doctor` sample MUST match the corrected binary
  output.

**Cross-cutting**

- **FR-020**: The documentation linter (`scripts/lint-docs.sh check` via `cargo
  xtask docs check`) MUST pass, and `cargo xtask ci` MUST be green.
- **FR-021**: Any new term introduced by the rewrites MUST carry a glossary entry in
  the same change (P-6); a page that references a glossary term MUST reference a
  defined one.

### Key Entities

- **Getting-started guide**: the first-run narrative
  (`site/content/docs/getting-started.mdx`); its endpoint is a capture file on disk.
- **Landing page**: `site/app/(home)/page.tsx`; the strong-opening problem
  statement, the worked `fragcap targets` example, the dependency diagram, one CTA.
- **CLI reference**: `site/content/docs/reference/cli.mdx`; the readable mirror of
  the `clap` grammar in `cli.rs`.
- **Doctor report**: the environment readiness report emitted by `fragcap doctor`;
  its identity section and section list are the contract the getting-started sample
  mirrors.
- **Dependency model**: npcap (required), Wireshark (recommended), extcap (optional),
  already a Mermaid diagram on Architecture, promoted to landing and getting started.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A reader following getting started literally, with no prior knowledge,
  reaches a capture file on disk with no dead-end step (every command runnable at the
  point it appears).
- **SC-002**: A site-wide search for the retired verbs (`fragcap run`, `fragcap tap`,
  `fragcap watch`), the retired commands (`steam profile`, `profile validate`), the
  `--profile` selector, the profile directory, and the retired `eso`/`<game-id>`
  profile slugs returns zero matches across `site/`.
- **SC-003**: The `fragcap doctor` output shown in getting started is byte-consistent
  with the corrected binary's output (no `profile dir`, no `Profiles`).
- **SC-004**: The landing page presents the settled opening copy, a `fragcap targets`
  worked example, the dependency diagram, and exactly one primary call to action,
  with none of the section-23.1 prohibitions present.
- **SC-005**: The documentation linter passes and `cargo xtask ci` is green.
- **SC-006**: The QA issues #130, #131, #132, #133 (docs half), #134, and #135 are
  each resolved by a concrete change in the guide, and can be closed as the slice
  lands.

## Assumptions

- The published master schema page (`reference/target-schema.mdx`) is current and
  correct (it documents the schema `fragcap schema validate/print` uses) and stays;
  only the stale `profile-schema.mdx` subset is removed.
- The internal `Profile`/`BundledSet`/`SearchPath` types stay (capture synthesizes a
  one-stage profile internally); only the user-facing profile-file documentation and
  the dead doctor profile rows are removed. The doctor change removes reporting, not
  the capture-config type.
- The IGDB credential walkthrough and IGDB enrichment are OUT OF SCOPE and deferred
  to a dedicated slice: the codebase has no IGDB/Twitch/credential storage plumbing
  (the S050 local.db columns issue #144 assumes exist were never built), so a
  credential-registration walkthrough would document unbuilt functionality (P-11).
  This is surfaced explicitly rather than silently absorbed.
- The dependency-model diagram already rendered on Architecture is reused (same
  Mermaid content) rather than redesigned.
- The doctor sample may show a representative version string; version drift in an
  illustrative sample is not a correctness defect, but the row set must match.
- The site MDX pages are not scanned by the documentation linter (it covers
  `docs/glossary/` and the canonical `README.md`/`docs/*.md`); the retired-verb
  acceptance criterion is verified by grep and the site build, not by the linter.

## Clarifications

### Session 2026-08-18

Resolved by decision under the operator's standing autopilot preference (decide and
proceed; surface, do not present confirmation menus). Recorded here for traceability
into planning and analysis.

- Q: What exactly does the landing page's worked `fragcap targets` example show?
  A: The S055 hero listing (numbered `# / TARGET / CAPTURE / KNOWN` columns) ending
  with the `fragcap capture <n>` hint line, mirroring the shipped output and the
  README quickstart, so the persuasive asset is the real tool output.
- Q: Are the two retired reference pages deleted outright or replaced with redirect
  stubs? A: Deleted outright, with navigation and inbound links updated. The static
  export has no configured redirect mechanism, and a pre-1.0 docs site can drop the
  routes; the conceptual content (stages, `descends_from`) already lives on the
  Architecture page and the master `target-schema` reference survives.
- Q: What replaces the retired landing-page capability bullet that linked to
  "Writing a profile"? A: A bullet describing discovery-and-registration
  (`fragcap targets`) linking to getting started, keeping every landing-page link
  pointed at a page that still exists.

## Dependencies

- Depends on S056 (merged): the `doctor --fix` npcap action is the behavior the #133
  narrative reconciles against.
- Reflects S054 (the CLI surface rework) and S055 (the `targets` hero command) as the
  shipped reality the docs converge onto.
