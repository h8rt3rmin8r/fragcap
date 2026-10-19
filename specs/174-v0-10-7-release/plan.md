# Implementation Plan: S174 v0.10.7 publication

**Branch**: `release/0.10.7` | **Date**: 2026-10-19 | **Spec**: [spec.md](spec.md)

## Summary

Publish accepted merged S173 and every unreleased fragment since v0.10.6 as v0.10.7 under #481. Prepare and certify one internally consistent candidate, integrate through checked PRs under explicit owner delegation, tag exact accepted merged source, complete normal protected registry approval, independently verify all public bytes, then reconcile current publication and site state. No IGDB work.

## Technical Context

**Language/Version**: Existing Rust workspace, version 0.10.6 to 0.10.7, MSRV 1.88 and existing pinned tools.

**Primary Dependencies**: Existing cargo-release, xtask, GitHub Actions package certification and registry workflow; no new dependency or runtime behavior planned.

**Storage**: Release metadata, generated evidence and public artifacts only; no owner application store or bundle changes.

**Testing**: Existing release identity/spec/notes/supply-chain/review/package validators, generated golden tests, complete cargo xtask ci, MSRV/neutral, site production export/accessibility, exact-head hosted checks, final tagged Windows certification and independent public file/crate checksum reconciliation.

**Target Platform**: Existing x86_64 Windows packages and ten registry crates; site production deployment.

**Project Type**: Release orchestration and truthful publication records.

**Performance Goals**: Existing short production-path performance gate remains mandatory; manual two-hour soak is not newly required.

**Constraints**: Exact tag/source and closed package identity, normal TLS/checksum, no Npcap redistribution, no changed environment protection, hidden/noninteractive child launch, UTF-8 without BOM/LF, no owner-sensitive runtime execution.

**Scale/Scope**: One candidate PR and separate actual-publication records PR; six downloadable files, three primary-sidecar pairs, ten crates.

## Constitution Check

P-1 through P-10 remain unchanged: no new target interaction, architecture, loss path, output format, vocabulary or wrapper logic. P-8 requires full spec-kit analysis, dated release decision, text hygiene and existing gates. P-11 keeps actual published v0.10.6 separate from 0.10.7 candidate applicability until public delivery is verified. Latest explicit owner instruction delegates release-specific PR merges, tag, normal protected approval and publication without another prompt; this overrides the ordinary checkpoint only for this release and does not amend global instructions or protection settings. Pre-design gate passes.

## Project Structure

Candidate edits cover Cargo.toml, Cargo.lock, fuzz/Cargo.lock, performance/native-proxy/Cargo.lock, sink version assertions, CLI Windows integration identity, conformance/native-http-tls/matrix-v1.json and report-v1.json, docs/security/native-product-review-record.v1.json, master Applies-To, generated fixture/CLI goldens, supply-chain/policy-v1.json, release-notes/v0.10.7.md, docs/maintainers/v0.10.7-release-handoff.md, chronological docs/plans/README.md and this slice. Existing CHANGELOG.md is assembled only through the release command. Actual publication later updates docs/published-release.json, its mechanically enumerated current markers and frozen review-candidate registry through a separate records PR.

## Decisions and Research

1. Use established release/0.10.7 because release.toml allows only release/*; explicit version-only cargo-release commands avoid the clean-main wrapper after spec commit. Keep --no-publish --no-tag --no-push during preparation; preview first.
2. Include all existing fragments plus one dated S174 release decision; use cargo xtask changelog --release 0.10.7 2026-10-19 once. Do not mutate historical release sections.
3. Refresh isolated locks offline. Snapshot governed graphs and compare third-party identities/features/counts before rebinding first-party version digests; preserve review dates and exception expiry.
4. Existing package/report authorities already include S173's exact User32 import. Do not touch SDK URL/pin, workflows or wrappers without a demonstrated required failure and dated decision.
5. Automatic candidate review is round one. At most one final manual request is round two, with every finding reconciled. Dedicated security report availability is disclosed truthfully; bot review does not imply independent whole-product audit. Records PR automatic review is distinct record-only acceptance and must not retrigger candidate review.
6. Existing crates-io configuration currently requires owner46768484, permits self-review, retains admin bypass and permits only v* tags. Use its normal pending-deployment approval under owner delegation, never policy weakening or bypass.
7. After exact-head green checks and actual findings resolved, merge candidate through its PR with expected-head protection, fetch/verify exact accepted tree, then push annotated tag to that exact merged source. No direct main push or tag move.
8. Download six unauthenticated release files and ten registry archives, verify sizes/digests/sidecars, certification/source and non-yanked state before records markers move. Never install downloaded product locally.

## Implementation Sequence

1. Complete specify/clarify/checklists/design/tasks and blocking analysis; commit specification gate.
2. Preview/execute version-only preparation, update explicit candidate authorities and regenerate goldens sequentially.
3. Rebind only measured first-party graph digests, assemble every fragment and bounded highlights, prepare handoff and run full verification.
4. Publish candidate PR, reconcile bounded reviews and all final-head checks, merge under delegated authority and tag accepted merged source.
5. Follow all four tagged workflow jobs; submit normal protected registry approval and independently reconcile public files/crates.
6. Prepare records-only PR with actual immutable authorities, validate repository/site/public links and automatic review, merge after exact-head checks and verify deployment.
7. Close #481 only after actual acceptance, clean merged branches/local feature state and stop any release timer.

## Post-Design Constitution Recheck

Pass. Owner delegation is recorded rather than changing future workflow rules. Candidate status cannot substitute for public publication. No gate, security test, package inventory, pin or historical evidence is weakened.

## Complexity Tracking

No new architecture. Existing release mechanism and validators are reused end to end.
