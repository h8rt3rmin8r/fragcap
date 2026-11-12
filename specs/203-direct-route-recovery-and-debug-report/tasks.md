# Tasks: Direct Route Recovery and Debug Report

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), [route and report contract](contracts/route-and-report.md), [quickstart.md](quickstart.md)

**Tests**: TDD and exact route, artifact, security, and package gates are required by the S203 spec and repository constitution.

## Phase 1: Setup

- [x] T001 Confirm clean branch, current issue and source authority, existing test/package commands, and protected artifact owners in AGENTS.md, crates/fragcap/src/deep_capture, and crates/fragcap-cli/src.
- [x] T002 Register S203 scope and issue traceability in docs/plans/README.md and verify .specify/feature.json points at the slice without staging it.

## Phase 2: Foundational selected-flow evidence

- [ ] T003 Add failing typed route-stream tests for selected TCP/UDP, refusal, no-byte closure, queue gap, writer failure, crash prefix, finite limits, and source access in crates/fragcap/tests/deep_capture_route_events.rs.
- [x] T004 Implement a bounded nonblocking typed producer, protected route.jsonl writer, prefix reader, sequence/header/trailer, and accounting in crates/fragcap/src/deep_capture/route_events.rs and lifecycle.rs.
- [x] T005 Bind a stable session-local route flow ID without exposing RouteReceiptId in crates/fragcap/src/deep_capture/direct_route.rs, native.rs, and direct_runtime.rs.

## Phase 3: User Story 1 - Working external TCP (#593)

**Goal**: Prove and repair client-visible non-loopback TCP forwarding while preserving unrelated traffic.

**Independent Test**: Controlled IPv4 and IPv6 clients exchange exact bytes with an owned non-loopback peer; empty, failed, and canceled cases report exact stages.

- [ ] T006 [US1] Add a failing external handshake and exact bidirectional transfer regression with active sentinels in crates/fragcap/src/deep_capture/direct_runtime.rs and tests/direct_route_windows.rs.
- [x] T007 [US1] Instrument selected packet flags, interface choice, generated/sent synthetic replies, observed client ACK, bridge transfer counts, and terminal reasons in crates/fragcap/src/deep_capture/direct_runtime.rs and direct_bridge.rs.
- [ ] T008 [US1] Diagnose and fix the demonstrated reply-interface, checksum, or other bridge defect in crates/fragcap/src/deep_capture/windivert.rs, direct_stack.rs, and direct_runtime.rs; preserve exact regression assertions.
- [ ] T009 [US1] Distinguish upstream-connected zero-byte selected tunnels from client data exchange in crates/fragcap/src/deep_capture/routing.rs, crates/fragcap-cli/src/commands/calibrate/assessment.rs, and terminal summaries.
- [ ] T010 [US1] Run controlled non-loopback IPv4/IPv6 selected TCP, refusal, cancellation, MSI/ZIP, and unrelated-sentinel gates with diagnostics on and off using the repository's Windows package matrix.

## Phase 4: User Story 2 - Exact direct calibration (#594)

**Goal**: Reachability facts follow route and client proof independently from passive first-packet retention.

**Independent Test**: Original-present and original-absent cases agree when client and proxy transfer are proven; all ambiguous and incomplete cases remain non-positive.

- [ ] T011 [US2] Add failing first-packet-present/absent, TCP/UDP, reuse, competing-owner, incomplete-history, refusal, and no-payload tests in crates/fragcap/src/deep_capture/native.rs and crates/fragcap-cli/src/commands/calibrate/assessment.rs.
- [x] T012 [US2] Bind selected receipt, final-client process generation, proxy leg, client delivery, and independent packet-evidence status in crates/fragcap/src/deep_capture/native.rs, model.rs, and direct_runtime.rs.
- [x] T013 [US2] Reconcile compatibility.json, calibration verdict, target-store fact applicability, and later ordinary admission in crates/fragcap/src/deep_capture/policy.rs and crates/fragcap-cli/src/commands/calibrate and deep_capture.rs.
- [ ] T014 [US2] Verify no helper-as-game attribution, no false positive from upstream CONNECT or consumed count, and exact positive route facts in crates/fragcap/tests/native_conformance.rs and crates/fragcap-cli/tests/cli_deep_capture.rs.

## Phase 5: User Story 3 - Native debug report (#595)

**Goal**: Export a complete or partial source-preserving report from native session evidence.

**Independent Test**: Normal, empty, refused, queue-gap, writer-failure, crash-prefix, missing-artifact, and optional-capture fixtures yield exact report authority without changing the bundle.

- [ ] T015 [US3] Add failing artifact registration, access, sharing, Doctor, and recovery tests in crates/fragcap/tests/bundle_access.rs, crates/fragcap/tests/deep_capture_lifecycle.rs, and crates/fragcap-cli/src/doctor.
- [x] T016 [US3] Register route.jsonl authority, sensitivity, finalization, completeness, and loss in crates/fragcap/src/deep_capture/artifacts.rs, collection.rs, session.rs, and crates/fragcap-cli/src/commands/deep_capture.rs.
- [x] T017 [US3] Add failing CLI parser and read-only report fixtures in crates/fragcap-cli/src/cli.rs and crates/fragcap-cli/src/commands/bundle.rs.
- [x] T018 [US3] Implement bounded source validation, exact evidence joins, declared endpoint/path transformation, atomic report output, and shared summary projection in crates/fragcap/src/deep_capture and crates/fragcap-cli/src/commands/bundle.rs.
- [x] T019 [US3] Include route stream in Doctor inventory/recovery and source-preserving bundle export decisions in crates/fragcap-cli/src/doctor/probe.rs, residue.rs, and crates/fragcap/src/deep_capture/artifacts.rs.
- [ ] T020 [US3] Verify human and --json terminal summaries reconcile with the report for complete and partial sessions in crates/fragcap-cli/tests/cli_deep_capture.rs.

## Phase 6: User Story 4 - Honest auto scope help (#598)

**Goal**: Active help and docs state the current auto and game behavior.

**Independent Test**: Direct and launcher-mediated resolver cases agree with shipped help and guided plan.

- [x] T021 [US4] Add a failing resolver-bound help and behavior test in crates/fragcap-cli/src/cli.rs and crates/fragcap-cli/src/commands/deep_capture.rs.
- [x] T022 [US4] Correct active help, guided guidance, and current user documentation in crates/fragcap-cli/src/cli.rs, docs/, and site/ where applicable, preserving dated historical records.

## Phase 7: Qualification and pull request

- [x] T023 Reconcile master specification, glossary, plan index, and dated changelog decisions with the proved behavior in docs/fragcap-specification.md, docs/glossary/, docs/plans/README.md, and changelog.d/.
- [x] T024 Run spec-kit analyze read-only after tasks and resolve every critical/high finding in specs/203-direct-route-recovery-and-debug-report/ before implementation.
- [ ] T025 Run complete foreground format, lint, test, documentation, shell, controlled Windows, and package gates on the final code and verify UTF-8, LF, and no mojibake.
- [ ] T026 Commit the slice on codex/s203-direct-route-recovery, push the authorized branch, and open an official PR that separately traces #593, #594, #595, and #598.
- [ ] T027 Address every received CI, Codex, and security review finding; trigger at most one explicit second @Codex review round; re-run exact-head gates after fixes.
- [ ] T028 Present the green reviewed PR, remaining deviations and unverified behavior to the owner for final review and merge without merging it.

## Dependencies and Execution Order

The selected-flow stream and stable ID precede runtime instrumentation. Runtime reproduction and verified forwarding precede positive calibration semantics. Calibration and stream authority precede the derived report. Scope help can be implemented independently after its failing conformance test. Analyze is mandatory before source implementation; after a material spec or plan change, repeat it. Every issue closes only when its own acceptance and repository-controlled gates are complete.

## Parallel Opportunities

Independent read-only issue, code, and documentation checks may be batched. Route, calibration, and report edits touch shared files and proceed sequentially. No test may be marked passing before it has actually run.

## Implementation Strategy

Deliver the observable selected-flow stream first, then a demonstrated external TCP repair, exact calibration evidence, a complete debug report, and the small help correction. Keep every increment independently testable. Commit and publish only the coherent complete S203 result.
