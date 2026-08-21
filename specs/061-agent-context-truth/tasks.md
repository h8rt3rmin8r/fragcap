---

description: "Task list for slice S061, agent context truthfulness"
---

# Tasks: Agent context truthfulness

**Input**: Design documents from `/specs/061-agent-context-truth/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md)

**Tests**: None. This slice changes prose in two governance files and adds no
code path. The applicable mechanical gate is `cargo xtask lint`, which already
runs over both files; adding a test that asserts the content of a sentence would
pin wording rather than behavior, and OOS-001 records why the one mechanical
gate the originating issue proposed was declined.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: can run in parallel (different files, no dependency)
- **[Story]**: US1 (the standing block) or US2 (the completion claim)

---

## Phase 1: Foundational

**Purpose**: pin the evidence the rewrite is checked against, so no sentence is
written from memory.

- [x] T001 Confirm the workflow run history with `gh run list --workflow=audit.yml`
  and `gh run list --workflow=platform.yml --limit 200`, and record the counts
  and dates. Expected: `audit` 2 scheduled runs (2026-08-10, 2026-08-17) both
  success; `platform` 85 runs, 79 success, 5 failure, 1 cancelled, most recent
  2026-08-19 success. A figure that does not match halts the slice rather than
  being written down as remembered.
- [x] T002 Re-read the block in place (`AGENTS.md`, the six bullets under "Two
  things are scaffolded but not exercised") and the two completion claims
  (`AGENTS.md` "Current state" opening, `CLAUDE.md` "S01 through S17 are
  complete") so the edit targets are exact and the surrounding text that must
  survive is known.

---

## Phase 2: User Story 1 - The standing block reports accurately (P1)

**Goal**: FR-001 through FR-006. No false claim, the rule survives, live capture
is split across discharged and outstanding, the count is gone.

**Independent test**: read the rewritten block and check every claim against the
evidence table in `spec.md`.

- [x] T003 [US1] Replace the block heading in `AGENTS.md` with one that states
  the standing rule (a check that has not run is not a check that passed) and
  carries no count. Satisfies FR-002 and FR-006.
- [x] T004 [US1] Rewrite the `platform` / `audit` item as discharged, naming the
  run counts and dates from T001. Satisfies FR-005.
- [x] T005 [US1] Rewrite the `cargo deny` item as discharged, naming the `audit`
  workflow and the two dates. Satisfies FR-005.
- [x] T006 [US1] Rewrite the live capture item as discharged **for the manual
  run only**: date, title class, duration, 18,234 captured, 16,427 written, and
  the subsystems observed to run. Satisfies FR-003 first half.
- [x] T007 [US1] Add the outstanding item stating that live capture is still not
  exercised in continuous integration, preserving all three unchanged facts
  (Tier 2 tests do not run in CI, a runner without npcap exits
  `STATUS_DLL_NOT_FOUND` before `main`, installing npcap on a runner is the
  operator's licensing decision) and naming what would discharge it. Satisfies
  FR-003 second half and FR-004.
- [x] T008 [US1] Trim the superseded final clause from the socket-table item
  ("This says nothing about live capture, which remains unexecuted for the
  reasons above"), keeping the rest of the item, which is still accurate.
  Satisfies FR-001 for the superseded row.
- [x] T009 [US1] Leave the minimum-toolchain and npcap-SDK items intact; confirm
  by reading that both are still true and that neither needs a date it does not
  have. Satisfies FR-001 for the two accurate rows.
- [x] T010 [US1] Group the items under the new heading into discharged and
  outstanding, in that order, per the clarification recorded in `spec.md`.

**Checkpoint**: every row of the spec's evidence table is answered by exactly
one place in the rewritten block, and nothing in it is false.

---

## Phase 3: User Story 2 - The completion claim stops going stale (P2)

**Goal**: FR-007 and FR-008. No slice-numbered completion claim in either file;
the architectural narrative untouched.

- [x] T011 [P] [US2] In `AGENTS.md`, replace the "Slices S01 through S17 are
  complete ..." opening of "Current state" with a statement naming no number and
  routing the reader to `specs/` and `changelog.d/`. Leave the following
  architectural narrative, which declares itself written as of S11, untouched.
  Satisfies FR-007 and FR-008.
- [x] T012 [P] [US2] In `CLAUDE.md`, replace the "S01 through S17 are complete:
  ..." bullet, including its enumeration that stops at S17, with the same
  non-perishable form. Keep the surrounding sentences about `cargo xtask ci` and
  the fixture corpus drift check, which are accurate. Satisfies FR-007.

**Checkpoint**: searching both files for a slice-numbered completion claim
returns nothing.

---

## Phase 4: Record and verify

- [x] T013 Write `changelog.d/S061-agent-context-truth.fixed.md`, the defect
  fragment.
- [x] T014 Write `changelog.d/S061-agent-context-truth.decisions.md` recording
  two calls: the declined `cargo xtask lint` gate over "X has never run" claims
  with its reasoning (OOS-001), and the decision to keep the block's governing
  rule while reorganizing its items rather than renaming or splitting the block.
- [x] T015 Run `cargo xtask ci` in the foreground and watch it to completion.
  The load-bearing member for this slice is `cargo xtask lint` (UTF-8 no BOM,
  LF, no trailing whitespace, single final newline, no em-dashes or en-dashes);
  the rest must stay green to prove nothing else moved. Satisfies FR-009 and
  SC-004.
- [x] T016 Read the rewritten block back against the evidence table in
  `spec.md`, claim by claim, and confirm SC-001, SC-002, SC-003, and SC-005.
- [x] T017 Stage only this slice's files and commit. Never stage
  `.specify/feature.json` (gitignored local state) and never edit `CHANGELOG.md`
  from a feature branch.

---

## Dependencies

- T001 and T002 block everything: no sentence is written before its evidence is
  confirmed.
- Phase 2 (T003 through T010) is one contiguous rewrite of one block in one
  file, so its tasks are sequential, not parallel.
- Phase 3 (T011, T012) touches two different files and is parallel with itself,
  and is independent of Phase 2 (different regions of `AGENTS.md`, different
  file for `CLAUDE.md`).
- Phase 4 follows both.

## Out of scope

Per `spec.md`: no code, no tests, no `docs/fragcap-specification.md` change, no
`cargo xtask lint` rule policing run-history claims, and no rewrite of the
architectural narrative that declares itself written as of S11.
