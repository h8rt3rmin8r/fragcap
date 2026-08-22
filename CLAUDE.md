# Claude Code guide

The house rules for this repository are provider-agnostic and live in
`AGENTS.md`. That file is the source of truth: what fragcap is, the reference
documents, the spec-kit workflow, the non-negotiables, verification discipline,
and the integration workflow. It is imported below so Claude Code loads it in
full.

@AGENTS.md

## Claude Code specifics

- Spec-kit exposes native `/speckit-*` skills here (specify, clarify,
  checklist, plan, tasks, analyze, implement, converge, constitution,
  taskstoissues). Prefer them over hand-running the phases; they drive the same
  shared `.specify/` engine described in `AGENTS.md`. Three other surfaces
  exist for other agents (Codex, Cursor, opencode) and are kept in sync by the
  spec-kit CLI, not by hand.
- Vendored skill content lives in `.agents/skills/` (committed, and read
  directly by Codex). Claude Code reads `.claude/skills/` instead, which an
  external skills CLI may populate with machine-local symlinks; those are
  gitignored because they carry absolute paths, and since that CLI is not part
  of this repository, a checkout may carry none of them. If yours does not, the
  vendored content is still under `.agents/skills/` and nothing here depends on
  the symlinks. The `speckit-*` directories under `.claude/skills/` are real
  tracked files and are the exception. The vendored set is small and governed
  by an admission test; `skills/README.md` is the authority.
- The spec-kit agent-context updater writes the per-slice narrative to
  `.specify/active-slice.md` (gitignored, local per-branch state), not into
  this tracked file. That keeps per-slice pointer churn out of the diff and
  stops parallel slices conflicting here. The durable artifact for the current
  slice is its `specs/NNN-slug/plan.md`, pointed to by `.specify/feature.json`.
- The per-slice record lives in `changelog.d/` and `specs/`, not in this file,
  and those two directories are the authority for what has landed. Read the
  highest-numbered `specs/` directory to see where the work has reached, and
  `.specify/feature.json` for what is in flight. No slice number is named here
  as a completion marker, because any number written here is wrong one slice
  later and a reader will quote it anyway. Run `cargo xtask ci` before proposing
  any change; it is the same set the automated checks run, so the two cannot
  drift. It now includes the fixture corpus drift check, so a hand-edited
  fixture fails there.
