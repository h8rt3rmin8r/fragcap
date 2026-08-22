# Feature Specification: Help accuracy audit and gate

**Feature Branch**: `070-help-accuracy-audit`

**Created**: 2026-08-22

**Status**: Draft

**Input**: User description: "S070: help accuracy audit and gate, closing issue
#183. A line-by-line accuracy and concision pass over every string fragcap's
binary can print as help, producing a reviewable per-line audit record, fixing
every real defect found, and adding a test gate that stops the defect class
recurring."

## Context

Issue #183 opens with the operator who read `fragcap capture --help`, composed
an invocation straight out of it, and was refused (#181, closed by S062). That
issue's own text draws the boundary this slice keeps: "a wrapped, jargon-free
line that is false is still false." S062 (issue #176, #177, #178, #181, #182)
made every page wrap, stripped internal development vocabulary, and fixed the
two accuracy defects sharp enough to have been filed as their own issues. #183
is the remainder: the findings that are about a line being **true**, complete,
and short, not about it fitting the terminal or naming a slice number, plus the
gate the issue itself specifies to keep the class from recurring the way #178
recurred after #67.

Explicitly sequenced last in the 2026-08 issue-clearing campaign
(`docs/plans/README.md` ordering; see the `issue-clearing-campaign-2026-08`
project record), specifically so its audit runs over text that already wraps
and has had the vocabulary stripped, per S062's own OOS-004.

## Evidence

Verified against `target/debug/fragcap.exe` built from `main` at `5a3862c`
(v0.5.1) on 2026-08-22, by running the binary rather than reading the issue's
2026-08-20 inventory as current. Three prior slices (S062, S063, plus the
S064-S069 capture/detection/steam work) landed on this surface since #183 was
filed, and the re-check changes the scope materially:

| # | Issue finding | Current status |
| --- | --- | --- |
| 1 | `capture --launch` documents an unresolvable invocation | Fixed, closed as #181 (S062) |
| 2 | `targets list` claims a column it never prints and claims to read | Fixed, closed as #182 (S062) |
| 3 | `capture` summary names two of four target inputs | Fixed by S062 (the summary now names three categories matching the four-way usage group) |
| 4 | `--sink` help lists four schemes; the parser accepts more | **Still live, and wider than filed.** `args.rs` accepts seven schemes (`file`, `pcapng`, `jsonl`, `pipe`, `fifo`, `unix`, `tcp://`) and six `,key=value` modifiers (`format`, `payload`, `rotate-size`, `rotate-duration`, `queue`, `timeout`). Help still names four schemes and zero modifiers |
| 5 | `--target`'s own help never states the row-index exclusivity | Fixed by S062 (FR-011): both the positional selector and `--target` state it directly |
| 6 | No option ever prints its default | **Still live for all four.** `--scope` prints `[default: target]` (added by S062 for an unrelated reason). `--mode`, `--direction`, `--roles`, and `--wait` all resolve a default in `assemble.rs` and none is printed |
| 7 | Spec section 17.2 documents `-m, --mode` and `-q, --quiet`; the grammar has neither | **Still live.** `docs/fragcap-specification.md:2577` and `:2591` are unchanged |
| 8 | Unimplemented surface marked only in prose | Fixed by S062: `--mode`'s value list and `replay`/`--ring` carry `(implemented)`/`(not yet implemented)` tags in the possible-values list and short help itself |
| 9 | `extcap --extcap-version`: "Accepted; not acted on" | **Still live** (unchanged; the audit re-evaluates whether this is a defect or an honest statement) |
| 10 | The 45-word `--json` paragraph repeats verbatim on all 29 pages | **Still live**, confirmed on `schema --help`, where it is entirely about other commands |
| 11 | `--quiet`/`--silent` sit inside the target-input group instead of beside each other | **Still live.** On `capture --help`, `--process` is separated from `--target`/`--id` by `--silent` and `--json` |
| 12 | `--catalog-db`/`--local-db` render as full paragraphs even in `-h` | **Still live**, confirmed by running `capture -h` |
| 13 | No option grouping | **Still live** |
| 14 | No worked examples on any page | **Still live** |
| 15 | `steam list` names no route to using an app id | **Still live.** The route ("registering a title as a capture target is `targets add --steam <app_id>`") exists only as a doc comment on the `SteamCommand` enum, which clap's derive never renders anywhere |

The gate section of #183 proposes four checks. None exist in
`crates/fragcap-cli/tests/cli_help.rs` today; its seven current tests cover
wrapping, vocabulary leaks, parser-internal leaks, page-set coverage, and the
two #181/#182 accuracy fixes. This slice's gate is net-new, not an extension of
an equivalent existing check.

## Clarifications

### Session 2026-08-22

Resolved under the autopilot decision policy: alternatives enumerated,
evaluated against the constitution, the master specification, existing code
patterns in S062's own spec, and this slice's scope; best-supported option
taken and recorded.

- Q: Finding 7's spec/grammar disagreement (`-m, --mode`, `-q, --quiet`) can be
  resolved by adding the short flags to the grammar or by correcting the spec.
  -> A: **Correct the spec.** Adding `-m`/`-q` would be a grammar change to a
  surface S062 just finished stabilizing, with no operator demand recorded
  anywhere, and clap short flags are a one-way door once released (removing one
  later is a breaking change; the spec's prose is not). The shipped grammar is
  also what issue #181's operator actually exercised, so it is the side with
  the stronger claim to being right.
- Q: Finding 9 (`--extcap-version`: "Accepted; not acted on"): is this an
  accuracy defect, or an honest statement of real behavior? -> A: **Not a
  defect; leave the text.** The extcap protocol requires the flag to be
  accepted for Wireshark's version negotiation to succeed; fragcap has one
  supported version and needs no branch on the query's value. The help already
  says exactly what happens. Removing the line would be less accurate, not
  more, so this finding closes with no code change, recorded as a closed
  finding in the audit record.
- Q: Finding 10 (the repeated `--json` paragraph) can be fixed by shortening
  the paragraph everywhere, or by giving each command a one-line pointer to a
  fuller explanation elsewhere. -> A: **Verified during implementation that
  the one-line `-h` split (FR-009, S062) was already in place**: `--json`'s
  doc comment already carries the blank-line split, and `capture -h` already
  renders one line ("Emit machine-readable output instead of human text").
  Finding 10's remaining defect is narrower than first scoped: the full
  cross-command paragraph still repeats on every page's `--help` (long form),
  including pages, like `schema --help`, it has nothing to do with. **A global
  `clap` arg cannot carry different `--help` text per subcommand** (`global =
  true` shares one `Arg` definition, confirmed by reading `clap`'s derive
  behavior rather than assumed), so making the paragraph appear only on the
  root page is not reachable without either dropping `global = true` (a larger
  grammar change, risking the `fragcap capture --json` placement working
  today) or duplicating the paragraph as literal text in the root page's
  existing hand-written template (a second copy of the same fact, which is the
  drift risk the single-source doc comment exists to avoid). Accepted as a
  documented clap limitation rather than fixed further: a source comment
  records why, and the existing phrasing (already generic, already names every
  command rather than pretending to be about whichever one the reader ran) is
  kept rather than duplicated or shortened for its own sake.
- Q: Finding 11 (`--quiet`/`--silent`/`--json` interleaved with the
  target-input group): clap 4.5.32 has no native arg grouping in rendered
  `--help` (the same constraint the root command's hand-written `Commands:`
  block works around). Is a hand-written template justified for `capture
  --help`? -> A: **No, and not a field reorder either, verified during
  implementation that clap's actual sort key is `(display_order, flag-name)`,
  not source declaration order.** Reading `clap_builder-4.5.32`'s
  `Command::_propagate_global_args` and `option_sort_key` directly (not
  assumed) showed that a propagated `global = true` arg keeps the
  `display_order` value it was assigned on `Cli` itself (sequential by
  declaration order: 0, 1, 2 for `quiet`/`silent`/`json`), and that value ties
  with each subcommand's own first few non-positional fields at the same
  slot, broken alphabetically by flag name. `CaptureArgs`' own field order was
  never the lever: `--process` already sat immediately after `--id` in source
  order before any edit in this slice, and the interleaving persisted
  regardless (confirmed by rendering `capture -h` before touching the struct).
  The real, minimal fix is `display_order = 1000` on the three global flags on
  `Cli`, which sorts them after every subcommand's own options uniformly
  (verified: `capture`'s own fields top out at display_order 21), fixing every
  subcommand's list in one place rather than one at a time, with no new
  template and no per-struct reordering.
- Q: Finding 14 (no worked examples): where do examples belong, every page, or
  a subset? -> A: **`capture --help` and `targets --help` only**, matching the
  issue's own "What to do" item 6. These are the two commands with worked
  invocations already published in the specification (section 9.1's five
  captures) and `README.md`, so an example section here can be drawn from an
  existing source rather than authored fresh, and cannot drift into a third,
  disagreeing copy. Every other page is a narrower surface where the option
  list itself is the complete usage story.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Every remaining accuracy finding is resolved or closed with a reason (Priority: P1)

An auditor (a future contributor, or this slice) needs to know, for each of
findings 4 through 15, whether it was fixed and how, or why it was
deliberately left. Today that record does not exist anywhere reviewable; the
issue itself is the closest thing, and it is a starting inventory, not a
verdict.

**Why this priority**: the issue's own acceptance criterion is "every finding
above is either fixed or closed with a recorded reason," and "the audit record
exists and is reviewable per line." Without the record, a fix cannot be told
apart from an oversight.

**Independent Test**: read the audit record and confirm every one of findings
4 through 15 has a disposition (fixed, with the change; or closed, with the
reason) and that the disposition matches what the rendered help actually says.

**Acceptance Scenarios**:

1. **Given** the audit record, **When** it is read against findings 4 through
   15, **Then** every finding has exactly one disposition and no finding is
   silently absent.
2. **Given** a finding marked "fixed", **When** the corresponding help page is
   rendered, **Then** the defect it named is gone.
3. **Given** a finding marked "closed, not a defect" (finding 9), **When** the
   record is read, **Then** it states the reason, not merely the verdict.

---

### User Story 2 - `--sink` and every option with a real default state it fully (Priority: P1)

An operator composing a `--sink` value from `--help` alone needs every scheme
and modifier the parser accepts; today's text would lead them to conclude
`pcapng:`, `fifo:`, and `unix:` do not exist, and that none of the four modifier
keys exist at all. Separately, an operator relying on `--mode`, `--direction`,
`--roles`, or `--wait` behaving one way by default has to read the source to
learn what "by default" means, because `--help` states none of the four.

**Why this priority**: equal to US1; these are the two remaining findings
where the printed text actively contradicts the accepted grammar rather than
merely omitting polish.

**Independent Test**: enumerate every scheme and modifier `parse_sink` accepts
and diff against the schemes and modifiers named in `--sink`'s help text; parse
a value using every default-bearing option's absence and diff the resolved
value against what `--help` states.

**Acceptance Scenarios**:

1. **Given** `capture --sink --help`, **When** it is read, **Then** it names
   every scheme `parse_sink` accepts and every `,key=value` modifier
   `apply_option` accepts.
2. **Given** `capture --mode`, `--direction`, `--roles`, and `--wait`,
   **When** each is rendered, **Then** it states its default in the same
   `[default: ...]` form `--scope` already uses, or, where the true default is
   conditional (mode's file default applies only absent a loaded profile),
   states the conditional plainly rather than omitting it.

---

### User Story 3 - The spec and the shipped grammar agree (Priority: P2)

A reader of `docs/fragcap-specification.md` section 17.2 sees `-m, --mode` and
`-q, --quiet` as part of the grammar. Neither exists in the shipped binary. One
of the two documents is wrong, and right now both are published.

**Why this priority**: lower than US1/US2 because it affects a reader of the
specification rather than a reader of `--help` directly, but it is a
architecture-of-record document making a false claim, which the master
specification's own authority depends on not doing.

**Independent Test**: diff the short-flag set the specification documents for
`capture` against the short-flag set clap actually declares.

**Acceptance Scenarios**:

1. **Given** `docs/fragcap-specification.md` section 17.2's `capture` grammar
   block, **When** it is compared against the shipped `CaptureArgs` short-flag
   set, **Then** they agree, with the specification corrected to match the
   shipped grammar (this slice's clarified decision) rather than the reverse.
2. **Given** a future divergence between the two, **When** the gate's
   spec-agreement check runs, **Then** it fails, naming the flag that disagrees.

---

### User Story 4 - The four proposed gate checks exist and hold (Priority: P1)

The audit is only as durable as its gate. #183 proposes four checks (short
help stays one line, every defaulted option prints its default, every
backticked cross-reference resolves to something real, and `capture`'s
flag/short-flag set matches the specification), and none exists in
`crates/fragcap-cli/tests/cli_help.rs` today.

**Why this priority**: matches US1/US2; without the gate, this slice's own
fixes are exactly as durable as #67's were, and #178 is the demonstrated
counterexample.

**Independent Test**: add a temporary regression of each kind (a two-line short
help, a defaulted option with the default suppressed, a cross-reference to a
nonexistent flag, a specification/grammar mismatch) and confirm each fails the
new gate without editing the gate.

**Acceptance Scenarios**:

1. **Given** any option's `-h` rendering, **When** it is measured, **Then** its
   summary is one line.
2. **Given** any option resolving a default in `assemble.rs`, **When** its
   `--help` is rendered, **Then** the default is stated.
3. **Given** any backticked `` `command` `` or `` `--flag` `` token inside a
   rendered help page, **When** it is checked, **Then** it names a real command
   or flag on the current command surface.
4. **Given** `capture`'s flag and short-flag set, **When** compared against
   specification section 17.2, **Then** they agree.
5. **Given** a deliberately introduced violation of any of the four, **When**
   the gate runs, **Then** it fails without the gate itself being edited.

---

### User Story 5 - Global flags are grouped, and `capture --help`/`targets --help` carry worked examples (Priority: P3)

An operator reading `capture --help` cannot see the four target-input options
adjacent to each other, because two global flags sit inside that group in
declaration order. An operator on either `capture --help` or `targets --help`
has to go to `README.md` or the specification to see a worked invocation.

**Why this priority**: real, but concision and onboarding rather than a false
or missing fact, so it ranks below the three accuracy-bearing stories.

**Independent Test**: read `capture --help`'s option order and confirm the
four target inputs are contiguous; read `capture --help` and `targets --help`
and confirm each carries at least one worked example.

**Acceptance Scenarios**:

1. **Given** `capture --help`, **When** the option list is read top to bottom,
   **Then** the positional selector, `--target`, `--id`, and `--process` are
   contiguous, with no other option's help text between the first and the
   last.
2. **Given** `capture --help`, **When** it is read, **Then** it carries at
   least one worked invocation drawn from the specification's section 9.1
   examples or `README.md`.
3. **Given** `targets --help`, **When** it is read, **Then** it carries at
   least one worked invocation.
4. **Given** `steam list --help` or its rendered output, **When** read,
   **Then** it names the route (`targets add --steam <app_id>`) from a printed
   app id to a capturable target.

---

### Edge Cases

- **A global `clap` arg cannot carry different help text per subcommand.**
  `--json`'s `global = true` arg is one `Arg` definition shared by every
  command. The clarified fix (a one-line short summary, full paragraph behind
  `--help`, on the root page) is the highest-fidelity fix available within that
  constraint; a per-command paragraph is out of reach without either dropping
  `global = true` (repeating the flag on every subcommand, a larger and
  riskier grammar change) or a custom help renderer (the root page's existing
  hand-budgeted-template risk, applied everywhere).
- **`--mode`'s default is conditional on a loaded profile, and that path is
  genuinely reachable, not merely theoretical.** `assemble.rs`'s `resolve_mode`
  reaches `profile.capture().mode()` whenever `--mode` is omitted; a stored
  target's resolved profile can declare a mode (proven by the pre-existing
  test `a_profile_declared_ring_mode_is_resolved_and_validated`, which
  constructs a real `CaptureArgs` from actual CLI parsing with no `--mode`
  and confirms `resolve_mode` returns the profile's declared `Ring`). A
  `default_value_t = ModeArg::File` attempt during implementation collapsed
  `Option<ModeArg>` to a plain `ModeArg`, destroying the "nothing passed"
  state that fallback depends on, and was caught failing that exact test; it
  was reverted (`--mode` stays `Option<ModeArg>`; see plan.md's Phase 0 for
  the full account). The correct precedence to document, and the one FR-003
  and the shipped `--mode` help text both actually state, is: an explicit
  `--mode` wins, else a profile-declared mode, else `file`. An earlier
  version of this edge case asserted the profile branch was unreachable on
  the shipped CLI surface and told the reader to state the default as an
  unconditional `file`; review of PR #198 correctly identified that claim as
  false against the code and the test proving otherwise.
- **A worked example embedded in help text can drift from the specification's
  own worked examples if either is edited alone.** The gate's cross-reference
  check (US4) verifies flags and commands named exist; it does not verify
  prose equivalence between three copies of an example. Recorded as a known
  gap, not fixed here: closing it fully would mean generating help examples
  from the specification's examples mechanically, which is a larger change
  than this slice's scope and not requested by the issue.
- **`no_match_message`'s row-index error text (finding 12 in the issue's
  numbering, already implemented) must not regress while other target-input
  help text is reordered (US5).** The existing test
  `a_bare_integer_is_documented_as_unconditionally_a_row_index` already covers
  the `--help` half of this; reordering fields in `CaptureArgs` must not touch
  `target_resolve.rs`, which is untouched by this slice's scope.

## Requirements *(mandatory)*

### Functional Requirements

**The audit record**

- **FR-001**: A reviewable per-finding audit record MUST exist covering every
  finding numbered 4 through 15 in issue #183, each with a disposition (fixed,
  with what changed; or closed, with the reason) and a reference to the test
  or manual verification that confirms it.

**Accuracy fixes**

- **FR-002**: `capture --sink`'s help MUST name every scheme `parse_sink`
  accepts (`file:`, `pcapng:`, `jsonl:`, `pipe:`, `fifo:`, `unix:`, `tcp://`)
  and every `,key=value` modifier `apply_option` accepts (`format`, `payload`,
  `rotate-size`, `rotate-duration`, `queue`, `timeout`).
- **FR-003**: `--mode`, `--direction`, `--roles`, and `--wait` MUST each state
  their effective default in rendered `--help`, in the same `[default: ...]`
  form `--scope` already uses where the default is unconditional, or in
  explicit prose where it is conditional (`--mode`, per the edge case above).
- **FR-004**: `docs/fragcap-specification.md` section 17.2's `capture` grammar
  block MUST be corrected to match the shipped short-flag set (no `-m`, no
  `-q`), resolving the disagreement without adding short flags to the grammar.
- **FR-005**: `steam list`'s rendered help or output MUST name the route from a
  listed app id to a capturable target (`targets add --steam <app_id>`),
  reachable from `--help` or `-h`, not only from a doc comment clap never
  renders.

**Concision fixes**

- **FR-006**: The global `--json` flag's `-h` summary MUST be a one-line
  statement (already true, verified against S062's FR-009 fix; re-verified
  here rather than re-fixed). Its full cross-command `--help` paragraph MUST
  name every command it actually applies to (already true) rather than reading
  as specific to whichever command the reader ran. The paragraph repeating
  verbatim on every page's `--help` is accepted as a documented clap
  limitation (a single shared global `Arg` cannot vary its help text by
  subcommand) rather than fixed further; a source comment MUST record the
  reason so a future reader does not attempt the same fix and rediscover the
  constraint.
- **FR-007**: The rendered `--help`/`-h` option list for `capture` (and, as a
  side effect of the same fix, every other subcommand) MUST place the
  positional selector, `--target`, `--id`, and `--process` contiguously, with
  `--quiet`, `--silent`, and `--json` sorted after every command-specific
  option rather than interleaved among them. Achieved via each global flag's
  `display_order`, not via `CaptureArgs`' field declaration order (verified
  during implementation that the latter has no effect on rendered order).
- **FR-008**: `--catalog-db` and `--local-db` MUST each carry a one-line `-h`
  summary distinct from their full `--help` paragraph, following the same
  short/long split FR-009 of S062 established for other options.
- **FR-009**: `capture --help` MUST carry at least one worked invocation drawn
  from specification section 9.1 or `README.md`.
- **FR-010**: `targets --help` MUST carry at least one worked invocation.

**The gate**

- **FR-011**: `crates/fragcap-cli/tests/cli_help.rs` MUST gain a check that
  every option's `-h` (short) rendering is one line, over every page the
  existing page-set enumeration discovers.
- **FR-012**: The suite MUST gain a check that every option resolving a default
  value in `assemble.rs` states that default in its rendered `--help`. The
  check MUST enumerate the defaulted options from `assemble.rs`'s own
  resolution sites (mirroring how `no_subcommand_requires_a_store_path`
  enumerates from the command tree) rather than from a hand-maintained list, so
  a new defaulted option is covered without an edit to the guard.
- **FR-013**: The suite MUST gain a check that every backticked `` `token` ``
  appearing in rendered help that names a command word or a `--flag` resolves
  to a real command or flag on the current command surface (the page-set
  enumeration already in `help_pages()`), failing on a stale cross-reference
  such as a renamed or removed flag still named in another page's prose.
- **FR-014**: The suite MUST gain a check comparing `capture`'s flag and
  short-flag set (from `fragcap_cli::command()`) against the flag block in
  `docs/fragcap-specification.md` section 17.2, failing when they disagree.
- **FR-015**: Each of FR-011 through FR-014 MUST be demonstrated to fail on a
  deliberately introduced violation before being demonstrated to pass on the
  corrected surface, per the project's standing verification discipline.

### Key Entities

- **Audit record**: the per-finding disposition table required by FR-001,
  living in this feature's `plan.md` or a dedicated file it points to.
- **Help page**: one `--help` or `-h` rendering, enumerated by
  `help_pages()` in `crates/fragcap-cli/tests/cli_help.rs`; unchanged by this
  slice.

### Out of scope

- **OOS-001**: Findings 1, 2, 3, 5, and 8, already fixed by S062 (see Evidence
  table). Re-verified, not re-fixed.
- **OOS-002**: Finding 9 (`--extcap-version`), closed with no code change per
  the clarification above: the text is accurate as written.
- **OOS-003**: Generating help examples mechanically from the specification's
  worked examples (the drift gap named in Edge Cases). A real follow-up if the
  three copies are found to have drifted in a future audit; not requested by
  #183 and larger than this slice's scope.
- **OOS-004**: Any change to `--sink`'s accepted schemes or modifiers, or to any
  other flag's actual parsing behavior. This slice corrects what `--help`
  claims about existing behavior; it does not add or remove accepted syntax.
- **OOS-005**: Issue #197 (prune vendored `.agents/skills/`), #155 (IGDB), and
  #94 (community sync). Unrelated surfaces, explicitly out of this campaign's
  remaining scope.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Every finding 4 through 15 has a recorded disposition, verified
  by reading the audit record against the Evidence table above.
- **SC-002**: `capture --sink --help` names all seven schemes and all six
  modifiers; verified by a test enumerating `parse_sink`'s accepted schemes and
  `apply_option`'s accepted keys against the rendered text.
- **SC-003**: `--mode`, `--direction`, `--roles`, and `--wait` each state a
  default in rendered help; zero of the four state none, down from four today.
- **SC-004**: `docs/fragcap-specification.md` section 17.2 and the shipped
  `capture` short-flag set agree, verified by the new spec-agreement gate
  check.
- **SC-005**: All four new gate checks (FR-011 through FR-014) exist, each
  demonstrated to fail on a deliberately broken case and pass on the shipped
  surface.
- **SC-006**: `cargo xtask ci` is green.

## Assumptions

- The page set, its enumeration (`help_pages()`), and the existing seven tests
  in `cli_help.rs` are unchanged in shape; this slice adds tests and does not
  restructure the existing ones.
- `docs/fragcap-specification.md` section 17.2 is the only specification
  location documenting `capture`'s short-flag set; corrected in place rather
  than duplicated elsewhere.
- No new crate or dependency is needed: this is prose and test work inside
  `fragcap-cli`, consistent with "wrappers stay thin."
- The audit record's home (`plan.md` versus a dedicated file) is a planning
  decision, not a spec decision; either satisfies FR-001 as long as it is
  reviewable per line.
