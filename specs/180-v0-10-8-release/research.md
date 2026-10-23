# Research: S180 release baseline

## Baseline

The current public GitHub release and `docs/published-release.json` identify v0.10.7. The latest accepted main commit at kickoff is `fe0bc6096ff103389f8fe5400a8968c31a495ebc` (S179). Ten unreleased fragments cover S174 through S179. GitHub issue #155 is the only open issue and describes a separate IGDB storage, fetch and documentation feature.

## Existing mechanisms

`release.toml` confines cargo-release to `release/*` and disables automatic tag, push and publish. `scripts/New-Release.ps1` supports a patch-level preview and preparation. `.github/workflows/release.yml` validates tag/version identity, certifies Windows packages, publishes a GitHub release from certified files, and then waits on protected crates.io approval before publishing ten crates. `release-notes/README.md` requires short authored highlights and `cargo xtask notes` validation.

## Alternatives

Patch 0.10.8 preserves the established corrective-release sequence. Minor 0.11.0 would imply a larger product boundary change that this slice does not introduce. Direct publication from the S179 main commit would leave workspace, artifact and documentation version identities at 0.10.7 and is therefore invalid.
