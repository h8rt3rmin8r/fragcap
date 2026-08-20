# Implementation Plan: Agent context truthfulness

**Branch**: `061-agent-context-truth` | **Date**: 2026-08-20 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/061-agent-context-truth/spec.md`

## Summary

Rewrite the standing verification block in `AGENTS.md` so that no claim in it is
false, and replace the slice-numbered completion claim in `AGENTS.md` and
`CLAUDE.md` with a pointer to the authorities that do not go stale. Two files
change. No code, no tests, no specification.

The approach is prose surgery against a fixed evidence table, not authoring: the
block keeps its governing rule and its voice, its items are regrouped into
discharged and outstanding, and each surviving claim gains the dated evidence
that keeps it true. The single design decision, taken in clarification, is that
the rule survives and the items reorganize under it rather than the block being
renamed or split.

## Technical Context

**Language/Version**: Markdown prose. No compiled artifact.

**Primary Dependencies**: None. The evidence was gathered with `gh run list` at
plan time and is transcribed, not re-queried at build time (see OOS-001).

**Storage**: N/A

**Testing**: `cargo xtask lint`, which is the only mechanical gate that applies
to a prose change. It enforces UTF-8 without BOM, LF line endings, no trailing
whitespace, exactly one final newline, and no em-dashes or en-dashes. The
documentation linter and `cargo xtask ci` as a whole run unchanged.

**Target Platform**: N/A. `AGENTS.md` is read by any agent; `CLAUDE.md` imports
it for Claude Code.

**Project Type**: Repository governance and agent context.

**Performance Goals**: N/A

**Constraints**: `AGENTS.md` is the provider-agnostic source of truth and
`CLAUDE.md` imports it with `@AGENTS.md`. Anything asserted in either is loaded
into every agent session, so the cost of a wrong sentence here is paid by every
future session rather than once.

**Scale/Scope**: Two files. One block of roughly 45 lines rewritten in
`AGENTS.md`, one paragraph opening amended in `AGENTS.md`, one paragraph
amended in `CLAUDE.md`.

## Constitution Check

*GATE: passed before Phase 0. Re-checked after Phase 1 design; still passing.*

| Principle | Bearing | Verdict |
| --- | --- | --- |
| P-1 Passive Observation Only | No code changes; no handles, drivers, or hooks touched. | Not engaged |
| P-2 Core Stays Platform-Neutral | No crate changes. | Not engaged |
| P-3 Capture And Attribution Stay Separate | No crate changes. | Not engaged |
| P-4 No Silent Loss | The block's own accounting is the subject: a discharged claim must be replaced by what discharged it, never quietly deleted. The rewrite therefore states what changed, not just the new state. | **Satisfied by design** |
| P-5 Compatibility Outranks Richness | No output format changes. | Not engaged |
| P-6 Glossary First | No new term is introduced. "Discharged" is used in its ordinary sense within one block and is not project vocabulary. | Satisfied |
| P-7 Wrappers Stay Thin | No wrapper changes. | Not engaged |
| P-8 House Standards Apply | UTF-8 no BOM, LF, no em-dashes or en-dashes. Enforced by `cargo xtask lint`. | **Gated** |
| P-9 The Instrument Does Not Lie (NON-NEGOTIABLE) | This is the principle the slice exists to serve, applied to the instruction surface rather than to capture output. A report that misstates what was observed is a defect in either direction, and the current block errs toward underclaiming. | **Primary driver** |
| P-10 One Path To A Target | No target handling. | Not engaged |
| P-11 The Specification Describes What Shipped | `docs/fragcap-specification.md` is explicitly out of scope (OOS-002) and is not touched, so the version lock-step is undisturbed. | Satisfied |

No violations. Complexity Tracking is therefore empty and omitted.

## Project Structure

### Documentation (this feature)

```text
specs/061-agent-context-truth/
├── spec.md                    # the specification
├── plan.md                    # this file
├── tasks.md                   # produced by /speckit-tasks
└── checklists/
    └── requirements.md        # spec quality checklist
```

No `research.md`, `data-model.md`, `contracts/`, or `quickstart.md`. There is no
unknown to research (the evidence table in the spec is the research, and it is
already gathered and cited), no data model, no contract, and no quickstart for a
prose change. Generating empty scaffolds for them would be noise in the diff.

### Files changed (repository root)

```text
AGENTS.md                      # the standing verification block; the completion claim
CLAUDE.md                      # the completion claim only
changelog.d/S061-agent-context-truth.fixed.md      # feature fragment
changelog.d/S061-agent-context-truth.decisions.md  # the declined lint gate, and the framing call
```

**Structure Decision**: Two tracked prose files at the repository root, plus the
slice's own `specs/` artifacts and two `changelog.d/` fragments.
`CHANGELOG.md` is never edited from a feature branch (it is assembled from
fragments at release time), and `.specify/feature.json` is gitignored local
state and is never staged. Both rules are from `CONTRIBUTING.md` and
`AGENTS.md`.

## Design

### The rewritten block

One heading that states the rule instead of counting items, then two groups.

**Heading.** Replaces "Two things are scaffolded but not exercised, and must not
be reported as passing checks:". The replacement states the standing rule: a
check that has not run is not a check that passed, and neither is to be reported
as green until watched. It carries no count, which resolves FR-006 by removing
the thing that was wrong rather than by incrementing it.

**Group one, discharged, with what discharged it and when.** Six items:

1. `platform` and `audit` have run. `audit`: two scheduled runs, 2026-08-10 and
   2026-08-17, both green. `platform`: 85 runs, 79 green, most recent 2026-08-19
   green. This is the only item that also carries a caution, because
   `platform`'s green does not mean the Tier 2 steps inside it ran; see the live
   capture item.
2. The minimum-toolchain check runs for real (unchanged, still accurate, kept
   because it is the clearest illustration of the rule).
3. The npcap SDK acquisition step has run and the live source links (unchanged,
   still accurate).
4. `cargo deny` has run, through the `audit` workflow, on the two dates above.
5. Live capture has been executed **manually**, on a developer machine with
   npcap, on 2026-08-20: `fragcap capture --launch` against a Steam title, 16
   minutes, 18,234 packets captured, 16,427 written. Managed launch, stage
   matching, ETW process watch, socket-table attribution, kernel filter
   narrowing (engaging at t+22.5s), and graceful `terminal-stage-exited`
   shutdown all ran.
6. The socket table backend has run (kept; its final clause, which asserted live
   capture remains unexecuted, is removed as superseded).

**Group two, still outstanding, with what would discharge it.** One item:

- Live capture is **not** exercised in continuous integration. A runner with no
  npcap exits `STATUS_DLL_NOT_FOUND` before `main`, so Tier 2 tests do not run
  there, and the workflow says so rather than appearing green over nothing.
  Installing npcap on a runner is a licensing decision for the operator. What
  would discharge this is a runner with npcap installed, not another manual run.

Splitting live capture across both groups is deliberate and is the whole of
FR-003: the manual execution is a discharged claim, the CI coverage is an
outstanding one, and collapsing them in either direction produces a false
sentence. The spec's own edge case names the overcorrection to avoid.

### The completion claim

`AGENTS.md` currently opens "Current state" with "Slices S01 through S17 are
complete (S17, Steam integration and managed launch, integrates through the pull
request that carries this note)." `CLAUDE.md` carries the same claim in its own
words at line 29.

Both are replaced with a statement that names no slice number and routes the
reader to `specs/` and `changelog.d/`, which are already the recorded authority
for the per-slice narrative and are already cited two sentences later in
`AGENTS.md`. The surrounding architectural narrative, which declares itself
written "as of S11" and extended by those same records, is left untouched per
FR-008: it is honest about its own vintage, and rewriting it is a different and
much larger job than this slice.

The `CLAUDE.md` paragraph additionally enumerates what S01 through S17 built
("the workspace, the check set, ... Steam integration and managed launch"). That
enumeration stops at S17 and is the same perishable shape, so it goes with the
number; the sentences around it about `cargo xtask ci` and the fixture corpus
drift check are accurate and stay.

### What is deliberately not done

Issue #187 point 4, a `cargo xtask lint` rule asserting claims of the form "X
has never run", is declined. The reasoning is recorded in the spec at OOS-001
and repeated in a decisions fragment because it is a governance call a future
reader may want to revisit: the claims are about external forge history rather
than repository bytes, every other lint rule reads the working tree and nothing
else, and making the cheapest gate in the set require a network connection is a
worse trade than re-reading six sentences when a slice discharges one.

## Verification

`cargo xtask ci`, in the foreground, watched to completion. For a prose change
the load-bearing member is `cargo xtask lint`; the remainder must stay green to
prove nothing else moved.

Then a read-back against the spec's evidence table: each claim in the rewritten
block is checked against its row, and both files are searched for a surviving
slice-numbered completion claim.
