# Feature Specification: S166 v0.10.5 patch release

**Feature Branch**: `release/0.10.5`
**Created**: 2026-10-05
**Status**: Published; current-state records awaiting human review and merge
**Input**: The operator requested complete publication without further input. Repository governance reserves PR approval, merge, and tag push for the human operator. This slice prepares and verifies everything possible before that boundary.

## User Scenarios & Testing

### User Story 1 - Accurate release candidate (Priority: P1)

The operator needs a v0.10.5 candidate containing all merged changes since v0.10.4, including S165's managed route lifetime correction, with matching version identities, changelog, and public notes.

**Independent Test**: First-party versions, lockfiles, generated evidence, specification applicability, changelog, and highlights agree on the candidate version. Local and hosted gates pass on the final PR head.

### User Story 2 - Verified publication (Priority: P2)

After human approval, merge, and tag push, the operator needs a public release whose artifacts, certification report, and ten registry versions independently reconcile to the exact merged source.

**Independent Test**: Annotated tag, release run, six assets and checksums, package certification, and non-yanked crate versions are checked before current published-state records advance in a separate reviewed PR.

### Edge Cases

- Main or PR head changes after candidate verification.
- A conflicting tag or published version appears.
- A required CI job fails or protected deployment awaits human review.
- Controlled S165 tests pass but a real game remains unverified.

## Requirements

- **FR-001**: Prepare v0.10.5 from exact merged main and include every unreleased fragment since v0.10.4 in chronological order.
- **FR-002**: Align first-party versions, isolated lockfiles, generated output, native conformance identity, supply-chain snapshot, specification applicability, and release notes.
- **FR-003**: Preserve current published-state markers at verified v0.10.4 until independent v0.10.5 publication evidence exists.
- **FR-004**: Pass source, documentation, notes, specification, version, MSRV, neutral, generated-output, text-hygiene, and final-head hosted CI gates.
- **FR-005**: Open a candidate PR and address actionable review findings. Human approval, merge, and tag push remain required by `CONTRIBUTING.md`.
- **FR-006**: Do not claim public completion from a candidate PR or tag alone. Verify all release jobs, protected deployment, six public files, certification, and ten non-yanked crates.
- **FR-007**: Move published-state records only through a separate reviewed PR after public verification.
- **FR-008**: Do not claim universal game compatibility or an owner field trial from controlled release gates.

## Success Criteria

- **SC-001**: One internally consistent v0.10.5 candidate PR has green required final-head checks and no unresolved actionable review findings.
- **SC-002**: Publication is reported complete only after exact-source tagging, public-asset and registry reconciliation, and reviewed current-record updates.
- **SC-003**: The human-only gates are identified precisely if they prevent completion without operator action.
