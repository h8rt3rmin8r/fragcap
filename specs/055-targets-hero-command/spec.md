# Feature Specification: The targets hero command and interactive authoring

**Feature Branch**: `055-targets-hero-command`

**Created**: 2026-08-18

**Status**: Draft

**Input**: Slice S055 (issue #142, milestone v0.5.0). Source: fragcap-v0.5.0-UX-Handoff-Plan.md sections 3.5, 3.6, 9.5, 9.6. Depends on S054 (merged).

## Overview

`fragcap targets` is the product's hero command: the single command a new user
runs successfully on their own machine, with no arguments and no prior
configuration, that makes fragcap's value concrete using their own installed
software rather than an example. It lists the user's capturable targets, ends by
naming the next command to run, and offers an interactive path to author a new
target when discovery did not find one. It is deliberately named `targets`, not
`games`, because the underlying schema admits non-game software, emulated
titles, launchers, and mods.

This slice turns the existing `targets` subcommand family (which already carries
`list`, `add`, `show`, `discover`, `scan`) into that hero surface, adds the
lifecycle commands the surface needs (`remove`, `export`, `import`), makes the
listing's row indices durable so `fragcap capture <n>` refers to exactly what the
user just saw, and adds the interactive authoring flow whose reason for existing
is the honest `unsure` answer to "which process holds the sockets?".

## Clarifications

### Session 2026-08-18

- Q: Does interactive `targets add` replace or supplement the S051 flag-driven
  add? → A: Supplement. Interactive prompts run only when standard input is a
  terminal; otherwise the flag-driven form (name/exe/steam/anchor/handle) is used.
  This keeps the command testable in CI and scriptable (FR-015, FR-008).
- Q: Is `capture <n>` / `show <n>` row resolution now backed by a persisted
  listing snapshot rather than the live target ordering it used through S054? →
  A: Snapshot-backed. Any listing (bare `fragcap`, `fragcap targets`,
  `targets list`) writes the ordered snapshot it displayed, and every row-index
  selector resolves against that snapshot; an out-of-range index is a usage error
  (FR-004, FR-005). This is the load-bearing behavioral change of the slice.
- Q: What is the shape of the export/import document? → A: A dedicated JSON array
  of target-entry objects, carrying the entry identity (stable id, handle, name,
  classification, fidelity, anchor, launch entries, install root, evidence). It is
  NOT the published capture schema (`target-schema.v1.json`), whose export records
  are catalog games and which omits the entry identity that merge-on-id needs.
  `export <selector>` emits a one-element array; `export` with no selector emits
  all targets; `import` reads the array and merges each element on its stable
  identifier. S055 does not change or version the published capture schema; a
  published target-entry schema, if wanted, is an additive follow-up (FR-018,
  FR-019, FR-020). (Operator decision, 2026-08-18.)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - The hero listing (Priority: P1)

A new user, having just installed fragcap, runs `fragcap targets` (or bare
`fragcap`) with no arguments and no prior configuration. Discovery runs across
its tiers, and the user sees a numbered table of their own installed software,
each row marked as capturable-now or needing a target, with neutral evidence
about what each title is known to run (anti-cheat, DRM, launcher mediation, or
that nothing online was recorded). The listing ends by naming the exact next
command, e.g. `fragcap capture 1`. The row numbers the user sees are the numbers
`fragcap capture <n>` will honor.

**Why this priority**: This is the hero command itself and the whole point of
the slice. Delivered alone it satisfies all five hero acceptance criteria and
gives a fresh install its first successful, value-demonstrating interaction. The
other stories extend it but this is the MVP.

**Independent Test**: Run the listing against a store seeded with a few targets
of varying readiness and evidence; assert the table shape, the deterministic
handle ordering, the trailing next-command line, and that a subsequent row-index
selector resolves to the row that was displayed. Run it again against an empty
store and assert the empty-case next-commands block.

**Acceptance Scenarios**:

1. **Given** a fresh install with no prior config, **When** the user runs
   `fragcap targets` with no arguments, **Then** discovery runs and the command
   completes successfully in a few seconds without modifying any target data.
2. **Given** a store holding several targets of mixed readiness, **When** the
   user runs the listing, **Then** each row shows a 1-based index, the target
   handle, a CAPTURE status of `ready` or `needs a target`, and a KNOWN column
   carrying neutral evidence, ordered deterministically by handle.
3. **Given** a listing was just displayed, **When** the user runs
   `fragcap capture <n>` for a row number `n` from that listing, **Then** the
   capture resolves to exactly the target that occupied row `n` in the listing
   the user saw, even if the underlying set has not changed.
4. **Given** discovery returns nothing and the store is empty, **When** the user
   runs the listing, **Then** the output names the concrete commands that would
   populate it (how to add or scan for a target) rather than printing an empty
   table.
5. **Given** any listing, **When** it finishes, **Then** its final line names
   the next command the user should run.

---

### User Story 2 - Interactive authoring with the honest `unsure` branch (Priority: P2)

A user whose title discovery did not find (or who wants to register software
explicitly) runs `fragcap targets add`. The tool points at an executable (the
user supplies a path, or presses Enter to browse for one), runs the detection
scan and shows the engine, anti-cheat, and DRM evidence inline, prompts for a
display name and a handle (offering a derived default), and then asks the one
question that matters: "Is the executable above the process that holds the
sockets? [Y/n/unsure]". The `unsure` answer is a first-class outcome: it
registers the entry with its launch chain left unresolved, so the first capture
run observes the real socket-holding process and promotes the row. The tool
never guesses the socket holder and presents the guess as fact.

**Why this priority**: This is how the empty or incomplete case from Story 1
becomes a capturable target using the user's own data, and it encodes the P-1 /
P-9 honesty posture (observe, do not fabricate). It depends on Story 1 existing
as the surface a new entry returns to.

**Independent Test**: Drive the authoring flow with scripted answers for each
branch (Y, n, unsure) and assert the resulting stored entry: a `Y`/`n` answer
records a resolved launch chain, `unsure` records an unresolved one, and none of
the three fabricates a socket holder. Separately, run `targets add --steam <app_id>`
and assert it scaffolds a registered entry with a `steam:<app_id>` anchor.

**Acceptance Scenarios**:

1. **Given** the user runs `targets add` and points at an executable, **When**
   the scan completes, **Then** the engine, anti-cheat, and DRM evidence is shown
   inline before any prompt that depends on it.
2. **Given** the authoring prompt for name and handle, **When** the user accepts
   the offered default handle, **Then** the stored handle is the derived default,
   disambiguated if it collides with an existing one.
3. **Given** the socket-holder question, **When** the user answers `unsure`,
   **Then** the entry is registered with its launch chain unresolved and no
   fabricated socket-holding process, and a subsequent capture run promotes the
   row to `verified` once it observes the real holder.
4. **Given** the socket-holder question, **When** the user answers `Y` or `n`,
   **Then** the entry records the corresponding resolved launch chain.
5. **Given** `targets add --steam <app_id>` for an installed title, **When** it
   runs, **Then** it scaffolds and registers a target carrying a `steam:<app_id>`
   anchor (the capability formerly named `steam profile`).

---

### User Story 3 - Target lifecycle: scan, remove, export, import (Priority: P3)

A user manages their registered targets: scans a directory to register what is
found there, removes a target they no longer want, exports one (or all) to a
JSON file that conforms to the master schema, and imports a JSON file another
user or machine produced. Import merges on the stable identifier so the same
target from two sources becomes one row, not two.

**Why this priority**: These round out the target store into something a user can
curate and move between machines, but the hero interaction and authoring deliver
value without them. Import/export portability is the most valuable of the four
and is what makes a target shareable.

**Independent Test**: Register a set of targets, export them to a file, import
that file into a second empty store, and assert the identifiers and row set are
identical with no duplicates. Separately assert `remove` deletes exactly the
selected target and `scan <dir>` registers the titles found under a directory.

**Acceptance Scenarios**:

1. **Given** a registered target selected by any valid selector, **When** the
   user runs `targets remove <selector>`, **Then** exactly that target is removed
   and others are untouched.
2. **Given** a set of registered targets, **When** the user runs
   `targets export <selector>` (or exports all), **Then** the output is JSON
   conforming to the master target schema.
3. **Given** an exported JSON file, **When** the user runs
   `targets import <file>` into another store, **Then** every target round-trips
   with identical identifiers and no duplicate rows, merging on `id` when a
   target with that identifier already exists.
4. **Given** a directory containing installed titles, **When** the user runs
   `targets scan <dir>`, **Then** the titles found there are registered as
   targets.

---

### Edge Cases

- **Empty everything**: no registered targets and discovery finds nothing. The
  listing prints actionable next commands (satisfying hero criterion 5 in the
  empty case), not an empty table, and exits successfully.
- **Non-interactive invocation of `add`**: when standard input is not a terminal
  (CI, a pipe, a script) the interactive prompts cannot run. Authoring falls back
  to the flag-driven form (name/exe/steam/anchor/handle supplied as arguments),
  and a required-but-missing value is a usage error, never a hung prompt.
- **Row index goes stale**: the user lists, then registers or removes a target,
  then runs `fragcap capture <n>` against the old number. Resolution is against
  the snapshot the listing wrote, so `n` still names the row the user saw; a
  number beyond the snapshot's range is an out-of-range usage error, not a
  silent mis-hit.
- **Import collision**: an imported target whose identifier already exists merges
  into the existing row rather than creating a duplicate; an imported file that
  does not conform to the master schema is rejected with diagnostics, not
  partially applied.
- **`add --steam` for a title that is not installed**: a usage error naming the
  app id, not a fabricated entry.
- **Ambiguous selector on remove/export**: a name matching more than one target
  lists the matches and refuses to act, rather than removing or exporting an
  arbitrary one (P-9).
- **`unsure` answer, then capture**: the first capture promotes the row; a
  capture that never observes a socket-holding process leaves the row unresolved
  rather than inventing one.

## Requirements *(mandatory)*

### Functional Requirements

**Hero listing (Story 1)**

- **FR-001**: `fragcap targets` with no arguments, and bare `fragcap`, MUST run
  discovery across its tiers, register any newly discovered titles into the local
  store idempotently (so each listed row is a registered, capturable target under
  the S054 register-then-capture rule), and present the registered targets as a
  numbered table. Registration is additive and idempotent (an already-registered
  title is not duplicated); no existing target data is modified or removed. A
  title is registered by the same operation every source uses (P-10).
- **FR-002**: Each listing row MUST show a 1-based index, the target handle, a
  CAPTURE status distinguishing a target that is capturable now (`ready`) from
  one that still needs launch information (`needs a target`), and a KNOWN column
  carrying neutral evidence (for example launcher mediation and the resolved
  client image, named anti-cheat or DRM technologies, or that no online mode was
  recorded).
- **FR-003**: The listing MUST order rows deterministically by handle so the same
  store produces the same numbering every time.
- **FR-004**: The listing MUST persist a snapshot of what it displayed to the
  local store, and row-index selectors (`fragcap capture <n>`, `targets show <n>`,
  and any other row-index consumer) MUST resolve against that snapshot so a row
  number names the row the user last saw.
- **FR-005**: A row-index selector referring to a position outside the persisted
  snapshot MUST be an out-of-range usage error, distinct from a clean no-match on
  a handle or name.
- **FR-006**: Every listing MUST end by naming the next command to run; when the
  store is empty and discovery finds nothing, that ending MUST be the concrete
  commands that would populate the store (how to add or scan for a target).
- **FR-007**: The listing MUST complete in a few seconds on a typical machine and
  MUST be non-destructive in the sense that it never modifies or removes existing
  target data: its only writes are the display snapshot and the idempotent
  registration of newly discovered titles (FR-001). A second listing over an
  unchanged environment registers nothing new and leaves the target set
  byte-identical.

**Interactive authoring (Story 2)**

- **FR-008**: `targets add`, when run interactively, MUST let the user point at an
  executable by path or press Enter to browse for one.
- **FR-009**: `targets add` MUST run the detection scan on the chosen executable
  and present the engine, anti-cheat, and DRM evidence inline before prompts that
  depend on it.
- **FR-010**: `targets add` MUST prompt for a display name and a handle, offering
  a derived default handle, and MUST disambiguate a colliding handle rather than
  overwrite an existing target.
- **FR-011**: `targets add` MUST ask whether the shown executable is the process
  that holds the sockets, accepting `Y`, `n`, and `unsure`, and MUST offer the
  `unsure` branch as a first-class answer.
- **FR-012**: The `unsure` answer MUST register the entry with its launch chain
  unresolved and MUST NOT record any fabricated socket-holding process; the tool
  MUST never guess the socket holder and present the guess as fact.
- **FR-013**: A subsequent capture run against an entry authored `unsure` MUST
  observe the real socket-holding process and promote the row to `verified`.
- **FR-014**: `targets add --steam <app_id>` MUST scaffold and register a target
  carrying a `steam:<app_id>` anchor for an installed title, and MUST be a usage
  error naming the app id when the title is not installed. This is the capability
  formerly exposed as `steam profile <app_id>`.
- **FR-015**: When standard input is not a terminal, `targets add` MUST accept the
  flag-driven form (name and executable/steam/anchor/handle as arguments) and MUST
  report a required-but-missing value as a usage error rather than blocking on a
  prompt.

**Lifecycle (Story 3)**

- **FR-016**: `targets scan <dir>` MUST register the titles discovered under the
  given directory.
- **FR-017**: `targets remove <selector>` MUST remove exactly the selected target
  and leave others untouched; an ambiguous selector MUST list the matches and
  refuse to act.
- **FR-018**: `targets export <selector>` MUST emit a single JSON document that is
  a dedicated array of target-entry objects carrying each entry's identity (stable
  id, handle, name, classification, fidelity, anchor, launch entries, install root,
  evidence): a one-element array for a selector, all registered targets when no
  selector is given. This representation is distinct from the published capture
  schema and does not change or version it.
- **FR-019**: `targets import <file>` MUST accept JSON conforming to the master
  target schema, merging each incoming target on its stable identifier so an
  existing identifier updates in place rather than creating a duplicate, and MUST
  reject a non-conforming file with diagnostics without applying it partially.
- **FR-020**: An export followed by an import into another store MUST round-trip
  every target with identical identifiers and no duplicate rows.

**Cross-cutting**

- **FR-021**: Every target that appears in the listing MUST be capturable in
  principle (the CAPTURE column reports how close, never whether the row is
  valid); the KNOWN column MUST remain neutral evidence and MUST NOT be phrased as
  a blocker or an endorsement.
- **FR-022**: New terminology introduced by this slice MUST receive a glossary
  entry in the same change, and any master-specification section this surface
  changes MUST be updated in lock-step (P-6, P-11).

### Key Entities *(include if feature involves data)*

- **Target entry**: a registered capture target. Carries a stable identifier, a
  normalized handle, a display name, a classification (game, launcher, tool, mod,
  emulator, unknown), a fidelity tier (`Authored`, `Verified`,
  `HeuristicUnverified`, `Observed`), an optional anchor (e.g. `steam:<app_id>`),
  launch entries (resolved or unresolved), an optional install root, and
  supporting evidence. Already exists; this slice reads and authors it.
- **Listing snapshot**: the ordered set of targets a listing displayed, persisted
  to the local store so row-index selectors resolve to what the user saw. New
  backing for the existing row-index selector kind.
- **Capture readiness**: a derived, per-target status (`ready` vs
  `needs a target`) shown in the CAPTURE column, computed from whether the entry
  has a resolved launch chain / client image. Presentational; not stored.
- **Evidence summary**: the neutral, human-readable KNOWN column, derived from an
  entry's evidence, launch mediation, and recorded online mode. Presentational.
- **Target export document**: a JSON representation of one or more target entries
  conforming to the master target schema, used by export and import; merges on
  stable identifier.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: On a fresh install with no arguments and no prior configuration, a
  user reaches a listing of their own installed targets and the next command to
  run in a single invocation that completes in a few seconds. *(Hero criterion 1,
  2, 4, 5.)*
- **SC-002**: The listing demonstrates attribution, the core value proposition,
  rather than a peripheral capability: every listed row leads to a capture that
  names the process behind the traffic. *(Hero criterion 3.)*
- **SC-003**: A row number shown in a listing resolves, on the next command, to
  exactly the target that occupied that row, in 100% of cases where the store is
  unchanged between the listing and the command.
- **SC-004**: A target authored with the `unsure` answer contains no fabricated
  socket-holding process, and its first capture promotes it to `verified` once a
  real holder is observed.
- **SC-005**: A set of targets exported and then imported into another store
  yields an identical set of identifiers with zero duplicate rows.
- **SC-006**: The empty case (no targets, discovery finds nothing) ends by naming
  the concrete commands that would populate the store, in 100% of empty
  invocations.
- **SC-007**: The listing and authoring flows write nothing to target data beyond
  the idempotent registration of newly discovered titles and the display snapshot;
  a repeat listing over an unchanged environment registers nothing new and leaves
  the registered target set byte-identical, and no listing ever modifies or removes
  an existing entry.

## Assumptions

- The interactive authoring flow only prompts when standard input is a terminal;
  otherwise it uses the flag-driven `targets add` form. This keeps the command
  testable in CI (which has no terminal) and scriptable, and is why Story 2's
  automated tests drive the flag form and scripted-answer form rather than a live
  prompt.
- "Browse for an executable" is a guided path-entry flow within the CLI (the tool
  helps the user reach an executable path), not a graphical file picker; the
  precise interaction is a plan-phase detail.
- The row-index snapshot is scoped per local store and reflects the most recent
  listing; a new listing replaces it. `--id <stable_id>` remains the durable,
  machine-facing selector unaffected by snapshots.
- `targets export` with no selector exports all registered targets; with a
  selector it exports the one it resolves. The output is a single JSON document.
- Target-entry export/import uses a dedicated target-entry array representation,
  not the published capture schema (`target-schema.v1.json`, S025/S026 lineage);
  this slice neither uses that schema for targets nor changes or versions it. The
  catalog export/import surface (`catalog`, S054) keeps using the published schema
  unchanged.
- Removing `steam profile` from the `steam` command surface is already done in
  S054; this slice provides its replacement (`targets add --steam`) as the sole
  path, and `steam` retains only enumeration and metadata reads.
- The CAPTURE readiness derivation treats an entry with a resolved launch chain
  (or a directly usable client image) as `ready` and one whose launch chain is
  unresolved as `needs a target`; the exact wording of intermediate states is a
  plan-phase detail.

## Dependencies

- **S054 (merged)**: the `capture` verb, the `targets`/`catalog` namespace split,
  and the row-index selector consumed by `capture <n>`.
- **S051**: the target entry model (handles, stable ids, selector resolution,
  fidelity tiers, cascade collapse) and the row-index selector kind (§5.4) this
  slice makes durable.
- **S052**: the TargetSource discovery seam and tiers the listing runs.
- **S053**: the detection signatures the authoring scan reports as evidence.
- **fragcap-steam**: the Steam enumeration and scaffold reused by
  `targets add --steam`.
