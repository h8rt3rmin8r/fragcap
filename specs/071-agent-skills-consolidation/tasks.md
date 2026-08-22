---
description: "Task list for slice S071, agent skills consolidation, closing issue #197."
---

# Tasks: Agent skills consolidation

**Input**: [spec.md](spec.md), [plan.md](plan.md)

**Prerequisites**: A clean tree on branch `071-agent-skills-consolidation`; `gh`
authenticated for the release download; `bash` and `pwsh` available for the
wrappers gate.

**Tests**: Unit tests accompany the new xtask module and cover its three
assertions plus the `speckit-` exclusion against constructed fixtures. Those
prove the function; they do not prove the wiring, so FR-011 additionally
requires each assertion be demonstrated failing against the **real** tree by a
deliberate, reverted regression. No test is added for the prose changes: the
applicable mechanical gate is `cargo xtask lint`, which already runs over every
file touched, and a test asserting the content of a sentence pins wording rather
than behavior. This is the posture S061 set for an instruction-surface slice.

## Format: [ID] [P?] [Story]

`[P]` marks tasks with no dependency on another unfinished task in the same
phase. `[USn]` names the user story the task serves.

## Phase 1: Foundational - pin the evidence before anything is written

Nothing in this slice is written from memory. A figure that does not match halts
the slice rather than being written down as remembered.

- [x] T001 Re-verify E-01 by reading `xtask/src/wrappers.rs:187,193,226-243`
      and confirming the `else` arm counts a failure rather than skipping.
- [x] T002 [P] Re-verify E-04 with `git check-ignore -v .agents/skills/debug`
      and confirm the directory is absent from `git ls-files`.
- [x] T003 [P] Re-verify E-07 and E-15: per-directory tracked file counts, and
      the lock entry count against the "Thirty-five skills" claim at
      `docs/plans/000-repository-foundation.md:91`.
- [x] T004 [P] Re-verify E-17: list `.claude/skills/` and `.cursor/skills/`, and
      confirm `git ls-files -s` reports no mode `120000` entry anywhere.
- [x] T005 Record the upstream release facts for E-19 and E-20: tag `v1.11.0`,
      commit `46ba297d`, published 2026-08-22T08:22:03Z, and the four archive
      digests, kept for the decisions fragment.

**Checkpoint**: Every Evidence row that the slice acts on has been seen this
session, not recalled.

## Phase 2: Vendor the four (US4)

**Goal**: The three standards P-8 binds, plus the workflow protocol, present at
current upstream and byte-identical to it.

**Independent Test**: Each vendored tree compared against its extracted archive
reports no differences.

Vendoring precedes deletion so the window in which the wrappers gate has no
checker never opens (plan.md §9).

- [x] T006 Download the four `v1.11.0` archives and `SHA256SUMS.txt`, and verify
      with `sha256sum -c` **before** extraction. A digest mismatch halts (FR-002).
- [x] T007 Check each of the four against constitution P-1 and record the check;
      `AGENTS.md` requires this of any vendored skill (FR-004).
- [x] T008 Replace `.agents/skills/shruggie-powershell/` with the extracted
      archive, unmodified. No hand-edit for hygiene (FR-003).
- [x] T009 [P] Replace `.agents/skills/shruggie-markdown/` the same way.
- [x] T010 [P] Replace `.agents/skills/shruggie-speckit/` the same way. Content
      is already current (E-13); this re-vendors for provenance.
- [x] T011 [P] Add `.agents/skills/shruggie-bash/`, closing the known gap that
      `skills/README.md` has carried unmet since before S18 (E-18).
- [x] T012 Confirm FR-003 and E-21 hold on what actually landed: no em-dash,
      en-dash, CRLF, or byte-order mark in any of the four trees.

**Checkpoint**: Run `cargo xtask wrappers` now, before anything is deleted. E-12
predicts it passes; this run confirms it on the real tree, and a failure here is
attributable to the refresh alone.

## Phase 3: Prune, and reconcile the three sources of truth (US1)

**Goal**: `.agents/skills/` holds exactly the four vendored skills plus the
CLI-owned `speckit-*` directories, and disk, git, and the lock agree.

**Independent Test**: `ls`, `git ls-files`, and `skills-lock.json` each report
the same four names.

- [x] T013 Remove the 33 dropped directories from `.agents/skills/`, enumerated
      in spec.md's Drop list (FR-005).
- [x] T014 Remove their 33 entries from `skills-lock.json`.
- [x] T015 Set `source` to `shruggietech/skills@v1.11.0` on the four surviving
      entries. The four-field schema gains and loses nothing (FR-006).
- [x] T016 Recompute `computedHash` for the four using the algorithm stated in
      plan.md §3, and record in the decisions fragment both the algorithm and
      what would falsify it (FR-007, CHK011).
- [x] T017 Confirm the reconciliation (FR-001): 14 directories, 4 non-`speckit-*`, 4 lock
      entries, and no file present-but-untracked or tracked-but-absent (SC-001,
      SC-002).

**Checkpoint**: The tree is consolidated but not yet defended.

## Phase 4: The structural gate (US3)

**Goal**: Disk, git, and the lock cannot disagree again without a check failing.

**Independent Test**: Each of the three disagreements introduced in turn makes
the gate fail for its own reason; reverting makes it pass.

- [x] T018 Add `xtask/src/skills.rs` with `run(&Path) -> io::Result<usize>` carrying
      the three assertions of FR-008 and no hash check (FR-009, OOS-003),
      modelled on `xtask/src/wrappers.rs`. Assertion 3 reads
      `git ls-files -- .agents/skills`; git being unavailable returns `Err`, so
      the caller exits 2 and the check can never degrade to a pass (CHK016).
- [x] T019 Exclude CLI-owned directories by the `speckit-` name prefix rather
      than an enumerated list that would go stale (CHK017, OOS-001).
- [x] T020 Add unit tests over constructed fixtures for all three assertions and
      the `speckit-` exclusion.
- [x] T021 Wire it up in `xtask/src/main.rs`: `mod skills;`, a `"skills"`
      dispatch arm modelled on the `"wrappers"` arm, a `ci` step, and a `USAGE`
      line, each placed adjacent to `wrappers` (FR-010).
- [x] T022 Add the `ci.yml` step mirroring line 76. `.github/workflows/**` is a
      pinned artifact, so this is what makes T032's decisions fragment mandatory
      rather than optional.
- [x] T023 Demonstrate each of the three assertions failing against the **real**
      tree via a deliberate, reverted regression, then confirm it passes
      (FR-011, SC-006). A fixture test proves the function; this proves the
      wiring.

**Checkpoint**: The gate has been seen red for three distinct real reasons and
green once.

## Phase 5: The instruction surface (US2)

**Goal**: Four files stop asserting a repository that does not exist.

**Independent Test**: Every rewritten claim checks out against the tree on a
fresh read.

- [x] T024 `.gitignore` line 3: `debug` becomes `/debug`. The latent bug outlives
      the dropped skill, so the prune does not make this moot (FR-012).
- [x] T025 `skills/README.md`: state the admission test, add the removal
      procedure mirroring the existing add procedure, name the single upstream,
      and delete both the generic-name-collision paragraph and the expired
      known gap, neither of which describes this repository any more (FR-013,
      CHK021).
- [x] T026 [P] `AGENTS.md` Skills section: policy summary only, pointing at
      `skills/README.md` for procedure. Stop asserting a symlink mechanism this
      checkout does not have (FR-014, E-17).
- [x] T027 [P] `CLAUDE.md` skills paragraph: same correction, same discipline.
- [x] T028 [P] `docs/plans/000-repository-foundation.md:91`: correct the count
      and name what superseded the original vendoring decision. It stays a
      record of a 2026-08-06 decision, not a description of today (FR-015).

**Checkpoint**: The instruction surface is true.

## Phase 6: Record, verify, commit

- [x] T029 [P] `changelog.d/S071-agent-skills-consolidation.removed.md`: the 33
      dropped, and the admission test that now governs the set.
- [x] T030 [P] `...changed.md`: the two refreshed standards, the newly vendored
      Bash standard, and the new gate.
- [x] T031 [P] `...fixed.md`: the `.gitignore` exclusion, the untrue symlink
      claim, and the stale count.
- [x] T032 `...decisions.md`, dated 2026-08-22: the single-upstream rule; why
      vendored content is never hand-edited; the three declined in-brand skills;
      the hash algorithm, its confidence, and its falsifier; the three
      long-standing lock divergences resolved and by what mechanism (FR-016);
      and **the `ci.yml` pinned-artifact change**, which the constitution
      requires be recorded here.
- [x] T033 Every fragment opens `<!-- spec-impact: none -->`; no specification
      section describes this mechanism (OOS-002).
- [x] T034 File the OOS-004 follow-up issue: wire the vendored Bash checker into
      `cargo xtask wrappers` to replace the hand-rolled `check_bash`, attaching
      the 2026-08-22 measurement that `scripts/fragcap.sh` already passes it
      cleanly, ShellCheck included.
- [x] T035 Run `cargo xtask ci` in the foreground, watched to completion (SC-004),
      confirming `wrappers` passes with the refreshed checker (SC-005).
- [x] T036 Structural read-back per plan.md's Verification section, plus the
      byte-level comparison of each vendored tree against its archive (SC-003),
      and the tracked-file count landing at 42 (SC-008).
- [x] T037 Manual read-back of every rewritten sentence against the Evidence
      table, at commit time rather than at spec-writing time (SC-007). A claim
      that no longer holds is corrected in place.
- [x] T038 Commit. Stage only this slice's files. Never stage
      `.specify/feature.json`; never edit `CHANGELOG.md` from a branch.
- [x] T039 Halt before push with the breakdown the autopilot protocol requires.

## Dependencies

- Phase 1 gates everything: no prose before the evidence is re-seen.
- Phase 2 precedes Phase 3 (plan.md §9). The wrappers checkpoint sits between
  them so a regression is attributable.
- Phase 4 depends on Phase 3: the gate asserts the consolidated state, so
  writing it first would mean writing a check against a tree that fails it.
- Phase 5 is independent of Phase 4 and could run in parallel, but T025 and
  T026 describe the state Phase 3 produces, so they follow it.
- T023 depends on T021 and T022; the regression is against the wired gate.
- Phase 6's fragments depend on every preceding phase, because they state what
  actually happened rather than what was planned.

## Parallel example

T002, T003, T004 in Phase 1. T009, T010, T011 in Phase 2, once T008 has
established the replace-in-place pattern. T026, T027, T028 in Phase 5. T029,
T030, T031 in Phase 6.

## Implementation strategy

The MVP is Phases 1 through 3: the prune and the re-sourcing close issue #197 on
their own. Phase 4 is what stops it recurring, and is the reason the slice is
worth a gate rather than a commit. Phase 5 is the S061-class correction the
audit surfaced along the way, and is separable if it has to be.

If any phase has to be dropped for time, drop Phase 5 and file it, never Phase 4:
a prune with no gate re-rots, and E-04 is the proof: that skill went uncommitted for the entire life of the project and no person and no check noticed.
