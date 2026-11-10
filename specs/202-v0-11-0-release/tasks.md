# Tasks: v0.11.0 Direct Route Release

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), [publication contract](contracts/publication.md)

**Sequence**: Tasks are chronological. Candidate work stops at owner merge; publication resumes under the same slice only after owner authorization.

## Phase 1: Accepted Source And Spec-Kit

- [x] T001 Confirm clean accepted S201 merge, final main checks, v0.10.14 public baseline, v0.11.0 tag absence, and S197-S201 release scope (FR-001, FR-006).
- [x] T002 Create the v0.11.0 release tracking issue and milestone, with #580 explicitly independent (FR-001, FR-010).
- [x] T003 Complete S202 specify, clarify, requirements and release checklists, research, data model, publication contract, plan, and quickstart (FR-001 to FR-010).
- [x] T004 Run non-destructive Spec-Kit analysis on the S202 spec, plan, and tasks; resolve every critical or high finding before release preparation (FR-004).

## Phase 2: Coherent Candidate

- [x] T005 Run the repository's release-preparation dry run on `release/0.11.0`; record the exact expected edits and any blocker (FR-001, FR-002).
- [x] T006 Advance all first-party version and generated identities to v0.11.0 using the repository release authority, with no tag, push, or publication side effect (FR-002).
- [x] T007 Assemble all unreleased S197-S201 fragments into one chronological changelog and write concise v0.11.0 release highlights (FR-001, FR-002).
- [x] T008 Reconcile package, dependency, conformance, CLI, and golden identities without accepting unrelated third-party drift (FR-002, FR-004).
- [x] T009 Reconcile specification, README, contributor guidance, security handoff, site, and parser examples as v0.11.0 candidate material while the current-public marker stays v0.10.14 (FR-002, FR-003).
- [x] T010 Validate the exact candidate version, notes, changelog, documentation, link and parser examples, dependency policy, supply-chain notices, and package contents (FR-002 to FR-004).
- [x] T011 Run the complete local repository gate and relevant Windows or package checks in the foreground; record exact outcomes and evidence limits in `verification.md` (FR-004, FR-010).
- [x] T012 Run post-implementation Spec-Kit convergence and address every substantive finding without widening release scope (FR-004).
- [x] T013 Mark completed candidate tasks, commit the complete release source, and verify the clean branch and exact diff from accepted `main` (FR-001 to FR-004).

## Phase 3: Official Candidate Review

- [ ] T014 Push the authorized `release/0.11.0` branch and open the official candidate PR, referencing the release issue without closing it before publication (FR-005, FR-006).
- [ ] T015 Watch applicable exact-head CI, Windows package certification, docs, and dependency checks to terminal results and repair concrete failures (FR-004, FR-005).
- [ ] T016 Inspect code and security review surfaces, respond to every actual finding, resolve review threads, and request at most one explicit second Codex round (FR-005, FR-010).
- [ ] T017 Reconfirm the final candidate head, checks, review disposition, current-public baseline, and PR merge readiness; hand the PR to the owner for final review and merge (FR-005, FR-006).

## Phase 4: Owner-Controlled Public Release

- [ ] T018 After owner merge, record the exact accepted source and verify no candidate source drift or conflicting v0.11.0 tag (FR-006, FR-007).
- [ ] T019 After separate owner authorization, create and push the annotated v0.11.0 tag selecting the accepted merge, then watch release identity and Windows package certification (FR-006, FR-007).
- [ ] T020 Observe GitHub release creation and the normal owner-approved registry deployment; require terminal successful jobs for both (FR-006, FR-007).
- [ ] T021 Independently retrieve six public files, compare three checksum sidecars, public API digests, and package certification report against the tagged source (FR-008).
- [ ] T022 Independently verify all ten non-yanked v0.11.0 registry archives and their checksums (FR-008).
- [ ] T023 Reconcile the complete public receipt set before claiming v0.11.0 current (FR-008, FR-009).

## Phase 5: Current Records And Housekeeping

- [ ] T024 Advance published-release metadata, current-version documentation, frozen review candidate, and release handoff only from verified public receipts; preserve historical v0.10.14 records (FR-009).
- [ ] T025 Validate records locally, publish a reviewed records PR, reconcile its checks and actual review findings, and obtain owner merge (FR-004, FR-005, FR-009).
- [ ] T026 Verify public site deployment and current v0.11.0 guidance; close release tracking and milestone from public evidence (FR-009).
- [ ] T027 Sync `main`, remove spent release branches and local Spec-Kit pointer, and confirm final GitHub, registry, site, issue, and repository state (FR-009).

## Dependency Order

- T001 through T004 establish release scope before version edits.
- T005 through T013 create one reviewable candidate. T014 through T017 require that candidate and stop at the owner merge boundary.
- T018 through T023 depend on owner merge and separate tag and registry actions. T024 through T027 depend on verified public receipts.
- A missing security-bot receipt is recorded, not retriggered beyond the second round and not treated as an invented publication prerequisite.
