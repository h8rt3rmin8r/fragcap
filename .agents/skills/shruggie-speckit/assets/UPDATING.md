# Updating the bundled reference

This skill bundles two documents that drift over time and need occasional
refresh. This file records where they come from and how to update them.

## Version pins

- `assets/autopilot-protocol.md` - generalized protocol, version 1.0.0, derived
  from the ShruggieGraph Build-Phase Autopilot Protocol v1.2.0. When the source
  protocol is amended in a way that changes behavior (not just project-specific
  wording), reconcile the generalized version here and bump its
  `Protocol version` line.
- `assets/speckit-reference.md` - condensed from upstream `github/spec-kit` and
  from the `/speckit-*` command skills installed in a reference project.

## When to refresh `speckit-reference.md`

Refresh when upstream spec-kit changes in a way the reference would now
misstate, in particular:

- A command is added, removed, or renamed (the SDD flow list changes).
- The artifacts a command produces change (for example `plan` emits a new
  supporting file).
- The `.specify/` layout changes (new config files, moved templates).
- The extension-hook wiring or the dotted-to-hyphen slash-command rule changes.
- The `specify init` invocation changes.

Do not mirror upstream verbatim. This is a condensed operating reference; keep
it short and keep it accurate.

## How to refresh

1. Run the helper to pull the current upstream command and docs set into a
   scratch directory for diffing:

   - PowerShell: `scripts/update-speckit-reference.ps1 -OutDir <scratch>`
   - Bash: `scripts/update-speckit-reference.sh -o <scratch>`

   The helper is a maintainer aid only. It performs a network fetch from the
   upstream repository and is never run at skill runtime.

2. Diff the fetched command definitions and docs against
   `assets/speckit-reference.md`. Where the flow, artifacts, layout, or hook
   rules changed, edit the reference to match. Preserve the house style: no
   em-dashes or en-dashes, concise prose, LF endings, UTF-8 without BOM.

3. If a command was added or renamed, also check `SKILL.md` and
   `assets/autopilot-protocol.md` for any step that names the command, and
   update those too.

4. Record the change in the repository `CHANGELOG.md` under `## [Unreleased]`
   keyed as `` `shruggie-speckit` ``, and cut a release per the repository
   contributing guide.

## If the helper cannot fetch

The helper depends on network access and the upstream repository layout, both of
which can change. If it fails, refresh manually: open the upstream
`github/spec-kit` repository, read the current command templates and docs, and
edit `assets/speckit-reference.md` by hand against them. The reference is small
enough to maintain without the script.
