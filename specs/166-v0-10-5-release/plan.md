# Implementation Plan: S166 v0.10.5 patch release

**Branch**: `release/0.10.5` | **Date**: 2026-10-05 | **Spec**: [spec.md](spec.md)

## Summary

Prepare v0.10.5 from merged S165 and the remaining changes since v0.10.4. Reuse the established release machinery, align every first-party and generated identity, pass local and hosted gates, and open a candidate PR. Publication follows human approval, merge, and tag push, then independent public verification and a separate records PR.

## Technical Context

**Language/Version**: Existing Rust workspace and pinned toolchain.
**Dependencies**: No new third-party packages.
**Storage**: Versioned source, generated evidence, GitHub PR and release records.
**Testing**: `cargo xtask ci`, `cargo xtask msrv`, `cargo xtask neutral`, docs, notes, specification, supply chain, and hosted PR gates.
**Target**: Existing Windows official package and source checks.
**Scope**: Ten product crates, xtask, isolated lockfiles, generated evidence, changelog, notes, and release handoff.

## Constitution Check

P-1 through P-7 and P-10 remain unchanged because this slice changes release identity, not capture behavior. P-8 requires final-head source and hosted gates. P-9 requires exact public evidence before publication claims. P-11 requires specification applicability to move with candidate version while published markers remain at the verified release. The human review and tag gates in `CONTRIBUTING.md` remain in force.

## Decisions

1. Use the established `release/0.10.5` branch because `release.toml` permits the version-only cargo-release command there.
2. Include all five current fragments, including S164 publication records and the platform token decision, because they are unreleased changelog content on merged main.
3. Keep v0.10.4 current-publication markers until exact v0.10.5 public files and registry records are reconciled.
4. Treat the operator's no-input request as authorization for all routine preparation, branch push, PR, and release checks. Human review, merge, and tag push cannot be performed by the agent under repository governance.
5. Keep game-specific field testing outside release acceptance. S165's controlled lifecycle tests establish the scoped correction; real-title compatibility is unverified.

**Execution note, 2026-10-06 UTC:** The owner merged PR #454 and then explicitly instructed the agent to push the exact `v0.10.5` tag. That later direct instruction superseded decision 4 for this tag push only. The tag was verified as annotated and bound to the owner-merged source before publication continued. GitHub records no formal PR approval review object; the owner performed the merge.

## Implementation Sequence

1. Commit the analyzed specification gate before candidate mutation.
2. Preview and perform the version-only bump, then update generated, supply-chain, changelog, notes, and specification candidate identity.
3. Run local gates, push the branch, open the PR, resolve findings, and require green final-head hosted CI.
4. After human merge and tag push, verify release jobs, public files, certification, and crates before a separate reviewed records PR.

## Post-Design Constitution Recheck

The design preserves the established source, governance, and public-evidence gates. No constitutional exception is needed.
