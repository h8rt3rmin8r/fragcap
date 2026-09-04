# Tasks: Native Deep Capture Performance Envelope

**Input**: Design documents from `specs/128-native-performance-envelope/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Required by FR-015 and the autopilot TDD protocol.

## Phase 1: Setup and Frozen Contracts

- [x] T001 Validate the active feature selector, branch, issue scope, merged S127 baseline, and dependency locks in `.specify/feature.json`, issue #326, `Cargo.lock`, and `fuzz/Cargo.lock`
- [x] T002 [P] Finalize performance gate semantics and entities in `specs/128-native-performance-envelope/contracts/performance-gate.md` and `specs/128-native-performance-envelope/data-model.md`
- [x] T003 [P] Validate requirements and performance quality in `specs/128-native-performance-envelope/checklists/requirements.md` and `specs/128-native-performance-envelope/checklists/performance.md`
- [x] T004 Freeze all fourteen pre-measurement case budgets and both profiles in `performance/native-proxy-budgets-v1.json`

## Phase 2: Failing Registry and Runtime Accounting Evidence

- [x] T005 Add failing registry tests for schemas, matrix identity, budgets, profiles, evidence references, workflow coverage, comparability, and immutable budget behavior in `xtask/src/performance.rs`
- [x] T006 [P] Add failing runtime tests for bounded failure details, task gauges, and certificate-cache gauges in `crates/fragcap-proxy/src/runtime.rs`
- [x] T007 [P] Add failing bounded writer-queue occupancy tests in `crates/fragcap/src/deep_capture/application.rs`
- [x] T008 Add the isolated performance harness manifest and failing report/parser self-tests in `performance/native-proxy/Cargo.toml` and `performance/native-proxy/src/`

## Phase 3: User Story 1 - Gate the Complete Native Matrix (Priority: P1)

**Goal**: Execute and validate all seven production protocol families with retention on and off.

**Independent Test**: A short campaign emits exactly fourteen terminal case results and rejects any missing, duplicated, stale, or over-budget row.

- [x] T009 [US1] Implement the schema and closed protocol-retention matrix validator in `xtask/src/performance.rs`
- [x] T010 [US1] Implement source-derived runtime inventory and attributed evidence validation in `xtask/src/performance.rs`
- [x] T011 [US1] Add the `performance` command and static gate to `cargo xtask ci` in `xtask/src/main.rs`
- [x] T012 [US1] Implement common parent/worker ownership, fixed workload generation, paired direct/proxy measurements, and bounded JSON Lines reports in `performance/native-proxy/src/main.rs`
- [x] T013 [US1] Implement the HTTP/1.1 and WebSocket real-proxy driver family in `performance/native-proxy/src/workloads.rs`
- [x] T014 [US1] Implement the HTTP/2 and gRPC real-proxy driver family in `performance/native-proxy/src/workloads.rs`
- [x] T015 [US1] Implement the authenticated generic TCP and UDP real-proxy driver family in `performance/native-proxy/src/workloads.rs`
- [x] T016 [US1] Implement the scoped QUIC and HTTP/3 real-proxy driver in `performance/native-proxy/src/workloads.rs`
- [x] T017 [US1] Implement seven-window timing evaluation, one guard-band retry, and per-row terminal decisions in `performance/native-proxy/src/main.rs`

## Phase 4: User Story 2 - Prove Bounded Degradation and Cleanup (Priority: P2)

**Goal**: Measure and enforce independent memory, disk, queue, cache, task, loss, and shutdown bounds.

**Independent Test**: Pressure workloads breach finite evidence capacities without unexplained loss or forwarding failure, then every product owner reaches its declared terminal bound.

- [x] T018 [US2] Cap runtime failure details with drop-oldest behavior and an exact overflow counter in `crates/fragcap-proxy/src/model.rs` and `crates/fragcap-proxy/src/runtime.rs`
- [x] T019 [US2] Add accepted-connection task current, peak, spawned, completed, and aborted gauges with exact terminal reconciliation in `crates/fragcap-proxy/src/runtime.rs`
- [x] T020 [US2] Expose leaf-cache current, peak, byte, and eviction accounting through `RuntimeObservation` in `crates/fragcap-proxy/src/model.rs`, `crates/fragcap-proxy/src/certificate.rs`, and `crates/fragcap-proxy/src/runtime.rs`
- [x] T021 [US2] Add application writer queue capacity, current, and peak gauges with terminal reconciliation in `crates/fragcap/src/deep_capture/application.rs` and `crates/fragcap-proxy/src/application.rs`
- [x] T022 [US2] Implement Windows and Linux current-worker CPU and memory sampling without a new dependency in `performance/native-proxy/src/metrics.rs`
- [x] T023 [US2] Implement exact artifact logical-growth, retention, queue, cache, task, loss, and shutdown evaluation in `performance/native-proxy/src/main.rs` and `performance/native-proxy/src/workloads.rs`
- [x] T024 [US2] Retain deterministic overload, retention saturation, cache churn, connection pressure, and interrupted-cleanup coverage in the product tests referenced by the registry

## Phase 5: User Story 3 - Reproduce Short and Multi-Hour Evidence (Priority: P3)

**Goal**: Provide trustworthy short regression and genuine two-hour soak profiles from one registry.

**Independent Test**: Short runs are comparable and repeatable within the declared tolerance; a two-hour run emits periodic samples and one complete terminal without resource slope or residue.

- [x] T025 [US3] Implement profile selection, fixed-duration soak phases, periodic sampling, incomplete-prefix handling, and one terminal reconciliation in `performance/native-proxy/src/main.rs`
- [x] T026 [US3] Implement report digest, environment comparability, repeatability, and report-validation tests in `xtask/src/performance.rs`
- [x] T027 [US3] Add required Windows and Ubuntu short jobs plus scheduled/manual Windows soak automation and report upload in `.github/workflows/performance.yml`
- [x] T028 [US3] Run two successive local short campaigns, compare their results, and retain sanitized reference evidence in `performance/native-proxy-reference-v1.json`
- [x] T029 [US3] Run the default two-hour soak campaign and retain its sanitized complete result in `performance/native-proxy-soak-v1.json`

## Phase 6: Documentation and Verification

- [x] T030 [P] Publish supported limits, degradation, comparability, report, short-run, and soak guidance in `docs/security/deep-capture-performance.md`
- [x] T031 [P] Add performance vocabulary in `docs/glossary/capture-and-networking.md` and regenerate `docs/glossary/index.md`
- [x] T032 [P] Record S128 and the #326/#327-#334 boundary in `docs/fragcap-specification.md`, `docs/fragcap-spec-outline.md`, `docs/plans/README.md`, and `AGENTS.md`
- [x] T033 [P] Add S128 feature and dated pinned-workflow/runtime-accounting decision fragments in `changelog.d/`
- [x] T034 Run spec-kit analysis across `spec.md`, `plan.md`, and `tasks.md` and remediate every finding
- [x] T035 Run the focused validator, runtime-accounting, harness, report, repeatability, and documentation checks from `specs/128-native-performance-envelope/quickstart.md`
- [x] T036 Run `cargo xtask ci`, the release-mode short campaign, text hygiene, dependency locks, forbidden-capability checks, and mojibake checks
- [x] T037 Run post-implementation convergence, complete any appended tasks, mark every task in `specs/128-native-performance-envelope/tasks.md`, and perform the final scope audit

## Dependencies and Execution Order

- Phase 1 freezes budgets before any measurement.
- Phase 2 establishes red contract and accounting tests before implementation.
- User Story 1 creates the complete real-proxy measurement path.
- User Story 2 adds the independent resource authorities required by User Story 1 evaluation.
- User Story 3 depends on complete matrix and resource accounting, then runs real short and soak campaigns.
- Documentation and full verification follow executable convergence.

## Parallel Opportunities

- T002 and T003 touch independent specification artifacts.
- T005, T006, T007, and T008 establish red tests in separate workspaces and modules.
- T013 through T016 use separate driver modules after T012 fixes their common interface.
- T018 through T022 touch separable accounting and platform-measurement surfaces, except shared runtime observation edits remain sequential.
- T030 through T033 touch independent documentation groups after behavior stabilizes.

## Implementation Strategy

1. Freeze the acceptance authority before collecting numbers.
2. Make incomplete registries, reports, and resource observations fail.
3. Execute each real proxy path with both retention modes.
4. Prove hard resource and loss invariants before interpreting timing.
5. Run repeatability and genuine soak evidence through the same contract.
6. Converge, document, and run the complete repository gate.

## Phase 7: Convergence

- [x] T038 Add a committed QUIC evidence coalescing test that proves the 16 KiB event bound, exact byte reconstruction, offsets, and terminal identity per FR-007 and SC-006 (partial)
- [x] T039 Add a seeded soak-report cadence rejection test covering intermediate and terminal gaps over 60 seconds per SC-008 and SC-010 (partial)
- [x] T040 Replace the persistent documentation development-server command with the finite documentation check in `specs/128-native-performance-envelope/quickstart.md` per T035 (partial)

## Phase 8: Final Review Remediation

- [x] T041 Preserve successfully flushed application records when a later buffered write fails, with a deterministic regression test for exact storage-loss accounting
- [x] T042 Reconstruct every report retry attempt from exact attempt and window identities, and reject any discarded attempt containing a hard failure
- [x] T043 Exercise five admitted QUIC certificate identities against one four-entry production leaf cache per worker, and require the reported peak to prove the churn occurred
- [ ] T044 Recollect two short campaigns and the complete two-hour soak on the final reviewed implementation, then rerun the complete repository gate
