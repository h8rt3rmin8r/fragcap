# Tasks: S159 v0.10.2 release

## Phase 1: Specification and planning

- [x] T001 [US1] Create `specs/159-v0-10-2-release/spec.md` with bounded candidate, publication and reconciliation requirements.
- [x] T002 [US1] Resolve release-version, authority and two-pull-request clarifications in the specification.
- [x] T003 [US1] Complete requirements and release-safety checklists.
- [x] T004 [US1] Create research, data model, publication contract, quickstart and implementation plan.
- [x] T005 [US1] Run blocking cross-artifact analysis and record `analyze.md` before candidate implementation.

## Phase 2: Candidate preparation

- [x] T006 [US1] Commit the S159 specification gate before modifying candidate version or release outputs.
- [x] T007 [US1] Move all ten workspace packages and lockfile identities from 0.10.1 to 0.10.2 using the established version-only release command.
- [x] T008 [US1] Update embedded output, conformance, staged Windows and specification Applies-To identities to 0.10.2.
- [x] T009 [US1] Regenerate every version-bearing golden through its owning test generator.
- [x] T010 [US1] Assemble S153 through S158 fragments into the dated v0.10.2 changelog and verify chronological order.
- [x] T011 [US1] Add bounded `release-notes/v0.10.2.md` highlights and validate their rendered output.
- [x] T012 [US1] Update candidate-only release handoff and planning records while preserving all actual v0.10.1 publication markers.

## Phase 3: Candidate verification and review

- [x] T013 [US1] Run focused version, changelog, note and generated-output checks.
- [x] T014 [US1] Run formatting, all-target all-feature clippy, locked workspace tests, aggregate CI, MSRV and neutral gates.
- [x] T015 [US1] Check strict UTF-8 without BOM, forbidden dash characters and mojibake across changed text.
- [ ] T016 [US1] Commit the complete candidate, push `release/0.10.2`, open the official closing pull request for #431 and attach it to this task.
- [ ] T017 [US1] Observe every first-round review and hosted check, address all findings, reply and resolve every thread.
- [ ] T018 [US1] Request at most one second Codex review if needed, address all resulting findings and require every final-head check green.
- [ ] T019 [US1] Ask the owner for the mandatory human merge only after candidate review and CI are complete.

## Phase 4: Public release

- [ ] T020 [US2] After merge, sync clean main and verify exact local, remote and merged-candidate identity.
- [ ] T021 [US2] Verify no conflicting v0.10.2 tag exists, create the tag at exact merged source and push it.
- [ ] T022 [US2] Monitor identity, package certification and GitHub release jobs without blind reruns or policy changes.
- [ ] T023 [US2] If the publish job waits, request the owner's protected-environment approval and continue after it is supplied.
- [ ] T024 [US2] Require all four release jobs green and record exact run, job and approval evidence.
- [ ] T025 [US2] Verify the public non-draft release and six expected file names, sizes and SHA-256 digests without installing or executing them.
- [ ] T026 [US2] Validate all three checksum sidecars and the downloaded certification summary against the exact tag source.
- [ ] T027 [US2] Verify all ten crates at 0.10.2 are available, non-yanked and carry registry checksums.

## Phase 5: Published-record reconciliation

- [ ] T028 [US3] Create a separate `codex/s159-v0-10-2-publication-record` branch from exact post-release main.
- [ ] T029 [US3] Update published-release JSON, current baseline markers, review candidate identity, release handoff and S159 verification with observed evidence.
- [ ] T030 [US3] Preserve historical v0.10.1 evidence and keep #372, #333, #334 and #278 open with accurate scope.
- [ ] T031 [US3] Run records-focused and full repository gates, text-integrity checks and published-site validation.
- [ ] T032 [US3] Push the records branch, open the official pull request, address no more than two review rounds and require final-head green CI.
- [ ] T033 [US3] Ask the owner for the final records merge, then complete #431 and set its Project item to Done after merge.

## Dependencies and execution order

- T001 through T006 block all candidate changes.
- T007 through T012 form one ordered release-preparation transaction.
- T013 through T019 require the complete candidate.
- T020 through T027 require the owner's candidate merge.
- T023 is conditional on GitHub requesting environment approval.
- T028 through T033 require complete public verification.

## Independent acceptance

- **US1**: Candidate version, outputs, changelog and notes agree while the published baseline remains v0.10.1.
- **US2**: Exact tag, four green jobs, six certified files and ten non-yanked crates establish live v0.10.2.
- **US3**: A later records-only PR moves current baseline markers using observed evidence and preserves every unresolved external gate.
