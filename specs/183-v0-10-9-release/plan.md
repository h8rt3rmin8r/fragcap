# Implementation Plan: S183 v0.10.9 release candidate

**Branch**: `release/0.10.9` | **Date**: 2026-10-29 | **Spec**: [spec.md](spec.md)

## Summary

Prepare one patch candidate from merged S182 and corrected dependency source, qualify it on controlled and hosted gates, push an official PR, close actual review threads, and hand the accepted head to the owner before merge. Issue #519 remains open for later publication reconciliation.

## Technical Context

**Language/Version**: Existing Rust workspace, 0.10.8 to 0.10.9, MSRV 1.88.

**Dependencies**: Existing cargo-release, xtask, Windows packaging and advisory tooling. Preserve third-party versions, features and providers unless a fresh advisory requires a correction.

**Storage**: Release metadata and generated evidence only. No game store, capture or sensitive bundle mutation.

**Verification**: Spec-Kit analysis; `cargo xtask ci`, MSRV, neutral, docs and advisory checks; controlled package tests; exact-head hosted checks and bounded reviews. The physical trust row has a separate result.

**Scope**: One release candidate PR covering #519's pre-publication work. Publication, registry approval, current marker changes and real-title testing follow merge and are not claimed here.

## Constitution Check

P-1 through P-10 keep existing product and artifact contracts. P-11 binds candidate Applies-To to the workspace version while current public markers remain v0.10.8. The owner authorized the branch push and PR but reserved final review and merge. Tag and protected registry actions remain owner decisions. Spec-Kit analyze and repository checks are mandatory. Pre-design gate passes.

## Structure

Work in `specs/183-v0-10-9-release/`, release-bound workspace manifests and lockfiles, generated goldens and conformance identities, governed dependency graphs, `docs/fragcap-specification.md`, `CHANGELOG.md`, `release-notes/v0.10.9.md`, and `docs/plans/README.md`. Keep `docs/published-release.json`, current website markers, and historical release records on v0.10.8.

## Decisions

1. Use patch version 0.10.9 because S181 and S182 repair the active baseline and add no new product family.
2. Use `release/0.10.9` because `release.toml` accepts that branch family. The isolated S183 worktree already has the correct branch, so execute the release script's documented constituent operations here instead of asking it to create another branch from `main`.
3. Preserve published v0.10.8 pointers until actual public bytes and registry packages reconcile. Change candidate-bound source authorities only.
4. Rebind first-party graph hashes from a measured exact graph. Inspect any third-party identity or feature delta; no blanket advisory exception.
5. Record the physical trust interaction on exact candidate bytes with explicit cleanup. A declined or unavailable operator decision yields an unverified row.
6. Automatic bot activity is round one. One explicit `@Codex` request may start round two. Answer every actionable comment, resolve threads and rerun exact-head checks after edits.
7. Stop at a green, review-ready PR. Do not merge, tag, approve the protected registry environment, publish, close #519 or advance public markers under this instruction.

## Implementation Sequence

1. Specify, clarify, checklist, plan, tasks and blocking analysis on literal S183 identity.
2. Prepare patch version, generated evidence, changelog, notes and measured graph authorities.
3. Qualify candidate with local and controlled Windows gates, including attempted physical trust row and exact cleanup.
4. Push and open official PR; reconcile exact-head CI and at most two Codex rounds.
5. Hand the final reviewed SHA and unverified limits to the owner for final review and merge ritual.

## Post-Design Constitution Recheck

Pass. Candidate evidence and published release evidence remain separate. No product behavior or security technique changes are introduced by version preparation.
