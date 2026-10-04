# Implementation Plan: S164 v0.10.4 patch release

**Branch**: `release/0.10.4` | **Date**: 2026-10-03 | **Spec**: [spec.md](spec.md)

**Input**: Operator authorized S164 autopilot, candidate push and PR, at most two Codex review rounds, and green-CI handoff before their merge. Publication remains a later gate.

## Summary

Prepare a reviewed v0.10.4 patch candidate from merged S163 and every unreleased fragment since v0.10.3. Reuse existing release machinery, reconcile all first-party and generated identities, pass local and hosted checks, then hand the green reviewed PR to the operator. A later exact-source tag, public-file verification, and records-only PR follow the owner merge and separate publication authorization.

## Technical Context

**Language/Version**: Rust workspace on pinned toolchain, PowerShell 7 release wrapper.
**Primary Dependencies**: Existing cargo-release, `scripts/New-Release.ps1` dry run, `cargo xtask`, GitHub Actions release workflow. No new dependencies.
**Storage**: Versioned source and generated evidence, GitHub PR, later Git tag and public release.
**Testing**: `cargo xtask ci`, `cargo xtask msrv`, `cargo xtask neutral`, docs/spec/notes gates, locked isolated manifests, hosted PR CI.
**Target Platform**: Windows official package with Linux and Windows source CI.
**Project Type**: Multi-crate Rust CLI patch release.
**Performance Goals**: Preserve existing controlled performance gates and thresholds.
**Constraints**: No direct main push, agent merge, local installed-title test, real trust effect, sensitive capture, early tag, or premature published-state marker.
**Scale/Scope**: Ten product crates, xtask, three lockfiles, generated outputs, changelog, release notes, candidate PR, later publication and records.

## Constitution Check

**P-1 through P-7 and P-10**: Pass. This release preparation changes identity and generated evidence, not capture techniques or product scope.

**P-8**: Pass when all source, documentation, MSRV, neutral, text, and hosted checks complete on exact candidate source.

**P-9**: Pass. Candidate, tagged release, public files, registry state, and published records remain separate observed facts. An open or green PR cannot establish publication.

**P-11**: Pass when specification Applies-To and candidate package version advance together while published baseline remains at the verified public version.

**Workflow**: Pass. The operator authorized branch push and PR, and retains the merge ritual. Tag and protected registry approval remain later gates.

## Project Structure

```text
specs/164-v0-10-4-release/
├── checklists/
├── contracts/release-handoff.md
├── data-model.md
├── plan.md
├── quickstart.md
├── research.md
├── spec.md
├── tasks.md
└── verification.md

Cargo.toml
Cargo.lock
fuzz/Cargo.lock
performance/native-proxy/Cargo.lock
CHANGELOG.md
release-notes/v0.10.4.md
docs/fragcap-specification.md
docs/plans/README.md
docs/maintainers/v0.10.4-release-handoff.md
conformance/native-http-tls/
crates/fragcap-sink/
fixtures/goldens/
scripts/New-Release.ps1
scripts/cut-release.sh
release.toml
```

## Decisions

1. Use `release/0.10.4` because `release.toml` permits the version-only cargo-release command only on `release/*`. Spec-Kit owns the `164-v0-10-4-release` directory through `.specify/feature.json` independently of the Git branch name.
2. Preview the release wrapper but run its documented version-only and generator commands on the existing branch. The wrapper's execution preflight requires clean main and creates a branch, which conflicts with the committed specification gate.
3. Keep S164's candidate and later public release as ordered states. The current user authorization covers branch push and PR, and explicitly stops for owner merge. A release tag is a distinct external effect after merge.
4. Keep title-specific field testing outside release acceptance. S163's controlled tests and repository checks establish its implemented correction; any observed failure against published bytes becomes a concrete new issue.
5. Preserve public v0.10.3 markers until independent v0.10.4 verification, followed by a separate records-only PR.
6. Align both release wrappers and `release.toml` on the annotated-tag and ten-crate printed handoff. A first-round Codex review identified the Unix and configuration comments that the initial Windows-only correction missed; the dated changelog decision covers all three pinned artifacts.

## Implementation Sequence

1. Commit this analyzed Spec-Kit gate before candidate mutation.
2. Preview patch release; bump first-party versions and regenerate output, lockfile, conformance, supply-chain, changelog, and note evidence.
3. Update specification applicability and candidate handoff while preserving public baseline markers.
4. Run focused and full local gates, push the PR, address first-round review and CI, request at most one second Codex review, and require green final-head checks.
5. Ask the operator for the final review and merge ritual. After that separate event and authorization, verify exact merged source before tag and public publication checks; move published-state records only in a later PR.

## Post-Design Constitution Recheck

The design preserves all gates above. No constitutional exception, dependency-policy change, or release-workflow change is required.
