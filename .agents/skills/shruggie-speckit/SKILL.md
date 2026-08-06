---
name: shruggie-speckit
description: Run a spec-kit feature slice under autopilot, driving the project's installed /speckit-* commands end to end (specify, clarify, checklist, plan, tasks, analyze, implement, verify, commit) and halting exactly once before git push. Trigger on a kickoff like "autopilot the next slice", "kick off S09", "run the next slice under autopilot", "autopilot this feature", or "run S044 under autopilot".
disable-model-invocation: false
user-invocable: true
when_to_use: Use when the operator kicks off a full spec-kit slice or feature to run without inter-step authorization, naming the slice or saying "autopilot", "kick off", or "run the next slice". Do not use for a single spec-kit command (a bare /speckit-plan or "just run clarify"); let that command run on its own. Do not use in a project that has no spec-kit installed.
---

# Shruggie Speckit Autopilot

This skill runs a spec-kit feature slice under autopilot. On one verbal kickoff
it drives the full spec-kit sequence end to end, makes the routine decisions
itself and records them, and halts exactly once, right before the push, with a
review breakdown. It is an orchestration layer: it invokes the `/speckit-*`
command skills the target project already has installed and never reimplements
or fakes them.

The full operating procedure is in
[assets/autopilot-protocol.md](assets/autopilot-protocol.md). Read it once at
kickoff. This body is the standing summary.

## When to Use

Invoke on a kickoff that authorizes a full slice run without inter-step pauses:

- "Autopilot the next slice" / "run the next slice under autopilot"
- "Kick off S09" (or any slice or feature id)
- "Run S044 under autopilot" / "autopilot this feature"

Do not invoke for:

- A single spec-kit command (a bare `/speckit-plan`, "just clarify this"). Let
  that one command run; autopilot is the whole sequence.
- A project with no spec-kit installed. See Preconditions.
- Generic "write a spec" or "make a plan" requests that do not name autopilot or
  a slice and do not intend the unattended end-to-end run.

## Preconditions

Before running the sequence, confirm the project is set up and discover its
specifics. Do not assume; read the project.

1. Spec-kit is initialized. Confirm a `.specify/` directory exists and the
   project exposes `/speckit-*` command skills (or the dotted `/speckit.*`
   form). If neither is present, halt and tell the operator to initialize
   spec-kit first; point them at
   [assets/speckit-reference.md](assets/speckit-reference.md) for the
   `specify init` steps. Never invent the commands.
2. Command form. Detect whether the project's commands are hyphenated
   (`/speckit-plan`) or dotted (`/speckit.plan`) and use whichever it exposes
   consistently for the whole run.
3. Constitution. If `.specify/memory/constitution.md` exists, read it; its
   principles govern every decision below. Absent a constitution, fall back to
   the project's CLAUDE.md/AGENTS.md and architecture docs.
4. Verification commands. Discover the project's CI-parity checks (format,
   lint, test) from its CLAUDE.md, CI workflow (`.github/workflows/**`), or
   build docs. Do not assume a toolchain; run what the project actually uses.
5. Branch model. Determine whether the project is trunk-based or uses feature
   branches, and follow that model at commit time.

## Instructions

Once preconditions pass, run these standing rules for the slice. The long-form
rationale and edge cases are in the protocol asset; keep to these while active.

- Run the per-slice sequence in order with no inter-step halt:
  `specify` -> `clarify` -> `checklist` (where the slice warrants it) -> `plan`
  -> `tasks` -> `analyze` -> `implement` -> verify -> commit. Invoke the
  project's own spec-kit commands for each step.
- `analyze` is a blocking gate. Resolve its findings. A genuine CRITICAL that
  needs a human decision is an early halt, not a proceed.
- Answer `clarify` questions yourself from the spec, the constitution, the
  architecture of record, and the slice's scope lines. Escalate only genuinely
  unanswerable questions.
- Decision policy: for any point the default behavior would raise to the user,
  enumerate the viable options, evaluate them against the constitution, the
  architecture of record, the slice's scope lines, and existing code patterns,
  pick the best-supported option, proceed, and record the decision and its
  rationale in the slice's `plan.md` or `spec.md` (and in the changelog when the
  choice is architecture-affecting). Halt to the operator only when: no option
  is clearly best and the choice is materially irreversible or
  architecture-defining; the slice intent or scope is genuinely ambiguous; or a
  constitution CRITICAL conflict cannot be resolved without a human call.
- Test discipline: implement under test-driven discipline. Security and tenancy
  tests are required, never optional or weakened.
- Verification discipline: run the discovered CI-parity checks in the
  foreground and watch them to completion. Never launch the test suite in the
  background and poll for its output; buffered test runners make a background
  run indistinguishable from a dead one. A red result that cannot be fixed
  within the slice is a halt with the failure.
- Commit and changelog: commit locally with a conventional message and the
  project's co-author trailer convention, and update the changelog's unreleased
  section (a feature line, plus a dated decisions entry for any
  architecture-affecting choice).
- Branching: follow the project's model. If trunk-based, commit onto local main
  and fold any harness-created working branch into main before the halt (rebase
  or squash, verify the result is exactly what ships, delete the working branch
  only after confirming its commits are on main). If the project uses feature
  branches, respect that instead. An unmergeable branch is a halt, not a silent
  deletion.
- Single pre-push halt: after verification and commit, halt and present the
  breakdown, then wait for explicit authorization. Present: the slice id and
  title and what was built (spec, plan, tasks artifacts, code, tests); the
  notable decisions and why; the verification results with pass/fail evidence;
  any deviations or open risks against the slice's done-gate; and the exact
  push command awaiting authorization.
- Always-halt guardrails, regardless of the decision policy: never `git push`,
  tag, or cut a release without explicit authorization; never skip the
  `analyze` gate; never weaken or skip security and tenancy tests. Pinned
  process artifacts (CI workflows, toolchain files, release config, scripts) may
  be changed when a slice's scope requires it, recorded as a dated changelog
  decision, and surfaced at the pre-push halt.

## Examples

### Example: kickoff to halt

**User input:**

```
Autopilot the next slice.
```

**Expected behavior:**

```
1. Confirm .specify/ and /speckit-* are present; read the constitution; discover
   the fmt/lint/test commands and the branch model.
2. Run specify -> clarify -> checklist -> plan -> tasks with no pause, answering
   clarifications from the spec and constitution and recording decisions in
   plan.md.
3. Run analyze; resolve findings; proceed since none are CRITICAL.
4. Run implement under TDD; write the required security tests.
5. Run the project's fmt, lint, and test commands in the foreground; all green.
6. Commit locally as feat(...) and add the changelog unreleased entry.
7. HALT with the pre-push breakdown and the exact `git push` command, awaiting
   authorization. Do not push.
```

## Additional Resources

- [assets/autopilot-protocol.md](assets/autopilot-protocol.md) - the full
  operating procedure: per-slice sequence, decision policy, branching, the
  pre-push halt breakdown, and the always-halt guardrails.
- [assets/speckit-reference.md](assets/speckit-reference.md) - what spec-kit is,
  each command and the artifacts it produces, the `.specify/` layout, extension
  hooks, command-name forms, and how to run `specify init` in a project.
- [assets/UPDATING.md](assets/UPDATING.md) - how to refresh the bundled spec-kit
  reference from upstream, and the pinned protocol version.
