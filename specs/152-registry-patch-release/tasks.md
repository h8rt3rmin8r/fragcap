# Tasks: S152 registry approval and patch release

**Input**: [spec.md](spec.md), [plan.md](plan.md), research, data model and release-guard contract.

## Phase 1: Setup

- [x] T001 Establish S152 spec, clarifications and requirements checklists under specs/152-registry-patch-release/.
- [x] T002 Record scoped authorization, immutable published baseline and hidden-tooling boundary in specs/152-registry-patch-release/plan.md.

## Phase 2: Foundation

- [x] T003 Complete independent read-only environment and release research plus design artifacts in specs/152-registry-patch-release/research.md, data-model.md, contracts/release-guard.md and quickstart.md.
- [x] T004 Pass blocking requirement/task/constitution analysis in specs/152-registry-patch-release/analysis.md before implementation.

## Phase 3: User Story 1 - Registry approval (Priority: P1)

**Goal**: Exact verified protection, fresh fail-closed automation and retained operator approval.

**Independent Test**: Pure protection and wiring fixtures reject every weakened/incomplete policy without GitHub writes or registry execution.

- [x] T005 [US1] Add negative protection and release-wiring tests in xtask/src/release_guard.rs, observing red before implementation.
- [x] T006 [US1] Implement bounded metadata policy validation and command dispatch in xtask/src/release_guard.rs and xtask/src/main.rs.
- [x] T007 [US1] Add fresh read-only verification before certification/release creation and immediately before registry execution in .github/workflows/release.yml; wire offline validation into xtask/src/main.rs ordinary CI.
- [x] T008 [US1] Configure only existing approved reviewer and tag policy, verify administrator bypass and unchanged environment identity, and record scrubbed before/after evidence in specs/152-registry-patch-release/verification.md. Any unsupported settings action stays operator-owned.
- [x] T009 [US1] Align exact reproducible operator approval instructions in release.toml and docs/maintainers/v0.10.1-release-handoff.md, with a dated changelog.d/S152-registry-patch-release.decisions.md entry (consumed into CHANGELOG.md during preparation).

## Phase 4: User Story 2 - Doctor patch candidate (Priority: P2)

**Goal**: Checked 0.10.1 candidate containing S151, with published 0.10.0 untouched.

**Independent Test**: Candidate identity/golden/conformance/spec/notes/package gates reconcile all ten crates and do not change any actual publication marker.

- [x] T010 [US2] Preview and execute installed version-only cargo-release for 0.10.1 in Cargo.toml and Cargo.lock; reconcile first-party metadata in fuzz/Cargo.lock and performance/native-proxy/Cargo.lock without dependency upgrades.
- [x] T011 [US2] Align writer assertions, staged-binary assertion, portable conformance matrix/report and not-started review record in crates/fragcap-sink/src/{pcapng,json}/mod.rs, crates/fragcap-cli/tests/windows_native_integration.rs, conformance/native-http-tls/ and docs/security/native-product-review-record.v1.json; bind xtask/src/review_handoff.rs fixture to candidate version.
- [x] T012 [US2] Regenerate owned synthetic corpus and CLI output goldens in fixtures/goldens/ and crates/fragcap-cli/tests/goldens/ using the three existing generators; inspect changes.
- [x] T013 [US2] Reconcile candidate Applies-To and dated architecture/release decisions in docs/fragcap-specification.md and docs/plans/README.md, retaining all actual v0.10.0 baseline markers.
- [x] T014 [US2] Refresh first-party-only reviewed graph digest in supply-chain/policy-v1.json from existing snapshot command without changing expiry, exceptions, third-party packages or acceptance claims.
- [x] T015 [US2] Assemble S151/S152 candidate release records via the existing release-only changelog generator into CHANGELOG.md and author release-notes/v0.10.1.md; validate short notes and preserved prior history.

## Phase 5: Cross-Cutting Verification and Handoff

- [x] T016 Run targeted tests, full watched cargo xtask ci/MSRV and documentation site build/unit/browser contracts; record exact evidence in specs/152-registry-patch-release/verification.md.
- [x] T017 Check UTF-8/no-BOM/LF, no mojibake, clean patch scope and unchanged published-release.json; commit with conventional message and required co-author trailer, then automatically push release/0.10.1 and open official PR against main.
- [ ] T018 Monitor current-head hosted CI and every bot review, reply to all comments, verify corrections and resolve threads; retain at most one manual second review request in specs/152-registry-patch-release/verification.md.
- [ ] T019 Update #416/project tracking to PR review with truthful dispositions and hand the official green PR to the operator, without closing independent #372/#331/#333/#413/#334/#278 or deferred #155/#94 and without merge/tag/publication.

## Dependencies and Execution Order

T001-T004 block implementation. US1 tests precede validator and workflow changes; configuration depends on the approved design and live reread. US2 version preparation precedes embedded reconciliation, golden generation and graph digest refresh. Release notes validation follows changelog assembly. Full gates precede commit/push; current-head hosted gates and complete review dispositions precede human handoff. US1 is the minimal protection increment, but S152 delivers both approved stories.

## Parallel Opportunities

Independent read-only policy research and release-identity research ran concurrently as requested by the plan skill. After foundations, environment setting verification can proceed independently of candidate notes authoring; US1 fixture policy checks and US2 immutable-baseline inspection do not require product effects. Cargo verification and rebuilding are sequential on Windows to avoid executable locks. No implementation task is marked parallel where it shares files or depends on an incomplete prerequisite.

## Implementation Strategy

Protect publication first using failing negative tests and fresh verification, then prepare and reconcile the patch identity without actual publication. Incremental controlled checks precede the full gate. Configuration success is established only from readback; required operator settings are not skipped. Official PR and bot-review handling follow explicit user authorization, with final human merge and a later separately authorized release remaining outside S152.
