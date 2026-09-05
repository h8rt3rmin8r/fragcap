# Tasks: Native Supply-Chain and Compatibility Gate

**Input**: Design documents from `specs/130-native-supply-chain/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Required by FR-015, SC-002, SC-004, SC-006, and the autopilot TDD protocol.

## Phase 1: Setup and Frozen Contracts

- [x] T001 Validate the active feature selector, branch, issue #328 scope, closed #280 prerequisite, merged S129 baseline, and S131/#329 handoff in `.specify/feature.json`, `specs/130-native-supply-chain/`, and live issues
- [x] T002 [P] Finalize graph, policy, exception, report, evidence, and workflow contracts in `specs/130-native-supply-chain/contracts/supply-chain-gate.md` and `specs/130-native-supply-chain/data-model.md`
- [x] T003 [P] Validate requirement and supply-chain quality in `specs/130-native-supply-chain/checklists/requirements.md` and `specs/130-native-supply-chain/checklists/supply-chain.md`
- [x] T004 Freeze the versioned policy shape and exact tool identities in `supply-chain/policy-v1.json`, `supply-chain/about.toml`, and `supply-chain/about.hbs`

## Phase 2: Failing Policy and Evidence Tests

- [x] T005 Add failing policy-schema tests for unknown fields, closed vocabularies, required graph views, date parsing, and sorted unique identities in `xtask/src/supply_chain.rs`
- [x] T006 Add failing normalized-graph tests for stable workspace identities, sources, checksums, licenses, declared MSRV, activated features, dependency kinds, target expressions, and deterministic digests in `xtask/src/supply_chain.rs`
- [x] T007 Add failing exception tests for missing fields, wildcards, duplicate identities, expired records, unused records, broad scopes, and exact finding matches in `xtask/src/supply_chain.rs`
- [x] T008 Add failing critical-dependency and compatibility-line tests for pin, feature, default-feature, cadence, review expiry, pre-1.0 semantics, and unapproved lineages in `xtask/src/supply_chain.rs`
- [x] T009 Add failing release-evidence tests for CycloneDX identity, component and dependency reconciliation, notices markers, missing and duplicate packages, stale graph binding, and absolute-path leakage in `xtask/src/supply_chain.rs`
- [x] T010 Add failing workflow and WiX contract tests for triggers, immutable tool pins, blocking behavior, generation-before-validation-before-packaging order, and both package payloads in `xtask/src/supply_chain.rs`

## Phase 3: User Story 1 - Block Unsafe Dependency Drift (Priority: P1)

**Goal**: Make the complete reviewed graph a blocking offline and network-backed pull-request authority.

**Independent Test**: Controlled graph, feature, source, compatibility, MSRV, unsafe-review, and workflow mutations each fail with an exact stable rule.

- [x] T011 [US1] Implement strict policy parsing, closed-schema validation, ISO date handling, and bounded sorted diagnostics in `xtask/src/supply_chain.rs`
- [x] T012 [US1] Implement locked offline Cargo metadata collection for Windows all-feature, Linux all-feature, and exact Windows release views in `xtask/src/supply_chain.rs`
- [x] T013 [US1] Implement stable package and edge normalization without absolute workspace paths in `xtask/src/supply_chain.rs`
- [x] T014 [US1] Implement SHA-256 graph, policy, and lockfile identities with exact count reconciliation in `xtask/src/supply_chain.rs`
- [x] T015 [US1] Implement allowed source, registry checksum, license metadata, and declared dependency MSRV validation in `xtask/src/supply_chain.rs`
- [x] T016 [US1] Implement critical pin, resolved feature, default-feature, cadence, compatibility-line, and unsafe-review graph binding validation in `xtask/src/supply_chain.rs`
- [x] T017 [US1] Populate reviewed graph digests, duplicate compatibility records, critical native dependency records, unsafe containment reviews, and empty finite exception set in `supply-chain/policy-v1.json`
- [x] T018 [US1] Add `supply-chain` command routing and the offline gate to `cargo xtask ci` in `xtask/src/main.rs`
- [x] T019 [US1] Tighten cargo-deny graph, duplicate, advisory, license, ban, and source policy with explicit all-feature Windows/Linux coverage in `deny.toml`
- [x] T020 [US1] Replace stale audit automation with blocking pull-request, `main`, weekly, and manual coverage using immutable action and exact toolchain pins in `.github/workflows/audit.yml`

## Phase 4: User Story 2 - Govern Dependency Maintenance and Exceptions (Priority: P2)

**Goal**: Make ordinary updates, emergency fixes, and temporary exceptions finite and reviewable.

**Independent Test**: Procedure fixtures pass for legitimate maintenance and fail for expired, unused, incomplete, or bypassing exceptions.

- [x] T021 [US2] Implement exact exception-to-finding matching and reject malformed, expired, duplicate, wildcard, unused, or infrastructure-bypass records in `xtask/src/supply_chain.rs`
- [x] T022 [US2] Implement critical review-cadence expiry and emergency expectation checks in `xtask/src/supply_chain.rs`
- [x] T023 [US2] Document routine single-package update, coordinated critical-stack update, emergency advisory patch, rollback, evidence regeneration, and approval preservation in `docs/maintainers/supply-chain.md`
- [x] T024 [US2] Bind the procedure's required commands, policy fields, decision record, and rollback checkpoints to static validation in `xtask/src/supply_chain.rs`

## Phase 5: User Story 3 - Ship Auditable Dependency Evidence (Priority: P3)

**Goal**: Embed accurate, deterministic, independently validated dependency evidence in both existing release packages.

**Independent Test**: Exact-pinned generation produces evidence that reconciles to the shipped Windows closure, while seeded missing, duplicate, stale, or path-leaking evidence fails before packaging.

- [x] T025 [US3] Finalize exact cargo-about license configuration and deterministic notice template in `supply-chain/about.toml` and `supply-chain/about.hbs`
- [x] T026 [US3] Implement CycloneDX 1.5 metadata, root, component, dependency, target, feature, version, policy, and lock binding validation in `xtask/src/supply_chain.rs`
- [x] T027 [US3] Implement third-party notice package-marker, legal-text, ordering, duplicate, omission, and path-hygiene validation in `xtask/src/supply_chain.rs`
- [x] T028 [US3] Wire exact-pinned cargo-cyclonedx and cargo-about generation plus independent validation before package assembly in `.github/workflows/release.yml`
- [x] T029 [US3] Add `fragcap.cdx.json` and `THIRD-PARTY-NOTICES.txt` to fixed installer components and feature ownership in `crates/fragcap-cli/wix/main.wxs`
- [x] T030 [US3] Add both evidence files to portable archive staging and assert their installed presence in the existing MSI smoke path in `.github/workflows/release.yml`

## Phase 6: Documentation and Architecture Record

- [x] T031 [P] Add supply-chain gate, software bill of materials, third-party notices, compatibility line, and finite exception vocabulary to `docs/glossary/platform-and-distribution.md` and regenerate `docs/glossary/index.md`
- [x] T032 [P] Record S130 policy, audit triggers, release evidence, and S131 handoff in `docs/fragcap-specification.md`, `docs/fragcap-spec-outline.md`, `docs/plans/README.md`, and `AGENTS.md`
- [x] T033 [P] Add S130 feature and dated pinned audit/release/tool decisions in `changelog.d/`

## Phase 7: Analysis, Convergence, and Verification

- [x] T034 Run spec-kit analysis across `spec.md`, `plan.md`, and `tasks.md` and remediate every finding before implementation
- [x] T035 Run policy schema, graph mutation, exception, critical dependency, compatibility, unsafe-review, workflow, WiX, SBOM, and notices tests from `specs/130-native-supply-chain/quickstart.md`
- [x] T036 Run the live offline graph gate and confirm every all-feature Windows/Linux and shipped Windows package and edge reconciles
- [x] T037 Run exact-pinned cargo-deny against fresh advisory data and generate plus validate release evidence twice under one fixed `SOURCE_DATE_EPOCH`
- [x] T038 Run `cargo xtask ci`, formatting, locked tests, MSRV build, text hygiene, forbidden-capability checks, dependency locks, and mojibake checks
- [x] T039 Run post-implementation convergence, complete appended tasks, mark every task in `specs/130-native-supply-chain/tasks.md`, and perform the final #328/#329/#334 scope audit

## Dependencies and Execution Order

- Phase 1 freezes the policy and standard-generator contracts before code.
- Phase 2 establishes red tests before implementation.
- User Story 1 is the blocking graph authority required by both later stories.
- User Story 2 extends that authority with finite maintenance and exceptions.
- User Story 3 consumes only the exact approved release closure and cannot package before validation.
- Documentation follows stable contracts; analysis is blocking before implementation and convergence is blocking before commit.

## Parallel Opportunities

- T002 and T003 touch independent specification artifacts.
- T005 through T010 describe separable test domains, though they share one implementation file and are executed sequentially in this workspace.
- T019 and T020 are separate from repository-owned validator implementation after policy semantics freeze.
- T031 through T033 touch independent documentation groups after behavior stabilizes.

## Implementation Strategy

1. Freeze one exact closed policy over the graph already selected by #280.
2. Make every missing authority fail in synthetic tests before implementing it.
3. Reuse cargo-deny, cargo-cyclonedx, and cargo-about for the standards they own.
4. Keep repository code limited to normalization, governance, reconciliation, and workflow ordering.
5. Embed evidence in both existing packages without expanding release downloads or taking S131's certification scope.
6. Converge, run the complete offline and network-backed gates, then publish one reviewed pull request.
