<!-- spec-impact: none -->
**2026-08-22** Audit disposition for issue #183's findings 4 through 15 (1, 2,
3, 5, and 8 were already fixed and closed by S062; re-verified here against
`5a3862c`, not re-fixed). Each finding below is fixed, with what changed, or
closed, with the reason.

- **Finding 4** (`--sink` undocumented schemes/modifiers). Fixed. Help now
  names all seven schemes and six modifiers `args.rs` actually accepts; the
  real gap was wider than the issue recorded (it also missed `pcapng:`,
  `fifo:`, and `unix:`, and three of six modifiers).
- **Finding 6** (no option states its default). Fixed for all four.
  `--direction` takes `default_value_t`, matching `--scope`'s existing
  pattern (it has no profile-priority behavior to lose). `--mode`, `--roles`,
  and `--wait` state their default in prose: `--mode` and `--roles` because
  each has a real, tested profile-priority fallback in `assemble.rs`
  (`default_value_t` was tried for `--mode` too and reverted after breaking
  that fallback; see the review-round entry below), `--wait` because its
  default is "no timeout," not a fixed value.
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

**Review round on PR #198 (Codex + Copilot, 18 findings, all verified and
addressed).** Every finding was checked against the actual code before being
accepted; none were rejected.

- **`--roles`' profile-priority default was never documented (Codex).**
  `assemble.rs`'s `.or_else(|| defaults.roles()...)` gives a profile-declared
  roles list priority over "every role," exactly the same shape as `--mode`'s
  fallback; the help text said only "every role." Fixed to state the real
  precedence.
- **FR-011's gate did not render `-h` (Codex + Copilot, independently).**
  Correct as filed. Fixing this properly required two further rendered-check
  attempts, both measured wrong, before landing on the final design: see the
  dedicated account below.
- **`DEFAULTED_OPTIONS` was a hand-maintained list FR-012 says must be
  structural (Codex + Copilot, independently).** Correct. Split into a
  structural half (any `Arg` with `get_default_values()` non-empty, walked
  from `fragcap_cli::command()`, covering `--direction` and any future
  `default_value_t` option with no list edit) and a minimal, explicitly-
  justified `PROSE_ONLY_DEFAULTS` list for exactly the two cases
  (`--roles`, `--wait`, and `--mode` after its own revert) that resolve a
  default inside `assemble.rs` with no clap-visible signal at all, which
  cannot be derived from the command tree by any structural walk.
- **The `DEFAULTED_OPTIONS` comment describing `--mode` was stale
  (Copilot).** It still said `--mode` used `default_value_t`; `--mode` was
  reverted to `Option<ModeArg>` with a prose default. The whole constant and
  its comment were removed as part of the structural/prose split above.
- **T017's sink-scheme regression assertion was never actually written
  (Copilot).** Correct: the task was marked done in `tasks.md` but no such
  test existed. Added `sink_help_names_every_scheme_and_modifier_the_parser_accepts`,
  deriving the scheme and modifier sets from `args.rs`'s own match-arm keys
  (`parse_destination`'s and `apply_option`'s), not a copied list, so the
  test and the parser cannot drift independently. Demonstrated failing (a
  temporarily removed `unix:` scheme) before being confirmed passing.
- **The specification still claimed `--mode`'s default is unconditionally
  `file` (Copilot).** Correct; `resolve_mode` preserves the profile-priority
  fallback. Corrected in `docs/fragcap-specification.md` section 17.2 to
  `[default: profile mode, else file]`, and `--roles`' row the same way.
- **"With neither this flag set" on `--wait` is ungrammatical (Copilot).**
  Correct (only one flag is being discussed, not two). Changed to "Without
  this flag."
- **Scheme count miscounted as six instead of seven (Copilot).** Correct
  arithmetic error in `spec.md`'s Evidence table and the audit-gate
  checklist; both list seven items. Fixed in both places.
- **"Still live for three of four" undercounts by one (Copilot).** Correct;
  all four remaining options (`--mode`, `--direction`, `--roles`, `--wait`)
  stated no default at time of writing. Fixed to "all four."
- **The `--profile`-flag edge case in `spec.md` was false (Copilot).**
  Correct, and the sharpest finding of the round: it asserted the
  profile-mode fallback was unreachable on the shipped CLI and told the
  reader to document `--mode` as unconditionally `file`, which is exactly
  backwards from what the pre-existing `a_profile_declared_ring_mode_is_
  resolved_and_validated` test proves and what FR-003 itself already said
  correctly. Rewritten to state the real, tested precedence.
- **`plan.md` still said `--mode` gains `default_value_t` (Copilot).**
  Correct; stale from before the revert. Rewritten together with the
  "one of four" miscount below (Copilot: the true total, `--scope` plus four
  more, is five) into a single corrected passage covering both.
- **`tasks.md`'s T018 record still described the reverted plan (Copilot).**
  Correct. Rewritten to describe the actual specification edit (conditional
  defaults, plus the three flags found missing entirely, below), and a new
  T019a records the FR-014 extension that surfaced them.
- **FR-013's cross-reference check was circular (Copilot).** Correct, and
  the second-sharpest finding: gating bare-word checking on "the backtick
  span's first word is itself a real command" meant a genuinely stale,
  *standalone* reference (a lone `` `watch` `` naming a retired command)
  could never trigger the check, because `watch` failing to be real is
  exactly what made the span fail to "read as an invocation." Redesigned:
  value literals (enum possible-values, `value_name` hint tokens) are now
  excluded structurally via `command_surface()`, and every bare
  command-word-shaped token is checked unconditionally, with no span
  gating at all. Fixing this exposed two more real gaps found only once the
  gate ran for real: `fragcap` itself (the binary's own name, appearing in
  every worked example) was not in the known-words set, and a bare word
  immediately followed by a colon (`` `kind: "export"` ``, a JSON field
  name in prose) needed to stay excluded from bare-word candidacy even
  though a flag reference's own trailing colon must still be trimmed
  (`` `--target`: ``). Both fixed; demonstrated catching a deliberately
  introduced `targets watch` stale reference before being reverted.
- **FR-014 only compared short flags, though the requirement names "flag and
  short-flag set" (Copilot).** Correct. Extended to compare long flags too
  (`get_long()` alongside `get_short()`), excluding hidden offline-substrate
  args (`is_hide_set()`) that never render and adding the propagated globals
  (`--quiet`/`--silent`/`--json`, invisible on an unbuilt `Command`) to the
  binary side. Turning this on immediately surfaced three real,
  previously-undiscovered spec gaps: `--catalog-db`, `--local-db`, and
  `--scope` are real, shipped `capture` flags the specification's section
  17.2 never documented at all. Added all three rows.

**FR-011's gate, the long way round.** Getting a genuinely rendered check
right took three attempts, and the failure of the second is worth recording
in full because it is not obvious in advance. Attempt one (a raw
continuation-line scan of `-h` output) flagged `--target`'s own
already-correctly-split, single-sentence summary as a violation, because
`capture -h`'s description column, pushed right by *other* long flag names on
the same page, is narrow enough to wrap even one short sentence. Attempt two
cross-checked each continuation against that same page's `--help` rendering
for an internal blank-line split, which fixed the `--target` case, but then
flagged `extcap -h`'s `--extcap-interfaces`, `--capture`, several more of its
own fields, and even clap's own auto-generated, unauthorable `-h, --help`
text ("Print help (see more with '--help')") as violations: `extcap`'s
longest flag name (`--extcap-interfaces`, 21 characters) is long enough to
push clap into a next-line layout for the *entire* page, so every entry wraps
regardless of whether it has, or even could have, a second sentence to
defer. Neither failure is the defect FR-011 exists to catch; both are caused
by sibling flags on the same page, not by the field's own doc comment. The
final design renders every page's `-h` (satisfying "render `-h`... iterate
`help_pages()`" literally, and confirming each exits 0), but the actual
pass/fail judgment is a source-level scan of `cli.rs`'s doc comments for a
sentence boundary (`.`, `?`, or `!` followed by whitespace and a capital
letter, outside backticked code) with no blank `///` split, because that is
a fact about the field's own content, not about what else happens to share
its page. The sentence detector was also corrected per the two `-011` review
comments to recognize `?`/`!`, not only `.`; verified by temporarily joining
a doc comment with `!` and confirming the gate caught it before reverting.
