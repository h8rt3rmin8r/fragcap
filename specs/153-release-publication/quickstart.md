# S153 Verification Quickstart

## Preconditions

Use direct git/gh and verified hidden non-Git launchers on Windows. Do not run the installed application or local package lifecycle harness. Registry secrets remain hosted.

## Release Evidence

Inspect exact merged source checks, git tag identity, the tag-triggered release run and any pending owner deployment. Download six official assets plus windows-package-certification-summary from the same release run to ignored target/release-verification-v0.10.1/. Compare hashes and sizes with sidecars and report, and separately compare report.build_identity.version/source_revision with 0.10.1 and the peeled tag.

```sh
gh release view v0.10.1 --json tagName,isDraft,isPrerelease,publishedAt,assets,url
gh run view RELEASE_RUN_ID --json headSha,status,conclusion,jobs
cargo xtask package-certification validate-report target/release-verification-v0.10.1/certification/report.json target/release-verification-v0.10.1/dist
```

Query https://crates.io/api/v1/crates/CRATE/0.10.1 for each crate in the ten-entry xtask/src/publish.rs ORDER. Require exact version and yanked=false. A public GitHub release without these registry records is partial, not complete.

## Records Verification

```sh
cargo xtask release-guard
cargo xtask notes 0.10.1
cargo xtask spec
cargo xtask ci
```

Expected: intact release wiring, valid short notes, matching candidate/publication fields, fourteen matching current markers and green parity. Hosted records-PR checks provide independent source CI; operator field measurements and security audit remain separate.
