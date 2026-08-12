### Decisions

- **2026-08-12** `release.toml` and `scripts/**` are pinned artifacts, changed
  here to add the release-preparation script pair (`scripts/cut-release.sh`,
  `scripts/New-Release.ps1`) and to record the sequence they consolidate. The
  script wraps `cargo release` rather than replacing it, so the version bump
  stays the tool `release.toml` configures; it stops before the tag push and the
  crates-io environment approval, so both authorizations the constitution
  requires are still separate human acts rather than side effects of a version
  bump.
- **2026-08-12** The changelog assembler and the release-notes derivation live
  in `cargo xtask` (Rust), not in the shell scripts, per the house rule that a
  wrapper which parses text is a missing capability in Rust. The transform has a
  canonical section order and a merge rule that are worth a unit test, and a
  reimplementation in two shell dialects would be two untested copies of the
  same logic. The scripts stay thin orchestrators over git, cargo, and the task
  runner.
