# Tasks: S183 v0.10.9 release candidate

## Phase 1 - Setup and Spec-Kit gate

- [x] T001 Confirm merged main, issue #519, absent v0.10.9 tag, branch `release/0.10.9`, and active selector in `.specify/feature.json`.
- [x] T002 Complete specification, clarification, requirements and release checklists, plan, research, data model, contract and quickstart in `specs/183-v0-10-9-release/`.
- [x] T003 Run blocking Spec-Kit analysis on `specs/183-v0-10-9-release/` and resolve critical findings before candidate mutation.

## Phase 2 - User Story 1: Coherent candidate

**Goal**: One v0.10.9 source and artifact identity with published markers left on v0.10.8.

**Independent test**: Version, changelog, notes, generated evidence, graph and spec checks agree.

- [x] T004 [US1] Bump workspace and product versions on `release/0.10.9` in `Cargo.toml`, crate manifests and release-bound lockfiles with no third-party resolution drift.
- [x] T005 [US1] Update embedded versions and regenerate owned goldens in `conformance/native-http-tls/`, `crates/fragcap-cli/tests/`, and `fixtures/`.
- [x] T006 [US1] Assemble chronological `CHANGELOG.md`, add `release-notes/v0.10.9.md`, update `docs/fragcap-specification.md` Applies-To and `docs/plans/README.md`, preserving current public v0.10.8 markers.
- [x] T007 [US1] Measure and rebind exact first-party dependency authorities in `supply-chain/` and verify no unreviewed third-party change.

## Phase 3 - User Story 2: Controlled qualification

**Goal**: Evidence for the exact candidate without overstating physical or published behavior.

**Independent test**: Local checks, current advisories, packages and physical trust receipt have exact identities and terminal outcomes.

- [x] T008 [US2] Run product and isolated locked advisory checks and graph diff, recording exact results in `specs/183-v0-10-9-release/verification.md`.
- [x] T009 [US2] Run `cargo xtask ci`, MSRV, neutral, docs, release-notes and text gates; record results in `specs/183-v0-10-9-release/verification.md`.
- [ ] T010 [US2] Run controlled Windows package certification and physical current-user trust attempt with exact cleanup; record measured or unverified rows in `specs/183-v0-10-9-release/verification.md`.

## Phase 4 - User Story 3: Review-ready PR

**Goal**: A green reviewed candidate PR ready for owner review and merge.

**Independent test**: Final head, checks, review round count and every actual thread reconcile.

- [ ] T011 [US3] Commit candidate and push `release/0.10.9`; open and attach the official PR referencing #519 without closing it.
- [ ] T012 [US3] Inspect exact-head hosted checks and automatic bot comments, fix findings, answer and resolve every actionable thread; record in `specs/183-v0-10-9-release/verification.md`.
- [ ] T013 [US3] If useful, request at most one explicit second `@Codex` round; resolve all resulting threads and reconfirm green final-head checks.
- [ ] T014 [US3] Hand the final PR SHA, check/review receipts and physical limits to the owner, with merge and publication pending.

## Dependencies

T001-T003 precede candidate changes. T004-T007 prepare one source identity. T008-T010 qualify it. T011-T014 publish and reconcile the PR. A changed PR head renews exact-head checks. Issue #519 closes only after later public delivery.

## Parallel opportunities

Independent read-only graph, documentation and package inventory may run together. Version, generated evidence, changelog and graph authority writes are sequential because they bind one candidate identity. No write task is marked parallel.

## Implementation strategy

Complete each phase in order, retain an exact source-to-evidence chain, and stop before merge/tag/publication as the owner requested.
