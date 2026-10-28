# Tasks: S182 Installed-product recovery

## Phase 1: Setup

- [X] T001 Verify S182 branch, feature pointer, issue authority, and clean base in specs/182-installed-product-recovery/spec.md.
- [X] T002 Record planned source, evidence, and package contracts in specs/182-installed-product-recovery/contracts/cli-and-evidence.md and contracts/windows-qualification.md.

## Phase 2: Foundational evidence

- [X] T003 [P] Add deterministic time/cause omission regression fixtures in crates/fragcap-core/src/flow.rs and crates/fragcap/tests/loopback_correlation.rs.
- [X] T004 [P] Add real Capture adapter budget regressions in crates/fragcap-cli/src/commands/deep_capture.rs.
- [X] T005 [P] Add exact-owner/helper and stop-timeout regressions in crates/fragcap-cli/src/commands/deep_capture/shutdown.rs and crates/fragcap/src/deep_capture/native.rs.
- [X] T006 [P] Add clean package and physical-reference failure cases in xtask/src/package_certification.rs and xtask/src/windows_integration.rs.

## Phase 3: User Story 1, moved target

**Goal**: Recover one uniquely moved stored target with its identity and roles unchanged.
**Independent test**: Disposable store with synthetic old/new platform libraries, ambiguous and missing cases.

- [X] T007 [US1] Add relocation proposal and stale-root tests in crates/fragcap-targets/src/register.rs and crates/fragcap-cli/src/commands/targets.rs.
- [X] T008 [US1] Implement exact platform/store relocation proposal and atomic update in crates/fragcap-targets/src/ and crates/fragcap-cli/src/commands/targets.rs.
- [X] T009 [US1] Revalidate calibration root and workflow applicability before effects in crates/fragcap-cli/src/commands/calibrate.rs.
- [X] T010 [US1] Verify old root present, missing manifest/executable, duplicate libraries, changed executable, and unchanged root in crates/fragcap-cli/tests/.

## Phase 4: User Story 2, evidence-backed calibration

**Goal**: Finish reachability and TLS calibration only when exact bounded packet and proxy evidence supports it.
**Independent test**: Production adapter and controlled packet/proxy boundary, including real budget and omission cases.

- [X] T011 [US2] Make real Capture duration use remaining facade budget and finite return margin in crates/fragcap-cli/src/commands/deep_capture.rs.
- [X] T012 [US2] Retain bounded timestamped omission causes without losing counts in crates/fragcap-core/src/flow.rs and crates/fragcap-core/src/pipeline/mod.rs.
- [X] T013 [US2] Reconcile scoped omission overlap and exact ownership in crates/fragcap/src/deep_capture/native.rs and crates/fragcap/src/deep_capture/process.rs.
- [X] T014 [US2] Test IPv4/IPv6, orientations, tuple reuse, global unknown loss, overflow, and observation cutoff in crates/fragcap/tests/loopback_correlation.rs.
- [X] T015 [US2] Prove a controlled production-path reachability to TLS calibration sequence and preserve failures in crates/fragcap-cli/tests/.

## Phase 5: User Story 3, managed Deep Capture

**Goal**: Close proven owners while observing helpers, and reconcile final session truth.
**Independent test**: Controlled root/client/helper tree and stop-timeout with later nonzero cleanup evidence.

- [X] T016 [US3] Separate proven owner selection from passive descendant accounting in crates/fragcap-cli/src/commands/deep_capture/shutdown.rs.
- [X] T017 [US3] Verify all potentially routed descendant exits before route release and preserve incomplete outcomes in crates/fragcap-cli/src/commands/deep_capture/shutdown.rs.
- [X] T018 [US3] Refresh proxy diagnostics from final runtime/artifact authorities after timeout in crates/fragcap/src/deep_capture/native.rs and session.rs.
- [X] T019 [US3] Reconcile terminal/manifest/application/cleanup counts and earliest causal failure in crates/fragcap-cli/src/commands/calibrate/assessment.rs.
- [X] T020 [US3] Prove controlled ordinary Deep Capture artifacts and route release in crates/fragcap-cli/tests/.

## Phase 6: User Story 4, package and docs

**Goal**: Start clean Windows packages, validate physical evidence, and give accurate same-version instructions.
**Independent test**: Fresh supported Windows guest, exact official v0.10.8 reference, candidate qualification, site/parser checks.

- [X] T021 [US4] Configure static MSVC CRT in .cargo/config.toml or the reviewed release build path and verify PE imports.
- [X] T022 [US4] Update exact package import contract and clean-guest startup certification in integration/windows-package-contract-v1.json and xtask/src/package_certification.rs.
- [X] T023 [US4] Correct intentional non-admin refusal assertion in crates/fragcap-cli/tests/windows_native_integration.rs.
- [ ] T024 [US4] Produce fresh exact official v0.10.8 physical reference and separate candidate evidence in integration/windows-native-reference-v1.json and xtask/src/windows_integration.rs.
- [ ] T025 [US4] Verify MSI/ZIP version, help, Doctor, install, repair, uninstall, and user-state preservation in .github/workflows/package-certification.yml.
- [X] T026 [US4] Update plain-English lifecycle and relocation guidance in site/content/docs/getting-started.mdx, reference/cli.mdx, reference/deep-capture-compatibility.mdx, guides/packaging-and-migration.mdx, and guides/doctor-and-troubleshooting.mdx.
- [X] T027 [US4] Add parser-checked examples and lifecycle semantic checks in site/ and verify documentation build.

## Phase 7: Cross-cutting qualification and handoff

- [ ] T028 Run focused crate and full repository gates, native protocol conformance, advisory checks, and two-hour soak; record exact failures and fixes in specs/182-installed-product-recovery/quickstart.md.
- [ ] T029 Run all seven physical Windows rows and clean package lifecycle against exact bytes; retain honest reference and candidate authority in integration/.
- [X] T030 Add chronological S182 entry to docs/plans/README.md, changelog fragment in changelog.d/, and synchronize master specification where behavior changes.
- [ ] T031 Check UTF-8/no BOM, mojibake, private identity absence, source format, and task completion across specs/182-installed-product-recovery/ and changed files.
- [ ] T032 Commit, push codex/s182-installed-product-recovery, and open one official PR linked to #514, #515, #516, #517, #518, #520, #521, #522.
- [ ] T033 Resolve CI and every actionable review comment, use at most one explicit @Codex second-round request, and stop at owner merge handoff.

## Dependencies and execution

T001-T002 precede all work. T003-T006 are independent regression setup. US1 can run independently; US2 timing and omission converge before US3 session truth; US4 package work is independent while documentation follows final behavior. T028-T033 follow all story implementations. The MVP is US1 recovery, but the approved S182 completion requires all four stories.

Parallel opportunities: T003-T006 affect separate ownership surfaces; T007 and T011 can be prepared independently; package build T021-T025 can proceed beside source work, but no concurrent Cargo builds. The single-agent execution remains sequential where file or gate ownership intersects.
