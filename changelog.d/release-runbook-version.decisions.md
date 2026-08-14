**2026-08-14** De-hardcoded the release version from the release-runbook
documentation in two pinned artifacts. `release.toml`'s step-3 comment and
`scripts/New-Release.ps1`'s minor-bump `.EXAMPLE` both named a literal `v0.2.0` /
`release/0.2.0`, which went stale the moment the version moved and would misdirect
an operator who read the file rather than the script's printed next steps. Both
now use a `vX.Y.Z` / `release/X.Y.Z` placeholder, so they stay correct across
releases. No behavior changed: `release.toml` still tags, pushes, and publishes
nothing (`tag`, `push`, and `publish` are all false), and the scripts already
print the tag command with the actual bumped version at runtime
(`git tag v${version} ...`), which is unaffected. Recorded here because both files
are pinned artifacts (surfaced by the Codex review of the v0.3.0 release,
pull request 99).
