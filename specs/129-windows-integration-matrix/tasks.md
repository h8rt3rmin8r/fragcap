# Tasks: Native Windows Integration Matrix

**Input**: Design documents from `specs/129-windows-integration-matrix/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Required by FR-003, FR-018, and the autopilot TDD protocol.

## Phase 1: Setup and Frozen Contracts

- [x] T001 Validate the active feature selector, branch, issue #327 scope, merged S128 baseline, installed Windows capabilities, and release-critical boundary in `.specify/feature.json`, `specs/129-windows-integration-matrix/`, and issue #327
- [x] T002 [P] Finalize the registry, report, execution, evidence, and packaging-handoff contracts in `specs/129-windows-integration-matrix/contracts/windows-integration-gate.md` and `specs/129-windows-integration-matrix/data-model.md`
- [x] T003 [P] Validate requirements and Windows integration quality in `specs/129-windows-integration-matrix/checklists/requirements.md` and `specs/129-windows-integration-matrix/checklists/windows-integration.md`
- [x] T004 Freeze the complete domain and row identities before implementation in `integration/windows-native-matrix-v1.json`

## Phase 2: Failing Static and Execution Evidence

- [x] T005 Add failing registry tests for schema, closed vocabularies, row identity, domain coverage, evidence references, and exact workflow ownership in `xtask/src/windows_integration.rs`
- [x] T006 Add failing report tests for header, row, terminal, capability stability, effect reconciliation, binary identity, and evidence currency in `xtask/src/windows_integration.rs`
- [x] T007 Add failing publication-hygiene tests for every prohibited secret, identity, path, payload, certificate, and raw-output class in `xtask/src/windows_integration.rs`
- [x] T008 [P] Add failing Windows integration probes for staged binary execution, privilege state, trust, Npcap, address family, recovery, key log, analyzer, and residue in `crates/fragcap-cli/tests/windows_native_integration.rs`
- [x] T009 Add failing finite child-runner tests for direct argv, bounded output, timeout, termination, and hidden Windows process creation in `xtask/src/windows_integration.rs`

## Phase 3: User Story 1 - Block Release Regressions on Windows (Priority: P1)

**Goal**: Establish one non-skippable, source-checked Windows completion authority.

**Independent Test**: Mutating any required row, completion domain, source test, workflow step, or terminal report makes `cargo xtask windows-integration` fail with the exact missing authority.

- [x] T010 [US1] Implement schema and closed-vocabulary parsing for `integration/windows-native-matrix-v1.json` in `xtask/src/windows_integration.rs`
- [x] T011 [US1] Implement duplicate-free exact row and completion-domain coverage validation in `xtask/src/windows_integration.rs`
- [x] T012 [US1] Implement repository-confined source-test lookup and required-test attribution validation in `xtask/src/windows_integration.rs`
- [x] T013 [US1] Derive completion inventory drift from existing native authority registries and product sources in `xtask/src/windows_integration.rs`
- [x] T014 [US1] Add `windows-integration` command routing and the static authority to `cargo xtask ci` in `xtask/src/main.rs`
- [x] T015 [US1] Implement immutable registry digest and staged executable version/SHA-256 identity validation in `xtask/src/windows_integration.rs`

## Phase 4: User Story 2 - Prove Windows Effects Are Scoped and Reversible (Priority: P2)

**Goal**: Execute real Windows paths with exact authority, outcome, and residue accounting.

**Independent Test**: Hosted and physical tier runs execute their complete row sets against a staged binary and finish only when capability and effect inventories reconcile.

- [x] T016 [US2] Implement immutable Windows preflight for privilege, Npcap runtime and compatibility mode, analyzer, IPv4/IPv6 loopback, interactive desktop, staged binary, and source revision in `xtask/src/windows_integration.rs`
- [x] T017 [US2] Implement direct finite child execution with redirected bounded output, per-row timeout, cancellation, and `CREATE_NO_WINDOW` in `xtask/src/windows_integration.rs`
- [x] T018 [US2] Implement staged binary, no-consent, non-admin, UAC-denial, and Doctor readiness probes in `crates/fragcap-cli/tests/windows_native_integration.rs`
- [x] T019 [US2] Implement exact current-user trust add/remove and mismatch probes with before/after state in `crates/fragcap-cli/tests/windows_native_integration.rs`
- [x] T020 [US2] Implement Npcap present/absent, live capture coexistence, and independent Deep Capture readiness probes in `crates/fragcap-cli/tests/windows_native_integration.rs`
- [x] T021 [US2] Implement IPv4/IPv6 exact loopback, no-wildcard, and no-firewall/system-proxy-mutation probes in `crates/fragcap-cli/tests/windows_native_integration.rs`
- [x] T022 [US2] Bind crash/restart rows to existing resource journal and Doctor recovery authorities with unrelated sentinel preservation in `crates/fragcap-cli/tests/windows_native_integration.rs`
- [x] T023 [US2] Bind key-log and unmodified analyzer rows to existing native conformance artifacts in `crates/fragcap-cli/tests/windows_native_integration.rs`
- [x] T024 [US2] Implement normalized before/after inventory and exact cleanup reconciliation for each effect-bearing row in `xtask/src/windows_integration.rs`
- [x] T025 [US2] Implement hosted and physical row selection that refuses capability mismatch and never converts it to a skip in `xtask/src/windows_integration.rs`

## Phase 5: User Story 3 - Retain Public-Safe Release Evidence (Priority: P3)

**Goal**: Produce bounded reviewable Windows evidence without publishing raw sensitive data.

**Independent Test**: Complete reports validate, incomplete prefixes remain incomplete, and every seeded secret or machine-identity value makes summary validation fail.

- [x] T026 [US3] Implement append-safe header, row, and terminal report records with exact completeness reconciliation in `xtask/src/windows_integration.rs`
- [x] T027 [US3] Implement closed-field public-safe summary derivation without raw child output in `xtask/src/windows_integration.rs`
- [x] T028 [US3] Implement prohibited-value, absolute-path, free-form-output, and evidence-size validation in `xtask/src/windows_integration.rs`
- [x] T029 [US3] Implement physical evidence registry digest, revision, product version, binary digest, row completeness, residue, and expiry validation in `xtask/src/windows_integration.rs`
- [x] T030 [US3] Execute the authorized non-admin physical tier and retain its validated sanitized authority in `integration/windows-native-reference-v1.json`

## Phase 6: Required Automation and Documentation

- [x] T031 Add the required finite Windows hosted workflow, temporary Npcap SDK build input, staged binary, summary validation, and safe upload in `.github/workflows/windows-integration.yml`
- [x] T032 Assert workflow triggers, required step identities, absence of row-level conditions and schedules, and non-upload of raw/SDK artifacts in `xtask/src/windows_integration.rs`
- [x] T033 [P] Add Windows completion-matrix vocabulary and primary references in `docs/glossary/capture-and-networking.md` and regenerate `docs/glossary/index.md`
- [x] T034 [P] Record S129, the #327/#329 staged-layout boundary, and the remaining release path in `docs/fragcap-specification.md`, `docs/fragcap-spec-outline.md`, `docs/plans/README.md`, and `AGENTS.md`
- [x] T035 [P] Add S129 feature and dated pinned-workflow/dependency-cycle decisions in `changelog.d/`

## Phase 7: Analysis, Convergence, and Verification

- [x] T036 Run spec-kit analysis across `spec.md`, `plan.md`, and `tasks.md` and remediate every finding
- [x] T037 Run registry mutation, report, sanitizer, child runner, staged binary, Windows effect, and workflow contract checks from `specs/129-windows-integration-matrix/quickstart.md`
- [x] T038 Run the hosted tier locally where capability-compatible and the physical tier on the authorized Windows host, then validate retained evidence and zero residue
- [x] T039 Run `cargo xtask ci`, formatting, locked tests, text hygiene, forbidden-capability checks, dependency locks, and mojibake checks
- [x] T040 Run post-implementation convergence, complete any appended tasks, mark every task in `specs/129-windows-integration-matrix/tasks.md`, and perform the final #327/#329/#334 scope audit

## Dependencies and Execution Order

- Phase 1 freezes the matrix before executable evidence is written.
- Phase 2 establishes red contract, runner, and Windows probe tests.
- User Story 1 creates the static release authority needed by all later work.
- User Story 2 depends on the registry and produces exact hosted/physical row evidence.
- User Story 3 derives only from complete raw results and never consumes unbounded child output.
- Automation and documentation follow stable row and report contracts.
- Analysis is blocking before implementation; convergence and full verification are blocking before commit.

## Parallel Opportunities

- T002 and T003 touch independent design/checklist artifacts.
- T008 can establish Windows probe tests while T005 through T007 and T009 establish task-runner tests.
- T018 through T023 are separable probe domains after common preflight and runner contracts exist, though shared test-file edits remain sequential.
- T033 through T035 touch independent documentation groups after behavior stabilizes.

## Implementation Strategy

1. Freeze required rows and explicit hosted/physical authority.
2. Make registry, evidence, runner, and sanitizer defects fail before building behavior.
3. Reuse existing production authorities for every row.
4. Run finite hosted and physical Windows tiers against one staged binary identity.
5. Retain only validated closed-field summaries.
6. Wire the release gate, document the packaging handoff, converge, and run all repository checks.
