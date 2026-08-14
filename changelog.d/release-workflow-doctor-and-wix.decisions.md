**2026-08-14** Fixed two defects in the release workflow
(`.github/workflows/release.yml`) that only surfaced when the release path first
ran for real at the v0.3.0 tag; both had sat latent because the affected steps run
only on a tag push and none had occurred since they were added.

First, the "Assert the featured binary starts without npcap" step (added with the
release capability features, issue #62) accepted `doctor` exiting 0 or 1 in its
guard and printed its success message, but never reset `$LASTEXITCODE`. On a
runner without npcap, `doctor` correctly exits 1 (its "not ready" code), so the
step inherited that 1 and GitHub Actions failed a check that had in fact passed,
which blocked the whole release before any artifact was assembled. The step now
ends with an explicit `exit 0` on the success path; a `doctor` that fails to start
(any exit other than 0 or 1) still fails the step through the existing guard.

Second, the "Install WiX and cargo-wix" step (added for the MSI, issue #96)
installed WiX through choco but did not propagate the `WIX` environment variable
and its `bin` directory to the later "Build the MSI installer" step, which runs in
a fresh shell that does not inherit a mid-job machine-environment change. cargo-wix
resolves the WiX binaries through the `WIX` variable, so the MSI build would not
have found them. The step now reads the machine `WIX` variable after install and
appends it to `$GITHUB_ENV` and its `bin` to `$GITHUB_PATH`, failing loudly if WiX
is absent. This is a fix-forward: the v0.3.0 tag builds from the workflow as it was
at the tag, so consuming the fix requires the operator to re-point the tag or cut
the next version.
