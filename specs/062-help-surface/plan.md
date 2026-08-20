# Implementation Plan: Help surface, wrapping, vocabulary, and accuracy

**Branch**: `062-help-surface` | **Date**: 2026-08-20 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/062-help-surface/spec.md`

## Summary

Turn on clap's help wrapper and cap it at 100 columns; strip every internal
identifier, section reference, appendix letter, feature-name phrase, and bare
tier number from the doc comments clap publishes; correct the two help lines
that describe behavior the code does not have and the error that results; and
replace the guard that failed with one that enumerates the command tree from
clap itself.

The order is fixed by dependency: wrapping first, because every later assertion
is made against rendered output; then the guard, written to fail against the
current text; then the scrub and the two corrections, which turn it green.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.82, toolchain pinned in
`rust-toolchain.toml`.

**Primary Dependencies**: `clap =4.5.32`, gaining the `wrap_help` feature. The
pin does not move.

**Storage**: N/A

**Testing**: `crates/fragcap-cli/tests/cli_help.rs` (the guard, rewritten),
`crates/fragcap-cli/tests/cli_targets.rs` (the numeric no-match message),
`xtask/src/lint.rs` (the source-side rule and its unit tests). All drive
`fragcap_cli::run_with` in process; no spawned binary.

**Target Platform**: Windows is the product target; the help surface is
platform-neutral and the guard runs anywhere.

**Project Type**: CLI.

**Performance Goals**: N/A

**Constraints**: `cargo xtask msrv` compiles clap, because clap is non-optional
in `fragcap-cli`. The new transitive package is therefore built under the 1.82
floor, unlike `pcap` behind `live`. `cargo xtask deps` is unaffected: clap lives
in `fragcap-cli` and never in `fragcap-core`.

**Scale/Scope**: 29 help pages. One doc-comment file (`cli.rs`, 757 lines), one
guard test, one lint rule, four error sites, one documentation page, one
manifest, one dependency-inventory table.

## Phase 0: Research (complete)

Three claims were load-bearing enough to verify before planning around them.
All three were measured on this branch, not taken from the issues.

**1. The two-line wrap fix works, and the issue's account of why is right.**
`Cargo.toml` gained `wrap_help`; `cli.rs` gained `max_term_width = 100` on the
root `#[command(...)]`. Measured on `fragcap catalog --help`:

| `COLUMNS` | widest, `wrap_help` only | widest, plus `max_term_width` |
| --- | --- | --- |
| 400 | 253 | 98 |
| 200 | 197 | 98 |
| 100 | 98 | 98 |
| 60 | 60 | 60 |

`wrap_help` alone follows the terminal and is unbounded, so `max_term_width` is
required for the hard limit, exactly as #177 said. Across all pages the count of
lines over 100 columns went from **82 to 0**. Continuation lines land on the
description column with no further work, because `StyledStr::indent` is not
feature-gated.

**2. The `Cargo.lock` delta is exactly one package.** `terminal_size v0.4.4`,
and nothing else moved. It resolves against the `windows-sys 0.61.2` anstream
already brought through clap's default `color` feature, so the `windows-sys
0.36` pin shared by `pcap` and the socket-table backend is untouched and no
second tree appears.

**3. Enumerating pages by scraping help text is not sound.** A first attempt
walked the `Commands:` block of each rendered page. Before the wrap fix it found
29 pages; after the wrap fix the same script reported 38, because wrapped
continuation lines begin with spaces and a lowercase word and were read as
subcommand names. The page set must therefore come from clap's own command tree,
not from its output. This is the single most important design consequence of
Phase 0 and is why the guard is built the way it is below.

Phase 0 also corrected two figures in the filed issues: there are 29 pages, not
27 (#177's table omits `extcap install` and `extcap uninstall`), and `fragcap
extcap --help` leaks `section 14.5`, which appears in no issue inventory. Both
are recorded in the spec's Evidence section because they are the argument for
FR-017.

## Constitution Check

*GATE: passed before Phase 0. Re-checked after design; still passing.*

| Principle | Bearing | Verdict |
| --- | --- | --- |
| P-1 Passive Observation Only | No capture, handle, or driver code touched. The new dependency reads terminal dimensions. | Not engaged |
| P-2 Core Stays Platform-Neutral | clap and `terminal_size` live in `fragcap-cli`. `fragcap-core` is untouched, and `cargo xtask deps` proves it. | Satisfied |
| P-3 Capture And Attribution Stay Separate | No crate-boundary change. | Not engaged |
| P-4 No Silent Loss | FR-021: a page that fails to render fails the guard rather than being skipped. A page silently dropping out of coverage is the documentation form of an uncounted discard. | **Satisfied by design** |
| P-5 Compatibility Outranks Richness | No output format changes. Help is not machine-consumed; `--json` is untouched. | Not engaged |
| P-6 Glossary First | No new term. "Title tier" and "engine tier" rename an existing concept rather than introducing one, and S063 will settle the surface. | Satisfied |
| P-7 Wrappers Stay Thin | `scripts/fragcap.sh` and `Invoke-FragCap.ps1` pass through and do not parse help. Unaffected; the wrapper checks in `cargo xtask ci` confirm. | Satisfied |
| P-8 House Standards Apply | UTF-8 no BOM, LF, no em-dashes or en-dashes across every changed file. | **Gated** |
| P-9 The Instrument Does Not Lie (NON-NEGOTIABLE) | The driver. A help line that describes an invocation that cannot resolve (#181) and a summary that claims a column it never prints (#182) are the instrument misreporting itself. FR-012 extends this to the error path: a resolver that knows why it failed and says only "no target matches" is withholding an observation it made. | **Primary driver** |
| P-10 One Path To A Target | Bears on OOS-001. Adding `capture --steam` would be a fourth target input; deferring it to the targeting work is the P-10-consistent choice, since the question is which paths exist, not how they are described. | Satisfied by deferral |
| P-11 The Specification Describes What Shipped | `docs/fragcap-specification.md:2542` is already correct on `--launch`; the shipped help drifted from it. This slice moves help toward the specification, so no specification edit is needed and the version lock-step is undisturbed. | Satisfied |

No violations. Complexity Tracking omitted.

## Design

### 1. Wrapping (FR-001 to FR-004)

Done in Phase 0 and kept:

- `Cargo.toml:141`: `features = ["derive", "wrap_help"]`, pin unchanged.
- `cli.rs` root `#[command(...)]`: `max_term_width = 100`, which propagates to
  every subcommand, so it is one place.
- A comment at the `help_template` literal recording that the `Commands:` block
  is hand-budgeted and does not wrap (FR-004). Its lines are 76 columns today.
- `AGENTS.md` dependency inventory gains a `terminal_size` row naming S062 and
  the reason, matching the table's existing form.

### 2. The guard (FR-017 to FR-022)

This is the part that must not be built the way it was built before.

**Page enumeration comes from clap, not from text.** `crates/fragcap-cli/src/lib.rs`
currently declares `mod cli;` privately, so a test cannot reach the command
tree. Add one narrow public accessor:

```rust
/// The clap command tree. Public so the help guard can enumerate every page
/// from clap itself; a guard that reads a hand-written list is the defect
/// issue #178 records.
pub fn command() -> clap::Command
```

The guard walks `get_subcommands()` recursively, skipping clap's generated
`help` command, and yields every path. A new subcommand then appears in the
guard's page set the moment it is declared, which is FR-017 and the whole
lesson of #178. The alternative, making `mod cli` public, exports 757 lines of
argument structs as API for one test; a single function is the smaller surface.

**Assertions, applied to every page:**

| Check | Requirement |
| --- | --- |
| Renders, exit 0 | FR-021 |
| No line over 100 columns | FR-019 |
| No leak pattern match | FR-018 |
| No `value_parser`, `value_delimiter`, `Vec<String>` | FR-020 |

**The leak pattern**, as amended by FR-018a:

| Pattern | Catches |
| --- | --- |
| `slice S\d+` | `slice S051` |
| `\bS\d{2,3}\b` | the bare `(S051)` form the old guard missed on a page it covered |
| `[Ss]ection \d+\.\d+` | `section 17.2`, `section 14.5` |
| `Appendix [A-Z]\b` | `Appendix B` |
| `\bP-\d\b` | constitution principle identifiers |
| `` `\w[\w-]*` feature ``, `the \w[\w-]* feature`, `feature "\w[\w-]*"` | `` the `net` feature `` |
| `\bTier \d\b` | `Tier 1`, `Tier 3` |

The feature clause matches the *phrasing*, never the declared names. Four of the
five workspace features (`live`, `net`, `targets`, and `etw` in prose) are
ordinary words; matching `net` bare fires on "network" and `targets` fires on
most of the `targets` pages. FR-018a records why, because the tempting
implementation is the wrong one and a future editor will reach for it.

**The match runs over the whole page with whitespace normalized, never line by
line** (FR-018b). This is not a refinement, it is the difference between a guard
that works and one that does not. With wrapping on, `fragcap extcap --help`
renders `specification section` at the end of one line and `14.5` at the start
of the next; a line-based scan reports that page clean while it still leaks.
The analyze gate caught this by noticing that the leaking-page count fell from
15 to 14 with no text having changed, which is the only symptom it produces. A
guard defeated by the wrap fix shipped in its own slice would be #178 repeating
itself one slice after being fixed.

The workspace has no regex engine in `fragcap-cli`'s dev-dependencies today.
`regex` is already a runtime dependency of `fragcap-profile`, so it is in the
graph and in `Cargo.lock`; adding it as a dev-dependency of `fragcap-cli` adds
no package. If that proves to pull anything, the patterns are simple enough to
hand-match and the fallback is a hand-rolled scanner, consistent with the glob
matcher and the pcapng writer. Decide by measuring the lock, not by preference.

**The source-side rule** (FR-022) goes in `xtask/src/lint.rs`, scoped by path to
`crates/fragcap-cli/src/cli.rs` and to `///` lines only. `run()` already has the
precedent for a path-scoped rule (the `FORBIDDEN_CALLS` scan is gated on
`is_source && ext == "rs" && !shown.starts_with("xtask/")`), so this follows an
established shape rather than inventing one. It is deliberately redundant with
the test: the test proves the rendered output is clean, the lint proves the
source is, and the lint runs in seconds without a build.

Both are needed. The rendered check cannot be satisfied by a source literal that
stops matching; the source check catches a leak in a doc comment that clap does
not currently publish but might after a later refactor.

### 3. The vocabulary scrub (FR-005 to FR-009)

24 doc-comment sites in `cli.rs` carry a leak. For each: remove the provenance
from the `///` text, and where it is useful to a maintainer, restate it as a
`//` comment above the item (FR-008), which clap does not read.

Two are not simple deletions:

- **`Tier 1` / `Tier 3`** become "the title tier" and "the engine tier"
  (FR-007). Per the clarification, the numbering is not defined here, because
  S063 removes these verbs entirely.
- **`` needs the `net` feature ``** on `catalog update` becomes a statement
  about capability, not about a build switch. The subcommand itself stays until
  S063; only the string changes (OOS-003).

Separately, FR-009: insert a blank `///` line after the first sentence of every
entry whose doc comment is a single paragraph, so clap takes a real one-line
summary for `-h` and keeps the rest for `--help`. This is the fix for the
`catalog seed-signatures` and global `--json` cases that render as three-line
table entries.

### 4. The two accuracy corrections (FR-010 to FR-016)

**`--launch`** is reworded to describe the stored target:

> Start the target through its platform launcher, then capture it. Windows
> only. The target must be Steam anchored; register one with
> `targets add --steam <app_id>`.

**Integer namespaces** (FR-011): the row-index rule moves onto `--target` and
the positional `SELECTOR`, where it is read, and stays on `--id`. Today it
appears only on `--id`, phrased as a negation of `--target`.

**The error** (FR-012). `resolve_positional` gates on `is_row_index` first, so a
numeric token never reaches the handle or name path. The message must say so.
Shape:

```
no target matches
  `1333350` was read as a listing row index; the listing has 33 rows.
  If 1333350 is a Steam app id, register the title first:
      fragcap targets add --steam 1333350
  Then capture it by handle or row:
      fragcap targets
```

Four sites emit the message (`target_resolve.rs:117`, `targets.rs:373`, `:416`,
`:838`). They get one shared constructor rather than four copies, so a fifth
site cannot drift. The snapshot row count comes from the store the resolution
already opened; the non-numeric case keeps today's message unchanged.

**`targets list`** (FR-013 to FR-015) is reworded to name the four columns
`render_table` prints and to say the command registers. `--db` stops saying
"read".

**The site page** (FR-016): `site/content/docs/reference/cli.mdx:61` takes the
new `--launch` sentence, resolving its contradiction with `:43`.

## Project Structure

### Documentation (this feature)

```text
specs/062-help-surface/
├── spec.md
├── plan.md                    # this file, carrying Phase 0 research inline
├── tasks.md
└── checklists/requirements.md
```

No separate `research.md`: Phase 0 produced three measured answers and they are
short enough to live in this file, where the design that depends on them can
cite them directly. No `data-model.md` or `contracts/`; nothing persistent and
no interface changes.

### Files changed

```text
Cargo.toml                                      # clap wrap_help
Cargo.lock                                      # +terminal_size, one package
crates/fragcap-cli/src/cli.rs                   # max_term_width, 24 scrub sites, 2 rewordings
crates/fragcap-cli/src/lib.rs                   # pub fn command()
crates/fragcap-cli/src/commands/target_resolve.rs   # the numeric no-match message
crates/fragcap-cli/src/commands/targets.rs      # three further no-match sites
crates/fragcap-cli/Cargo.toml                   # regex dev-dependency, if measured clean
crates/fragcap-cli/tests/cli_help.rs            # the guard, rewritten
crates/fragcap-cli/tests/cli_targets.rs         # the numeric no-match assertion
xtask/src/lint.rs                               # the source-side leak rule
site/content/docs/reference/cli.mdx             # the --launch row
AGENTS.md                                       # dependency inventory row
changelog.d/S062-help-surface.fixed.md
changelog.d/S062-help-surface.decisions.md      # the dependency, and the feature-phrase rule
```

**Structure Decision**: no new module and no new crate. The guard stays in the
existing `cli_help.rs`, which is where #67 put it and where #183 will extend it.

## Verification

`cargo xtask ci` in the foreground, watched to completion, plus `cargo xtask
msrv` separately (FR-024), which `ci` does not run and which this slice can
break because clap is non-optional.

Then, against the rebuilt binary: re-run the page enumeration at `COLUMNS=400`,
`100`, and `60` and confirm 0 lines over the limit and 0 leaking pages, and run
the failing invocation from #181 to read the new error.

The guard is written before the scrub and must be observed failing first. A
guard that has never been red is a guard that has not been shown to work, which
is the #67 lesson stated as a test discipline.
