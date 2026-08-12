### Added

- **A single release-preparation command, `scripts/cut-release.sh` and its
  PowerShell twin `scripts/New-Release.ps1`.** It prepares a `release/X.Y.Z`
  branch in one step: it bumps the workspace version through `cargo release`,
  assembles `CHANGELOG.md` from the `changelog.d/` fragments, corrects the two
  embedded-version assertions and the golden corpus that the bump moves, and
  runs the full check set. It performs no tag, push, or publish, so the two
  authorizations the constitution requires (pushing the tag, approving the
  crates-io environment) remain manual. A `--dry-run` previews the plan and the
  assembled changelog without writing anything. Both scripts are held to the
  ShruggieTech shell standards by `cargo xtask wrappers`.
- **`cargo xtask changelog`, which folds the `changelog.d/` fragments into
  `CHANGELOG.md`.** `--check` prints the assembled body and changes nothing;
  `--release <version> <date>` rewrites the changelog, moving the assembled body
  into a dated version section, resetting `[Unreleased]`, and removing the
  consumed fragments. The section order is canonical, existing `[Unreleased]`
  content is preserved, and an unknown section name fails loudly rather than
  dropping the entry.
