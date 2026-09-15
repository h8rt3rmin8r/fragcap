# Tasks: S153 v0.10.1 publication

**Input**: [Specification](spec.md), [plan](plan.md), research, data model and publication contract.\
**Tests**: Reuse existing negative policy, publication-order and identity regression tests before effects or records edits; no production code change.

## Phase 1: Setup

- [x] T001 Confirm full ordered artifacts, authorization and clean release-source identity in specs/153-release-publication/verification.md.
- [x] T002 Verify ignored temporary downloads and publication-record branch isolation in .gitignore and .specify/feature.json.

## Phase 2: Foundation

- [x] T003 Run existing release/publish/spec regression tests and short-note validation for xtask/src/release_guard.rs, xtask/src/publish.rs and xtask/src/spec.rs; record output in specs/153-release-publication/verification.md.
- [x] T004 Inspect all applicable merged a7d2496 source workflows, tag absence and fresh public crates-io policy; record exact source gate evidence in specs/153-release-publication/verification.md.

## Phase 3: User Story 1 - Obtain the release

**Goal**: A fully published, green, certified patch without sensitive local execution.\
**Independent Test**: Public assets, certification, ten registry version records and exact release workflow conclusions.

- [x] T005 [US1] Create and push annotated v0.10.1 selecting exact merged a7d2496, preserving existing tags; record tag/source identity in specs/153-release-publication/verification.md.
- [x] T006 [US1] Drive existing .github/workflows/release.yml to completion, requesting only an essential owner environment decision if held; retain failed/partial/retry evidence in specs/153-release-publication/verification.md.
- [x] T007 [US1] Download six official assets and exact-run summary into target/release-verification-v0.10.1/; independently reconcile checksum/size/report version and source without launching product.
- [x] T008 [US1] Verify all ten non-yanked registry versions from xtask/src/publish.rs ORDER and all mandatory release jobs green; record scrubbed identities/digests in specs/153-release-publication/verification.md.

## Phase 4: User Story 2 - Reconcile published baseline

**Goal**: Actual release identity and current documentation agree without erasing history.\
**Independent Test**: Existing fourteen-marker publication gate and documentation checks pass.

- [x] T009 [US2] Update docs/published-release.json only after T008, identifying exact actual 0.10.1 publication.
- [x] T010 [US2] Reconcile fourteen CURRENT_RELEASE_SURFACES listed in xtask/src/spec.rs and current applicability prose, preserving historical evidence and independent acceptance.
- [x] T011 [US2] Record actual publication/checksum/certification and owner decision in docs/maintainers/v0.10.1-release-handoff.md, with a dated changelog.d/S153-publication.decisions.md fragment.
- [x] T012 [US2] Append chronological S153 entry to docs/plans/README.md and preserve unresolved #372/#333/#413/#331/#334/#278 tracking in specs/153-release-publication/verification.md.

## Phase 5: Polish and Handoff

- [x] T013 Run full local cargo xtask ci, short notes, specification, documentation/site and strict encoding/diff checks; record exact outcomes in specs/153-release-publication/verification.md.
- [x] T014 Commit and push bounded records branch, open official human-merge PR with actual released source identity distinct from PR source; record URL in specs/153-release-publication/verification.md.
- [ ] T015 Satisfy every received review comment and all applicable current-head PR checks within the two-round review cap; reconcile S153 tracker and final evidence in specs/153-release-publication/verification.md.
- [ ] T016 Deliver fully live release/checksum/green-check links and separate human-owned records merge/independent acceptance handoff in specs/153-release-publication/verification.md.

## Dependencies and Execution Order

T001-T004 precede US1. US1 orders tag, hosted effects, certified download and complete inventory. US2 requires verified actual publication; its tests are independently runnable afterward. T009 precedes T010, which precedes final gate/PR work. Shared records edits stay serial. Within US1 independent metadata/checksum reads may overlap; within US2 independent marker reads may overlap, but no extra agent work is required. Product release is the first viable deliverable; documentation reconciliation follows without delaying it for another merge.

## Implementation Strategy

Reuse established release machinery, validate before effects, publish once, verify exact public bytes and registry identity, then reconcile records through a narrow PR. A failed boundary remains failed; a warranted registry-only rerun preserves original evidence. Never weaken tests or approve/bypass deployments. Explicit release authorization supersedes the default pre-push pause, not the human-only merge boundary.
