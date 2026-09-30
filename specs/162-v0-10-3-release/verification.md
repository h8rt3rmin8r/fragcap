# Verification Record: S162 v0.10.3 release

**Date**: 2026-09-29; public release verified 2026-09-30

## Candidate preparation

The S162 specification gate was committed before candidate mutation. The patch wrapper dry run selected v0.10.3 and seven unreleased fragments. Its release command required a clean main checkout, so the established version-only `cargo release 0.10.3 --workspace --execute --no-confirm` command was run on the already committed `release/0.10.3` branch. This changed ten product crates and xtask in Cargo.toml and Cargo.lock without creating a tag, pushing, or publishing.

The seven fragments were assembled into the chronological v0.10.3 changelog. A fragment dated 2026-09-21 was renamed before assembly so date order agreed with its content. Embedded output assertions, generated goldens, native conformance data, staged Windows identity, specification applicability, release highlights, and the current review-handoff template were bound to 0.10.3. The verified public baseline and frozen published review-candidate registry remain at v0.10.2.

The supply-chain snapshot changed because first-party package versions are included in graph digests. Inspection of the Cargo.lock diff found only eleven first-party version-field changes. All three graph package and edge counts are unchanged; no third-party package, feature, provider, unsafe implementation, or review exception changed. The reviewed digests and unsafe-review binding were advanced with the same review expiry.

The first hosted short-performance jobs on the candidate PR failed before compilation because the standalone `performance/native-proxy/Cargo.lock` retained nine v0.10.2 path-crate entries. First-round Codex review identified the same stale identities in `fuzz/Cargo.lock`. Both isolated locks now align those nine first-party versions to 0.10.3 without changing third-party dependencies. Full `cargo metadata --locked --offline --format-version 1` passed for each isolated manifest; final-head hosted short and fuzz campaigns remain the authority for their execution.

## Local source evidence

The following passed on the candidate without installing or running released fragcap bytes:

- `cargo test -p fragcap --test goldens --quiet` with `FRAGCAP_UPDATE_GOLDENS=1` (seven golden checks);
- `cargo test -p fragcap-cli --test cli_capture --quiet` with `FRAGCAP_UPDATE_GOLDENS=1` (24 checks);
- `cargo test -p fragcap-cli --test cli_extcap --quiet` with `FRAGCAP_UPDATE_GOLDENS=1` (19 checks);
- `cargo xtask notes 0.10.3`;
- `cargo xtask spec`;
- `cargo xtask supply-chain snapshot` and `cargo xtask supply-chain`;
- `cargo xtask ci` with `CARGO_BUILD_JOBS=1` and `CARGO_PROFILE_TEST_DEBUG=0`;
- `cargo xtask msrv` (Rust 1.88 workspace build);
- `cargo xtask neutral` (three neutral Linux crates);
- `cargo xtask docs build` (81 static pages);
- `git diff --check` and strict changed-text UTF-8, no BOM, LF, one final newline, trailing-whitespace, and replacement-character checks.

The initial full CI attempt exhausted host memory while compiling quinn-proto test debuginfo. Reducing test debuginfo allowed the suite to run; the next attempt identified the stale current review-handoff template, and the following attempt identified the old first-party graph digests. Both source records were corrected and the final full gate passed. No gate was skipped.

## Candidate review and merge

[Candidate PR #440](https://github.com/h8rt3rmin8r/fragcap/pull/440) was pushed and operator-merged after final-head hosted checks passed. First-round Codex review found stale first-party versions in the two isolated harness lockfiles; both were corrected and the review thread resolved. One authorized second Codex round raised no new finding. The reviewed head `1bf909424f39498ab3177a277f682629c575376b` and operator squash merge `9decf214dd58f2467c4f2fdd7228893ffafd17f9` share tree `d2deea2b893442e034bfb6ac34e2e13911ed0feb`.

## Public release evidence

Annotated tag object `d6116de97d2480c2e8db132003bd9be2361d0565` peels to exact merged source `9decf214dd58f2467c4f2fdd7228893ffafd17f9`. [Release run 36586311847](https://github.com/h8rt3rmin8r/fragcap/actions/runs/36586311847) completed successfully on that source. Identity job 109467482038, Windows certification job 109467776395, GitHub release job 109473556952, and registry job 109473827433 are all green. GitHub records normal owner `h8rt3rmin8r` approval, state `approved`, for the `crates-io` environment 21294112472; no agent approval or bypass occurred.

The public [v0.10.3 release](https://github.com/h8rt3rmin8r/fragcap/releases/tag/v0.10.3) is non-draft and non-prerelease, published `2026-09-29T15:10:13Z`. All six assets were downloaded and independently matched by exact name, byte count and SHA-256 against the release record in `docs/maintainers/v0.10.3-release-handoff.md`; the three checksum sidecars also match their primaries. The downloaded schema-version-4 certification report identifies official 0.10.3, exact source, native backend, six artifacts, two controlled smokes and zero findings. Canonical `cargo xtask package-certification validate-report` passed. All ten public crates.io records independently report `0.10.3`, `yanked=false`, and a nonempty archive checksum; the handoff retains each exact checksum.

## Records boundary

A separate records-only branch advances `docs/published-release.json`, fourteen current baseline markers, frozen review-candidate identity, and this evidence after public verification. `cargo xtask review-record`, `cargo xtask spec`, `cargo xtask docs check`, full `cargo xtask ci` and `cargo xtask docs build` passed on the records branch. The static export generated 81 pages; `git diff --check` and strict UTF-8 without BOM, mojibake marker and final-newline checks passed for 28 changed text files. Historical v0.10.2 evidence, the unsigned policy, release workflow, dependencies and product bytes remain unchanged. The records PR still requires operator review and merge. Agent verification did not install or execute released fragcap, a real game, production Doctor, real trust mutation, or sensitive live capture on the operator machine. The focus-dependent Print Screen observation has no demonstrated fragcap-specific cause or fix; independent whole-product security review and universal live-title compatibility are not claimed.
