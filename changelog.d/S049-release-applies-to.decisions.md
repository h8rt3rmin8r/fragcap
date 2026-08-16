<!-- spec-impact: none -->

**2026-08-16** Extended the release-preparation scripts (`scripts/cut-release.sh`
and `scripts/New-Release.ps1`, slice S049) to move the specification's
`Applies-To` field to the new version alongside the workspace bump. The field is
bound to the workspace version by the new `cargo xtask spec` check that runs in
`cargo xtask ci` during release preparation, so without this the check would fail
on the stale field on every release.
