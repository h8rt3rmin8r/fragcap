# Tasks: Installed Direct Route Acceptance

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), [acceptance contract](contracts/acceptance.md), [quickstart.md](quickstart.md).

**Tests**: Write failing regressions before application fixes. Physical, package, and fixture evidence remain distinct.

## Phase 1: Setup and authority

- [x] T001 Confirm branch, issue states, product and package source, and gate commands in AGENTS.md and specs/204-direct-route-acceptance/.
- [x] T002 Inspect default target store, session registry, installation CA, and Doctor recovery without changing native state; record privacy-scrubbed restoration prerequisites in specs/205-installed-route-acceptance/evidence.md and retain exact canonical paths for the owner handoff.
- [x] T003 Run read-only speckit-analyze across S205 artifacts, constitution, and master specification; resolve critical and high findings before implementation.

## Phase 2: Foundational controlled journey

- [x] T004 Add a no-console finite raw-TCP client with exact request, response, and after-reply acknowledgement in integration/controlled-route-client.rs.
- [x] T005 Add a source-preserving installed and portable acceptance harness using public registration, cold launch, authorization, native defaults when claimed, sentinels, and cleanup in crates/fragcap-cli/tests/cli_direct_route_hosted.rs. The existing integration/direct-route-matrix-v1.json remains the separate signed-driver library matrix.

## Phase 3: User Story 1 - Installed selected TCP (#593)

**Goal**: Candidate package bytes route a cold client to an external peer with exact two-way transfer.

**Independent Test**: MSI and ZIP, IPv4 and IPv6, required route stream, separate library diagnostics on and off, sentinels, refusal, cancellation, repeat, and teardown.

- [ ] T006 [US1] Run the ignored public package-level external byte and route-stage assertions in crates/fragcap-cli/tests/cli_direct_route_hosted.rs against candidate package bytes; preserve the existing scripts/Test-PackageCertification.ps1 identity and help gates.
- [ ] T007 [US1] Fix any demonstrated launch, selected packet, bridge, reply, transfer, or terminal defect in crates/fragcap/src/deep_capture/ and preserve its regression.
- [ ] T008 [US1] Run published v0.11.0 comparison and candidate MSI/ZIP external cases with native route stream, separate library diagnostics-on/off cases, sentinel latency/loss, failure, repeat, and cleanup; record results and cloudbase port closure in specs/205-installed-route-acceptance/evidence.md.

## Phase 4: User Story 2 - Physical correlation (#594)

**Goal**: Exact route facts survive passive original-packet absence without creating packet truth.

**Independent Test**: Observed original-present and original-absent TCP/UDP plus negative proof cases agree across compatibility, verdict, store, and admission.

- [ ] T009 [US2] Audit original-present/absent, TCP/UDP, competing-owner, reuse, helper, history-gap, refusal, and no-payload coverage and add failing gaps in crates/fragcap/src/deep_capture/native.rs and crates/fragcap-cli/tests/cli_deep_capture.rs.
- [ ] T010 [US2] Fix demonstrated correlation or four-surface disagreement in crates/fragcap/src/deep_capture/native.rs, crates/fragcap/src/deep_capture/policy.rs, and crates/fragcap-cli/src/commands/calibrate/.
- [ ] T011 [US2] Observe both physical Npcap original-packet orderings with packaged WinDivert, and record passive artifacts separately from route logs in specs/205-installed-route-acceptance/evidence.md.
- [ ] T012 [US2] Verify a positive fact admits only the matching later route and changed or incomplete cases refuse before effects in crates/fragcap-cli/tests/cli_deep_capture.rs.

## Phase 5: User Story 3 - Debug report (#595)

**Goal**: Complete and partial native reports agree with source and terminal facts.

**Independent Test**: Fixture corpus, source digests, private/shareable output, access, Doctor, retention, manifest, and human/JSON summaries.

- [ ] T013 [US3] Audit complete, zero-byte, refused, queue-gap, writer-failure, crash-prefix, missing-artifact, incomplete-cleanup, and optional-capture coverage and add failing gaps in crates/fragcap/tests/deep_capture_route_events.rs.
- [ ] T014 [US3] Add failing manifest, Doctor/recovery, retention/share, recipient access, source immutability, and human/JSON parity checks in crates/fragcap/tests/bundle_access.rs and crates/fragcap-cli/tests/.
- [ ] T015 [US3] Fix demonstrated route stream, report, access, Doctor, or summary disagreement in crates/fragcap/src/deep_capture/route_report.rs, crates/fragcap/src/deep_capture/route_events.rs, and crates/fragcap-cli/src/.
- [ ] T016 [US3] Verify private receipt and unrelated payload do not appear in shareable output, and partial bundles retain declared authority in specs/205-installed-route-acceptance/evidence.md.

## Phase 6: Qualification and pull request

- [ ] T017 Reconcile master specification, chronological plans index, unreleased changelog, and pinned-artifact decisions in docs/fragcap-specification.md, docs/plans/README.md, and changelog.d/.
- [ ] T018 Run focused driver-free and ordinary passive Capture tests, foreground cargo xtask ci, required Windows/package checks, and UTF-8/LF/mojibake checks on final code.
- [ ] T019 Verify every #593, #594, and #595 acceptance row against actual evidence and retain exact unverified outcomes in specs/205-installed-route-acceptance/evidence.md.
- [ ] T020 Commit S205 files, push the authorized branch, and publish an official PR tracing each issue separately.
- [ ] T021 Resolve every CI, Codex, and security finding; request at most one explicit second @Codex review round and recheck the exact pushed head.
- [ ] T022 Close only issues with full scoped acceptance through merge keywords or documented closeout; hand a green reviewed PR to the owner without merging it.

## Dependencies and Execution Order

Analyze precedes implementation. The controlled client and harness precede live package claims. Failing tests precede fixes. Physical and package outcomes precede issue closure. Review closeout follows the official PR.

## Parallel Opportunities

Independent read-only inventories may be batched. Report fixtures and controlled-client source affect separate files, but route and CLI edits proceed sequentially. No parallel agent delegation is required.

## Implementation Strategy

Deliver the package route, then reconcile packet-order calibration and debug-report authority from the same source evidence. Preserve negative outcomes and decline issue closure when an acceptance row has not run.
