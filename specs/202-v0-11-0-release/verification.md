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

## Release boundary

The official candidate PR, exact-head hosted checks, and actual review disposition will be recorded here. Candidate acceptance does not publish v0.11.0 or advance current-public records. Owner merge and later annotated tag and registry approval remain separate actions.
