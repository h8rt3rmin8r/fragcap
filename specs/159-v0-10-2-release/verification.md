# Verification Record: S159 v0.10.2 release

**Date**: 2026-09-20

## Candidate preparation

The S159 specification gate was committed before any candidate mutation. The established version-only cargo-release command moved the workspace to 0.10.2 without tagging, pushing or publishing. Embedded writer assertions, native conformance evidence, staged Windows identity and specification Applies-To moved to the same version. The three owning golden generators regenerated every version-bearing capture output.

Release assembly consumed fourteen accumulated S153 through S159 fragments into the dated v0.10.2 section and reset Unreleased. Assembly identified two existing documentation fragments whose unsupported `docs` suffix had never been accepted by the closed changelog grammar. Their unchanged content was reclassified to the supported `changed` section before successful assembly. No changelog schema or release tool changed.

The bounded v0.10.2 highlights validate and link to the tagged complete changelog. The candidate handoff preserves v0.10.1 as actual publication and keeps independent review, Doctor field validation and final Deep Capture acceptance open.

## Local source evidence

The following passed on the prepared candidate without installing or running released fragcap bytes:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all --locked`
- `cargo xtask ci`
- `cargo xtask msrv`
- `cargo xtask neutral`
- `cargo xtask docs build`
- `cargo xtask spec`
- `cargo xtask supply-chain`
- `cargo xtask review-handoff`
- `cargo xtask review-record`
- `cargo xtask notes 0.10.2`

The aggregate gate reports thirteen documentation topics, thirteen guided-calibration criteria, ten threat-model rows, six fuzz targets, fifteen failure boundaries, fourteen performance cases, a closed Windows integration registry, complete package certification, current release-guard wiring and all source tests passing. Independent review remains explicitly not performed.

## Safety boundary

No installed product, real game, production Doctor, real trust mutation or sensitive live capture ran locally. Controlled source tests use synthetic loopback and repository-owned fixtures. Packaged execution remains the responsibility of disposable hosted Windows certification after push.

## Pending evidence

Hosted pull-request checks, review dispositions, exact human-merged candidate source, tag identity, release workflow, public files, registry versions and post-publication records do not exist yet. They will be appended only after observation.
