# Spec-Kit Reference

Condensed reference for spec-kit, the spec-driven-development (SDD) toolkit this
skill orchestrates. Upstream: `github/spec-kit`. This file is bundled so the
agent can operate against a project's spec-kit install and recognize what each
command produces. It is a reference, not a reimplementation; the authoritative
commands are the `/speckit-*` skills installed in the target project.

See [UPDATING.md](UPDATING.md) for how and when to refresh this file from
upstream.

## What spec-kit is

Spec-kit structures feature work as a sequence of commands that each produce a
durable Markdown artifact, so the specification (not ad-hoc chat) is the source
of truth for a feature. A project adopts it by running `specify init`, which
lays down a `.specify/` directory and installs the command skills for the chosen
agent (Claude, and others).

## The SDD flow and commands

Commands run in this order. Each is invoked as a slash command; see
Command-name forms below.

- `constitution` - creates or amends `.specify/memory/constitution.md`, the
  project's governing principles. Run once per project and amend deliberately;
  it is not part of a per-feature slice.
- `specify` - from a natural-language feature description, creates the feature
  directory (`specs/<NNN-short-name>/` by default), `spec.md`, and
  `checklists/requirements.md`. Focuses on what and why, not implementation.
- `clarify` - surfaces ambiguities in the spec as targeted questions and folds
  the answers back into `spec.md`. Under autopilot the agent answers these
  itself from the spec, constitution, and scope, escalating only the genuinely
  unanswerable.
- `checklist` - adds domain-specific quality checklists to the feature where it
  warrants them.
- `plan` - produces the implementation plan and its supporting artifacts, which
  per the project's templates may include `research.md`, `data-model.md`,
  `contracts/`, and `quickstart.md`.
- `tasks` - decomposes the plan into `tasks.md`, an ordered, testable task list.
- `analyze` - cross-checks the spec, plan, and tasks for conflicts, gaps, and
  constitution violations. It is the blocking gate before implementation; a
  CRITICAL finding must be resolved (or escalated) before proceeding.
- `implement` - executes `tasks.md` under test-driven discipline, writing code
  and tests.

Optional and auxiliary commands some installs add:

- `converge` - assesses the current codebase against the feature's spec, plan,
  and tasks, then appends any remaining unbuilt work as new tasks to `tasks.md`
  so `implement` can complete it. Useful as a completion check; not part of the
  core autopilot sequence.
- `taskstoissues` - converts `tasks.md` into dependency-ordered GitHub issues
  (requires GitHub tooling).
- an agent-context updater, typically run via extension hooks after `specify`
  and `plan`.

Treat any command the project exposes but this list omits as project-specific;
read its own SKILL.md before invoking it.

## The `.specify/` layout

A spec-kit project keeps its configuration under `.specify/`:

- `memory/constitution.md` - the governing principles.
- `templates/` - the templates each command fills (`spec-template.md`,
  `plan-template.md`, `tasks-template.md`, `checklist-template.md`,
  `constitution-template.md`).
- `scripts/` - helper scripts (a PowerShell and/or shell variant) for
  prerequisites, feature creation, and plan/task setup.
- `extensions.yml` and `extensions/` - installed extensions and the hook wiring
  (see Extension hooks).
- `init-options.json` - options such as feature numbering (`sequential` yields
  `NNN-` prefixes; `timestamp` yields `YYYYMMDD-HHMMSS-` prefixes).
- `feature.json` - the currently resolved feature directory, written by
  `specify` so downstream commands locate the feature without relying on git
  branch names.
- `integrations/` and `workflows/` - integration manifests and the command
  workflow registry.

Feature artifacts themselves live under `specs/<feature-dir>/`, not under
`.specify/`.

## Extension hooks

`.specify/extensions.yml` can register hooks that run before or after a command
(for example `after_specify`, `after_plan`). A hook names an extension, a
command, and whether it is optional. Commands read this file and dispatch
registered hooks; when constructing a slash command from a hook's dotted command
name, dots become hyphens (`speckit.agent-context.update` becomes
`/speckit-agent-context-update`). Under autopilot, let mandatory hooks run and
treat optional hooks by the project's settings; do not evaluate hook
`condition` expressions yourself.

## Command-name forms

Two forms appear in the wild:

- Hyphenated: `/speckit-specify`, `/speckit-plan`. This is how the command
  skills are installed for the Claude agent.
- Dotted: `/speckit.specify`, `/speckit.plan`. This is the canonical spec-kit
  naming used in some docs and configs.

Detect which form the target project exposes and use it consistently. When in
doubt, list the project's installed command skills to see their actual names.

## Initializing spec-kit in a project

If a project has no `.specify/` directory and no `/speckit-*` commands,
autopilot cannot run. Direct the operator to initialize spec-kit first. The
upstream tool is the `specify` CLI (from `github/spec-kit`); a typical init runs
`specify init` in the project root and selects the Claude agent, which creates
`.specify/` and installs the command skills. Consult the upstream README for the
current install and init commands, since they change over time. Once spec-kit is
initialized and a constitution exists, autopilot can drive a slice.
