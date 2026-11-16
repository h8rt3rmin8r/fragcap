# Feature Specification: Verified Test Cleanup

**Feature Branch**: `codex/s208-test-cleanup`

**Created**: 2026-11-16

**Status**: Implemented; hosted CI and owner review pending

**Input**: Owner-approved S208 bundle for #605 and #606: require every application test to own and verify cleanup of its generated local artifacts, repair the demonstrated Deep Capture and skills-test leaks, and audit other test producers without deleting unrelated or product-managed evidence.

## Scope

**In**: Verified lifecycle for test-owned local artifacts, repair of proven fixture leaks, a bounded residue check, and a versioned governance amendment with aligned contributor and reviewer guidance.

**Out**: Deletion of historic unproven directories, product-managed native-session retention changes, live-game qualification, routing and inspection repairs in epic #607, and release publication.

**Done when**: #605 and #606 meet their controlled acceptance criteria, the required repository gates pass, and the PR is ready for owner review. A future private field run is not a completion gate.

## Clarifications

### Session 2026-11-16

- **Exact ownership**: A run may reconcile only its own recorded, canonical, inactive fixture roots. A matching name prefix is never ownership proof.
- **Abnormal termination**: A process killed before finalization cannot truthfully report successful cleanup. The next bounded check must identify any exact inactive residue and either reconcile it or report why it remains.
- **Native evidence**: Product-managed bundles keep their retention and recovery contract. Test scratch cleanup cannot erase the only unresolved session evidence.
- **Audit scope**: Repair demonstrated persistent test writers in this slice; record producers with separate ownership rather than broadening product behavior.

## User Scenarios & Testing

### User Story 1 - Automated tests leave no owned residue (Priority: P1)

A contributor runs the ordinary test suite, including Deep Capture session tests. Every test-created directory and file has an exact owner, remains available while assertions need it, and is gone when the test reports completion.

**Why this priority**: The observed accumulation includes 10,872 directories with the name used by the Deep Capture session test. Preventing further accumulation is the immediate defect correction.

**Independent Test**: Run the affected controlled tests against an isolated effective temp root, inspect their artifacts during assertions, and confirm that no newly owned directory remains after success or a controlled assertion failure.

**Acceptance Scenarios**:

1. **Given** a test that creates a session bundle, **when** its assertions finish, **then** the test closes the bundle and verifies its complete owned tree is absent.
2. **Given** a test that fails an assertion after creating files, **when** the test terminates, **then** an exact owner record permits bounded reconciliation and the failure report names any residue.
3. **Given** concurrent test runs, **when** they create fixtures, **then** their ownership and cleanup remain separate and neither run deletes the other's files.

### User Story 2 - Test handoffs prove cleanup (Priority: P2)

A contributor or reviewer can tell which effective local roots a test used, what it cleaned, and what remains. A passing assertion alone never stands in for a cleanup result.

**Why this priority**: Test output can be redirected into alternate temp or application-data roots. A nominal path or silent destructor does not establish absence of residue.

**Independent Test**: Run a controlled gate with one successful cleanup and one forced cleanup failure; verify the first passes with no new owned residue and the second fails with the exact remaining scope.

**Acceptance Scenarios**:

1. **Given** a test run that finishes normally, **when** its handoff is prepared, **then** effective canonical roots, cleanup actions, absence checks, and residuals are stated.
2. **Given** a cleanup failure or interruption, **when** the run is reconciled, **then** only proven inactive fixtures are eligible for removal and an unresolved path is reported as residual.
3. **Given** a product-managed native bundle or an unrelated same-prefix directory, **when** the residue check runs, **then** it does not delete or claim ownership of that material.

### User Story 3 - Cleanup guidance is consistent (Priority: P3)

Agents, contributors, and reviewers follow the same end-of-test rule for automated tests, package certification, controlled application tests, and installed-product field tests.

**Why this priority**: Existing guidance covers installed default paths but leaves automated and interrupted test cleanup underspecified.

**Independent Test**: Check the constitution, agent and contributor guidance, and review instructions against the same success, failure, interruption, and unresolved-native-session examples.

**Acceptance Scenarios**:

1. **Given** a completed application test, **when** its results are reported, **then** the handoff names exact effective locations, cleanup verification, and residuals.
2. **Given** an unresolved native session, **when** cleanup would destroy its only recovery evidence, **then** the guidance requires preserving and reporting it rather than claiming cleanup succeeded.

### Edge Cases

- A test panics after writing but before explicit close.
- A child process or open file still holds a fixture at cleanup time.
- A test process is interrupted before any finalizer runs.
- A redirected temp root differs from the nominal environment variable.
- An attacker or unrelated user places a same-prefix path or a reparse point near the fixture.
- A pre-existing historic test directory has no trustworthy owner record.
- Cleanup reports success but a directory is recreated before the absence check.

## Requirements

### Functional Requirements

- **FR-001**: Every scoped automated test writer MUST create a unique, exact-owned local output tree and preserve its artifacts through all assertions.
- **FR-002**: Normal completion MUST perform fallible cleanup and verify absence before reporting test success; cleanup failure MUST fail the test or gate with the exact remaining path.
- **FR-003**: Assertion-failure and interruption handling MUST retain an exact ownership record whenever residue remains; later reconciliation MUST require exact path identity and inactive ownership before deletion, and MUST preserve uncertain material with a named reason.
- **FR-004**: The Deep Capture session test and fixed skills-test scratch writer MUST stop creating persistent or colliding directories.
- **FR-005**: An audit MUST identify other automated test, xtask, certification, and controlled application-test writers that produce persistent local artifacts, repair proven leaks in this slice, and name separately owned producers without assuming every temp path leaks.
- **FR-006**: A bounded post-run check MUST distinguish this run's owned residue from pre-existing, unrelated, product-managed, and unresolved recovery material, including across effective temp and redirected roots.
- **FR-007**: The versioned constitution and agent, contributor, test, and review guidance MUST state end-of-test cleanup and verification duties for success, failure, timeout, and interruption without weakening native session retention or recovery.
- **FR-008**: Public reports MUST omit private paths, identities, raw captures, and credentials; local failure reports MAY state the exact residual path to the operator.

### Key Entities

- **Test fixture owner**: One run-scoped identity tied to an exact canonical root, active lifetime, and cleanup disposition.
- **Residual inventory**: Bounded list of exact owned paths remaining after cleanup, each with a cause or unresolved status.
- **Native session evidence**: Product-managed bundle and recovery state with a separate owner and retention policy; never inferred to be disposable test scratch by a name prefix.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Controlled Windows before/after inventories show zero new test-owned directories after the corrected test and xtask gates finish successfully.
- **SC-002**: Every forced cleanup failure in the controlled matrix causes a failing result that names its exact residual path; no failure is silently converted to success.
- **SC-003**: Success, assertion failure, interruption, concurrent-run, same-prefix unrelated path, redirected-root, and unresolved-native-session cases each produce their specified outcome without cross-owner deletion.
- **SC-004**: The repository's required documentation, source, and test gates pass, and the handoff states all effective roots, cleanup verification, and any remaining material.

## Assumptions

- S208 repairs test ownership and guidance, not product native-session retention or the Deep Capture routing defects in epic #607.
- Historic directories are evidence for diagnosis but are not deleted by this code slice without independently proven ownership and inactivity.
- The controlled validation uses synthetic local roots and needs no live game, real trust mutation, or deletion of existing native bundles.
