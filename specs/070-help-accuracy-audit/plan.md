# Implementation Plan: Help accuracy audit and gate

**Branch**: `070-help-accuracy-audit` | **Date**: 2026-08-22 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/070-help-accuracy-audit/spec.md`

## Summary

Close out issue #183, the twelve findings (4 through 15) S062 deliberately left
for this slice: fix five real remaining accuracy and concision defects
(`--sink`'s undocumented schemes and modifiers, four undocumented defaults, a
specification/grammar disagreement, `steam list`'s missing route to
registration, the repeated `--json` paragraph, the interleaved global flags,
missing worked examples), close two findings with no code change and a
recorded reason (the two already fixed by S062, re-verified rather than
re-fixed; `--extcap-version`'s text, judged accurate as written), and add the
four gate checks #183 itself specifies, none of which exist in
`crates/fragcap-cli/tests/cli_help.rs` today.

The order is fixed by the same dependency S062 established: write each gate
check first and observe it fail against the current text, then fix the text
until each check passes. A gate that has never been red has not been shown to
catch anything, which is the standing lesson from #67/#178 this project keeps
relearning (see `fragcap-recurring-self-inflicted-bugs`).

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.82, toolchain pinned in
`rust-toolchain.toml`.

**Primary Dependencies**: None added. `clap =4.5.32` (with `wrap_help`, from
S062) and `regex` (already a dev-dependency of `fragcap-cli` since S062) are
sufficient for every new gate check.

**Storage**: N/A

**Testing**: `crates/fragcap-cli/tests/cli_help.rs`, extended with four new
tests. `xtask/src/lint.rs` is unaffected (its rule already covers the same
leak patterns S062 built; this slice adds no new leak pattern). All drive
`fragcap_cli::command()` and `fragcap_cli::run_with` in process, following the
existing file's own pattern; no spawned binary, no new test file.

**Target Platform**: Windows is the product target; the help surface is
platform-neutral and every new check runs anywhere, matching the existing
guard.

**Project Type**: CLI.

**Performance Goals**: N/A

**Constraints**: No dependency or MSRV change. `cargo xtask deps` is
unaffected: nothing here touches `fragcap-core`.

**Scale/Scope**: One doc-comment file (`crates/fragcap-cli/src/cli.rs`), one
value-grammar file (`crates/fragcap-cli/src/args.rs`, prose only, no parser
change), one specification page
(`docs/fragcap-specification.md` section 17.2), one guard test file gaining
four tests, one changelog fragment. 29 help pages (unchanged from S062;
re-verified, not re-measured, since no subcommand was added or removed since).

## Phase 0: Research (complete)

Three claims were load-bearing enough to verify before planning around them,
each measured against `target/debug/fragcap.exe` built from `5a3862c`, not
taken from the issue's 2026-08-20 inventory.

**1. The issue's own finding 4 undercounts the `--sink` gap.** Reading
`crates/fragcap-cli/src/args.rs` directly (not the help text) shows
`parse_destination` accepting seven schemes (`file`, `pcapng`, `jsonl`, `pipe`,
`fifo`, `unix`, `tcp://`) and `apply_option` accepting six modifier keys
(`format`, `payload`, `rotate-size`, `rotate-duration`, `queue`, `timeout`).
The issue's own inventory names four schemes and mentions three modifiers by
example (`format=`, `rotate-size=`, `rotate-time=`, the last of which does not
even match the real key, `rotate-duration`). FR-002 is written against the
source, not the issue text, for this reason.

**2. Only one of four defaulted options prints its default, and the mechanism
differs between them.** `--scope` uses `default_value_t`, a clap feature that
renders `[default: ...]` automatically; S062 added it for an unrelated
reason (S062 plan does not mention it directly, but the field predates this
slice). `--direction` and `--roles` resolve their default inside `assemble.rs`
(`args.direction.unwrap_or(Direction::Both)` at two call sites;
`.or_else(|| defaults.roles().map(...))` reading a `Profile` default object
that is always absent on the CLI-only path, so the practical default is
whatever `Profile::default()` supplies, traced to be the specification's
"all"). `--mode` resolves through `profile.capture().mode().unwrap_or(CaptureMode::File)`,
reached only when `args.mode` is `None`, which on the shipped CLI (`CaptureArgs`
carries no `--profile` flag) is every invocation that omits `--mode`. `--wait`
resolves to no default at all: `acquisition_timeout: args.wait` passes `None`
straight through, meaning "wait indefinitely," which is itself a fact `--wait`'s
help should state rather than merely defaulting to printing nothing.

This means FR-003's four fixes are not uniform: `--mode` and `--direction`
gain a literal `[default: ...]` (via `default_value_t`, matching `--scope`'s
existing pattern, since both resolve to one fixed value on every reachable
code path in the shipped binary); `--roles` gains a stated default matching
"all" (in prose, since `ValueEnum` and `default_value_t` do not apply to a
`Vec<String>` built from a comma-delimited value); `--wait` gains a stated "no
timeout; wait until the target starts or the process is interrupted" (in
prose, since `None` here is a real behavior, not an unstated one).

**3. The three checklist-flagged edges (CHK007, CHK015, CHK020) each resolve
to "no special case needed," verified by reading the actual code:**

- **CHK007** (a conceptual cross-reference misfiring the FR-013 check):
  `` `the title tier` `` and similar phrases never match either candidate shape
  (a bare lowercase-hyphenated token equal to a real subcommand name, or a
  `--`/`-`-prefixed token). The check only evaluates backticked text matching
  one of those two shapes; a multi-word phrase inside backticks is never a
  candidate. No exemption list is needed because the shape rule itself
  excludes it.
- **CHK015** (FR-011's one-line check against an item with no doc comment):
  every `#[arg(...)]` and `#[derive(Subcommand)]` variant in `cli.rs` today
  carries a doc comment (verified by reading the file in full during
  specification); an item with none would render no summary line at all under
  clap's derive, which is a missing-documentation defect the existing
  `no_help_page_leaks_internal_vocabulary`-style enumeration would not be the
  right test for. FR-011 is written as "the rendered summary, when present, is
  one line," which is satisfied vacuously by an absent summary and is not
  weakened by this; the codebase's actual state makes the case moot rather
  than needing a carve-out.
- **CHK020** (FR-007 contiguity versus the flattened `OfflineArgs`):
  `OfflineArgs`' five fields are all `#[arg(long, hide = true)]`, so clap
  excludes every one from any rendered `--help`, verified directly against the
  current `capture --help` output (zero offline-flag lines appear). FR-007's
  "no other option's help text between the first and the last" is checked only
  over the rendered page, which structurally cannot contain a hidden field, so
  reordering `CaptureArgs`' visible fields cannot be read as violating or
  satisfying the rule against invisible ones.

## Constitution Check

*GATE: passed before Phase 0. Re-checked after design; still passing.*

| Principle | Bearing | Verdict |
| --- | --- | --- |
| P-1 Passive Observation Only | No capture, handle, or driver code touched. | Not engaged |
| P-2 Core Stays Platform-Neutral | Every change is in `fragcap-cli` (doc comments, tests) or `docs/`. `fragcap-core` is untouched. | Satisfied |
| P-3 Capture And Attribution Stay Separate | No crate-boundary change. | Not engaged |
| P-4 No Silent Loss | FR-015: each of the four new gate checks must be demonstrated failing before it is demonstrated passing, so a check that silently never fires cannot ship as if it were verified. | **Satisfied by design** |
| P-5 Compatibility Outranks Richness | No output format changes; `--json`'s machine-readable behavior is untouched, only its `-h`/`--help` prose splits. | Satisfied |
| P-6 Glossary First | No new term introduced. | Not engaged |
| P-7 Wrappers Stay Thin | `scripts/fragcap.sh` and `Invoke-FragCap.ps1` do not parse help text; unaffected. | Satisfied |
| P-8 House Standards Apply | UTF-8 no BOM, LF, no em-dashes or en-dashes across every changed file, including this plan. | **Gated** |
| P-9 The Instrument Does Not Lie (NON-NEGOTIABLE) | The driver. `--sink` claiming four schemes when seven exist, four options never stating a default they have, and a specification claiming short flags the binary refuses are all the instrument's own description of itself being wrong. | **Primary driver** |
| P-10 One Path To A Target | Not engaged. No new target input is added; FR-005 only makes an existing route (`targets add --steam`) discoverable from `steam list`'s own surface. | Not engaged |
| P-11 The Specification Describes What Shipped | The driver for FR-004. Section 17.2 currently describes a grammar (`-m`, `-q`) the shipped binary refuses, which is exactly the drift this principle exists to prevent; the clarified fix moves the specification to match the shipped grammar rather than the reverse. | **Primary driver** |

No violations. Complexity Tracking omitted.

## Design

### 1. `--sink` accuracy (FR-002)

`cli.rs`'s `sink` field doc comment is rewritten to name all seven schemes and
all six modifiers, grouped for readability (destination schemes, then
modifiers), for example:

> An output sink, repeatable. `file:PATH`, `pcapng:PATH` (an alias for `file:`),
> `jsonl:PATH`, `pipe:NAME`, `fifo:PATH`, `unix:PATH`, or `tcp://HOST:PORT`.
> Modifiers append as `,key=value`: `format=pcapng|jsonl`, `payload=true|false`,
> `rotate-size=SIZE`, `rotate-duration=DURATION`, `queue=N`, `timeout=DURATION`.

The exact wording is an implementation-time judgment call (fitting the
100-column wrap limit may force a shorter phrasing or a `--help`-only
expansion via the blank-line short/long split); the requirement is that every
name in `parse_destination`'s `match` and every key in `apply_option`'s `match`
appears somewhere in the rendered text, which FR-013's cross-reference-style
enumeration (see below) can assert mechanically by reading those two match
arms' literals at test time via a small local list mirrored from the source,
OR, more robustly, by a doctest-style assertion that greps `args.rs` itself.
Chosen approach, decided now rather than left to task time: **the test reads
the scheme and modifier literals directly out of the two error messages
`parse_destination`/`apply_option` already construct** ("expected one of
file:, pcapng:, jsonl:, pipe:, fifo:, unix:, tcp://" and the four modifier
match arms), rather than hand-copying the list into the test, so the test and
the parser cannot drift independently. This is the same "assert structure, not
a hand-maintained list" discipline FR-012/FR-013/FR-014 apply to the gate
itself, applied here to a single finding's own regression test.

### 2. Stated defaults (FR-003)

- `--mode`: **planned as `default_value_t = ModeArg::File`, reverted during
  verification.** `cargo xtask ci` failed a pre-existing test,
  `a_profile_declared_ring_mode_is_resolved_and_validated`: `resolve_mode`'s
  `None` arm falls back to a profile-declared `[capture] mode`, a real,
  tested behavior this plan's Phase 0 research missed by reasoning about
  "every reachable path in the shipped CLI" instead of running the existing
  test suite before deciding. Collapsing `Option<ModeArg>` to `ModeArg` erases
  the "user passed nothing" state that fallback needs. `mode` stays
  `Option<ModeArg>`, `resolve_mode` keeps its `None` arm unchanged, and the
  default is stated in prose: "Defaults to a profile-declared mode if one
  exists, else `file`."
- `--direction`: add `default_value_t = Direction::Both`, mirroring the same
  pattern; `assemble.rs`'s two `unwrap_or(Direction::Both)` sites become
  unreachable defaults and can be simplified the same way.
- `--roles`: cannot take `default_value_t` (the field is `Option<Vec<String>>`
  built from a delimited value, not a `ValueEnum`). State the default in prose
  instead, in the existing doc-comment's second sentence: "Defaults to every
  role" (verifying "every role" is the correct wording against
  `Profile::default()`'s roles value during task execution, not assumed here).
- `--wait`: state in prose that omitting it waits with no timeout. No
  `[default: ...]` form applies because `None` here is not a stand-in for a
  fixed value, it is the behavior.
- The new FR-012 gate check (below) enumerates its subject set from
  `assemble.rs`'s own default-resolution call sites (a small fixed list
  reflecting the four sites named above) rather than walking every `Option`
  field on `CaptureArgs`, because not every optional flag has a default in the
  sense FR-003 means (for example `--out` has no default; it is simply
  optional). The set is recorded as a named constant in the test file with a
  comment pointing at the four `assemble.rs` sites it mirrors, so a future
  reader auditing "did this check drift from the code" has one place to check
  both against.

### 3. Specification agreement (FR-004, and the new gate FR-014)

`docs/fragcap-specification.md:2577` and `:2591` are corrected: the `capture`
grammar block drops `-m, --mode` and `-q, --quiet`, keeping the long forms and
the `[default: file]` annotation (which becomes true once FR-003 lands).

FR-014's gate check parses the same specification block (a fixed line range or
a delimited section, whichever `check-prerequisites`-style parsing in this
codebase already prefers; the existing `no_subcommand_requires_a_store_path`
test's structural-enumeration style over `fragcap_cli::command()` is the model
for the *binary* side of the comparison) and compares its short-flag set
against `CaptureArgs`' actual `#[arg(short = ...)]` declarations, read via
`clap::Command::get_arguments()` the same way the existing store-path test
reads `get_arguments()`. A drift in either direction (the spec claims a flag
that does not exist, or the binary has a short flag the spec never
documents) fails the check, so the fix is symmetric even though today's defect
is one-directional.

### 4. `steam list`'s registration route (FR-005)

The `SteamCommand::List` variant doc comment ("List the installed Steam titles
this machine can enumerate.") gains a second sentence naming the route,
matching the phrasing already used correctly elsewhere on the enum-level
comment that never renders: "Register one as a capture target with `targets
add --steam <app_id>`." This is the minimal fix: move the existing, correct
sentence from the non-rendering enum doc comment to the rendering variant doc
comment, rather than composing new prose.

### 5. The `--json` split (FR-006)

`Cli::json`'s doc comment gains the same blank-line short/long split FR-009 of
S062 established for options: a one-line summary ("Emit machine-readable
output instead of human text.") stays as the first line, unchanged; the
existing second paragraph (naming `capture`, `steam`, `extcap`, and `doctor`
by name) is kept as the `--help`-only continuation, since removing it would
lose real cross-command information a root-level `--help` reader benefits
from, and per the clarification, the root command is where a reader deciding
whether to script against fragcap at all is the one who needs the whole
picture. No other command's help text changes: this is a single doc-comment
edit on the one shared `Arg`.

### 6. Option grouping (FR-007)

Not a field reorder: verified by rendering `capture -h` against the unmodified
struct that `--process` already sat immediately after `--id` in source order,
yet `--silent`/`--json` still interleaved between `--id` and `--process`. So
`CaptureArgs`' own field order was never the lever. Reading
`clap_builder-4.5.32`'s `Command::_propagate_global_args` (`sc.args.push(a.clone())`,
so a propagated global keeps whatever `display_order` it had on `Cli`) and
`option_sort_key` (sorts by `(display_order, flag-name)`, alphabetical
tiebreak) explains it exactly: `quiet`/`silent`/`json` get sequential
`display_order` values 0/1/2 from their declaration order on `Cli`, which tie
with `capture`'s own first three non-positional fields (`target`=0, `id`=1,
`process`=2) at those same slots, and the alphabetical tiebreak scatters them
(`quiet` before `target`; `id` before `silent`; `json` before `process`).

The fix: add `display_order = 1000` to `quiet`, `silent`, and `json` on `Cli`.
Since capture's own fields top out at display_order 21, this sorts all three
globals after every subcommand's own options uniformly, fixing the interleave
on every page in one place. Verified: after the change, `capture -h` shows
`--target`, `--id`, `--process` contiguous immediately after the positional
argument, with `--json`/`--quiet`/`--silent` clustered at the very end, after
`--help`.

### 7. Short/long split for `--catalog-db`/`--local-db` (FR-008)

Both gain a blank-line-separated one-line summary ("The shipped catalog store
consulted while resolving a target." / "The local store, where registered
targets and learned launch data live."), with the existing paragraph kept
behind `--help`, following the same FR-009-of-S062 pattern applied twice more.

### 8. Worked examples (FR-009, FR-010)

`capture`'s and `targets`' `Command` variant doc comments each gain an
`Examples:` block in the `--help`-only (post-blank-line) portion, drawn from
specification section 9.1 and `README.md` rather than composed fresh, so a
future edit to either source is the one an example here should track. Kept
short (two to three lines) to respect the 100-column wrap budget across
multiple example lines; the exact commands chosen are a task-time drafting
step, constrained to already-published invocations per FR-009/FR-010's own
wording ("drawn from... rather than authored fresh").

### 9. The four new gate checks (FR-011 through FR-015)

All four extend `crates/fragcap-cli/tests/cli_help.rs`, reusing its existing
`help_pages()`, `render()`, `label()`, and `normalize()` helpers rather than
duplicating them, matching the file's own stated design (a new subcommand
inherits every check by construction).

| Check | FR | Subject-set source | Shape |
| --- | --- | --- | --- |
| Short help is one line | FR-011 | `help_pages()`, rendered with `-h` (not `--help`), same enumeration the existing wrap test already uses with `--help` | For each page's `-h` rendering, every option/subcommand row's description must not wrap onto a continuation line (detected by absence of a bare-indent-only line following it, mirroring how the existing tests already parse rendered option blocks) |
| Every defaulted option states its default | FR-012 | A fixed list of four flags (`--mode`, `--direction`, `--roles`, `--wait`), each paired with the exact default-resolution site in `assemble.rs` it mirrors, recorded as a comment | For each, the rendered `capture --help` text contains either `[default: ...]` or the flag's stated prose default |
| Cross-references resolve | FR-013 | Every backticked token across every rendered page, filtered to two shapes: a bare lowercase-hyphenated word equal to some subcommand name in `help_pages()`, or a `-`/`--`-prefixed token | For each candidate, it names a real subcommand path or a real `Arg` reachable from `fragcap_cli::command()`, walked the same way `no_subcommand_requires_a_store_path` already walks the tree |
| Spec agreement | FR-014 | `capture`'s `Arg` set from `fragcap_cli::command()`, and a parsed short-flag set from `docs/fragcap-specification.md` section 17.2 | The two short-flag sets are equal |

Each is written and observed to fail against the pre-fix text (FR-015) before
the corresponding Design section's fix lands, in the same order as the table,
matching the dependency S062's own plan stated: gate before scrub.

## Project Structure

### Documentation (this feature)

```text
specs/070-help-accuracy-audit/
├── spec.md
├── plan.md                          # this file, carrying Phase 0 research inline
├── tasks.md
└── checklists/
    ├── requirements.md
    └── audit-gate.md
```

No separate `research.md`: Phase 0 produced three measured answers short
enough to live in this file, matching S062's own precedent. No
`data-model.md` or `contracts/`: nothing persistent, no interface change, and
the CLI grammar itself (already fully described in FR-002 through FR-010) is
the only "contract" this slice touches. No `quickstart.md`: the Verification
section below is the runnable validation guide.

### Files changed

```text
crates/fragcap-cli/src/cli.rs                   # --sink, --mode, --direction, --roles, --wait,
                                                 # --json, --catalog-db/--local-db, steam list,
                                                 # field reorder, worked examples
crates/fragcap-cli/src/assemble.rs              # simplified default resolution for --mode/--direction
                                                 # (unreachable-branch cleanup only, no behavior change)
crates/fragcap-cli/tests/cli_help.rs            # four new gate tests
docs/fragcap-specification.md                   # section 17.2 short-flag correction
AGENTS.md                                       # no dependency change expected; touched only if
                                                 # the audit record's home is recorded here
changelog.d/S070-help-accuracy-audit.fixed.md
changelog.d/S070-help-accuracy-audit.decisions.md   # the audit disposition table (FR-001)
```

**Structure Decision**: no new module, no new crate, no new test file. The
audit record required by FR-001 lives in
`changelog.d/S070-help-accuracy-audit.decisions.md`, which is reviewable per
line (one bullet per finding), durable (changelog fragments persist until
release, then fold into `CHANGELOG.md`), and matches the project's existing
practice of recording architecture-affecting decisions there rather than
inventing a new artifact location.

## Verification

`cargo xtask ci` in the foreground, watched to completion.

Each of the four new gate tests (FR-011 through FR-014) is written first, run
and read failing against the unmodified text (FR-015), then the corresponding
Design fix lands and the same test is re-run to confirm it passes. This is not
optional polish: it is the mechanism that makes the difference between "a gate
exists" and "a gate has been shown to catch the thing it exists for," which is
the exact distinction #67's guard failed on and #178 exploited.

After all fixes land: re-render every page enumerated by `help_pages()` and
confirm the existing wrap and vocabulary-leak tests still pass unchanged
(neither this slice's scope nor its fixes should touch wrapping or vocabulary,
so a regression there would indicate an unintended side effect), then manually
read `capture --help`, `capture -h`, `steam list --help`, and `targets --help`
end to end against the audit disposition table to confirm every FR is visibly
satisfied in the rendered text, not only in the passing test.
