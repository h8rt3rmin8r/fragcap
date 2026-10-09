# Tasks: S170 initiating-user bundle access

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md) and [access contract](contracts/access.md).

## Phase 1: Setup and specification

- [x] T001 Verify literal S170/#464/branch/directory identity and clean human-merged base; persist ignored .specify/feature.json.
- [x] T002 Specify and autonomously clarify both new-output and historical-repair journeys in spec.md.
- [x] T003 Complete requirement-quality and access/recovery checklists in checklists/requirements.md and checklists/access.md.
- [x] T004 Dispatch required research agents and consolidate exact token, handoff and pinned repair decisions in research.md.
- [x] T005 Produce plan.md, data-model.md, contracts/access.md and quickstart.md with constitution gates and fixed shared ownership.
- [x] T006 Run blocking read-only speckit-analyze across specs/170-initiating-user-bundle-access artifacts; resolve findings before implementation (FR-001 through FR-014).

## Phase 2: Foundational access authority

- [x] T007 Establish failing Windows identity, private-child and actual ordinary-context access regressions in crates/fragcap/src/deep_capture/access.rs and access/windows.rs (FR-001 through FR-006, FR-012).
- [x] T008 Implement exact recipient/current-session/linked-token context, finite authenticated handoff and pinned descriptor operations in crates/fragcap/src/deep_capture/access.rs and access/windows.rs; add required existing Windows feature edges in crates/fragcap/Cargo.toml (FR-001, FR-003, FR-005, FR-006).
- [x] T009 Export the additive shared access interfaces in crates/fragcap/src/deep_capture/mod.rs and agree repair/CLI signatures in contracts/access.md before dependent edits (FR-004, FR-007 through FR-011).

## Phase 3: US1 normal desktop access

**Goal**: New elevated output remains readable in its exact intended ordinary context, and unprovable identity refuses before effects.

**Independent test**: Controlled actual Windows enumeration/read covers inherited/protected/atomic artifacts and denied unrelated access.

- [x] T010 [US1] Correct private directory and file preparation in crates/fragcap/src/deep_capture/artifacts.rs and prove selected recipient persists across writer threads and independent protection (FR-002, FR-004, FR-005).
- [x] T011 [P] [US1] Correct explicit producer identity for ephemeral private CA ACL in crates/fragcap-proxy/src/windows/acl.rs without granting retained-recipient access to issuer-private state (FR-005).
- [x] T012 [US1] Add recipient/helper arguments and plan-bound preflight, protection and post-reconciliation verification in crates/fragcap-cli/src/cli.rs, commands/deep_capture.rs and commands/calibrate.rs (FR-001 through FR-006, FR-010).
- [x] T013 [US1] Replace producer-context accessibility inference with verified recipient outcomes and guarded analyzer guidance in crates/fragcap-cli/src/session_ux.rs and commands/deep_capture.rs (FR-010).
- [x] T014 [P] [US1] Prove actual new-output inheritance, atomic publication, ordinary read, denied-parent refusal/correction and unrelated denied-read positive controls in crates/fragcap/tests/bundle_access.rs (FR-004, FR-004a, FR-005, FR-006, FR-012, SC-001).

## Phase 4: US2 exact historical repair

**Goal**: Repair recognized historical inaccessible output without changing content, retention or recovery obligations.

**Independent test**: Old-policy fixtures fail ordinary reads before correction, then repair conserves all bytes and enables those reads; unrelated/stale/escaped/partial cases remain truthful.

- [x] T015 [P] [US2] Establish failing provenance/population/old-sidecar/content-conservation and denied proven-owned-parent traversal regressions in crates/fragcap/src/deep_capture/access_repair.rs (FR-004a, FR-007 through FR-009, FR-011, FR-012).
- [x] T016 [US2] Implement bounded pinned inspection, digest-bound repair, exact non-propagating owned-parent traversal grants, artifact ACL effects and fresh-preview idempotent retry in crates/fragcap/src/deep_capture/access_repair.rs using shared access/windows.rs operations (FR-004a, FR-007 through FR-011).
- [x] T017 [US2] Add read-only inspect and exact-authorized repair commands, owner-lease validation and structured/shared-layout reports in crates/fragcap-cli/src/commands/bundle.rs, cli.rs and doctor/residue.rs (FR-007 through FR-011).
- [x] T018 [US2] Prove historical denied reads, mixed sidecars, owned-parent repair without sibling descriptor changes, byte conservation, active/stale/reparse/hard-link/unknown refusals and partial retry in crates/fragcap/tests/bundle_access.rs and crates/fragcap-cli/tests/cli_bundle.rs (FR-004a, FR-008, FR-009, FR-011, FR-012, SC-002, SC-003).

## Phase 5: Completion

- [x] T019 Update help, master specification, chronological slice plan, site access/recovery documentation and changelog.d/S170.*.md with exact shipped semantics in crates/fragcap-cli/src/workflow_help.rs, docs/fragcap-specification.md, docs/plans/README.md and site/content/docs (FR-013).
- [x] T020 Audit every #464 criterion and all FR/SC evidence in issue-acceptance.md and verification.md; run focused owning suites and cargo xtask ci, text hygiene and final diff review (FR-012 through FR-014, SC-001 through SC-004).
- [ ] T021 Commit and automatically push/open/attach the official PR, resolve every returned finding, allow at most one manual second review round, verify final-head CI and hand off for owner merge; external disposition is recorded in the PR conversation (FR-014).

## Dependencies and parallel execution

T001-T006 block implementation. T007 precedes T008; T009 fixes shared signatures before root CLI/repair edits. Recipient agent owns T007-T010 and access.rs/windows.rs/artifacts.rs/Cargo.toml. Repair agent owns T011/T015-T016 in proxy ACL and access_repair.rs. Root owns T009/T012-T013/T017/T019-T021 and all shared module exports. A controlled evidence agent may own T014/T018 only in bundle_access.rs after the interface contract stabilizes. Agents must communicate before changing shared signatures and never edit another owner's files concurrently. CLI integration and final security evidence follow foundational proof, then full verification follows all changes.

## Implementation strategy

Deliver new-output recipient authority as the first increment, then exact historical repair. Both stories are mandatory for #464 closure. Establish red security evidence before implementation, preserve independent content and recovery authority, and finish through exact-head hosted verification and bounded external reviews. No owner-machine ACL mutation, retention cleanup, merge or release is performed by this slice.
