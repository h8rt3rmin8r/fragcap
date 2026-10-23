# Tasks: S180 v0.10.8 publication

## Phase 1 - Specification and baseline

- [x] T001 Verify clean accepted main, current v0.10.7 public release, absent v0.10.8 tag and separate #155 scope; create `release/0.10.8` and literal S180 feature selector.
- [x] T002 Complete specification, autonomous clarifications and requirements-quality checklists.
- [x] T003 Complete plan, research, data model, publication contract and quickstart.
- [x] T004 Run blocking Spec-Kit analysis across specification, plan, tasks and constitution; resolve all critical findings before implementation.

## Phase 2 - Coherent candidate

**Goal**: One reviewable v0.10.8 source and artifact identity that contains all work merged since v0.10.7.

**Independent test**: Existing version, generated-evidence, supply-chain, notes, changelog, CI and package gates agree while actual public markers remain at v0.10.7.

- [x] T005 Preview and execute patch-only version preparation; update root and isolated lockfiles without altering third-party resolution.
- [x] T006 Update all explicit first-party version assertions and regenerate fixture, CLI, conformance and review-record evidence using existing generators.
- [x] T007 Snapshot governed graphs, compare third-party identities and features, and rebind measured first-party graph digests only.
- [x] T008 Add a dated S180 release decision, assemble all unreleased fragments exactly once, write short v0.10.8 highlights and validate notes.
- [x] T009 Update chronological release plan and release handoff. Run full local CI, MSRV, neutral, fresh audit, site export/accessibility and encoding/whitespace checks; record exact results.

## Phase 3 - Accepted integration

**Goal**: An exact reviewed candidate becomes the merged release source.

**Independent test**: Every required exact-head check is green and every actual review thread resolved before a PR merge whose resulting tree matches the reviewed candidate.

- [x] T010 Commit/push the release branch, open and attach the official candidate PR.
- [x] T011 Reconcile automatic and at most one requested second Codex review round, security feedback and exact-head hosted checks; fix scoped failures and respond to every finding.
- [x] T012 Merge the accepted candidate under owner delegation, verify the exact merged tree, create and push one annotated v0.10.8 tag on that commit.

## Phase 4 - Public delivery

**Goal**: Certified Windows assets and ten crates are actually public and internally consistent.

**Independent test**: Tagged certification and normal protected approval finish, public bytes match their authority, and registry archives match their recorded checksums.

- [x] T013 Follow every tagged release job; submit normal crates.io protected approval without changing protection or bypassing it.
- [x] T014 Independently reconcile six public files, three checksum sidecars and final certification report against the exact tagged source.
- [x] T015 Independently reconcile ten non-yanked v0.10.8 crates and downloaded archive digests.

## Phase 5 - Current public records and completion

- [x] T016 Prepare separate records branch updating only verified current markers, published-release JSON, review-candidate authority and dated decision.
- [ ] T017 Run local records/site gates, open and attach records PR, resolve actual reviews and exact-head checks, then merge under delegation.
- [ ] T018 Verify production site deployment and public links, clean merged release branches/selector/worktrees, and report exact immutable identities plus unverified field/audit limits.

## Dependencies

T001-T004 precede release mutation. T005-T009 prepare and verify the candidate. T010-T012 precede the tagged release. T013-T015 precede any public marker change. T016-T018 complete actual public-state reconciliation. No future external check is complete because it is planned.
