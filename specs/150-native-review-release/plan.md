# Implementation Plan: Native Documentation and Reviewable Release Handoff

**Branch**: `release/0.10.0` | **Date**: 2026-09-15 | **Spec**: [spec.md](spec.md)

**Input**: Approved S150, tracked by #411 beneath #278 with related #331 and #333.

## Summary

Align current native product guidance, expand existing no-dispatch CLI documentation tests, publish a bounded review-scope registry and explicit not-started review template, and prepare v0.10.0 for the operator tag ritual. Reuse actual manifest-reader specimen tests and existing protocol, threat, fuzz, recovery, performance, Windows, supply-chain, packaging, and stable-API gates. No new product effect or security approval is introduced.

## Technical Context

**Language/Version**: Rust, MSRV 1.88; authored Markdown/MDX and JSON; existing Fumadocs static site.

**Primary Dependencies**: Existing clap parser, serde_json, repository validators, Cargo release 1.1.2, and pinned site dependencies. No new dependency or provider change.

**Storage**: A versioned review-scope registry and an explicitly unperformed review-record template; no runtime store or artifact schema change.

**Testing**: TDD expansion of `crates/fragcap-cli/tests/cli_reference.rs`; pure `xtask/src/review_handoff.rs` positive and negative fixture tests; existing product manifest example reader; full `cargo xtask ci`, declared MSRV build, static site and production accessibility.

**Target Platform**: Portable file-read/parser checks and existing Windows CI. Agent performs no installed-product or host trust test.

**Project Type**: Multi-crate library and CLI with a static documentation site and release tooling.

**Performance Goals**: File-read validation only, no product performance change or relaxed runtime budget.

**Constraints**: Keep #331/#333/#334/#278/#372 open; user tags and publishes after merge. Record version-bound evidence honestly and never rewrite physical Windows historical measurements to the new version.

**Scale/Scope**: Eleven documentation surfaces, twelve independent-review areas, one bounded GitHub delivery issue, and one minor release preparation.

## Constitution Check

GATE: Passed before research and after design.

- P-1: Passive Capture and deliberate scoped Deep Capture, exact consent and denylist remain unchanged; installed sensitive testing is explicitly external.
- P-2/P-3: No new platform dependency, capture/attribution edge, or product integration layer.
- P-4/P-9: Documentation separates protocol, loss, artifact, correlation, and cleanup facts; unperformed independent review is not approval.
- P-5: Existing parser/readers and standard analyzer authority are reused, not replaced by decorative examples.
- P-6: Reuse glossary terms and add any newly introduced public terminology with references.
- P-7/P-8: Rust owns validation; no wrapper parser or loosened check. New prose remains soft-wrapped, UTF-8 without BOM, and LF.
- P-10: Target storage and discovery operation remain unchanged.
- P-11: Version-bound source, specification Applies-To, release preparation, and current-versus-published status reconcile explicitly.

## Project Structure

### Documentation

`specs/150-native-review-release/` contains spec, quality checklists, research, data model, contracts, quickstart, tasks, and acceptance evidence. Current public content lives under `site/content/docs/`; maintainer review guidance and registry live under `docs/security/` and `docs/maintainers/`. Root README and contributor/release guidance remain entry points.

### Source Code

`crates/fragcap-cli/tests/cli_reference.rs` extends the existing example parser over the current authored documentation corpus, excluding historical changelog and generated glossary. `xtask/src/review_handoff.rs` validates scope/source/test references and the explicit not-started template, wired through `xtask/src/main.rs` and ordinary `ci`. Existing manifest-reader example tests validate artifact specimens.

**Structure Decision**: Do not add a parallel product parser, general security-audit engine, or fabricated completed-review record. Static handoff checks establish readiness only. A completed independent review is externally recorded against exact published identity under #333.

## Decisions and Release Preparation

- D-1: Choose v0.10.0 because changes since v0.9.0 include guided calibration, resumable workflows, exact candidate choices, safer discovery, installer cleanup, and session UX. A patch label understates this user-visible release increment; final feature completion is still not claimed.
- D-2: Use `release/0.10.0`, explicitly deviating from the default `codex/` branch prefix because `release.toml` permits local version preparation only on `release/*`. Spec directory identity is independent of Git branch.
- D-3: Existing New-Release orchestration assumes clean synchronized main and directly launches console children. Do not run that launcher or weaken its preflight for S150. Invoke its existing lower-level release-only operations through a verified hidden non-interactive process path, preserve targeted staging, and record this local orchestration deviation. No release script or workflow changes are needed.
- D-4: Commit validated documentation preparation before the version operation. Preview and execute only `cargo release version 0.10.0 --workspace --config release.toml`, commit the local result separately, update exact version assertions and regenerated goldens, assemble all fragments with `cargo xtask changelog --release`, validate the one-screen notes, then run the full gate. The aggregate dry run prints a Publishing banner even with publication disabled, so no aggregate release execution is used. The version subcommand has no tag/push/publication stage.
- D-5: Freeze independent source identity only after operator merge/tag and final package publication. The committed template remains not-started and does not contain a guessed final SHA, package checksum, reviewer, or zero-findings assertion.
- D-6: #411 closes only after its bounded implementation and CI pass. #331 retains security-review-driven reconciliation, #333 retains independent installed-build review, #372 retains operator-published reproduction, and #334/#278 retain final acceptance.

## Complexity Tracking

No constitutional violation. Release documentation and version-bound policy/evidence changes require the dated S150 decisions fragment. Workflows, budgets, crypto selection, product authorization, physical evidence, and final gate requirements remain unchanged.
