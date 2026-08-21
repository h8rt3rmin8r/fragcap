# Feature Specification: Agent context truthfulness

**Feature Branch**: `061-agent-context-truth`

**Created**: 2026-08-20

**Status**: Implemented

**Input**: Correct the stale "scaffolded but not exercised" block in `AGENTS.md`
and the perishable completion claim in `AGENTS.md` and `CLAUDE.md`. Closes
issue #187.

## Context

`AGENTS.md` line 495 opens a block with a standing prohibition:

> Two things are scaffolded but not exercised, and must not be reported as
> passing checks:

Four of the six bullets under it are false as of 2026-08-20, including the one
an agent is most likely to repeat verbatim: "Live capture has still never
executed."

This is not an ordinary stale comment. `AGENTS.md` is the canonical
agent-neutral instruction file; `CLAUDE.md` imports it in full; and this block
is phrased as an instruction rather than as narrative. Every agent session loads
it and is actively directed to assert things that are no longer true. The block
exists to stop agents overclaiming. It now makes them underclaim with equal
confidence, which damages exactly the thing it was built to protect.

Constitution P-9 (The Instrument Does Not Lie) is the principle in play, applied
to the project's own instruction surface rather than to capture output: a report
that misstates what was observed is a defect regardless of which direction it
errs in.

## Evidence

Gathered 2026-08-20 against the live repository through `gh run list`.

| Claim in the block | Status | Evidence |
| --- | --- | --- |
| `platform` and `audit` "still have not [run]" | **stale** | `audit`: 2 scheduled runs, 2026-08-10 and 2026-08-17, both success. `platform`: 85 runs, 79 success, 5 failure, 1 cancelled, most recent 2026-08-19 success. |
| "The minimum-toolchain check now runs for real" | accurate | unchanged |
| "The npcap SDK acquisition step has now run, and the live source links" | accurate | unchanged |
| "Live capture has still never executed" | **stale** | 2026-08-20: `fragcap capture --launch` against a Steam title, 16 minutes wall clock, 18,234 packets captured and 16,427 written to a pcapng, on a developer machine with npcap installed. Managed launch, stage matching, ETW process watch, socket-table attribution, kernel filter narrowing (observed engaging at t+22.5s), and graceful `terminal-stage-exited` shutdown all ran. |
| "`cargo deny` has never run" | **stale** | The `audit` workflow owns it and ran green on 2026-08-10 and 2026-08-17. |
| "the socket table backend has run ... This says nothing about live capture, which remains unexecuted" | **stale in its last clause** | superseded by the live capture row |

Two adjacent staleness signals in the same two files:

- The header undercounts its own list. "Two things" introduces six bullets, and
  has said "two" since it had two.
- `AGENTS.md:46` and `CLAUDE.md:29` both claim "S01 through S17 are complete".
  `specs/` currently runs to `060-installer-npcap`, so the agent context files
  understate the project by roughly forty slices. `AGENTS.md` anticipates
  narrative drift ("the architectural summary below is written as of S11 and is
  extended by those records rather than rewritten here every slice"), which is a
  reasonable policy for the architecture narrative but does not extend to a
  numbered completion claim that a reader will take literally.

## Clarifications

### Session 2026-08-20

Both questions below were resolved under the autopilot decision policy: the
alternatives were enumerated, evaluated against the constitution (P-9), the
originating issue, and the slice's scope lines, and the best-supported option
was taken and recorded here rather than raised.

- Q: The block's own heading, "Two things are scaffolded but not exercised", is
  now inaccurate as a label and not only as a count, since four of its six items
  have been exercised. Does the block keep that framing, get renamed, or split?
  -> A: **Keep the standing rule, reorganize the items under it into discharged
  and outstanding.** FR-002 requires the governing framing to survive, and issue
  #187 is explicit that "the instruction to distinguish 'a check that did not
  run' from 'a check that passed' is the durable part and is worth keeping even
  when every bullet under it has been discharged". Renaming it to a status
  report would drop the instruction; splitting it into two blocks would separate
  the rule from the evidence that makes it concrete. Reorganizing under one
  heading also dissolves the count problem in FR-006, because a rule needs no
  count.
- Q: What non-perishable form replaces "S01 through S17 are complete"? -> A:
  **Point at `specs/` and `changelog.d/` as the authority and name no number.**
  Naming the current slice reintroduces the same defect one slice later; an "as
  of DATE" range is still a number a reader will quote without the date. Both
  directories are already the recorded authority for the per-slice narrative,
  and `AGENTS.md` already routes readers to them for everything after S11, so
  this extends an existing convention instead of inventing one.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - An agent session reports the project's state accurately (Priority: P1)

An agent starts a session, loads `CLAUDE.md`, which imports `AGENTS.md` in full,
and is asked whether live capture has ever been demonstrated. Today the loaded
instruction directs it to answer "no, and it must not be reported as a passing
check". The correct answer is that live capture executed manually on a developer
machine with npcap on 2026-08-20, and that it is still not exercised in
continuous integration.

**Why this priority**: This is the whole defect. Every other item in this slice
is a secondary staleness signal found in the same two files.

**Independent Test**: Read the rewritten block and check each claim against the
evidence table above. Every claim either names its discharging evidence or names
what keeps it true.

**Acceptance Scenarios**:

1. **Given** the rewritten `AGENTS.md`, **When** a reader looks for the state of
   live capture, **Then** they find that it executed manually on a developer
   machine with npcap on 2026-08-20, and that it is still unexercised in
   continuous integration, stated as two distinct facts.
2. **Given** the rewritten block, **When** a reader looks for the state of the
   `platform` and `audit` workflows, **Then** they find both have run and both
   are green, with the dates.
3. **Given** the rewritten block, **When** a reader looks for `cargo deny`,
   **Then** they find it has run, through the `audit` workflow, on the dates
   named.
4. **Given** the rewritten block, **When** a reader looks for its governing
   instruction, **Then** the distinction between a check that did not run and a
   check that passed is still stated and still binding.

---

### User Story 2 - The completion claim stops going stale (Priority: P2)

A reader of either agent context file wants to know how far the project has
progressed. Today both files say "S01 through S17 are complete", which was true
roughly forty slices ago and will be wrong again one slice after any number is
written.

**Why this priority**: Real, and misleading, but a reader who follows the
existing pointer to `specs/` recovers immediately. The live capture claim has no
such escape hatch, because it is phrased as a prohibition.

**Independent Test**: Search both files for a slice-numbered completion claim.
There is none, and what replaces it names `specs/` and `changelog.d/` as the
authority.

**Acceptance Scenarios**:

1. **Given** either agent context file, **When** a reader looks for the
   project's completion state, **Then** they are pointed at `specs/` and
   `changelog.d/` rather than given a number.
2. **Given** a future slice landing, **When** nothing in this slice's changed
   text is edited, **Then** the completion statement is still true.

---

### Edge Cases

- **What happens when a future reader assumes the block is retired because every
  bullet has been discharged?** The instruction to distinguish "a check that did
  not run" from "a check that passed" is the durable part and must survive even
  when no bullet under it is outstanding. The rewrite keeps the framing as a
  standing rule, not as a list that empties out.
- **What happens if the live capture entry is overcorrected?** "Live capture is
  covered" would be a new false claim in the opposite direction. Tier 2 tests
  still do not run in continuous integration, a runner with no npcap still exits
  `STATUS_DLL_NOT_FOUND` before `main`, and installing npcap on a runner is
  still a licensing decision for the operator. The workflow still declines to
  appear green over nothing, which is correct and stays.
- **What happens to the architecture narrative written "as of S11"?** It is left
  alone. It already declares its own vintage and its own extension mechanism.
  Only the numbered completion claim is in scope.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The `AGENTS.md` block MUST contain no claim that is false as of
  the date this change lands, and each surviving claim MUST name the evidence
  that keeps it true.
- **FR-001a**: The block MUST distinguish a claim about something **observed
  once** (which carries a date, and is untrustworthy without one) from a claim
  about how a check **behaves** (which is invariant, carries no date, and names
  how to see the behavior instead). Review of PR #188 found the first draft
  asserting that every claim needs a date while three entries beneath it
  correctly had none, which is the internal inconsistency this splits.
- **FR-001b**: The pointer to the authoritative record MUST name `CHANGELOG.md`
  for released history and `changelog.d/` only for unreleased. `cargo xtask
  changelog --release` consumes the fragments and deletes them, so a reader sent
  to `changelog.d/` alone to find a landed slice usually finds nothing.
- **FR-002**: The block MUST retain its governing framing: the distinction
  between a check that did not run and a check that passed is a standing rule,
  and MUST NOT be presented as a list that is retired once emptied. Its items
  MUST be organized as discharged (with the evidence and date that discharged
  them) and outstanding (with what would discharge them), under one heading that
  states the rule rather than counting the items.
- **FR-003**: The live capture entry MUST state that live capture executed
  manually on a developer machine with npcap, with the date and the run's
  measured figures, AND MUST separately state that it is still not exercised in
  continuous integration. It MUST NOT imply the second from the first.
- **FR-004**: The live capture entry MUST preserve the three facts that did not
  change: Tier 2 tests do not run in continuous integration, a runner with no
  npcap exits `STATUS_DLL_NOT_FOUND` before `main`, and installing npcap on a
  runner is a licensing decision for the operator.
- **FR-005**: The `platform`, `audit`, and `cargo deny` claims MUST be replaced
  with what discharged them and when.
- **FR-005a**: The outstanding live-capture item MUST NOT state that installing
  npcap on a runner would discharge it. `crates/fragcap-capture/tests/live.rs`
  prints a reason and returns when the environment is absent, because Rust's
  harness has no skip, so a test that declined to run still passes and a green
  Tier 2 step can mean either that capture happened or that nothing did.
  Discharging it needs npcap present, loopback capture support, sufficient
  privilege, and the test output read. Found in review of PR #188.
- **FR-006**: The block header MUST NOT state a count that disagrees with the
  number of items beneath it. Either the count is correct or there is no count.
- **FR-007**: Neither `AGENTS.md` nor `CLAUDE.md` MUST carry a slice-numbered
  completion claim. Both MUST instead point at `specs/` and `changelog.d/` as
  the authority for what has landed.
- **FR-008**: The architectural narrative in `AGENTS.md` that declares itself
  written "as of S11" MUST NOT be rewritten. Only the numbered completion claim
  is in scope.
- **FR-009**: Both files MUST remain UTF-8 without BOM, LF-terminated, and free
  of em-dashes and en-dashes, so `cargo xtask lint` stays green.

### Out of scope

- **OOS-001**: Issue #187 point 4 proposes a `cargo xtask lint` gate over claims
  of the form "X has never run". **Declined, with the reason recorded here.**
  Such claims are assertions about external continuous-integration run history,
  not about repository content. A lint rule that checked them would have to
  query the forge over the network from a check that is meant to be hermetic,
  offline-runnable, and deterministic; every other `cargo xtask lint` rule
  (`OpenProcess`, the pcap transmit calls, BOM, CRLF, dashes, SPDX) reads bytes
  in the working tree and nothing else. Adding a network dependency to the
  cheapest gate in the set to police six sentences is a poor trade, and it would
  make `cargo xtask ci` fail without a network connection. Issue #187 itself
  concedes the point ("If that is too much machinery for the payoff, say so and
  close this point; the rewrite is still worth doing on its own").
- **OOS-002**: No code changes, no test changes, no changes to
  `docs/fragcap-specification.md`.
- **OOS-003**: The behavior of the `platform` workflow, the `audit` workflow,
  and the Tier 2 test gating are unchanged. This slice reports on them; it does
  not alter them.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Every one of the claims in the rewritten block can be checked
  against a named, dated piece of evidence, and none of them is false.
- **SC-002**: A reader asked "has live capture been demonstrated?" answers "yes,
  manually, on 2026-08-20, on a machine with npcap" and, asked "is it covered by
  continuous integration?", answers "no", from the same paragraph.
- **SC-003**: Searching both agent context files for a slice-numbered completion
  claim returns nothing.
- **SC-004**: `cargo xtask ci` is green.
- **SC-005**: The change touches exactly two files, `AGENTS.md` and
  `CLAUDE.md`, plus the slice's own `specs/` artifacts and two `changelog.d/`
  fragments, one for the feature and one recording the two decisions.

## Assumptions

- The workflow run history read on 2026-08-20 is the authoritative record of
  whether a check has executed. It was read through `gh run list` against
  `h8rt3rmin8r/fragcap` and is reproducible.
- The 2026-08-20 live capture run is recorded in this repository's memory and in
  issues #184, #185, #186, and #187, all of which were filed from it. It is
  treated here as an established fact rather than re-executed, since re-running
  it requires npcap, a Steam title, and sixteen minutes, and would demonstrate
  nothing the recorded run did not.
- Dated claims will age. The rewrite reduces the rate at which they age by
  attaching evidence to each, but it does not eliminate the class; OOS-001
  records why the mechanical alternative was rejected.
