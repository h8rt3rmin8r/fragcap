# Build-Phase Autopilot Protocol (generalized)

Protocol version: 1.0.0
Derived from: the ShruggieGraph Build-Phase Autopilot Protocol (v1.2.0)

This is the operating procedure for running a spec-kit feature slice under
autopilot in any spec-kit project. It is project-agnostic: where the source
protocol named a specific toolchain, constitution numbering, or slice range,
this document says to discover that from the project at runtime. Where this
document and a project's own constitution or governance appear to conflict, the
project's constitution wins.

## Purpose

The default agent behavior pauses for authorization between each spec-kit step
and raises routine decisions that, in practice, are approved as recommended.
Autopilot removes that friction: one verbal kickoff runs a full slice end to
end, the agent makes the routine decisions itself and records them, and the
agent halts once, right before the push, with a breakdown for review.

## Trigger

The operator starts an autopilot run with a verbal kickoff naming the slice, the
next slice, or the feature, for example:

- "Kick off S09"
- "Run the next slice"
- "Autopilot the next slice"
- "Run S044 under autopilot"
- "Autopilot this feature"

On trigger, run the entire sequence below without pausing for inter-step
authorization. An explicit autopilot request is itself the authorization for the
named work; it does not depend on the slice falling in any predefined range.

## Preconditions

Before running the sequence, confirm setup and discover project specifics.

- Spec-kit is initialized: a `.specify/` directory exists and the project
  exposes `/speckit-*` command skills (or the dotted `/speckit.*` form). If not,
  halt and direct the operator to initialize spec-kit; do not invent the
  commands. See `speckit-reference.md`.
- Command form: detect hyphenated (`/speckit-plan`) versus dotted
  (`/speckit.plan`) and use one form consistently for the whole run.
- Constitution: read `.specify/memory/constitution.md` if it exists. It governs
  every decision. Absent one, fall back to the project's CLAUDE.md/AGENTS.md and
  architecture docs.
- Verification commands: discover the project's CI-parity checks (format, lint,
  test) from CLAUDE.md, the CI workflow under `.github/workflows/**`, or build
  docs. Run what the project uses; do not assume a toolchain.
- Branch model: determine trunk-based versus feature-branch and follow it at
  commit time.

## Per-slice sequence

Run these steps in order, with no halt between them:

1. `specify` creates the feature directory, `spec.md`, and the requirements
   checklist.
2. `clarify` runs under the decision policy below. Answer clarification
   questions yourself from the spec, the constitution, the architecture of
   record, and the slice's scope lines ("Scope in", "Scope out", "Done when").
   Escalate only genuinely unanswerable questions.
3. `checklist` adds domain checklists where the slice warrants them.
4. `plan` produces the plan and its supporting artifacts (research, data model,
   contracts, quickstart, as the project's templates define).
5. `tasks` produces `tasks.md`.
6. `analyze` is the blocking gate. Resolve findings. A genuine CRITICAL conflict
   that needs a human decision triggers an early halt.
7. `implement` executes the tasks under test-driven discipline. Security and
   tenancy tests are required, not optional.
8. Verify with CI parity: run the discovered format, lint, and test commands in
   the foreground and watch them to completion. Never launch the test suite in
   the background and poll for its output; buffered test runners make a
   background run indistinguishable from a dead one, which has caused
   misdiagnosed hangs. A red result that cannot be fixed within the slice
   triggers a halt with the failure.
9. Commit locally with a conventional message (for example
   `feat(<slice>): <title>`) and the project's co-author trailer convention, and
   update the changelog's unreleased section: a feature line, plus a dated
   decisions entry for any architecture-affecting choice.
10. Halt before the push. Present the breakdown below and wait for explicit
    authorization.

## Decision policy

This is the core behavioral change. For any decision point that the default
behavior would raise to the operator, the agent instead:

- Enumerates the viable alternatives.
- Evaluates them against the constitution, the architecture of record, the
  slice's scope lines, and existing code patterns.
- Picks the best-supported option, proceeds, and records the decision and its
  rationale in the slice's `plan.md` or `spec.md`, and in the changelog's
  decisions section when the choice is architecture-affecting.

Halt to the operator only when one of these holds:

- No option is clearly best and the choice is materially irreversible or
  architecture-defining.
- The slice's intent or scope is genuinely ambiguous in the laid-out plan.
- A constitution CRITICAL conflict cannot be resolved without a human decision.

## Branching

Follow the project's branch model.

- Trunk-based: commit directly onto local main. Do not create or push feature
  branches. If the harness or a spec-kit kickoff creates a working branch, fold
  it into main before the pre-push halt (rebase or squash its commits onto local
  main, verify the result is exactly what ships, and delete the working branch
  locally and on the remote if it was pushed). Delete a branch only after
  confirming its commits are present on main. An unmergeable branch is a halt,
  not a silent deletion.
- Feature-branch: commit onto the slice's branch per the project's convention.
  The single pre-push halt still applies before the branch is pushed or a pull
  request is opened.

## The pre-push halt breakdown

At the single halt, present:

- The slice id and title, and what was built: the spec, plan, and tasks
  artifacts, the code modules, and the tests.
- The notable decisions made and why (the decision log).
- The verification results for format, lint, and tests, with evidence of pass
  or fail.
- Any deviations or open risks against the slice's "Done when" gate.
- The exact push command awaiting authorization.

## Always-halt guardrails

These hold regardless of the decision policy:

- Never push, tag a release, or run a release command without explicit
  authorization.
- Never weaken or skip security and tenancy tests, and never skip the `analyze`
  gate.

Pinned process artifacts (CI workflows, toolchain files, release config, build
scripts, release docs) may be modified when a slice's scope requires it,
provided the change is recorded as a dated decision in the changelog. Autopilot
does not halt separately for this; the changes surface at the once-per-slice
pre-push halt and must pass the project's merge gate. Cutting a release still
requires explicit authorization.

## Scope and expiry

An explicit autopilot request authorizes autopilot for the named slice or
feature regardless of any numbering. If a project defines standing autopilot
ranges (for example a laid-out build sequence), honor them as the project
documents; work outside those ranges without an explicit request falls back to
normal interactive mode.
