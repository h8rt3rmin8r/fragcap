# Implementation Plan: S180 v0.10.8 publication

**Branch**: `release/0.10.8` | **Date**: 2026-10-23 | **Spec**: [spec.md](spec.md)

## Summary

Publish all merged work after v0.10.7 as a coherent v0.10.8 release, then reconcile actual public records. The owner explicitly delegates the candidate and records PRs, integration, annotated tag, normal protected approval and publication for S180. IGDB remains separate.

## Technical Context

**Language/Version**: Existing Rust workspace, 0.10.7 to 0.10.8, MSRV 1.88.

**Dependencies**: Existing cargo-release, xtask, package certification and registry release workflows; no new third-party package.

**Storage**: Release metadata and generated evidence only. No game store, capture or bundle mutation.

**Verification**: Spec-Kit analysis, local cargo xtask ci and additional release gates, exact-head hosted CI, bounded reviews, tagged certification, unauthenticated public asset checks, registry archive reconciliation and site deployment.

**Scope**: One reviewed release candidate PR, one separate published-records PR, six release files, three checksum pairs and ten crate archives.

## Constitution Check

P-1 through P-10 retain their existing techniques, artifact protections and mechanical gates. P-11 requires candidate and actual publication states to remain separate. Release preparation is a version and evidence change, with no target execution or new proxy behavior. The owner explicitly authorizes this release's publication steps, which satisfies the autopilot pre-push and tag authorization requirement without changing repository governance. The normal protected registry approval remains mandatory. Pre-design gate passes.

## Structure

Candidate changes belong in `specs/180-v0-10-8-release/`, the workspace and isolated lockfiles, existing generated golden and conformance authorities, supply-chain graph policy, master specification version, changelog, release notes and chronological release plan. Current public markers stay on v0.10.7 in the candidate. A later records branch updates `docs/published-release.json`, current documentation markers and review-candidate authority only after public reconciliation.

## Decisions

1. Use patch version 0.10.8 because this release collects scoped corrections in S175 through S179 and the S174 publication record. The existing patch-release pattern is the closest fit; no new feature is added.
2. Use `release/0.10.8`, the branch family required by `release.toml`. Preview the existing Windows release script and run version-only preparation with explicit no-tag, no-push and no-publish flags.
3. Include every `changelog.d/` fragment since v0.10.7 once, with a dated S180 decision. Keep the source release current markers at 0.10.7 until actual delivery.
4. Rebind measured first-party graph digests only after inspecting the exact resolved graph. Preserve third-party versions, features, policy exceptions and review dates.
5. Automatic Codex code/security review is the first round. At most one explicit second candidate round is allowed. Resolve every actual finding, and disclose any absent review receipt without claiming an independent audit.
6. Integrate only an exact reviewed head through a PR, verify the resulting squash tree, and tag that accepted merged commit once. Normal crates.io approval follows existing protection; no bypass or protection edit.
7. Verify public assets and registry archives independently before a separate records PR advances current markers and site state. Keep historical review and field-testing non-claims intact.

## Implementation Sequence

1. Specify, clarify, checklist, plan, tasks and blocking analysis on the literal S180 branch and feature directory.
2. Preview and prepare version, changelog, generated evidence, graph digests and highlights. Run complete local release gates.
3. Push candidate, open official PR, reconcile checks and bounded reviews, merge exact reviewed candidate and annotate the accepted source with v0.10.8.
4. Follow tagged certification, release and protected registry publication. Independently verify all public files and archives.
5. Create and merge a separate publication-records PR, verify Pages deployment and links, then clean branches and local selector.

## Post-Design Constitution Recheck

Pass. The release procedure retains normal protection, review, package and data-integrity boundaries. No partial publication is represented as complete.
