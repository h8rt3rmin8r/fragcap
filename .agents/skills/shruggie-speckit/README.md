# shruggie-speckit

Runs a [spec-kit](https://github.com/github/spec-kit) feature slice under
autopilot. On one verbal kickoff ("autopilot the next slice", "kick off S09"),
the agent drives the full spec-kit sequence end to end (specify, clarify,
checklist, plan, tasks, analyze, implement, verify, commit), makes the routine
decisions itself and records them, and halts exactly once, right before the
push, with a review breakdown.

This skill is an orchestration layer. It drives the `/speckit-*` command skills
the target project already has installed; it does not ship or reimplement them.

## Prerequisite

The target project must already have spec-kit installed: a `.specify/` directory
and the `/speckit-*` command skills. If it does not, the skill halts and points
you at the setup notes rather than improvising the commands. To add spec-kit to
a project, run `specify init` from the upstream `github/spec-kit` tooling; see
`assets/speckit-reference.md` for the details.

The skill is strictly generic. It discovers the project's specifics at runtime
(the constitution at `.specify/memory/constitution.md`, the format/lint/test
commands from the project's CLAUDE.md or CI workflow, and the branch model)
rather than assuming any particular toolchain.

## Trigger and safety

The skill is model-invocable: a natural-language kickoff auto-triggers it. Its
description is scoped tightly to autopilot-kickoff phrasing so a bare
`/speckit-plan` or a generic "write a spec" request does not pull in the whole
autopilot run. The behavior is bounded by an always-halt-before-push guardrail:
the agent never pushes, tags, or cuts a release without explicit authorization,
and never skips the `analyze` gate or weakens security and tenancy tests.

## Files

- `SKILL.md` - the operating instructions loaded at invocation.
- `assets/autopilot-protocol.md` - the full generalized protocol.
- `assets/speckit-reference.md` - condensed spec-kit reference (commands,
  artifacts, `.specify/` layout, hooks, init).
- `assets/UPDATING.md` - how to refresh the bundled reference from upstream.
- `scripts/update-speckit-reference.{ps1,sh}` - maintainer aid that fetches
  upstream spec-kit material for diffing. Never run at skill runtime.

## Sourcing and updates

The generalized protocol (`assets/autopilot-protocol.md`, version 1.0.0) is
derived from the ShruggieGraph Build-Phase Autopilot Protocol (v1.2.0) with all
project-specific detail replaced by discover-at-runtime guidance. The spec-kit
reference is condensed from upstream `github/spec-kit` and the installed command
skills. See `assets/UPDATING.md` for when and how to refresh both, and the
update helper for pulling the current upstream material.

## Known conflicts

- Missing dependency: the skill cannot run without spec-kit installed in the
  project. The precondition gate detects this and halts with setup guidance.
- Command-name form drift: some installs expose hyphenated `/speckit-plan`,
  others dotted `/speckit.plan`. The skill detects and uses whichever form the
  project exposes.
- Trigger overlap: the `/speckit-*` command skills are themselves
  model-invocable. This skill's description matches only autopilot-kickoff
  phrasing to avoid firing when a single command is wanted.
