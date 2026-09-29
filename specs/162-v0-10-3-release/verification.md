# Verification Record: S162 v0.10.3 release

**Date**: 2026-09-29

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

## Current boundary

This is a locally verified release candidate, not a public release. The candidate PR, owner merge, annotated tag, hosted release jobs, protected crates.io approval, six public-file checks, ten registry checks, and subsequent records-only PR remain outstanding. Local agent verification did not install or execute the released product, a real game, production Doctor, real trust mutation, or sensitive live capture on the operator machine.
