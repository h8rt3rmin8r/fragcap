<!-- spec-impact: none -->

**2026-08-16** Renamed the shipped seed store from `hint.db` to `catalog.db` in
`.github/workflows/release.yml` (slice S050): the release now builds, stages,
archives, and checksums `catalog.db` beside the binary, which the first-run
bootstrap copies into the per-user catalog store. The MSI (`wix/main.wxs`)
installs the same file under the new name.
