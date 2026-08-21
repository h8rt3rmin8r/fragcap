# Feature Specification: Help surface, wrapping, vocabulary, and accuracy

**Feature Branch**: `062-help-surface`

**Created**: 2026-08-20

**Status**: Implemented

**Input**: Make `--help` wrap, strip internal development vocabulary from it,
correct two help lines that describe behavior the code does not have, and gate
all of it with a guard that enumerates every page. Closes issues #176, #177,
#178, #181, #182.

## Context

`--help` is the primary documentation surface for a command-line tool. An
operator on v0.5.1 read `fragcap capture --help`, composed an invocation
directly from it, and was refused (#181). The help line was not merely unclear;
it described a command that cannot work.

Five filed issues are five axes of that one surface, and they share a diff and a
guard:

| Issue | Axis |
| --- | --- |
| #177 | No page wraps. Every page overflows, the worst at 449 columns. |
| #176 | Internal development vocabulary reaches user help in at least eleven places. |
| #178 | Internal slice identifiers are back, regressing #67, and the guard test that was supposed to prevent it passes anyway. |
| #181 | `capture --launch` documents an invocation that cannot resolve. |
| #182 | `targets list` help is wrong on both of its claims. |

#178 is the reason all five are one slice rather than five. Issue #67 closed on
2026-08-12 with a guard test in `crates/fragcap-cli/tests/cli_help.rs`, and the
leak came straight back, because that guard matches a hardcoded token list over
three hand-picked pages out of twenty-nine. Scrubbing without fixing the guard
repeats 2026-08-12. Fixing the guard once and hanging every check off it is the
whole economy of the slice.

## Evidence

Measured 2026-08-20 against `target/debug/fragcap.exe` built from `main` at
`e2d655d` (v0.5.1), by enumerating every help page from the binary rather than
by hand-listing. Two figures correct the filed issues, which is itself the
argument for enumeration:

- **There are 29 help pages, not 27.** #177's table lists 27; `extcap install`
  and `extcap uninstall` are absent from it. A hand-listed page set was already
  two pages stale when the issue was filed.
- **`fragcap extcap --help` leaks `section 14.5`**, which appears in neither
  #176's nor #178's inventory.

| Measure | Value |
| --- | --- |
| Help pages | 29 |
| Pages with at least one line over 100 columns | 29 (all of them) |
| Total lines over 100 columns | 82 |
| Widest single line | 449 (`fragcap capture --help`) |
| Pages leaking internal vocabulary | 15 |

Leak forms present in rendered help, all confirmed from the binary:

| Form | Example page |
| --- | --- |
| `slice S0NN` | `targets`, `targets add`, `targets discover`, `targets scan`, `targets remove`, `targets export`, `targets import`, `technologies`, `catalog`, `catalog seed-signatures` |
| `(S0NN)`, bare parenthesized | `capture` (twice) |
| `section N.N` | `capture` (17.2), `extcap` (14.5) |
| `Appendix B` | `catalog`, `catalog seed-signatures` |
| `` `net` feature `` | `catalog`, `catalog update` |
| `Tier 1` / `Tier 3` | `catalog`, `catalog seed`, `catalog seed-engine` |

`COLUMNS=80 fragcap catalog --help` still reports a widest line of 253, which is
the tell: the width is not consulted at all. `Cargo.toml:141` takes clap without
the `wrap_help` feature, and in clap 4.5.32 the wrapper itself is compiled out
when that feature is absent (`styled_str.rs:75` makes `wrap` a no-op body;
`help_template.rs:1070` returns `(None, None)` from `dimensions()`). Setting
`term_width` or `max_term_width` alone changes nothing, because the function
they feed does nothing. The feature is the fix, not a width setting.

The re-indent that keeps continuation lines under the description column is
**not** gated on `wrap_help` (`help_template.rs:651` calls `help.indent`
unconditionally with `trailing_indent`), so turning the wrapper on gets the
column alignment for free.

### Why the #67 guard did not catch the regression

`crates/fragcap-cli/tests/cli_help.rs` fails on two axes at once:

1. It matches a hardcoded token list, `["S15", "S16", "S17", "slice S"]`, which
   was written when those were the current slices. `S051` through `S055` match
   none of those literals except through `"slice S"`, and the bare parenthesized
   form `(S051)` matches nothing at all. `capture --help` **is** a covered page
   and leaks `(S051)` twice today.
2. It covers three pages out of twenty-nine: `capture`, `extcap`, and the root.
   Nine other leaking pages are unguarded.

The same weakness applies to its parser-internals half (`value_parser`,
`value_delimiter`, `Vec<String>`), which is also enforced on only those three.

### The two accuracy defects

**`capture --launch` (`cli.rs:259`)** renders as "Launch the game through its
platform launcher before capturing, then capture it (Windows only; requires a
`--target` carrying a Steam app id)". That has two readings. The plain one, and
the one a first-time operator takes, is "pass the Steam app id to `--target`" -
reinforced by `fragcap steam list` printing app ids in the immediately preceding
command. The one the code means is "the target that `--target` resolves must
itself carry a Steam anchor", which is a statement about a stored row phrased as
a statement about the flag's argument.

Reading one cannot work. `resolve_positional`
(`crates/fragcap-targets/src/selector.rs`) gates on `is_row_index` first, and
that predicate is `!token.is_empty() && token.bytes().all(|b| b.is_ascii_digit())`,
so a bare integer is **unconditionally** the row-index path and never falls
through to a handle or name lookup. The operator's `1333350` was read as
"listing row 1333350" against a 33-row listing.

Three integer namespaces exist on this surface and two of them collide on
`--target`: the listing row index, the durable stable id (`--id`), and the Steam
app id (`targets add --steam`). The collision is documented in exactly one
place, `--id`'s help, phrased as a negation of `--target`, where a reader of
`--target` will never find it.

The error that results names no cause. `target_resolve.rs:117` emits "no target
matches; list targets with `fragcap targets`", and `targets.rs:373`, `:416`, and
`:838` emit a bare "no target matches". The resolver knows the token was
numeric, knows it therefore took the row-index path, and knows how many rows the
snapshot holds; it reports none of it.

This is propagated to the published docs.
`site/content/docs/reference/cli.mdx:61` carries the `--launch` sentence
verbatim, and `:43` gets `--target` right ("A bare number given here is a row
index"), so two rows of one table contradict each other. The master
specification does not have the bug: `docs/fragcap-specification.md:2542` reads
"Start the title through its platform (--target only)". The ambiguity was
introduced when the shipped help expanded on the spec's line.

**`targets list` (`cli.rs:358`)** renders as "List registered targets with their
row index, handle, and identifier". Both halves are wrong. The rendered table
has four columns (`#`, `TARGET`, `CAPTURE`, `KNOWN`) and none is an identifier;
the stable id is available only from `targets show` or `targets export`. And the
command is not a listing: `TargetsCommand::List` dispatches to `hero_listing`
(`targets.rs:47` to `:137`), which runs discovery, **registers newly discovered
targets into the store**, and rewrites the row-index snapshot on every
invocation. On a fresh store it registered 31 rows under a verb whose help says
"List", while the `--db` flag's own help says "The store file (local.db) to
**read**". The behavior is deliberate and documented in a source comment, so
this is a help defect, not a behavior defect. `targets discover` is careful
about exactly this distinction ("Reads only; a candidate becomes a stored target
when acted on"); the command that actually writes is the one that says it reads.

## Clarifications

### Session 2026-08-20

Resolved under the autopilot decision policy: alternatives enumerated, evaluated
against the constitution, the master specification, the originating issues, and
existing code patterns; best-supported option taken and recorded.

- Q: #178 leaves open whether published specification section references
  (`section 17.2`, `section 14.5`) should be stripped alongside slice ids, since
  the specification, unlike `specs/`, is published. -> A: **Strip them.** A
  section number is actionable only to a reader who has the specification open,
  which is not the reader of a terminal help page, and #178 itself calls them
  "equally noise in a terminal". Keeping one class of provenance while stripping
  another also makes the guard rule a list of exceptions rather than a rule.
  Where the reference is genuinely useful to a maintainer it moves to a `//`
  comment above the item, which clap does not read, preserving it in the source
  where a maintainer looks.
- Q: #176 asks whether `Tier 1` and `Tier 3` should be named or defined, noting
  they are a real user-visible concept whose numbering exists only in the
  specification. -> A: **Name them ("the title tier", "the engine tier") and do
  not define the numbering.** Naming removes the leak without inventing a
  glossary entry for a numbering scheme that slice S063 is about to remove
  entirely when it collapses the three seed verbs into one `--tier` flag.
  Defining the numbers here would be work undone one slice later.
- Q: #181 point 5 asks whether `capture --steam <app_id>` should exist as a
  fourth target input, so the number `steam list` prints is directly usable. ->
  A: **Out of scope, recorded.** #181 states it is "not required to close this
  issue". It is a new target input, which is a surface addition with a P-10
  bearing (one path to a target) and belongs with the targeting work, not with a
  help-text correction. What this slice owes is that no help line invites the
  wrong reading and that the error names the right route.
- Q: #182 point 4 asks whether a purely read-only listing should exist, given
  that `targets list` writes. -> A: **Out of scope, recorded.** It is a behavior
  question about the hero path, and #182's own acceptance asks only that the
  help match whatever is chosen. This slice makes the help match the behavior
  that exists.
- Q: The `` `net` feature `` string appears in `catalog` and `catalog update`
  help, and #175 owns the structural fix (compiling the subcommand out). -> A:
  **Scrub the string here; leave the `cfg` gating and the product decision to
  S063.** The string is a doc-comment leak and is #176's inventory item; the
  dead subcommand is #175's. Splitting on that line keeps each issue closing in
  exactly one slice, and the guard rule this slice adds is what will keep S063
  from reintroducing a feature name.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Help fits the terminal (Priority: P1)

An operator runs `fragcap catalog --help` in a normal console. Today one entry
is a single 253-column line; the console either wraps it at the raw terminal
edge with no regard for the description column, so continuation text starts at
column 0 and the two columns collide, or runs it off the right edge entirely.

**Why this priority**: it is the precondition for the other four. A page that
does not wrap cannot be read, so auditing its wording is premature, and #183
(the accuracy audit) is explicitly sequenced after it.

**Independent Test**: render every page under a fixed `COLUMNS` and measure the
widest line.

**Acceptance Scenarios**:

1. **Given** any of the 29 help pages, **When** rendered at `COLUMNS=100`,
   **Then** no line exceeds 100 columns.
2. **Given** a 400-column terminal, **When** any page is rendered, **Then** no
   line exceeds 100 columns, because the cap is a hard limit and not a
   terminal-width follow.
3. **Given** a 60-column terminal, **When** any page is rendered, **Then** the
   output shrinks to that width rather than staying at 100.
4. **Given** a wrapped multi-line description, **When** it is rendered, **Then**
   its continuation lines align under the description column, not under the
   command name.

---

### User Story 2 - Help speaks the user's vocabulary (Priority: P1)

A user holding a released binary runs `fragcap targets --help` and reads "slice
S051", "Appendix B", "Tier 1", and "the `net` feature". Every one of those is
provenance that means something to this repository and nothing to them. Six of
eight `targets` subcommands cite a slice number.

**Why this priority**: equal to US1. A user reading "slice S051" learns that
they are reading someone else's internal notes, which is a trust cost out of
proportion to the fix.

**Independent Test**: render every page and match against the leak pattern.

**Acceptance Scenarios**:

1. **Given** any of the 29 help pages, **When** rendered, **Then** it contains
   no slice identifier in any form, no specification section reference, no
   appendix letter, no Cargo feature name, and no bare tier number.
2. **Given** a maintainer reading `cli.rs`, **When** they look for the
   provenance that was removed, **Then** they find it in a `//` comment above
   the item, which clap does not publish.
3. **Given** any option or subcommand, **When** its short help is rendered with
   `-h`, **Then** the summary is one line and the paragraph is behind `--help`.

---

### User Story 3 - Help does not teach a command that fails (Priority: P1)

An operator reads `fragcap capture --help`, composes
`fragcap capture --launch --target <steam app id> ...`, and is refused with a
message that names no cause.

**Why this priority**: this is the concrete failure that motivated the whole
set. A false help line is worse than a missing one, because the reader acts on
it.

**Independent Test**: read `--launch` and `--target` help and try to derive the
failing invocation from them; then run the failing invocation and read the
error.

**Acceptance Scenarios**:

1. **Given** `fragcap capture --help`, **When** an operator reads the `--launch`
   entry, **Then** it describes a property of the stored target and contains no
   sentence that can be read as "pass a Steam app id to `--target`".
2. **Given** `fragcap capture --help`, **When** an operator reads `--target` or
   the positional selector, **Then** it says that a bare integer is a listing
   row index.
3. **Given** a numeric `--target` token matching no row, **When** resolution
   fails, **Then** the error names the row-index interpretation it used, the
   number of rows in the listing snapshot, and the `targets add --steam` route
   to the other namespace.
4. **Given** `site/content/docs/reference/cli.mdx`, **When** its `--launch` and
   `--target` rows are read together, **Then** they agree.

---

### User Story 4 - Help describes what the command does (Priority: P2)

A user reads `fragcap targets list --help`, learns it will list registered
targets with their identifier, runs it, and gets four columns with no identifier
among them and a line saying 31 targets were registered.

**Why this priority**: real and misleading, but the surprise is a superset of
what was promised rather than a failed command, so it ranks below US3.

**Independent Test**: compare the `list` help text against the columns
`render_table` prints and against what `hero_listing` writes.

**Acceptance Scenarios**:

1. **Given** `fragcap targets list --help`, **When** it is read, **Then** it
   names the columns the command prints and no column it does not.
2. **Given** the same page, **When** it is read, **Then** it states that the
   command registers newly discovered titles into the store.
3. **Given** the `--db` flag on `list`, **When** its help is read, **Then** it
   does not describe the store as read-only.
4. **Given** a reader looking for the durable identifier `--id` consumes,
   **When** they read `list` help, **Then** they are pointed at `targets show`
   or `targets export`.

---

### User Story 5 - The guard cannot be outrun by a new subcommand (Priority: P1)

A future slice adds a subcommand whose doc comment cites its own slice number,
exactly as six existing ones do. Today the guard would not notice, because the
new page is not one of the three it checks.

**Why this priority**: without it the other four stories are undone by the next
slice, which is not hypothetical: it already happened once between #67 and #178.

**Independent Test**: add a subcommand with a deliberate leak and confirm the
guard fails without being edited.

**Acceptance Scenarios**:

1. **Given** the guard test, **When** it runs, **Then** it discovers the page
   set from the binary rather than from a hand-written list.
2. **Given** a new subcommand carrying a slice identifier in its doc comment,
   **When** the guard runs, **Then** it fails, with no edit to the guard.
3. **Given** a doc comment in `cli.rs` carrying a leak, **When** `cargo xtask
   lint` runs, **Then** it fails, so the cheap check catches it as well as the
   rendered one.

---

### Edge Cases

- **The top-level `Commands:` block will not wrap.** It is a literal inside the
  custom `help_template` in `cli.rs`, not clap-rendered rows, so `wrap_help`
  does not reach it. Its lines are 76 columns today, inside the limit, but a
  future entry with a long description would overflow again. It must carry a
  comment saying the block is hand-budgeted, and the width guard must still
  cover the root page so an overflow there fails.
- **What if a leak pattern matches legitimate user-facing text?** `Tier 1` is
  the live case: it is a real concept, not only provenance. Resolved in
  clarification by renaming rather than by exempting the pattern. A future
  genuine collision must be resolved the same way, by changing the text, not by
  adding an exception to the rule, or the rule decays into a list.
- **What if a page cannot be rendered?** A subcommand whose `--help` exits
  non-zero must fail the guard rather than be silently skipped, or a page could
  drop out of coverage by breaking.
- **`cargo xtask msrv` compiles clap.** clap is a non-optional dependency of
  `fragcap-cli`, so the new transitive `terminal_size` is compiled under the
  1.82 floor, unlike `pcap` behind `live`. It declares `rust-version = "1.71"`
  and clears the floor, but this must be verified rather than assumed.

## Requirements *(mandatory)*

### Functional Requirements

**Wrapping (#177)**

- **FR-001**: `--help` output MUST wrap. No line of any help page MUST exceed
  100 columns at any terminal width.
- **FR-002**: At a terminal narrower than 100 columns, wrapped description text
  MUST shrink to the terminal width rather than holding at 100.

  **Two categories cannot shrink, and both are clap structure rather than
  fragcap text.** Measured at `COLUMNS=60`: the root page's `Commands:` block is
  a literal inside the custom `help_template`, so clap never wraps it (this is
  FR-004, and the block is hand-budgeted to 76 columns); and a `Usage:` line
  containing a required-one-of argument group renders as one unbreakable token
  sequence, 95 columns on `capture`. Neither is reachable without either
  abandoning the grouped command table or hand-wrapping usage lines clap
  regenerates. The 100-column limit that #177 actually asks for holds at zero
  violations on every page at every width at or above 100, which is what the
  guard asserts.
- **FR-003**: Continuation lines of a wrapped description MUST align under the
  description column, not under the item name.
- **FR-004**: The hand-budgeted literal `Commands:` block in the root
  `help_template` MUST carry a source comment stating that it does not wrap and
  is budgeted by hand.

**Vocabulary (#176, #178)**

- **FR-005**: No rendered help page MUST contain a slice identifier, in the
  `slice S0NN` form or the bare parenthesized `(S0NN)` form.
- **FR-006**: No rendered help page MUST contain a specification section
  reference, an appendix letter, a constitution principle identifier, or a Cargo
  feature name.
- **FR-007**: `Tier 1` and `Tier 3` MUST be replaced by names ("the title tier",
  "the engine tier"). No bare tier number MUST remain in rendered help.
- **FR-008**: Provenance removed from a doc comment that is genuinely useful to
  a maintainer MUST be preserved as a `//` comment above the item, which clap
  does not read.
- **FR-009**: Every option and subcommand MUST have a one-line summary that clap
  renders for `-h`, with any paragraph behind `--help`. This is achieved by a
  blank doc-comment line after the first sentence and requires no API change.

**Accuracy (#181, #182)**

- **FR-010**: The `--launch` help MUST describe a property of the stored target,
  and MUST contain no sentence that can be read as an instruction to pass a
  Steam app id to `--target`.
- **FR-011**: The `--target` flag and the positional selector MUST each state
  that a bare integer is **unconditionally** a listing row index, never a
  handle, a name, or a platform app id. The statement MUST remain on `--id` as
  well.

  Measured correction to this slice's own first reading: both already say "an
  exact handle, a case-insensitive exact name, or a 1-based row index over the
  current listing". That is not the missing fact. Phrased as a list of accepted
  forms it invites the reading that a number might be tried as other things too,
  which is exactly the inference the operator in #181 made. `is_row_index` gates
  first and never falls through, so the fact that has to be on the page is the
  exclusivity, not the membership.
- **FR-012**: A numeric selector token that matches no row MUST produce an error
  naming the row-index interpretation that was used, the number of rows in the
  listing snapshot, and the `targets add --steam` route to the Steam app id
  namespace. This applies at every site that emits the no-match message:
  `target_resolve.rs:117` and `targets.rs:373`, `:416`, `:838`.
- **FR-013**: The `targets list` summary MUST name the columns the command
  prints, MUST NOT name a column it does not print, and MUST state that the
  command registers newly discovered titles into the store.
- **FR-014**: The `list --db` help MUST NOT describe the store as read-only.
- **FR-015**: `targets list` help MUST point at `targets show` or `targets
  export` for the durable identifier.
- **FR-016**: `site/content/docs/reference/cli.mdx` MUST agree with the shipped
  `--launch` help, resolving its internal contradiction with the `--target` row.

**The guard (#178, and the reason the set is one slice)**

- **FR-017**: The guard in `crates/fragcap-cli/tests/cli_help.rs` MUST discover
  the set of help pages by walking the binary's own command tree, not from a
  hand-written list, so a new subcommand inherits every check.
- **FR-018**: The guard MUST match leaks by pattern, not by literal token list.
  The pattern MUST cover `slice S\d+`, a bare `S\d{2,3}` at a word boundary,
  `[Ss]ection \d+\.\d+`, `Appendix [A-Z]`, `P-\d`, and a phrase naming a Cargo
  feature.
- **FR-018a**: The Cargo-feature clause of FR-018 MUST match the *phrasing*
  that names a build feature to a user (the forms `` `X` feature ``, `feature
  X`, `the X feature`), NOT the set of feature names declared in the workspace.
  The declared names are `live`, `socket-table`, `etw`, `net`, and `targets`,
  and four of those five are ordinary English words that appear legitimately in
  help prose: matching `net` bare would fire on "network", and `targets` and
  `live` would fire on most of the `targets` and `capture` pages. A rule that
  cries wolf is a rule that gets an exception list, and an exception list is
  what #178 shows decaying into a hardcoded token set.
- **FR-018c**: The leak pattern MUST match a constitution principle identifier
  with **more than one digit**. The constitution has eleven principles, so a
  `P-\d` pattern silently exempts P-10 and P-11. Review of PR #189 caught this
  in both gates.
- **FR-018d**: The source-side `cargo xtask lint` rule MUST match the same
  pattern set as the rendered guard, not a subset. Review of PR #189 found it
  matching neither principle identifiers nor the unquoted `the <word> feature`
  phrasing, so the cheap gate advertised parity it did not have and a leak of
  either form reached only the expensive gate.

- **FR-018b**: The guard MUST match the leak pattern against the whole rendered
  page with whitespace normalized to single spaces, NOT line by line. Measured:
  once FR-001 wrapping is in place, `fragcap extcap --help` renders
  `specification section` at the end of one line and `14.5` at the start of the
  next, so a line-based scan finds nothing and reports the page clean while it
  still leaks. A guard defeated by the wrap fix introduced in its own slice is
  the #178 failure mode in new dress, and it is the one failure the guard exists
  to make impossible.
- **FR-019**: The guard MUST assert the 100-column limit on every page.
- **FR-020**: The guard MUST apply the existing parser-internals checks
  (`value_parser`, `value_delimiter`, `Vec<String>`) to every page.
- **FR-021**: A help page that fails to render MUST fail the guard, not be
  skipped.
- **FR-022**: `cargo xtask lint` MUST fail on a doc comment in
  `crates/fragcap-cli/src/cli.rs` matching the leak pattern, so the source-level
  defect is caught by the cheap gate as well as by the rendered one.

**Dependency**

- **FR-023**: The clap `wrap_help` feature MUST be added without changing the
  exact `=4.5.32` pin, and the `Cargo.lock` delta MUST be exactly one package,
  `terminal_size`. The workspace dependency inventory in `AGENTS.md` MUST record
  it with the slice that added it, matching the existing table's form.
- **FR-024**: This slice MUST NOT regress the minimum-supported-toolchain gate,
  verified by running it, since clap is non-optional in `fragcap-cli` and the
  new package is therefore compiled under the 1.82 floor.

  **Verified, and a correction to an earlier reading in this same slice.**
  `terminal_size 0.4.4` declares `rust-version = "1.71"` and clears the floor,
  and the `Cargo.lock` delta is exactly that one package plus a `regex`
  dev-dependency edge that adds none. The authoritative gate, the `minimum
  supported toolchain` job in `.github/workflows/ci.yml`, is **green on this
  branch and green on `main`**, both confirmed by reading the job result rather
  than the workflow conclusion.

  `cargo xtask msrv` fails on the developer machine this slice was written on,
  at `constant_time_eq 0.4.2` (`edition = "2024"`, `rust-version = "1.85.0"`,
  reached through `blake3` from S051). That failure was first read as a
  pre-existing repository defect. It is not one: the same job runs the same
  `cargo build --workspace --locked` under the same 1.82 toolchain on the runner
  and compiles that exact package successfully, on `main` and here. The
  divergence is local and its cause is not established, so it is recorded as an
  environment observation (OOS-006) rather than asserted as a defect.

### Out of scope

- **OOS-001**: `capture --steam <app_id>` as a fourth target input (#181 point
  5). Recorded in clarification; belongs with the targeting work.
- **OOS-002**: Whether a purely read-only `targets list` should exist (#182
  point 4). Behavior question; this slice makes the help match the behavior that
  exists.
- **OOS-003**: Compiling `catalog update` out under `#[cfg(feature = "net")]`,
  and the product decision about how a user refreshes the catalog. Both belong
  to #175 and slice S063. This slice scrubs the string only.
- **OOS-004**: The accuracy audit of #183 (findings 3 through 15: undocumented
  `--sink` forms, unstated defaults, unmarked unimplemented surface, option
  grouping, examples, spec reconciliation). #183 is explicitly sequenced after
  this slice so its audit runs over text that already wraps and has been
  scrubbed.
- **OOS-005**: The nine required store-path flags (#179). Belongs to S063.
- **OOS-006**: A local `cargo xtask msrv` failure, observed and not reproduced
  on the runner. On the Windows developer machine this slice was written on,
  `rustup run 1.82 cargo build --workspace --locked` fails parsing
  `constant_time_eq 0.4.2`, which declares `edition = "2024"`. The `minimum
  supported toolchain` job runs the same command under the same toolchain and
  compiles that package successfully, on `main` and on this branch. Both were
  checked at the job level, not the workflow level. Not filed as a defect,
  because the evidence does not support one; recorded here so the next person to
  hit it locally has the comparison already done.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: At `COLUMNS=100`, the count of help lines exceeding 100 columns
  falls from 82 to 0, across all 29 pages.
- **SC-002**: The count of help pages containing internal vocabulary falls from
  15 to 0.
- **SC-003**: At `COLUMNS=400` and `COLUMNS=100`, zero help lines exceed 100
  columns. At `COLUMNS=60`, wrapped description text shrinks to 60, and the only
  lines above it are the two structural categories named in FR-002.
- **SC-004**: A deliberately leaking doc comment added to `cli.rs` fails both
  the guard test and `cargo xtask lint`, with no edit to either.
- **SC-005**: `fragcap capture --help` contains no sentence readable as "pass a
  Steam app id to `--target`", and both `--target` and the positional selector
  state the row-index rule.
- **SC-006**: A numeric `--target` with no matching row produces an error naming
  the interpretation and the snapshot size, asserted by a test.
- **SC-007**: `fragcap targets list --help` names exactly the columns printed
  and states that the command writes.
- **SC-008**: The `Cargo.lock` delta is exactly one added package.
- **SC-009**: `cargo xtask ci` is green, and the `minimum supported toolchain`
  job is green on this branch, confirmed from the job result.

## Assumptions

- The page set is discovered from the binary, so "29 pages" is a measurement at
  this commit and not a constant. The guard must not hardcode the number.
- `clap` stays exactly pinned at `=4.5.32`. The MSRV reasoning recorded at
  `Cargo.toml:134` is about clap 4.6 and later 4.5 patches and is unaffected by
  a feature flag.
- `terminal_size` resolves against the `windows-sys 0.61.2` already in
  `Cargo.lock` through anstream, leaving the `windows-sys 0.36` pin shared by
  `pcap` and the socket-table backend untouched. To be verified, not assumed.
- The behavior of `targets list` (that it registers and rewrites the snapshot)
  is deliberate and stays. Only its description changes.
