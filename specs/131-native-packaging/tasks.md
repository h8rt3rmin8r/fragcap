# Tasks: Native Windows Packaging Certification

**Input**: Design documents from `specs/131-native-packaging/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Required by FR-019, SC-006, SC-008, and the autopilot TDD protocol.

## Phase 1: Setup and Frozen Contracts

- [x] T001 Validate the active feature selector, branch, issue #329 scope, merged S130 prerequisite, current package payload, and #334 handoff in `.specify/feature.json`, `specs/131-native-packaging/`, and live issues
- [x] T002 [P] Finalize package, build-identity, PE, signature, ownership, lifecycle, report, and publication contracts in `specs/131-native-packaging/contracts/package-certification.md` and `specs/131-native-packaging/data-model.md`
- [x] T003 [P] Validate requirement and packaging quality in `specs/131-native-packaging/checklists/requirements.md` and `specs/131-native-packaging/checklists/packaging.md`
- [x] T004 Freeze the closed machine-readable contract in `integration/windows-package-contract-v1.json`

## Phase 2: Failing Static and Report Tests

- [x] T005 Add failing contract-schema tests for unknown fields, closed vocabularies, exact artifacts, sidecars, shared entries, size ceilings, unique identities, and report bounds in `xtask/src/package_certification.rs`
- [x] T006 Add failing workflow and WiX tests for early release identity, exact package payload, pinned tools, blocking certification, certified-byte publication, upgrade behavior, and exact effect ownership in `xtask/src/package_certification.rs`
- [x] T007 Add failing report tests for artifact inventory, entry identity, checksums, signature states, PE machine/imports, build identity, smoke result, lifecycle completeness, timeouts, residue, and path hygiene in `xtask/src/package_certification.rs`
- [x] T008 Add failing CLI tests for the exact machine-readable native release build identity in `crates/fragcap-cli/tests/`
- [x] T009 Add failing PowerShell fixture checks for malformed packages, checksum drift, wrong signature state, lifecycle exit failure, user-state mutation, and residue in the package-certification harness fixtures

## Phase 3: User Story 1 - Certify Complete Native Artifacts (Priority: P1)

**Goal**: Prove that final ZIP and MSI bytes contain the complete native product with no undeclared prerequisite or entry.

**Independent Test**: Build both official package forms once, inspect every final entry and executable property, and complete the constrained native packaged-binary smoke.

- [x] T010 [US1] Implement strict contract parsing, closed-schema validation, deterministic diagnostics, size ceilings, prohibited-content rules, and exact repository wiring checks in `xtask/src/package_certification.rs`
- [x] T011 [US1] Add `package-certification` command routing and the offline gate to `cargo xtask ci` in `xtask/src/main.rs`
- [x] T012 [US1] Implement machine-readable release build identity with version, revision, target, architecture, features, native backend, and official marker in `crates/fragcap-cli/build.rs` and CLI command handling
- [x] T013 [US1] Implement safe ZIP inventory/extraction, exact MSI installed-file reconciliation, shared-file byte identity, per-entry and per-artifact size enforcement, and standalone catalog matching in `scripts/Test-PackageCertification.ps1`
- [x] T014 [US1] Implement PE machine, ordinary import, delayed import, version-resource, and packaged build-identity checks in `scripts/Test-PackageCertification.ps1`
- [x] T015 [US1] Implement the sanitized-environment packaged native smoke with bounded hidden child execution and explicit no-download/no-external-runtime evidence in `scripts/Test-PackageCertification.ps1`

## Phase 4: User Story 2 - Certify Installer Lifecycle and Ownership (Priority: P2)

**Goal**: Prove that the MSI installs, repairs, reinstalls, upgrades, refuses downgrade, and uninstalls within exact ownership boundaries.

**Independent Test**: Exercise every required lifecycle case in fresh Windows roots and reconcile owned plus seeded user-owned state after each transition.

- [x] T016 [US2] Correct WiX upgrade ordering and make Defender exclusion cleanup conditional on exact installer-created ownership in `crates/fragcap-cli/wix/main.wxs`
- [x] T017 [US2] Implement hidden finite Windows Installer invocation, exit accounting, local-only log retention, and finally cleanup in `scripts/Test-PackageCertification.ps1`
- [x] T018 [US2] Implement clean install, repair after deletion/mutation, exact-byte reinstall, and uninstall cases with exact owned-state reconciliation in `scripts/Test-PackageCertification.ps1`
- [x] T019 [US2] Implement digest-pinned v0.8.0 predecessor upgrade and newer-package downgrade-refusal cases in `scripts/Test-PackageCertification.ps1`
- [x] T020 [US2] Seed and prove byte preservation of per-user databases, captures, bundles, pre-existing exact Defender exclusion, and independently managed extcap state across applicable cases in `scripts/Test-PackageCertification.ps1`
- [x] T021 [US2] Emit one bounded public-safe versioned report and implement complete report reconciliation in `xtask/src/package_certification.rs`

## Phase 5: User Story 3 - Gate Published Integrity and Truth (Priority: P3)

**Goal**: Publish only the exact certified bytes with independently checked integrity and accurate unsigned state.

**Independent Test**: Alter each artifact, checksum, signature state, identity field, report row, and release dependency edge and require publication refusal.

- [x] T022 [US3] Implement exact primary-artifact checksum sidecars, duplicate/missing/extra/stale rejection, and independent post-transfer rehash in the harness and task-runner report validator
- [x] T023 [US3] Implement determinate Authenticode `NotSigned` checks for MSI and executable surfaces plus explicit not-applicable states for all other roles in the harness and report validator
- [x] T024 [US3] Add one required package-certification workflow for pull requests, `main`, manual dispatch, and tag release that builds candidate bytes once, acquires the pinned predecessor before offline testing, runs certification, and uploads only the certified bundle plus sanitized summary in `.github/workflows/package-certification.yml`
- [x] T025 [US3] Refactor `.github/workflows/release.yml` to validate tag identity before build, consume and independently revalidate the certified bundle, and preserve the strict certification-before-GitHub-release-before-crates order with no best-effort package path
- [x] T026 [US3] Pin and unify cargo-wix, WiX, and Npcap SDK package-build inputs without distributing them

## Phase 6: Documentation and Architecture Record

- [x] T027 [P] Correct `catalog.db` download naming, package-scoped no-download wording, native Doctor labeling, unsigned guidance, and current package prerequisites in `README.md`, `NOTICE`, site documentation, and CLI Doctor output
- [x] T028 [P] Document package certification, local validation, lifecycle ownership, unsigned state, tool updates, failure handling, and release consumption in `docs/maintainers/package-certification.md`
- [x] T029 [P] Add package contract, artifact identity, package entry, build identity, installer effect, lifecycle case, and certification report vocabulary to the glossary and regenerate its index
- [x] T030 [P] Record S131 certification, publication order, and #334 handoff in `docs/fragcap-specification.md`, `docs/fragcap-spec-outline.md`, `docs/plans/README.md`, and `AGENTS.md`
- [x] T031 [P] Add S131 feature and dated package/WiX/workflow decision fragments in `changelog.d/`

## Phase 7: Analysis, Convergence, and Verification

- [x] T032 Run spec-kit analysis across `spec.md`, `plan.md`, and `tasks.md` and remediate every finding before implementation
- [x] T033 Run contract schema, workflow, WiX, build-identity, package mutation, checksum, signature, PE, report, lifecycle, timeout, ownership, and path-hygiene tests from `specs/131-native-packaging/quickstart.md`
- [x] T034 Run the live Windows package-certification workflow and confirm every final-byte, smoke, lifecycle, integrity, and cleanup row reconciles
- [x] T035 Run `cargo xtask ci`, formatting, locked tests, MSRV build, text hygiene, PowerShell compliance, forbidden-capability checks, dependency locks, and mojibake checks
- [x] T036 Run post-implementation convergence, complete appended tasks, mark every task in `specs/131-native-packaging/tasks.md`, and perform the final #329/#334 scope audit

## Dependencies and Execution Order

- Phase 1 freezes the final package and lifecycle contracts before code.
- Phase 2 establishes red tests before implementation.
- User Story 1 produces certified final content required by both later stories.
- User Story 2 owns real MSI effects and cannot run before final candidate bytes exist.
- User Story 3 binds integrity evidence and publication only after artifact plus lifecycle certification.
- Documentation follows stable contracts; analysis is blocking before implementation and convergence is blocking before commit.

## Parallel Opportunities

- T002 and T003 touch independent specification artifacts.
- T005 through T009 describe separable failing-test domains, though shared implementation files require serialized edits in this workspace.
- T013 through T015 are independent harness concerns after its report skeleton exists.
- T027 through T031 touch independent documentation groups after behavior stabilizes.

## Implementation Strategy

1. Freeze the smallest closed contract over the release artifacts already shipped.
2. Make every missing authority fail through mutation tests before implementing it.
3. Reuse S129 child containment, S130 validated inputs, and native Windows package inspection APIs.
4. Build once, certify those bytes, and publish those bytes without a reconstruction gap.
5. Exercise the issue-required MSI transitions while avoiding a new generic installer fault-injection system.
6. Converge, run the complete local and hosted gates, then publish one reviewed pull request.

## Phase 8: Convergence

- [x] T037 Add exact primary-artifact, checksum-sidecar, package-entry, size, digest, role, signature, and PE version-resource rows to the bounded certification report and validator per FR-015, FR-016, FR-017, and CertificationReport (partial)
- [x] T038 Bind MSI ProductName, ProductVersion, Manufacturer, UpgradeCode, and installed executable identity to the certified build identity per FR-016 and US2/AC1 (partial)
- [x] T039 Add controlled mutation coverage for every missing, extra, altered, stale, mis-versioned, mis-featured, prohibited, unsigned-policy, checksum, traversal, lifecycle, timeout, and residue failure class per SC-006 (partial)
