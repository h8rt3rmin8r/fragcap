---

description: "Task list for slice S070, the help accuracy audit and gate"
---

# Tasks: Help accuracy audit and gate

**Input**: Design documents from `/specs/070-help-accuracy-audit/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md)

**Tests**: Required, and each of the four new gate checks is written and
observed failing against the current text before its corresponding fix lands.
A gate that has never been red has not been shown to work, which is the
standing lesson from #67/#178 this project keeps relearning.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: can run in parallel (different files, no dependency)
- **[Story]**: US1 audit record, US2 `--sink`/defaults accuracy, US3 spec
  agreement, US4 the four new gate checks, US5 grouping/examples/steam route

---

## Phase 1: Setup

- [x] T001 Build `target/debug/fragcap.exe` and re-render every page in
  `help_pages()`'s enumeration (already done once during specification; repeat
  if the branch has moved) to confirm the Evidence table in `spec.md` still
  holds before changing anything.

---

## Phase 2: Foundational

Shared infrastructure every Phase-4 (US4) gate test reuses. No fix lands here;
this only adds the plumbing the gate tests need.

- [x] T002 In `crates/fragcap-cli/tests/cli_help.rs`, add a `DEFAULTED_OPTIONS`
  constant: a fixed list of `(flag_name, assemble_rs_site_comment)` pairs for
  `--mode`, `--direction`, `--roles`, `--wait`, each annotated with a comment
  naming the exact `assemble.rs` line/site it mirrors (per plan.md's Design
  section 9 table), so a reader auditing drift has one place to check both
  sides against.
- [x] T003 [P] In the same file, add a small helper that extracts every
  backticked token from a rendered page and classifies it as a candidate
  cross-reference only if it is a bare lowercase-hyphenated word or is
  `-`/`--`-prefixed (the CHK007 shape rule from plan.md Phase 0, item 3),
  returning the filtered candidate list for FR-013.
- [x] T004 [P] Add a helper that reads `docs/fragcap-specification.md`,
  extracts the `capture` grammar block's short-flag set from section 17.2 (a
  parsed set of `-x` tokens from that block), for FR-014.

---

## Phase 3: User Story 4 - The four new gate checks exist and hold (P1)

**Goal**: FR-011 through FR-015. Each check is written now, against the
current (pre-fix) text, and observed failing, establishing the red state the
later phases turn green. This phase produces the tests only; the fixes that
turn them green are Phases 4 through 6.

**Independent Test**: run each new test now and confirm it fails with a
message naming the real, current defect (not an error in the test itself).

- [x] T005 [US4] Add `every_short_help_summary_is_one_line` to `cli_help.rs`
  (FR-011): for every page in `help_pages()`, render with `-h`, and for every
  option/subcommand row assert its description does not continue onto a bare
  wrapped line. Run and confirm it fails today on `--json`, `--catalog-db`,
  and `--local-db` (the three known multi-line `-h` paragraphs).
- [x] T006 [US4] Add `every_defaulted_option_states_its_default` to
  `cli_help.rs` (FR-012), driven by `DEFAULTED_OPTIONS` (T002): for each entry,
  render `capture --help` and assert the flag's block contains `[default:
  ...]` or a stated prose default. Run and confirm it fails today on all four
  (`--mode`, `--direction`, `--roles`, `--wait` currently state none).
- [x] T007 [US4] Add `every_cross_reference_resolves` to `cli_help.rs`
  (FR-013), driven by the T003 helper: for every page, extract candidate
  tokens and assert each names a real subcommand path (from `help_pages()`) or
  a real `Arg` (walked from `fragcap_cli::command()`, following
  `no_subcommand_requires_a_store_path`'s existing walk pattern). Run once
  against current text to confirm it passes vacuously or fails honestly (no
  known live cross-reference defect is expected here; record the actual
  result rather than assuming).
- [x] T008 [US4] Add `capture_short_flags_match_the_specification` to
  `cli_help.rs` (FR-014), driven by the T004 helper: compare
  `fragcap_cli::command()`'s `capture` subcommand short-flag set against the
  parsed specification set; assert equality. Run and confirm it fails today
  (`-m`, `-q` appear in the spec, not in the binary).
- [x] T009 [US4] Record the T005/T006/T008 failing output (and T007's actual
  result) verbatim in `changelog.d/S070-help-accuracy-audit.decisions.md`
  under a "gate: before" heading, satisfying FR-015's fail-then-pass
  demonstration for its first half.

**Checkpoint**: three of four new gates are red for a known, real reason; the
fourth's actual state is recorded rather than assumed. Nothing outside the
test file has changed yet.

---

## Phase 4: User Story 2 - `--sink` and every defaulted option state fully (P1)

**Goal**: FR-002, FR-003. Turns T005 and T006 green for `--mode`, `--direction`,
`--roles`, `--wait`, and separately fixes `--sink` (not gated by T005/T006,
verified by its own assertion below per plan.md Design section 1).

**Independent Test**: render `capture --sink --help` and diff against
`parse_destination`'s and `apply_option`'s match-arm literals; render
`capture --help` and confirm `--mode`, `--direction`, `--roles`, `--wait` each
show a default.

- [x] T010 [US2] In `crates/fragcap-cli/src/cli.rs`, rewrite the `sink` field's
  doc comment to name all seven schemes (`file:`, `pcapng:`, `jsonl:`, `pipe:`,
  `fifo:`, `unix:`, `tcp://`) and all six modifiers (`format`, `payload`,
  `rotate-size`, `rotate-duration`, `queue`, `timeout`), grouped for
  readability and fitting the 100-column wrap budget (per plan.md Design
  section 1).
- [x] T011 [P] [US2] **Superseded during verification, recorded rather than
  silently corrected.** Originally: change `mode`'s type from
  `Option<ModeArg>` to `ModeArg` and add `default_value_t = ModeArg::File`,
  matching `--scope`. Analyze's finding U1 correctly flagged that
  `default_value_t` needs a non-`Option` field, and that change was applied
  and compiled. But `cargo xtask ci`'s test phase then failed a pre-existing
  test, `assemble::tests::a_profile_declared_ring_mode_is_resolved_and_
  validated`: a profile's own `[capture] mode = "ring"` is meant to apply
  when `--mode` is omitted (`resolve_mode`'s original `None` arm), and
  collapsing `Option<ModeArg>` to `ModeArg` destroys the distinction between
  "user passed nothing" and "user explicitly asked for `file`" that this
  fallback depends on. **Reverted**: `mode` stays `Option<ModeArg>`, no
  `default_value_t`; its default is stated in prose instead ("Defaults to a
  profile-declared mode if one exists, else `file`"), and
  `assemble.rs::resolve_mode` keeps its original `None => profile...`
  fallback arm. `--scope` has no such profile-priority behavior to lose,
  which is why its own `default_value_t` was always safe.
- [x] T012 [P] [US2] In the same file, change the `direction` field on
  `CaptureArgs` (not `ExtcapArgs`'s separate `direction` field, which is
  unaffected) from `Option<Direction>` to `Direction` and add
  `default_value_t = Direction::Both`, same reasoning as T011.
- [x] T013 [US2] In `crates/fragcap-cli/src/assemble.rs`, update the
  `CaptureArgs`-path call site that read `args.direction.unwrap_or(Direction::Both)`
  (`:148`) to read `args.direction` directly (now infallible, since `direction`
  is `default_value_t`). The `ExtcapArgs`-path call site (`:225`, a separate
  field, never changed) is untouched. `resolve_mode`'s `match args.mode { ...
  None => ... }` is **not** touched (see T011's reversion): `mode` stays
  `Option<ModeArg>` and the `None` arm (falling back to a profile-declared
  mode) stays exactly as it was. Depends on T012 (not T011, which was
  reverted).
- [x] T014 [P] [US2] State `--roles`' default ("every role") in its doc
  comment's second sentence in `cli.rs`, verified against
  `Profile::default()`'s actual roles value before wording it.
- [x] T015 [P] [US2] State `--wait`'s no-timeout behavior in its doc comment in
  `cli.rs` ("Waits with no timeout" or equivalent, matching the real
  `None`-passthrough behavior in `assemble.rs`).
- [x] T016 [US2] Re-run T005 and T006; confirm both now pass for these four
  flags specifically (T005 may still show other pages if untouched; confirm
  those are addressed by Phase 6). Depends on T010 through T015.
- [x] T017 [US2] Add a regression assertion (in `cli_help.rs` or as an
  extension of an existing `args.rs` unit test) that reads `parse_destination`
  and `apply_option`'s own match-arm/error-message literals and asserts every
  one appears in the rendered `--sink` help text, so the help text and the
  parser cannot drift independently (per plan.md Design section 1). Depends on
  T010.

**Checkpoint**: `--sink`, `--mode`, `--direction`, `--roles`, `--wait` are all
individually verified accurate; T005/T006 pass for every flag they cover here.

---

## Phase 5: User Story 3 - The specification and the shipped grammar agree (P2)

**Goal**: FR-004. Turns T008 green.

**Independent Test**: diff `capture`'s short-flag set (from
`fragcap_cli::command()`) against `docs/fragcap-specification.md` section
17.2 and confirm they match.

- [x] T018 [US3] In `docs/fragcap-specification.md`, remove `-m, --mode` and
  `-q, --quiet` from the section 17.2 `capture` grammar block (lines 2577 and
  2591 as of `5a3862c`; verify line numbers before editing since prior edits
  in this slice may have shifted them), keeping the long forms and adding
  `[default: file]` to `--mode`'s row to match T011.
- [x] T019 [US3] Re-run T008; confirm it now passes. Depends on T018, T011.

**Checkpoint**: the specification makes no claim about the shipped grammar
that the grammar does not honor.

---

## Phase 6: User Story 5 - Global flags grouped; `capture`/`targets` carry examples; `steam list` names its route (P3)

**Goal**: FR-005 through FR-010. Turns the remainder of T005 green (the
`--json`/`--catalog-db`/`--local-db` short/long splits) and delivers the
concision findings.

**Independent Test**: read `capture --help`'s option order and confirm the
four target inputs are contiguous; read `capture --help` and `targets --help`
and confirm each carries a worked example; read `steam list`'s rendered
surface and confirm it names `targets add --steam <app_id>`.

- [x] T020 [P] [US5] In `crates/fragcap-cli/src/cli.rs`, split `Cli::json`'s
  doc comment into a one-line `-h` summary and the existing cross-command
  paragraph behind `--help` (FR-006), per plan.md Design section 5.
- [x] T021 [P] [US5] In `crates/fragcap-cli/src/cli.rs`, add
  `display_order = 1000` to `quiet`, `silent`, and `json` on `Cli` (FR-007),
  per plan.md Design section 6. **Not a `CaptureArgs` field reorder**:
  verified against the actual rendered `capture -h` that field order has no
  effect (clap's real sort key is `(display_order, flag-name)`, and a
  propagated global keeps the `display_order` it had on `Cli`, which ties
  with each subcommand's own early fields). Confirmed after the fix:
  `--target`/`--id`/`--process` are contiguous and the three globals cluster
  at the end of every subcommand's option list.
- [x] T022 [P] [US5] Split `--catalog-db` and `--local-db`'s doc comments into
  one-line `-h` summaries with the existing paragraphs behind `--help`
  (FR-008), per plan.md Design section 7. Two fields, in `CaptureArgs`.
- [x] T023 [P] [US5] Add an `Examples:` block (behind `--help`, after the
  blank-line split) to the `Capture` variant's doc comment in `cli.rs`,
  drawing invocations from specification section 9.1 or `README.md` verbatim
  rather than composing new ones (FR-009).
- [x] T024 [P] [US5] Add an `Examples:` block to the `Targets` variant's doc
  comment in `cli.rs` (FR-010), same sourcing constraint.
- [x] T025 [US5] Move the route sentence ("Register one as a capture target
  with `targets add --steam <app_id>`") from the non-rendering `SteamCommand`
  enum-level doc comment onto the rendering `SteamCommand::List` variant's doc
  comment in `cli.rs` (FR-005), per plan.md Design section 4.
- [x] T026 [US5] Re-run T005 (short-help-one-line) and T007
  (cross-reference); confirm both now pass across every page. Depends on
  T020 through T025.

**Checkpoint**: every page's short help is one line; `capture` and `targets`
each show a worked example; `steam list` names its own follow-up command.

---

## Phase 7: User Story 1 - Every finding has a recorded, verifiable disposition (P1)

**Goal**: FR-001. Depends on every prior phase's fixes being final, since the
record's content is a true report of what changed and what did not.

**Independent Test**: read the audit record against findings 4 through 15 and
confirm every one has exactly one disposition, each matching what the
rendered help actually says.

- [x] T027 [US1] Write `changelog.d/S070-help-accuracy-audit.decisions.md`
  with one entry per finding 4 through 15: fixed findings name the task that
  fixed them and the FR; closed findings (9, and re-verified 1/2/3/5/8 from
  S062) state the reason. Append the T009 "gate: before" section already
  captured, and add a "gate: after" section with the T005-T008 passing output.
- [x] T028 [US1] Add `changelog.d/S070-help-accuracy-audit.fixed.md`, the
  ordinary user-facing changelog fragment (one line per real fix, in the
  project's existing fragment style).
- [x] T029 [US1] Cross-read the audit record (T027) against the rendered
  output of every page named in it, confirming no disposition is aspirational
  (per the project's standing verification discipline: an unverified claim is
  worse than a known gap).

**Checkpoint**: the audit record exists, is reviewable per line, and every
line in it is true of the shipped binary.

---

## Phase 8: Polish & cross-cutting

- [x] T030 Run `cargo xtask ci` in the foreground, watched to completion.
- [x] T031 Re-run the existing `every_help_page_wraps_within_the_limit` and
  `no_help_page_leaks_internal_vocabulary` tests specifically (not just as
  part of T030) and confirm neither regressed, since this slice's fixes should
  not touch wrapping or vocabulary.
- [x] T032 Manually render `capture --help`, `capture -h`, `steam list
  --help`, and `targets --help` end to end and read them against the Evidence
  table in `spec.md`, confirming every FR is visibly satisfied in the
  rendered text and not only in the passing tests.
- [x] T033 Update `AGENTS.md` only if the audit record's location or any
  dependency fact needs recording there; expected to be a no-op (no new
  dependency).

---

## Dependencies

- Setup (T001) has no dependencies.
- Foundational (T002-T004) has no dependencies beyond T001 and must complete
  before any Phase-3 task.
- Phase 3 (US4, the gate tests) depends on Phase 2 and establishes the red
  baseline every later phase turns green; it does not depend on Phase 4/5/6.
- Phase 4 (US2) depends on Phase 3 existing (T016 re-runs T005/T006) but its
  fixes (T010-T015, T017) can be written in parallel with Phase 5's and
  Phase 6's fixes, since they touch disjoint doc comments and disjoint
  `assemble.rs` sites.
- Phase 5 (US3) depends on T011 (for the `[default: file]` annotation T018
  adds) but is otherwise independent of Phase 4's other flags and of Phase 6.
- Phase 6 (US5) is independent of Phase 4 and Phase 5's specific fixes (no
  shared field), but T026 re-runs T005/T007 and so should run after T020-T025.
- Phase 7 (US1) depends on Phases 3 through 6 all being complete, since the
  audit record's content is a report on their outcome.
- Phase 8 depends on everything before it.

## Parallel example

Within Phase 6, T020 through T025 touch disjoint doc comments in the same
file (`cli.rs`) and are safe to draft in parallel (as edits), but should be
applied as a single coherent set of changes to that one file rather than
landed as five separate commits, since they share a file and sequential
tool-level edits avoid conflicting diffs.

## Implementation strategy

**MVP scope**: Phase 3 (the four gate tests, red) plus Phase 4 (the `--sink`
and defaults fixes) delivers the sharpest, most-cited defects (findings 4 and
6) with their own gate turned green, and is independently valuable and
demonstrable on its own. Phases 5 and 6 are additive and can land in the same
slice (as planned) or, if time-boxed, be deferred with the audit record (T027)
noting them as open findings rather than closed ones, though the plan is to
complete all phases in this slice, since #183 is the last slice of the
campaign and deferring any finding would leave the campaign's own closing
issue open.
