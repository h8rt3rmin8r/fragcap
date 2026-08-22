<!-- spec-impact: none -->
**2026-08-22** Audit disposition for issue #183's findings 4 through 15 (1, 2,
3, 5, and 8 were already fixed and closed by S062; re-verified here against
`5a3862c`, not re-fixed). Each finding below is fixed, with what changed, or
closed, with the reason.

- **Finding 4** (`--sink` undocumented schemes/modifiers). Fixed. Help now
  names all seven schemes and six modifiers `args.rs` actually accepts; the
  real gap was wider than the issue recorded (it also missed `pcapng:`,
  `fifo:`, and `unix:`, and three of six modifiers).
- **Finding 6** (no option states its default). Fixed for `--mode`,
  `--direction` (now `default_value_t`, matching `--scope`'s existing
  pattern), `--roles` and `--wait` (stated in prose, since one is a
  `Vec<String>` and the other's default is "no timeout," not a fixed value).
- **Finding 7** (spec documents `-m`/`-q` the grammar refuses). Fixed by
  correcting the specification, per the recorded clarification (adding short
  flags to a just-stabilized grammar is the higher-risk direction). Also
  found and fixed in the same edit: the specification also documented
  `-V, --version` on `capture`, which the subcommand has never had (`-V` is
  root-only; clap does not propagate it without `propagate_version(true)`,
  which this grammar does not set). Caught by the new spec-agreement gate
  itself once it existed, not by the original audit inventory.
- **Finding 9** (`--extcap-version`: "Accepted; not acted on"). Closed, not a
  defect. The extcap protocol requires the flag to be accepted for
  Wireshark's version negotiation; the text already says exactly what
  happens.
- **Finding 10** (repeated `--json` paragraph). Partially closed as a
  documented clap limitation. The one-line `-h` split was already in place
  (S062's FR-009); verified, not re-fixed. The long-form paragraph still
  repeats on every page's `--help`, because a `global = true` clap arg cannot
  carry different help text per subcommand without either dropping
  `global = true` (a larger grammar change) or duplicating the paragraph as a
  second, driftable copy in the root page's hand-written template. A source
  comment records the reason.
- **Finding 11** (`--quiet`/`--silent`/`--json` interleaved with target
  inputs). Fixed, and not the way first planned. Reordering `CaptureArgs`'
  fields has no effect (verified: `--process` already sat beside `--id` before
  any edit, and the interleaving persisted). Reading `clap_builder-4.5.32`
  directly showed the real mechanism: a propagated global arg keeps the
  `display_order` it had on `Cli`, which ties with each subcommand's own early
  fields and is broken alphabetically. `display_order = 1000` on the three
  globals sorts them after every subcommand's own options uniformly.
- **Finding 12** (`--catalog-db`/`--local-db` long in `-h`). Fixed with the
  same short/long split as every other multi-sentence field this slice
  touched.
- **Finding 13** (no option grouping). Fixed as a side effect of finding 11's
  real fix: the four target inputs are now contiguous on every page.
- **Finding 14** (no worked examples). Fixed for `capture --help` and
  `targets --help`, drawn verbatim from specification section 9.1 and
  `README.md` rather than composed fresh.
- **Finding 15** (`steam list` names no route to registering a title). Fixed:
  the correct sentence already existed on the `SteamCommand` enum's own doc
  comment, which clap never renders; moved onto the `List` variant's doc,
  which does.

**Scope discovered during implementation, beyond the twelve findings above.**
Writing FR-011's gate (every `-h` summary is one line) against rendered output
first produced 75 apparent violations, the great majority false positives:
`capture -h`'s own column width (driven by its longest flag name) can wrap
even an already-correctly-split, inherently short sentence, and a page with
especially long flag names (`extcap`, `--extcap-interfaces` at 21 characters)
pushes clap into a next-line layout for every entry on that page, including
its own auto-generated, unauthorable `-h, --help` text. Neither is a doc
comment defect; both are `--width` concerns FR-001/FR-002's existing wrap gate
already owns. The check was redesigned as a source-level scan of `cli.rs`'s
doc-comment blocks (excluding the never-rendered outer doc on a struct or enum
used as a variant's payload, or a `#[command(subcommand)]`/`#[command(flatten)]`
field, verified against `doctor --help`'s and `extcap --help`'s actual
rendering): a real, source-only fact, independent of any page's column width.
That redesign surfaced 25 genuine, previously undiscovered violations spread
across `targets`, `catalog seed`, `doctor`, and `extcap` (fields whose entire
multi-sentence doc comment rendered in `-h`, never split); all 25 are fixed in
this slice (mechanical: one blank `///` line inserted after the first
sentence in each), since the fix was low-risk and leaving roughly two dozen
known violations unaddressed at the close of this campaign's last slice was
judged worse than the modest scope growth.

**The gate, before and after (FR-015).** Each of the four new checks in
`crates/fragcap-cli/tests/cli_help.rs` was demonstrated failing for the real
reason before the corresponding fix landed, or (where the fix predated the
test's first run) demonstrated failing again afterward via a deliberate,
reverted regression:

- `every_defaulted_option_states_its_default`: deliberately stripped `--wait`'s
  prose default statement; failed naming exactly that flag and block; restored
  and re-confirmed green.
- `capture_short_flags_match_the_specification`: deliberately reintroduced
  `-m, --mode` into the specification's section 17.2 block; failed with
  `spec-only ["-m"]`; restored and re-confirmed green.
- `every_cross_reference_resolves`: deliberately introduced a backticked
  reference to the retired `targets watch` command; failed on both pages that
  render `SteamCommand::List`'s doc, naming exactly `watch`; restored and
  re-confirmed green. (During development this same check also caught a real
  false positive of its own, `` `catalog seed --tier signature` `` reading
  `signature` as a stale command word when it is a `--tier` *value*, fixed
  in the checker itself, not in `cli.rs`, before the check was trusted.)
- `every_short_help_summary_is_one_line`: its first real run (against the
  as-yet-unfixed surface, before this slice's own doc-comment fixes landed)
  failed on all 25 genuine violations named above; every one now passes.

**Dependency and process notes.** No dependency added; `regex` was already a
`fragcap-cli` dev-dependency (S062). No change to `cargo xtask deps`,
`cargo xtask msrv`, or the workspace dependency inventory in `AGENTS.md`.
