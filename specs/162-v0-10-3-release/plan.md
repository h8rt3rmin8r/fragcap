# Implementation Plan: S162 v0.10.3 patch release

**Branch**: `release/0.10.3` | **Date**: 2026-09-29 | **Spec**: [spec.md](spec.md)

**Input**: The operator authorized publication of the next patch and will approve protected crates.io deployment when requested. Issue #439 tracks the outcome.

## Summary

Prepare a reviewed 0.10.3 candidate containing all merged changes since v0.10.2, then publish only the exact operator-merged source through the existing tagged release workflow. Verify public files, certification, and all ten crates before a separate records-only PR changes current published-state markers.

## Technical Context

**Language/Version**: Rust workspace on the pinned repository toolchain; PowerShell 7 release wrapper on Windows.

**Primary Dependencies**: Existing cargo-release, `scripts/New-Release.ps1`, `cargo xtask`, GitHub Actions release workflow, crates.io protected environment. No new dependency.

**Storage**: Source and generated release records, Git tag, GitHub release assets, crates.io package versions.

**Testing**: `cargo xtask ci`, `cargo xtask msrv`, `cargo xtask neutral`, docs/spec/notes gates, hosted PR CI, release-job and public-asset reconciliation.

**Target Platform**: Windows official distribution with Linux and Windows source CI.

**Project Type**: Multi-crate Rust CLI release with GitHub-hosted certified packages.

**Performance Goals**: Preserve S160 controlled performance gates without changing their thresholds during release preparation.

**Constraints**: No direct main push or agent PR merge; no agent crates.io approval; no installed product, real game, production Doctor, trust mutation, or sensitive live capture locally; no premature published-state claim.

**Scale/Scope**: Ten product crates, six public release files, three checksum sidecars, four release jobs, one candidate PR, one later records-only PR.

## Constitution Check

**P-1 through P-7 and P-10**: Pass. Release preparation changes product version and generated/versioned evidence, not capture or proxy architecture, target access, attribution, packet handling, or profile behavior.

**P-8**: Pass when the full source, documentation, shell, MSRV, neutral, hosted, and certification gates pass on exact candidate source.

**P-9**: Pass. Candidate, public tag, assets, registry publication, and current baseline remain separate observed facts. A failed or pending job cannot be described as release completion.

**P-11**: Pass. Specification applicability and candidate version move together, and actual publication markers move only after the public release is verified.

**Integration workflow**: Pass. The operator merges PRs; the agent publishes the explicitly authorized tag after merge. The owner alone approves protected crates.io deployment. This gate is rechecked after design.

## Project Structure

### Documentation

```text
specs/162-v0-10-3-release/
├── checklists/
├── contracts/release-publication.md
├── data-model.md
├── plan.md
├── quickstart.md
├── research.md
├── spec.md
├── tasks.md
└── verification.md
```

### Candidate surfaces

```text
Cargo.toml
Cargo.lock
CHANGELOG.md
release-notes/v0.10.3.md
docs/fragcap-specification.md
docs/plans/README.md
docs/maintainers/v0.10.3-release-handoff.md
conformance/native-http-tls/
crates/fragcap-sink/
crates/fragcap-cli/tests/
fixtures/golden/
```

### Post-publication record surfaces

```text
docs/published-release.json
docs/maintainers/v0.10.3-release-handoff.md
docs/security/
README.md
CONTRIBUTING.md
specs/162-v0-10-3-release/verification.md
```

**Structure Decision**: Reuse S159's two-PR release model and the existing release wrapper. Do not alter release tooling, workflow, dependency policy, or protection unless a verified blocker demands a separate reviewed correction.

## Implementation Sequence

1. Commit the analyzed S162 specification gate before candidate version changes.
2. Preview and run the existing Windows patch release wrapper on `release/0.10.3`; inspect all generated version and changelog changes.
3. Author bounded highlights and candidate-only records; keep public markers at v0.10.2.
4. Run local gates, push the candidate PR, resolve at most two review rounds, and wait for final-head hosted checks.
5. Request the required operator PR merge, then verify the exact merged source and conflicting-tag absence.
6. Create and push annotated v0.10.3 at that exact source, monitor release jobs, and request owner crates.io approval only if GitHub waits.
7. Verify four jobs, six public files and sidecars, certification evidence, and ten non-yanked registry packages.
8. Prepare a separate records-only PR with observed facts, pass its gates and reviews, and request operator merge.

## Post-Design Constitution Recheck

The design preserves every gate above. No constitutional exception or dependency change is required.
