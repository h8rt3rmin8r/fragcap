---

description: "Task list for slice S062, the help surface"
---

# Tasks: Help surface, wrapping, vocabulary, and accuracy

**Input**: Design documents from `/specs/062-help-surface/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md)

**Tests**: Required, and written before the text they police. The guard must be
observed failing against the current doc comments before the scrub turns it
green; a guard that has never been red has not been shown to work, which is the
whole lesson of #178 against #67.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: can run in parallel (different files, no dependency)
- **[Story]**: US1 wrapping, US2 vocabulary, US3 `--launch` accuracy,
  US4 `targets list` accuracy, US5 the guard

---

## Phase 1: Foundational (complete in Phase 0 research)

- [x] T001 [US1] Add `wrap_help` to the clap feature list in `Cargo.toml`,
  leaving the `=4.5.32` pin untouched. **Done and measured**: `Cargo.lock` delta
  is exactly one package, `terminal_size v0.4.4`. Satisfies FR-023 in part.
- [x] T002 [US1] Add `max_term_width = 100` to the root `#[command(...)]` in
  `crates/fragcap-cli/src/cli.rs`. **Done and measured**: lines over 100 columns
  across all pages went from 82 to 0; `COLUMNS=60` still shrinks to 60.
  Satisfies FR-001, FR-002, FR-003.

---

## Phase 2: User Story 5 - The guard (P1, and it comes first)

**Goal**: FR-017 to FR-022. Written against the current text so it fails, then
kept as the acceptance test for Phases 3 and 4.

- [x] T003 [US5] Add `pub fn command() -> clap::Command` to
  `crates/fragcap-cli/src/lib.rs`, returning the clap command tree through
  `CommandFactory`. One function rather than making `mod cli` public, so the
  exported surface is one item and not 757 lines of argument structs. Satisfies
  FR-017 in part.
- [x] T004 [US5] Measure whether adding `regex` as a dev-dependency of
  `fragcap-cli` changes `Cargo.lock`. It is already a runtime dependency of
  `fragcap-profile`, so it should add no package. If it does add one, hand-roll
  the matcher instead, consistent with the glob matcher and the pcapng writer.
  Record the measured answer either way.
- [x] T005 [US5] Rewrite `crates/fragcap-cli/tests/cli_help.rs`: walk the
  command tree from T003, skip clap's generated `help`, and build the page set.
  Assert per page that `--help` exits 0 (FR-021), that no line exceeds 100
  columns (FR-019), that no leak pattern matches (FR-018, FR-018a), and that the
  parser-internals tokens are absent (FR-020). Delete the three hand-picked
  page calls and the hardcoded token list.
  **The leak match runs over the whole page with whitespace normalized to single
  spaces, never line by line** (FR-018b). Line-based matching is defeated by the
  wrapping this slice turns on: `extcap --help` splits `specification section` /
  `14.5` across a line break and a per-line scan reports it clean. The width
  assertion stays per line, because that one is about lines.
- [x] T006 [US5] Confirm the guard fails, and read its output: it must name the
  pages and the offending text. Expected at this point: **15** pages leaking and
  0 width failures, Phase 1 having already fixed the widths. 15 rather than 14
  is the check on FR-018b: a whole-page normalized match sees the `extcap`
  `section 14.5` that wrapping split across two lines, and a line-based one does
  not. A guard that reports 14 here has the bug the gate was built to prevent. A
  guard that passes here has been written wrong.
- [x] T007 [US5] Add the source-side rule to `xtask/src/lint.rs`, scoped by path
  to `crates/fragcap-cli/src/cli.rs` and to `///` lines, following the existing
  path-scoped `FORBIDDEN_CALLS` shape in `run()`. Add unit tests beside the
  existing rule tests. Satisfies FR-022.
- [x] T008 [US5] Confirm `cargo xtask lint` now fails on the unscrubbed
  `cli.rs`, for the same reason and naming the same lines.

**Checkpoint**: two independent gates are red for exactly the reason the slice
exists. Neither has been satisfied by editing the gate.

---

## Phase 3: User Story 2 - The vocabulary scrub (P1)

**Goal**: FR-005 to FR-009. Turns T006 and T008 green.

- [x] T009 [US2] Strip `slice S0NN` and the bare `(S0NN)` form from every doc
  comment in `cli.rs` (24 sites, listed by the enumeration in `spec.md`). Where
  the provenance is useful to a maintainer, restate it as a `//` comment above
  the item, which clap does not read. Satisfies FR-005 and FR-008.
- [x] T010 [US2] Strip `section N.N` (`capture` 17.2, `extcap` 14.5) and
  `Appendix B` (`catalog`, `catalog seed-signatures`). Satisfies FR-006.
- [x] T011 [US2] Replace `Tier 1` with "the title tier" and `Tier 3` with "the
  engine tier" on `catalog`, `catalog seed`, and `catalog seed-engine`. Do not
  define the numbering; S063 removes these verbs. Satisfies FR-007.
- [x] T012 [US2] Reword the `` `net` feature `` strings on `catalog` and
  `catalog update` to describe capability rather than a build switch. The
  subcommand itself stays until S063 (OOS-003). Satisfies FR-006.
- [x] T013 [US2] Insert a blank `///` line after the first sentence of every
  single-paragraph doc comment, so clap takes a real one-line `-h` summary and
  keeps the paragraph for `--help`. Worst offenders: `catalog seed-signatures`,
  the global `--json`, `capture --catalog-db`, `capture --local-db`. Satisfies
  FR-009.
- [x] T014 [US1] Add the source comment at the `help_template` literal recording
  that the `Commands:` block is hand-budgeted and does not wrap, its lines being
  76 columns today. Satisfies FR-004.

**Checkpoint**: T006 and T008 are green. The guard was not touched to make them
so.

---

## Phase 4: User Story 3 and 4 - The accuracy corrections (P1, P2)

- [x] T015 [US3] Reword `--launch` (`cli.rs:259`) to describe the stored target:
  Windows only, the target must be Steam anchored, register one with
  `targets add --steam <app_id>`. Satisfies FR-010.
- [x] T016 [US3] Put the integer-namespace rule on `--target` and the positional
  `SELECTOR`, and keep it on `--id`. Satisfies FR-011.
- [x] T017 [US3] Add one shared constructor for the numeric no-match message and
  call it from all four sites (`target_resolve.rs:117`, `targets.rs:373`,
  `:416`, `:838`), so a fifth site cannot drift. It names the row-index
  interpretation, the snapshot row count, and the `targets add --steam` route.
  The non-numeric case keeps today's message. Satisfies FR-012.
- [x] T018 [US3] Add the assertion to `crates/fragcap-cli/tests/cli_targets.rs`:
  a numeric selector with no matching row produces a message naming the
  interpretation and the listing size. Satisfies SC-006.
- [x] T019 [P] [US4] Reword the `targets list` summary (`cli.rs:358`) to name
  the four columns `render_table` prints and to state that the command registers
  newly discovered titles. Satisfies FR-013.
- [x] T020 [P] [US4] Reword `list --db` so it does not describe the store as
  read-only, and point at `targets show` or `targets export` for the durable
  identifier. Satisfies FR-014 and FR-015.
- [x] T021 [P] [US3] Update `site/content/docs/reference/cli.mdx:61` to the new
  `--launch` sentence, resolving its contradiction with `:43`. Satisfies FR-016.

---

## Phase 5: Record and verify

- [x] T022 Add the `terminal_size` row to the `AGENTS.md` dependency inventory,
  naming S062 and the reason, in the table's existing form. Satisfies FR-023.
- [x] T023 Write `changelog.d/S062-help-surface.fixed.md`.
- [x] T024 Write `changelog.d/S062-help-surface.decisions.md` recording the
  dependency addition with its measured lock delta, and the FR-018a decision to
  match the feature-naming phrase rather than the declared feature names.
- [x] T025 Run `cargo xtask ci` in the foreground, watched to completion.
- [x] T026 Run `cargo xtask msrv` in the foreground. It is not part of `ci`, and
  this slice can break it: clap is non-optional in `fragcap-cli`, so
  `terminal_size` is compiled under the 1.82 floor. Satisfies FR-024.
- [x] T027 Re-run the page enumeration against the rebuilt binary at
  `COLUMNS=400`, `100`, and `60`, matching leaks over whole normalized pages.
  Confirm SC-001 (82 overflowing lines to 0), SC-002 (15 leaking pages to 0),
  and SC-003.
- [x] T028a Inject a doc comment carrying a deliberate leak into `cli.rs`,
  confirm that both the guard test and `cargo xtask lint` fail on it with no
  edit to either gate, then remove it. This is the only task that proves the
  gates still bite *after* the scrub; T006 and T008 prove they bit before it,
  which is a different claim. Satisfies SC-004.
- [x] T028b Read `fragcap targets list --help` back against the columns
  `render_table` actually prints and against what `hero_listing` writes.
  Satisfies SC-007.
- [x] T028 Run the #181 invocation and read the new error, confirming SC-005 and
  SC-006 against real output rather than against a test assertion alone.
- [x] T029 Stage only this slice's files and commit. Never stage
  `.specify/feature.json`; never edit `CHANGELOG.md` from a feature branch.

---

## Dependencies

- Phase 1 is complete (Phase 0 research) and unblocks everything, because every
  later assertion is made against rendered output.
- Phase 2 must precede Phases 3 and 4: the guard is written to fail first.
  T003 blocks T005; T004 blocks T005; T005 blocks T006; T007 blocks T008.
- Phase 3 turns T006 and T008 green. T009 to T013 all edit `cli.rs` and are
  sequential with each other.
- Phase 4: T015, T016 edit `cli.rs` (sequential with Phase 3 and each other).
  T017 and T018 are the error path. T019 and T020 edit `cli.rs`. T021 edits the
  site page and is genuinely parallel.
- Phase 5 follows everything.

## Out of scope

Per `spec.md`: `capture --steam` as a fourth target input (OOS-001), whether a
read-only `targets list` should exist (OOS-002), compiling `catalog update` out
and the catalog-refresh product decision (OOS-003, slice S063), the #183
accuracy audit (OOS-004), and the nine required store-path flags (OOS-005,
slice S063).
