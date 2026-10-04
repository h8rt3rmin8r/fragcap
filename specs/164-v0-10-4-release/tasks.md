# Tasks: S164 v0.10.4 patch release

## Phase 1: Specification gate

- [x] T001 [US1] Create the S164 feature specification from merged S163 and the operator's candidate PR instructions.
- [x] T002 [US1] Resolve candidate, publication, field-trial, review-count, and owner-merge clarifications in `spec.md`.
- [x] T003 [US1] Complete requirements and release checklists under `checklists/`.
- [x] T004 [US1] Create `plan.md`, `research.md`, `data-model.md`, `contracts/release-handoff.md`, and `quickstart.md`.
- [x] T005 [US1] Run the blocking read-only Spec-Kit analysis and record the result in `analyze.md`.
- [x] T006 [US1] Commit the analyzed specification gate before candidate changes.

## Phase 2: Candidate preparation

- [x] T007 [US1] Preview the patch wrapper and run its documented version-only cargo-release command on `release/0.10.4`.
- [x] T008 [US1] Align embedded output, generated goldens, native conformance, all three lockfiles, and supply-chain identity to 0.10.4.
- [x] T009 [US1] Assemble all unreleased fragments chronologically into `CHANGELOG.md`.
- [x] T010 [US1] Author public-safe v0.10.4 highlights and validate notes.
- [x] T011 [US1] Advance specification Applies-To, the release slice plan record, and the candidate handoff while retaining public v0.10.3 markers.
- [x] T012 [US1] Run focused version, changelog, notes, output, specification, supply-chain, and isolated locked-manifest checks.
- [x] T013 [US1] Run full source CI, MSRV, neutral, documentation, and relevant release gates in the foreground.
- [x] T014 [US1] Verify UTF-8 without BOM, LF, final newline, no mojibake, and `git diff --check`.
- [x] T015 [US1] Commit and push the complete candidate, open its official PR, and attach it to this task.
- [ ] T016 [US1] Address every first-round external comment and CI failure; request and address at most one second Codex round.
- [ ] T017 [US1] Require green required checks and no unresolved actionable threads on the final PR head; ask the operator for final review and merge.

## Phase 3: After owner merge and publication authorization

- [ ] T018 [US2] Verify exact owner-merged source and absence of a conflicting v0.10.4 tag.
- [ ] T019 [US2] Create and push the annotated v0.10.4 tag only with separate publication authorization.
- [ ] T020 [US2] Observe the four release jobs and request the owner to approve protected crates.io deployment if needed.
- [ ] T021 [US2] Independently verify six public files, three checksum sidecars, certification, and ten non-yanked crate records.
- [ ] T022 [US2] Prepare a separate records-only PR to advance current published-state markers from observed evidence.

## Dependencies and acceptance

T001 through T006 precede candidate mutation. T007 through T014 precede branch push. T015 through T017 require the final candidate. T018 through T022 require the operator's merge and separate publication authorization, so they are intentionally pending at this handoff. US1 is independently accepted by a green reviewed PR; US2 is independently accepted only by verified public release evidence.
