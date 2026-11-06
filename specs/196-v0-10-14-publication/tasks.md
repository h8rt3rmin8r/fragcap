# Tasks: v0.10.14 Publication

**Input**: Design documents from `specs/196-v0-10-14-publication/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/publication.md, quickstart.md

**Tests**: Every public or installed claim requires an observed receipt. Candidate, tagged workflow, public asset, registry archive, installed product, records PR, merge, and Pages evidence remain distinct.

**Organization**: Tasks follow the public release journey chronologically and are grouped by independently testable user story.

## Phase 1: Setup And Specification

**Purpose**: Freeze exact source and complete Spec-Kit before publication mutation.

- [x] T001 Confirm clean `main`, accepted merge `5efb38ee93daff140e1ba79f57eea0a93d5b572f`, green merge workflows, and absent or identical `v0.10.14` tag in `specs/196-v0-10-14-publication/verification.md`
- [x] T002 Create S196 specification and clarifications in `specs/196-v0-10-14-publication/spec.md`
- [x] T003 Create and satisfy requirement-quality checklists in `specs/196-v0-10-14-publication/checklists/requirements.md` and `specs/196-v0-10-14-publication/checklists/publication.md`
- [x] T004 Create research, data model, contract, implementation plan, and quickstart in `specs/196-v0-10-14-publication/`
- [x] T005 Run read-only Spec-Kit analysis, resolve every finding in `specs/196-v0-10-14-publication/`, and record the final analysis result in `specs/196-v0-10-14-publication/verification.md`

---

## Phase 2: User Story 1 - Obtain The Complete Public Release (Priority: P1)

**Goal**: Publish the exact accepted merge through GitHub and crates.io with complete independent readback.

**Independent Test**: The annotated tag peels to the accepted merge, all four release jobs pass, six GitHub files and sidecars validate, and ten non-yanked crate archives match registry checksums.

- [x] T006 [US1] Create annotated tag `v0.10.14` at the accepted merge and record tag object plus peeled source in `specs/196-v0-10-14-publication/verification.md`
- [x] T007 [US1] Push only tag `v0.10.14`, capture the resulting tagged workflow identity, and record it in `specs/196-v0-10-14-publication/verification.md`
- [x] T008 [US1] Watch tagged identity and Windows package certification to terminal success and record job receipts in `specs/196-v0-10-14-publication/verification.md`
- [x] T009 [US1] Verify the public GitHub release job completes and record release URL, publication state, source, and timestamp in `specs/196-v0-10-14-publication/verification.md`
- [x] T010 [US1] Approve the configured crates.io environment deployment through normal GitHub review without protection changes, then record deployment and job receipts in `specs/196-v0-10-14-publication/verification.md`
- [x] T011 [P] [US1] Independently download and hash all six public GitHub files in a transient directory and record bounded receipts in `specs/196-v0-10-14-publication/verification.md`
- [x] T012 [P] [US1] Validate all three public checksum sidecars, GitHub API digests, and public package-certification authority, recording results in `specs/196-v0-10-14-publication/verification.md`
- [x] T013 [P] [US1] Query and independently download all ten v0.10.14 crates, require non-yanked state, compare archive hashes to registry checksums, and record receipts in `specs/196-v0-10-14-publication/verification.md`
- [x] T014 [US1] Reconcile T006 through T013 against `specs/196-v0-10-14-publication/contracts/publication.md` before any current-public marker changes

**Checkpoint**: v0.10.14 is publicly available and independently verified on GitHub and crates.io.

---

## Phase 3: User Story 2 - Exercise The Released Product (Priority: P2)

**Goal**: Observe the verified public MSI at documented default storage, retention, and cleanup behavior.

**Independent Test**: The installed product reports official v0.10.14 identity, effective default product locations are observed without overrides, supported bounded commands run, and native retained state plus cleanup are accounted.

- [x] T015 [US2] Install the verified public v0.10.14 MSI through a hidden noninteractive parent and record installer identity in `specs/196-v0-10-14-publication/verification.md`
- [x] T016 [US2] Query installed build identity and bounded read-only product status without storage overrides, recording observed results in `specs/196-v0-10-14-publication/verification.md`
- [x] T017 [US2] Determine the effective canonical default target store, session registry, and native bundle locations from product output and filesystem observations, recording only privacy-safe paths in `specs/196-v0-10-14-publication/verification.md`
- [x] T018 [US2] Run supported installed-product cleanup or retention operations required by the observed journey, inventory retained native artifacts, and record exact results in `specs/196-v0-10-14-publication/verification.md`
- [x] T019 [US2] File privacy-scrubbed v0.10.14 GitHub issues for every concrete installed failure and record both verified and unavailable behavior in `specs/196-v0-10-14-publication/verification.md`

**Checkpoint**: The public installed-product observation is complete to the extent the current host and available titles permit, with no invented claim.

---

## Phase 4: User Story 3 - See v0.10.14 As Current Everywhere (Priority: P3)

**Goal**: Derive, review, merge, and publicly deploy current records from verified public evidence.

**Independent Test**: Repository validators, PR checks, hosted review, merge workflows, and Pages all succeed; public records name v0.10.14; #567 and its milestone are complete.

- [x] T020 [P] [US3] Create `docs/maintainers/v0.10.14-release-handoff.md` from verified tag, workflow, asset, registry, and installed observations
- [x] T021 [P] [US3] Advance `docs/published-release.json` to the verified v0.10.14 source, date, and public URL
- [x] T022 [P] [US3] Advance current-version prose in `README.md`, `CONTRIBUTING.md`, `docs/fragcap-specification.md`, `docs/security/native-product-review-handoff.md`, and `site/content/docs/`
- [x] T023 [US3] Advance `docs/security/native-product-review-candidate.v1.json` and related frozen review authority to exact verified public assets
- [x] T024 [US3] Update `.github/workflows/published-review-candidate.yml` only as required for the v0.10.14 frozen candidate, with a dated decision record if the pinned workflow changes
- [x] T025 [US3] Add the S196 roadmap entry to `docs/plans/README.md` and publication record to `CHANGELOG.md` or `changelog.d/` according to repository release-history policy
- [x] T026 [US3] Complete `specs/196-v0-10-14-publication/verification.md` and mark completed tasks in `specs/196-v0-10-14-publication/tasks.md`
- [x] T027 [US3] Run `cargo xtask ci`, release-note, specification, review-record, package, supply-chain, isolated advisory, site, encoding, and mojibake validation against the records change
- [ ] T028 [US3] Commit the complete S196 publication records, push `codex/s196-v01014-publication`, and open the official pull request closing #567
- [ ] T029 [US3] Watch all applicable PR checks to terminal success and retain exact-head evidence in the pull request and `specs/196-v0-10-14-publication/verification.md`
- [ ] T030 [US3] Reconcile automatic hosted code and security review, respond to every actual comment, resolve every thread, and apply required changes
- [ ] T031 [US3] If useful after complete round-one reconciliation, request exactly one second hosted code and security review round, process it fully, and never request a third
- [ ] T032 [US3] Merge the green reviewed records pull request under the owner's explicit S196 authorization and verify the exact main merge in `specs/196-v0-10-14-publication/verification.md`
- [ ] T033 [US3] Watch post-merge main checks and Pages deployment to terminal success, then verify the public site presents v0.10.14 current state
- [ ] T034 [US3] Close and reconcile issue #567, milestone v0.10.14, and project Stage/Slice from public evidence

**Checkpoint**: v0.10.14 is public, current, merged, deployed, and tracked as complete.

---

## Phase 5: Final Housekeeping

**Purpose**: Leave no spent branch or local Spec-Kit state.

- [ ] T035 Switch to `main`, fetch and fast-forward, remove the merged local and remote S196 branch, and delete `.specify/feature.json`
- [ ] T036 Verify `git status --short --branch` is clean synchronized main and perform one final GitHub release, crates.io, Pages, issue, milestone, and project readback

---

## Dependencies And Execution Order

- Phase 1 precedes every publication mutation.
- User Story 1 must complete before installed-product observation or current-record edits.
- User Story 2 follows public verification and may produce concrete version-specific issues without reopening publication.
- User Story 3 depends on public GitHub and registry receipts and incorporates the installed observation boundary.
- Final housekeeping follows the records merge and public Pages deployment.
- T011, T012, and T013 may run in parallel after the tagged workflow completes.
- T020, T021, and T022 may be drafted in parallel from the same verified receipt set; T023 through T027 reconcile them sequentially.
- T031 is optional and is the only permitted explicit second hosted review round.

## Implementation Strategy

1. Complete Spec-Kit and freeze exact release source.
2. Publish and independently verify public GitHub and registry bytes.
3. Exercise only the public installed product at defaults.
4. Build one records change from those receipts.
5. Push, review, merge, deploy, and close tracking without another owner halt.
