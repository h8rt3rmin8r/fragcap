# Tasks: S174 v0.10.7 publication

## Phase 1 - Setup

- [x] T001 Verify clean merged base, latest release, absent conflicting tag and literal S174 branch/directory identity; record specs/174-v0-10-7-release/spec.md and issue #481.
- [x] T002 Complete clarify and requirement/release checklists in specs/174-v0-10-7-release/checklists/.

## Phase 2 - Foundation

- [x] T003 Complete research, plan, data-model, contracts/publication.md and quickstart in specs/174-v0-10-7-release/.
- [x] T004 Run blocking installed spec-kit analysis and commit specification gate; record analysis.md.

## Phase 3 - Accurate candidate (US1)

**Goal**: Coherent release candidate containing every accepted correction.

**Independent test**: Existing identity, generated golden, notes, supply-chain and package authorities agree on0.10.7; published markers remain0.10.6.

- [ ] T005 [US1] Preview/execute version-only cargo-release for Cargo.toml and Cargo.lock; refresh fuzz/Cargo.lock and performance/native-proxy/Cargo.lock offline.
- [ ] T006 [US1] Update sink json/mod.rs and pcapng/mod.rs assertions, crates/fragcap-cli/tests/windows_native_integration.rs, conformance/native-http-tls/matrix-v1.json and report-v1.json, docs/security/native-product-review-record.v1.json and master Applies-To/revision.
- [ ] T007 [US1] Regenerate fixtures/goldens/ and crates/fragcap-cli/tests/goldens/ using existing facade, CLI capture and extcap update switches sequentially.
- [ ] T008 [US1] Snapshot governed graphs, compare third-party versions/features/counts, and rebind only measured digests in supply-chain/policy-v1.json while preserving dates/expiry.
- [ ] T009 [US1] Add dated release decision, assemble all changelog.d fragments once through xtask into CHANGELOG.md, and author release-notes/v0.10.7.md.
- [ ] T010 [US1] Prepare docs/maintainers/v0.10.7-release-handoff.md and chronological docs/plans/README.md entry; run full local CI, MSRV/neutral, fresh audit, site export/accessibility and strict changed-file hygiene; record verification.md.

## Phase 4 - Accepted integration (US3)

**Goal**: Reviewable candidate becomes exact accepted merged source under explicit owner delegation.

**Independent test**: All exact-head checks green, actual findings resolved and expected-head merge tree matches reviewed candidate.

- [ ] T011 [US3] Commit/push release/0.10.7, create and attach official candidate PR, and record review ledger in verification.md.
- [ ] T012 [US3] Reconcile automatic and at most second/final requested review, every actual thread and final-head hosted checks; fix only demonstrated scoped failures with required verification.
- [ ] T013 [US3] Merge accepted candidate PR under explicit delegation with expected head, fetch and verify exact tree, then push annotated v0.10.7 on exact merged source; record tag/merge identities in release handoff.

## Phase 5 - Public delivery (US2)

**Goal**: All release files and registry crates actually publish and independently reconcile.

**Independent test**: Tagged certification succeeds before release creation; normal approval permits ten crates; unauthenticated downloads and archives match their exact authorities.

- [ ] T014 [US2] Follow all four release workflow jobs and submit normal crates-io pending-deployment approval after fresh protection validation, recording actual workflow/approval in verification.md.
- [ ] T015 [US2] Independently download/reconcile six public files, three sidecars and schema4 certification report through existing report validator; record all identities in docs/maintainers/v0.10.7-release-handoff.md.
- [ ] T016 [US2] Independently verify ten non-yanked0.10.7 crate records and downloaded archive checksums; record exact identities in release handoff.
- [ ] T017 [US2] Prepare separate records branch advancing docs/published-release.json, fourteen current markers, docs/audits/native-documentation-coverage.v2.json, docs/security/native-product-review-candidate.v1.json, literal assets in xtask/src/review_record.rs and replay workflow label with dated decision.
- [ ] T018 [US2] Run repository/site/accessibility and hygiene gates for records changes, publish/attach records PR and reconcile every automatic finding plus final-head checks; record actual evidence in verification.md.
- [ ] T019 [US2] Merge records PR under delegation after checks, verify site deployment/public links and current release references; update #481 acceptance only from actual results.

## Phase 6 - Completion

- [ ] T020 Reconcile final verification.md and release handoff/task status without inventing future checks; external exact-head receipts may live in PR comments.
- [ ] T021 Close #481 after actual acceptance, clean merged branches and local feature state, stop any release timer and report immutable release URL/source and evidence limits.

## Dependencies and execution

T001-T004 precede version mutation. T005-T009 establish US1 identity, T010 verifies it. T011-T013 govern US3 integration before T014. T014-T016 precede any public-marker change. T017-T019 complete US2 public records; T020-T021 finish the complete requested outcome. Research agents may inspect independent files while authoring, but all mutations and Cargo operations are serial on Windows. No task is complete solely because a future operator/reviewer run is planned.

## Requirement Coverage

FR-001:T005,T006,T007,T009; FR-002:T009,T010; FR-003:T006,T017; FR-004:T010,T012,T018; FR-005:T013; FR-006:T014,T015; FR-007:T014; FR-008:T015,T016; FR-009:T017,T018,T019; FR-010:T001,T002,T013,T019; FR-011:T010,T012,T014,T015,T016,T020,T021. SC-001:T005-T013; SC-002:T014-T016; SC-003:T017-T019; SC-004:T013-T021.

## Implementation strategy

Candidate preparation is independently reviewable but not final completion. Deliver candidate, actual publication and records as one release outcome. Reuse existing validators rather than adding tests that mirror version replacements; preserve mandatory security/package tests. Current public state and history remain authoritative throughout partial delivery.
