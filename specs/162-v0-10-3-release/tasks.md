# Tasks: S162 v0.10.3 patch release

## Phase 1: Specification gate

- [x] T001 [US1] Create `specs/162-v0-10-3-release/spec.md` with candidate, publication, and records requirements tied to #439.
- [x] T002 [US1] Resolve version, operator merge, tag authorization, approval, and published-marker clarifications in the spec.
- [x] T003 [US1] Complete requirements and release checklists in `specs/162-v0-10-3-release/checklists/`.
- [x] T004 [US1] Create plan, research, data model, contract, and quickstart artifacts.
- [x] T005 [US1] Run blocking Spec-Kit cross-artifact analysis, correct findings, and record `analyze.md` before candidate changes.
- [x] T006 [US1] Commit the S162 specification gate before changing candidate versions or generated outputs.

## Phase 2: Reviewed candidate

- [ ] T007 [US1] Dry-run the patch wrapper, then execute its version-only cargo-release command on the existing `release/0.10.3` branch, moving ten workspace and lockfile package versions.
- [ ] T008 [US1] Inspect versioned embedded output, conformance, Windows staged identity, specification applicability, and regenerated golden files.
- [ ] T009 [US1] Assemble every unreleased fragment into chronological v0.10.3 changelog entries.
- [ ] T010 [US1] Add bounded `release-notes/v0.10.3.md` highlights and validate rendered notes.
- [ ] T011 [US1] Add candidate handoff and plan records while retaining public v0.10.2 markers.
- [ ] T012 [US1] Run focused version, changelog, notes, output, documentation, and specification checks.
- [ ] T013 [US1] Run full source CI, MSRV, neutral, and related release gates in the foreground.
- [ ] T014 [US1] Check UTF-8 without BOM, LF, final newline, whitespace, forbidden dash, and mojibake hygiene.
- [ ] T015 [US1] Commit and push the complete candidate, open the official PR referencing #439, and attach it to this task.
- [ ] T016 [US1] Address every first-round review comment and hosted failure; allow at most one second Codex review round.
- [ ] T017 [US1] Require green final-head PR checks and ask the operator to merge the reviewed candidate.

## Phase 3: Public release

- [ ] T018 [US2] After owner merge, sync clean main and verify exact merged candidate identity.
- [ ] T019 [US2] Reject a conflicting v0.10.3 tag, then create and push an annotated tag at the exact merged source.
- [ ] T020 [US2] Monitor identity, package certification, GitHub release, and crates.io jobs without blind reruns or policy changes.
- [ ] T021 [US2] If GitHub pauses `crates-io`, give the operator its exact approval link and continue after the owner acts.
- [ ] T022 [US2] Require four green release jobs and record run, job, and approval evidence.
- [ ] T023 [US2] Verify public release visibility and all six files, names, sizes, and SHA-256 identities.
- [ ] T024 [US2] Validate three checksum sidecars and downloaded certification evidence against exact tag source.
- [ ] T025 [US2] Verify all ten 0.10.3 crates are available, non-yanked, and checksum-bearing.

## Phase 4: Published-state records

- [ ] T026 [US3] Create a separate records-only branch from post-release main.
- [ ] T027 [US3] Update `docs/published-release.json`, current baseline markers, handoff, and S162 verification from observed public evidence.
- [ ] T028 [US3] Preserve historical v0.10.2 evidence and keep external field observations accurately scoped.
- [ ] T029 [US3] Run records-focused and full repository gates and text hygiene checks.
- [ ] T030 [US3] Push the records PR, address at most two review rounds, and require final-head green CI.
- [ ] T031 [US3] Ask the operator to merge records, then close #439 and update its project item if applicable.

## Dependencies and independent acceptance

T001 through T006 precede all candidate changes. T007 through T014 form the candidate preparation and verification transaction. T015 through T017 require the complete candidate. T018 through T025 require the operator's candidate merge; T021 is conditional on GitHub pausing the registry job. T026 through T031 require exact public verification.

US1 independently proves internally consistent candidate source while published markers remain at v0.10.2. US2 independently proves exact tagged public distribution and registry publication. US3 independently proves that only observed publication evidence moves current records.
