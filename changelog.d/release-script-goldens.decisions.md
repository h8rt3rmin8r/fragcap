<!-- spec-impact: none -->

**2026-08-18** Corrected the golden-regeneration step in the release-preparation
scripts (`scripts/cut-release.sh` and `scripts/New-Release.ps1`). Both named a
`fragcap-cli` test binary `cli_run`, which the S054 command-line surface rework
removed when it collapsed `run`, `tap`, and `watch` into `capture`. The goldens
that carry the embedded `fragcap/<version>` string moved with it: `cli_capture`
now owns `capture.fcapng` and `capture.jsonl`, and `cli_extcap` owns
`run.fcapng`. Preparing v0.5.0 failed at that step with `no test target named
cli_run`, after the version bump had already been committed, leaving a
half-prepared release branch. Both scripts now regenerate through the three
binaries that actually own the version-bearing goldens, and the comment above the
step names which golden belongs to which binary so the next rename is caught by
reading rather than by a failed release. Recorded as a dated decision per the
pinned-artifact rule (`scripts/**` changes only with one).
