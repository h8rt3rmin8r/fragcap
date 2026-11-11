# Verification: v0.11.0 Candidate

## Source and release identity

- Accepted base: S201 squash merge `f48d3525ad5317bc1d76c54b20ce5524d791242d` on `main`.
- Candidate branch: `release/0.11.0`; release tracking issue #589 and milestone v0.11.0.
- `cargo release 0.11.0 --workspace --no-confirm` previewed the version change. `cargo release 0.11.0 --workspace --execute --no-confirm` committed only `Cargo.toml` and `Cargo.lock` as `1983ac3`; release configuration disabled tag, push, and registry publication.
- `cargo xtask changelog --check` validated 13 pending fragments. `cargo xtask changelog --release 0.11.0 2026-11-10` assembled all 13 into `CHANGELOG.md` and reset `[Unreleased]`. The consumed fragments were removed.
- The isolated fuzz and native proxy performance locks changed only first-party package versions. Versioned output goldens and conformance identity were advanced to 0.11.0.
- Public v0.10.14 metadata, tagged publication, and frozen published-review evidence remain unchanged.

## Local checks

- `cargo xtask notes 0.11.0`: passed after matching the exact standard changelog guidance.
- `cargo xtask spec`: passed; Applies-To matches 0.11.0 and the 15 current public markers agree on v0.10.14.
- `git diff --check`: passed.
- Versioned golden regeneration tests: seven capture corpus, 31 sink pcapng, and 20 sink JSON tests passed.
- The first `cargo xtask ci` run found a stale 0.10.14 version in the unstarted independent-review template and a test whose hardcoded mismatch became the new version. The template now follows 0.11.0 while the frozen published candidate remains v0.10.14; the test uses a version-independent mismatch. All 11 focused review-handoff tests passed after correction.
- The next full gate passed the Rust tests, lint, dependency direction, and crate license checks, then found the expected first-party version change in three reviewed dependency graph digests. The graph snapshots retained 188/461 Linux, 197/455 Windows all-feature, and 172/382 Windows release package/edge counts. The policy now records the exact new digests without extending its unsafe-review expiry or changing third-party dependencies.
- The final `cargo xtask ci` run passed all checks, including formatting, Clippy, tests, repository lint, dependency and license policy, supply chain, package contract, wrappers, skills, documentation, specification, guided calibration, threat model, review readiness, frozen publication candidate, fuzz inventory, failure matrix, conformance, performance authority, and Windows integration authority.
- `cargo xtask docs check` passed the documentation contract, linter, and both CLI reference configurations. `cargo xtask docs build` rendered the 0.11.0 site and 114 static pages with `.nojekyll` and `CNAME`.
- `cargo xtask publish` validated the ten-crate registry order without publishing. `cargo xtask changelog --check` passed before assembly; after assembly it reports the expected empty unreleased section and no pending fragments.
- The edited source and documentation passed `git diff --check` and a targeted scan for common mojibake markers.
- Spec-Kit convergence found no missing candidate implementation task. The existing T014 through T027 already name official review, owner merge, publication, public readback, and current-record work; those stages are not yet claimed complete.

## Official candidate and accepted merge

- [PR #590](https://github.com/h8rt3rmin8r/fragcap/pull/590) reviewed candidate head `c08f0bf65b5243f82ccee09cf65f0533859868f6` and merged at `2026-11-11T01:38:05Z` as `5e2c0435fcb642ab9ca42d91854def3357f32a7b`.
- All 24 applicable exact-head PR checks passed, including final Windows package certification, Windows native integration, docs, audit, platform, analyzer, fuzz, performance, and CodeQL. Pages deployment and the manually dispatched soak job were intentionally skipped on the PR.
- The first Codex code review completed on the exact candidate head with no findings, no inline comments or review threads, and a thumbs-up reaction. The security review produced no receipt. No explicit second review was requested.
- The accepted merge tree has no diff from the reviewed candidate tree. The authenticated owner authorized public release after the squash merge.
- Annotated tag `v0.11.0` was pushed on the exact accepted merge. Tagged release run [42940637534](https://github.com/h8rt3rmin8r/fragcap/actions/runs/42940637534) passed identity and started final package certification. Public release and registry publication remain in progress.

## Release boundary

Candidate acceptance alone did not advance current-public records. The following public readback preceded every current-version edit in the publication-record branch.

## Tagged publication and independent readback

- Annotated tag object `659e65dd05f875833967505ee78c768c969906ac` selects accepted merge `5e2c0435fcb642ab9ca42d91854def3357f32a7b`. Tagged run 42940637534 passed identity, final Windows certification, and GitHub release creation. The public release was published at `2026-11-11T01:57:37Z`, neither draft nor prerelease.
- The normal crates.io environment approval for deployment `7699528996` allowed publication. Nine crates published; the tenth encountered a crates.io HTTP/2 framing error while fetching `brotli`. `gh run rerun --failed` retried only the failed job, with normal protected deployment approval `7699571514`. Attempt 2 succeeded. The publisher skipped the already uploaded crates, and no source or public package bytes changed.
- Six separately downloaded public files matched GitHub API lengths and SHA-256 digests. The three checksum sidecars independently matched their named downloads. Exact values are in [the release handoff](../../docs/maintainers/v0.11.0-release-handoff.md).
- Downloaded tagged schema-4 certification report passed `cargo xtask package-certification validate-report target/s202-public-readback/certification/report.json target/s202-public-readback`. It names official 0.11.0, the accepted source, exact feature closure including `direct-route`, ten package entries, two controlled loopback smokes, six lifecycle cases, complete fresh start, and zero findings.
- All ten separately downloaded v0.11.0 crates matched the crates.io API checksums and were non-yanked. The downloaded ZIP contains the WinDivert files and its `fragcap.exe --version` printed `fragcap 0.11.0` without elevation.
- Current metadata, documentation, frozen review candidate, and replay asset authority advanced after this reconciliation. Independent whole-product security review and private-title behavior remain unobserved.

## Publication-record validation

- `cargo xtask review-record`, `cargo xtask docs check`, `cargo xtask docs build`, `cargo xtask spec`, and `cargo xtask notes 0.11.0` passed on the current v0.11.0 records. The docs build generated 114 static pages.
- The first full `cargo xtask ci` run passed formatting, Clippy, all Rust tests, repository lint, dependency and license checks, supply chain, package certification, wrappers, skills, and documentation, then found that the new publication `.fixed.md` fragment lacked the required leading spec-impact comment. That comment was added. `cargo xtask spec` passed, and the subsequent complete `cargo xtask ci` passed every stage through guided calibration, threat model, review readiness, frozen candidate, fuzz inventory, failure matrix, conformance, performance, and Windows integration authority.
- `git diff --check` and a targeted mojibake scan passed after the record edits. [Records PR #591](https://github.com/h8rt3rmin8r/fragcap/pull/591), exact head `bce44a43f440ee521940d4c628fb0d75beb0617d`, squash-merged as `f46a9d54feac1556eafbc4c782d82a98c23bbb03` on 2026-11-11 UTC. The Codex code review completed on that head with no findings, and there were no inline review comments. The security review left no receipt. The locally complete gate, tagged package certification, and independent public readback provided the validation authority; follow-on hosted PR checks were observed for concrete failures and the documentation build passed. Public site readback follows below.

## Public site and closeout

- The merged [docs workflow 42944399795](https://github.com/h8rt3rmin8r/fragcap/actions/runs/42944399795) passed its static export, browser accessibility, and Pages deployment. Independent HTTP readback of `/docs/getting-started`, `/docs/reference/deep-capture-compatibility`, and `/docs/guides/packaging-and-migration` returned 200 and showed the v0.11.0 published baseline and direct-route guidance. The getting-started page links the exact v0.11.0 tag and says the released Windows package bundles WinDivert.
- GitHub's latest stable release resolves to v0.11.0. [Release issue #589](https://github.com/h8rt3rmin8r/fragcap/issues/589) closed with public evidence, and milestone v0.11.0 closed with zero open issues. The remote candidate and records branches were already removed; the local release branch and ignored Spec-Kit pointer were removed after confirming the candidate and tag trees match. The checkout returns to synced `main` after this closeout record merges.
