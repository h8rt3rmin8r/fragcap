# Feature Specification: Specification and constitution reconciliation

**Feature Branch**: `049-spec-constitution-reconciliation`

**Created**: 2026-08-16

**Status**: Draft

**Slice**: S049 (GitHub issue #136, milestone v0.5.0)

**Input**: fragcap v0.5.0 UX Handoff Plan sections 3.4, 3.7, 12 (S050 placeholder,
renumbered to S049), Appendix C, Appendix D.

## Clarifications

### Session 2026-08-16

- Q: During v0.5.0 development the workspace manifest is at 0.4.0; what value
  should `Applies-To` hold and what does the lock-step check assert? -> A: Track
  the workspace version. `Applies-To` equals `CARGO_PKG_VERSION` at all times
  (0.4.0 during development), the check is a simple equality against the
  workspace version, and both move to 0.5.0 together in the v0.5.0 release commit.
- Q: How deep should the version-currency sweep go beyond the enumerated
  sections? -> A: Full currency sweep. Correct every stale version reference
  bounded to version currency, including the document title, the section 1
  framing, and the section 28 heading (with its table-of-contents anchor and any
  inbound links), with no technical-content rewrites.

## User Scenarios & Testing *(mandatory)*

The actors here are the people and processes that consume the specification and
the constitution: an implementing agent that is instructed to trust the master
specification as the architecture of record, a contributor reading it cold, and
the release process that must not ship while the documents and the artifact
disagree.

### User Story 1 - The specification describes shipped reality (Priority: P1)

An agent or contributor opens `docs/fragcap-specification.md` to learn what
fragcap is and what has shipped. Today the document still frames v0.2.0 as the
first and only functional release, across its title, its document-control block,
and sections 3.3, 27.3, and 28, while the live release is v0.4.0. The reader is
therefore trusting a document that is two minor versions stale about the most
basic fact it asserts. After this slice, the document states the release history
through v0.4.0 as shipped and names v0.5.0 as the work in progress, and it
carries an explicit `Applies-To` field naming the released version it describes.

**Why this priority**: This is the defect the slice exists to correct. Every
other v0.5.0 slice depends on S049 precisely because they build on a document a
reader can trust. Constitution principle P-11 elevates this drift to the
severity of a failing test.

**Independent Test**: Read the specification with no other context. The latest
shipped release is determinable from the document alone (its `Applies-To` field
and its document-control history), and no section presents an unreleased version
as already shipped or omits a shipped one.

**Acceptance Scenarios**:

1. **Given** the specification's document-control history, **When** a reader
   looks for the release history, **Then** it lists v0.1.0 through v0.4.0 as
   shipped with their scope, and identifies v0.5.0 as in progress.
2. **Given** sections 3.3, 27.3, and 28, **When** a reader looks for the first
   functional release, **Then** the text reflects the real history rather than
   presenting v0.2.0 as the first or only functional release.
3. **Given** section 23.1, **When** a reader reads the landing-page description,
   **Then** it matches the settled Appendix D replacement text.
4. **Given** the whole document, **When** a version-currency sweep runs, **Then**
   no residual reference presents v0.2.0 (or any unreleased version) as the
   current or first functional release.

---

### User Story 2 - The durable rules survive every future session (Priority: P1)

A future agent session, with none of this conversation's context, must still be
bound by the two rules this release establishes: that every target is created by
one operation and stored in one form (the guiding light for future platforms),
and that the specification always describes what shipped. These are recorded as
constitution principles P-10 and P-11 so they hold without restatement.

**Why this priority**: P-10 governs the architecture of every later v0.5.0 slice
(S052 onward), and P-11 is the rule that makes this very reconciliation binding
rather than a one-time cleanup. Both must exist before the slices that rely on
them.

**Independent Test**: Read `.specify/memory/constitution.md`. It carries P-10 and
P-11 verbatim from Appendix C, its Sync Impact Report header records the
addition, and its version is bumped per the versioning policy.

**Acceptance Scenarios**:

1. **Given** the constitution, **When** a reader looks for the target-creation
   rule, **Then** P-10 (One Path To A Target) is present verbatim.
2. **Given** the constitution, **When** a reader looks for the specification-
   currency rule, **Then** P-11 (The Specification Describes What Shipped) is
   present verbatim.
3. **Given** the constitution header, **When** a reader checks the version,
   **Then** it has been bumped (MINOR, two principles added) and the Sync Impact
   Report records the change and reasoning.

---

### User Story 3 - Drift cannot recur silently (Priority: P2)

The condition this slice corrects took two minor versions to be noticed because
nothing mechanical was watching for it. A repository check now asserts that the
specification's `Applies-To` value equals the workspace package version, and it
runs in the same check set every contributor and the continuous-integration
pipeline already run, so the two can never diverge unnoticed again.

**Why this priority**: Without the mechanical gate, the reconciliation is a
snapshot that decays the next time a release ships without a matching
specification edit. It is the enforcement that makes P-11 real, but it depends on
the `Applies-To` field existing first (User Story 1).

**Independent Test**: With the `Applies-To` value equal to the workspace version,
the full check set passes; with them made to differ, the check set fails and
names the mismatch. The check appears in both the local check set and the CI
pipeline.

**Acceptance Scenarios**:

1. **Given** `Applies-To` equals the workspace version, **When** the full check
   set runs, **Then** the version check passes.
2. **Given** `Applies-To` and the workspace version differ, **When** the full
   check set runs, **Then** the version check fails and reports both values.
3. **Given** the continuous-integration configuration, **When** the pipeline
   runs, **Then** the version check is one of the steps it executes.

---

### User Story 4 - A named section change is backed by a real edit (Priority: P3)

When a release is assembled, a changelog fragment may declare that it changed a
specification section. A release-time gate asserts that any fragment naming a
section is backed by an actual specification edit within that release's diff, so
a fragment cannot claim a specification change that did not happen.

**Why this priority**: This closes the reverse of the drift, a fragment that
asserts a specification impact the specification does not reflect. It is the
lowest priority because it only binds at release assembly and depends on the
fragment-format change landing first.

**Independent Test**: A changelog fragment whose `spec-impact` names a section,
introduced in a release whose diff contains no change to the specification,
causes the release gate to fail. A fragment with `spec-impact: none`, or one
whose named section was actually edited in the diff, passes.

**Acceptance Scenarios**:

1. **Given** a fragment with `spec-impact: none`, **When** the release gate runs,
   **Then** it imposes no specification-change requirement for that fragment.
2. **Given** a fragment naming section 23.1 and a release diff that edits section
   23.1, **When** the release gate runs, **Then** it passes.
3. **Given** a fragment naming section 23.1 and a release diff that does not
   touch the specification, **When** the release gate runs, **Then** it fails and
   names the fragment and section.

---

### Edge Cases

- **Missing `Applies-To` field.** If the field is absent entirely, the version
  check treats it as a divergence and fails, rather than passing vacuously.
- **The release version bump.** Because `Applies-To` equals the workspace version
  at all times, the v0.5.0 release commit must move both together, or the version
  check fails on the release branch. This mirrors the existing requirement that
  the release commit regenerate the golden corpus and version assertions.
- **A corrected heading changes an anchor.** Renaming the section 28 heading away
  from "Roadmap Beyond v0.2.0" changes its generated anchor
  (`#28-roadmap-beyond-v020`); the table-of-contents entry and any inbound links
  must be updated in the same change, or a dead intra-document link is left
  behind.
- **A fragment naming a nonexistent section.** Out of scope to fully validate
  section existence; the gate's contract is "a named section is backed by a
  specification edit," and this behavior is recorded for the plan phase to
  decide rather than left implicit.
- **Multiple fragments naming sections in one release.** Every such fragment must
  be independently satisfied by a specification edit in the diff.
- **Defining "this release's diff".** The gate needs a boundary for what counts
  as the release's changes; the reasonable default is the range since the most
  recent release tag, resolved in the plan phase.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The specification's document-control history MUST cover v0.1.0
  through v0.4.0 as shipped releases with their scope, and MUST identify v0.5.0
  as the work in progress.
- **FR-002**: Sections 3.3, 27.3, and 28 MUST describe the released software
  through v0.4.0 rather than presenting v0.2.0 as the first or only functional
  release.
- **FR-003**: Section 23.1's landing-page paragraph MUST be replaced with the
  settled Appendix D text from the handoff plan, verbatim.
- **FR-004**: No specification section may present an unreleased version as
  already shipped, nor omit a shipped release. A full version-currency sweep MUST
  correct every residual stale reference, bounded to version currency and without
  rewriting technical content. The sweep MUST include the document title
  (currently "fragcap v0.1.0 Technical Specification"), the section 1 framing
  (currently naming v0.2.0 "the first public release"), and the section 28
  heading (currently "Roadmap Beyond v0.2.0"); when a corrected heading changes
  its table-of-contents anchor, the table of contents and any inbound links MUST
  be updated in the same change.
- **FR-005**: The specification's document-control block MUST carry an
  `Applies-To` field whose value is the workspace package version
  (`CARGO_PKG_VERSION`), that being the released version the document describes.
- **FR-006**: The constitution MUST add P-10 (One Path To A Target) and P-11 (The
  Specification Describes What Shipped) verbatim from Appendix C, with the Sync
  Impact Report header updated and the version bumped (MINOR, two principles
  added).
- **FR-007**: A repository check MUST assert that the specification's `Applies-To`
  value equals the workspace package version, failing and reporting both values
  when they diverge, including when the field is absent.
- **FR-008**: The version check MUST run as part of the local full check set
  (`cargo xtask ci`) and in the continuous-integration pipeline (`ci.yml`).
- **FR-009**: The changelog-fragment format MUST gain a `spec-impact` field whose
  value is either `none` or a list of specification section numbers, and the
  fragment documentation MUST describe it.
- **FR-010**: A release-time gate MUST assert that if any fragment in a release
  names a section in its `spec-impact` field, the specification was modified
  within that release's diff, failing and naming the fragment and section
  otherwise.
- **FR-011**: The change to the pinned artifact `ci.yml` MUST be accompanied by a
  dated `decisions` fragment under `changelog.d/`, per the constitution.
- **FR-012**: P-10 and P-11 MUST reside under `.specify/` (in the constitution)
  so they are loaded into every future agent session without restatement.

### Key Entities

- **`Applies-To` field**: a value in the specification's document-control block
  naming the released version the specification currently describes. The anchor
  the version check binds against.
- **Constitution principles P-10 and P-11**: the two durable rules added by this
  slice, verbatim from the handoff plan Appendix C.
- **Version lock-step check**: the repository check asserting `Applies-To` equals
  the workspace package version.
- **`spec-impact` fragment field**: a per-fragment declaration of which
  specification sections a change touched, or `none`.
- **Release-time spec-impact gate**: the release-assembly check that a named
  section is backed by a real specification edit in the release diff.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A reader can determine the latest shipped release from the
  specification alone, without consulting git history or the workspace manifest.
- **SC-002**: The specification contains zero references presenting an unreleased
  version as the current or first functional release.
- **SC-003**: 100% of divergences between `Applies-To` and the workspace version
  are caught by the check set: it passes when they are equal and fails when they
  differ.
- **SC-004**: A changelog fragment that names a specification section without a
  corresponding specification change in the release diff causes the release gate
  to fail, in every such case.
- **SC-005**: The constitution carries P-10 and P-11 verbatim and its version is
  bumped, verifiable by reading the file.
- **SC-006**: The full local check set (`cargo xtask ci`) and the CI pipeline both
  execute the version check and pass on the reconciled tree.

## Assumptions

- **`Applies-To` tracks the released (workspace) version.** (Confirmed in the
  2026-08-16 clarification.) The project carries the last-shipped version in the
  workspace manifest between releases (currently 0.4.0), so during v0.5.0
  development `Applies-To` reads 0.4.0 and both it and the workspace version move
  to 0.5.0 in the v0.5.0 release commit. The check is a simple equality against
  the workspace version. The definition-of-done statement that `Applies-To`
  "reads 0.5.0" describes the post-release end state.
- **The specification body may describe v0.5.0 work as in progress** while
  `Applies-To` names the last shipped version; `Applies-To` is the mechanism that
  keeps P-11 satisfied while forward-looking content exists, exactly as the
  document has always described roadmap work.
- **The sweep is a full version-currency pass.** (Confirmed in the 2026-08-16
  clarification.) The enumerated sections (document-control history, 3.3, 23.1,
  27.3, 28) are the named subjects; FR-004 extends the pass to every other stale
  version reference, including the document title, the section 1 framing, and the
  section 28 heading and its anchor, bounded to version currency rather than
  technical rewrites.
- **Appendix C and Appendix D text are taken verbatim** from the handoff plan;
  copy wording is delegated (the operator declined to review it).
- **The release gate defines "this release's diff"** as the range since the most
  recent release tag; the exact resolution is a plan-phase decision.
- **The version check is added to the existing `xtask` crate** as a new
  subcommand or module and wired into the existing `ci` aggregation and
  `ci.yml`; the precise shape is a plan-phase decision.
- **No source code behavior changes.** This slice touches documentation, the
  constitution, the changelog-fragment convention, and repository check tooling
  only; it does not alter capture, attribution, or output behavior.

## Constitution alignment

- **P-8 (House Standards Apply) and text hygiene**: all edited files stay UTF-8
  without BOM, LF line endings, no trailing whitespace, single trailing newline,
  and no em-dashes or en-dashes. Appendix C and D text must be transcribed under
  the same rule.
- **P-11 (added here)**: this slice is the first application of the principle it
  introduces; the version check is what makes the principle enforceable rather
  than aspirational.
- **Pinned-artifact rule**: `ci.yml` is pinned, so FR-011 requires the dated
  decisions fragment. No other pinned artifact (`rust-toolchain.toml`,
  `release.toml`, `scripts/**`, release docs) is changed unless the release-gate
  implementation requires it, in which case the same rule applies.
- **Amendment policy**: the constitution version bump is MINOR (two principles
  added), and the Sync Impact Report header is updated with the change and its
  reasoning.
