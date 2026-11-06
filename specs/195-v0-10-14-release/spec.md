# Feature Specification: S195 v0.10.14 release stabilization and candidate

**Feature Branch**: `release/0.10.14`

**Created**: 2026-11-06

**Status**: In progress

**Input**: Correct the concrete Windows controlled-calibration timing defect in issue #566, prepare merged S189 through S194 as one coherent v0.10.14 source candidate under issue #567, push and open the official pull request, resolve every review comment through at most two hosted Codex rounds, and hand the green candidate to the owner before merge, tag, or publication.

## Clarifications

### Session 2026-11-06

- Q: Does the owner's autopilot authorization include merge, tag, or publication? -> A: No. It authorizes candidate preparation, push, the official PR, CI, and bounded review handling, then requires a halt for the owner's final merge ritual.
- Q: Which tracked issues should the candidate PR close? -> A: The PR closes timing defect #566. It references release-delivery issue #567, which remains open until later public tag, asset, registry, and current-marker reconciliation.
- Q: How are the hosted Codex review rounds counted? -> A: Automatic hosted review activity is round one. S195 may send at most one explicit request for round two and must never request a third round.
- Q: Is installed private-title testing part of S195 acceptance? -> A: No. Installed-product testing occurs after publication, and any concrete defect returns as a version-specific GitHub issue.

## User Scenarios & Testing

### User Story 1 - Trust deterministic source readiness (Priority: P1)

The owner can rely on the controlled Steam setup-to-calibration regression without its result changing solely because a hosted Windows runner scheduled the same work more slowly, while the product continues to honor every explicit launch deadline exactly.

**Why this priority**: The first post-merge S194 Windows CI attempt failed at the exact merged source and only a rerun passed. A release candidate must not carry a known timing-dependent automated assertion.

**Independent Test**: Repeated controlled Windows executions establish the setup-to-calibration transition without depending on a one-second runner timing race, and separate deadline-boundary coverage still reports a late launch as failed.

**Acceptance Scenarios**:

1. **Given** a controlled Steam target that has just accepted its client declaration, **when** fresh calibration begins under ordinary hosted scheduling variation, **then** the regression reaches and records the intended fresh-calibration path without changing production timeout defaults.
2. **Given** a controlled launch that actually exceeds its accepted finite deadline, **when** the deadline boundary is evaluated, **then** the result remains a truthful named failure rather than being widened or converted to success.

---

### User Story 2 - Review one coherent patch candidate (Priority: P1)

The owner can review one v0.10.14 source candidate containing S189 through S194 and the S195 readiness correction, with version, changelog, short notes, generated evidence, dependency authorities, specification, roadmap, and package identity agreeing.

**Why this priority**: The merged work is not available to users until one exact candidate assembles it without confusing candidate source with the existing v0.10.13 public release.

**Independent Test**: Every candidate-bound identity names v0.10.14 and the same source tree, every unreleased fragment appears exactly once, and all current-public markers continue to identify verified v0.10.13 bytes.

**Acceptance Scenarios**:

1. **Given** merged S189 through S194 and the S195 correction, **when** the candidate is prepared, **then** chronological release history and concise notes describe each user-visible outcome once and use valid release-fragment formatting.
2. **Given** no public v0.10.14 bytes, **when** candidate documentation is reviewed, **then** current download, registry, and frozen published-review markers still point to v0.10.13.
3. **Given** stale current-state prose that says a completed issue remains open, **when** the candidate documentation is reconciled, **then** it accurately describes the shipped baseline, the candidate scope, and post-release defect handling.

---

### User Story 3 - Receive a review-ready official PR (Priority: P2)

The owner receives an official candidate pull request whose final head has terminal successful applicable checks, complete package evidence, and no unanswered or unresolved review comment, while merge and publication remain owner actions.

**Why this priority**: The candidate is useful only when its exact head is reviewable and the owner can perform the normal merge ritual without reconstructing CI or review state.

**Independent Test**: The final PR head matches the checked candidate, all applicable checks are terminal success, every actual review thread is answered and resolved, and no more than one explicit second Codex review request was sent.

**Acceptance Scenarios**:

1. **Given** an actionable code or security review comment, **when** the candidate changes, **then** the comment receives a specific response, its thread is resolved after the correction, and final-head checks run again.
2. **Given** a green reviewed head, **when** S195 reaches handoff, **then** no merge, tag, GitHub release, registry publication, installed private-title test, or current-public marker advance has occurred.

### Edge Cases

- The observed Windows failure may pass on rerun; the initial exact receipt remains valid evidence of nondeterminism and is not erased by the later success.
- Release preview may reject a fragment before version mutation. The invalid source fragment is corrected and previewed again rather than bypassing the validator.
- A fresh advisory or unexpected third-party package, feature, provider, or source change blocks candidate acceptance until corrected or explicitly scoped through the existing governance process.
- A hosted reviewer may leave only a reaction or no separate receipt. Actual comments, reactions, review submissions, and threads are reported literally without inferring an absent approval.
- Any source change after a review or successful check invalidates earlier final-head claims and requires renewed applicable checks and comment reconciliation.
- Package certification may run on GitHub's temporary merge commit. An exact branch-head package run remains required for the candidate that the owner will merge.

## Requirements

### Functional Requirements

- **FR-001**: S195 MUST begin from merged `main` commit `1e44519a8d34406fcd3178365c1dde68b689439d` or a verified fast-forward successor, on `release/0.10.14`, with issues #566 and #567 carrying the authoritative S195 mapping.
- **FR-002**: S195 MUST correct issue #566 by making the setup-to-fresh-calibration regression deterministic without changing production timeout defaults, silently widening operator-supplied values, or weakening separate launch-deadline boundary coverage.
- **FR-003**: S195 MUST update all candidate-bound first-party versions, generated output, conformance identity, dependency authorities, specification Applies-To value, changelog, short notes, and package identity together to v0.10.14.
- **FR-004**: S195 MUST include every unreleased fragment exactly once in chronological release history, correct fragment-format violations rather than bypassing validation, and describe S189 through S194 plus #566 in user-relevant language.
- **FR-005**: S195 MUST reconcile the roadmap and current-state documentation, including stale statements that issue #547 remains open, while preserving v0.10.13 as the actual current public baseline.
- **FR-006**: S195 MUST audit the product and every isolated locked dependency graph against current advisory, license, ban, and source policy without adding a blanket waiver or accepting unreviewed third-party resolution drift.
- **FR-007**: S195 MUST run the repository's complete local automated checks and exact-head hosted checks, including Windows package certification, release-notes validation, same-version documentation, controlled conformance, minimum-version, neutral-platform, supply-chain, text, and shell requirements.
- **FR-008**: S195 MUST push `release/0.10.14`, open its official pull request, close #566 through that PR, and reference #567 without closing the release-delivery issue before actual publication reconciliation.
- **FR-009**: S195 MUST inspect every hosted code and security review surface, answer every actionable comment, resolve every thread after any required correction, and trigger no more than one explicit second Codex review round after the automatic first round.
- **FR-010**: S195 MUST stop at owner final review and merge. It MUST NOT merge the PR, create or push a tag, approve protected publication, claim public v0.10.14 assets or crates, close #567, or advance current-public markers.
- **FR-011**: S195 MUST NOT perform or require installed private-title testing. Installed-product observations occur only after publication, and a concrete failure becomes a version-specific GitHub issue.

### Key Entities

- **Timing-defect receipt**: The exact failed Windows attempt, successful rerun, corrected regression design, and final controlled results for issue #566.
- **Candidate source**: One immutable reviewable tree whose first-party and generated authorities agree on v0.10.14.
- **Published release**: The independently verified v0.10.13 tag, assets, registry archives, and current markers that remain authoritative until later publication reconciliation.
- **Qualification receipt**: Exact candidate identity, command or hosted run, terminal outcome, artifact identity, and explicit evidence limit.
- **Review receipt**: PR number, exact head, reviewer or bot surface, round identity, comment disposition, thread resolution, and applicable check state.

## Success Criteria

### Measurable Outcomes

- **SC-001**: The corrected setup-to-fresh-calibration regression completes successfully across repeated controlled Windows executions, while an exact late-launch case continues to produce its named deadline failure.
- **SC-002**: All v0.10.14 candidate-bound identities agree, every unreleased fragment is consumed exactly once, and every current-public marker remains on v0.10.13.
- **SC-003**: Product and isolated dependency inventories show no unreviewed third-party identity or feature drift and pass current advisory, license, ban, and source policy.
- **SC-004**: The final official PR head has terminal success for every applicable check, exact-head package evidence, zero unresolved review threads, responses to every actionable comment, and no more than two total Codex review rounds.
- **SC-005**: The owner receives the final PR URL and exact head with merge, tag, publication, installed-product behavior, and #567 closure still unclaimed.

## Assumptions

- v0.10.14 is a patch release because it delivers bounded continuity, diagnostics, ownership, storage, relay, calibration, and test-determinism corrections without introducing a new product family.
- The official release preparation script creates a release branch from clean `main`; because Spec-Kit work is already on the required release branch, S195 will execute the script's documented constituent operations there after a successful dry-run preview.
- Automatic hosted bot activity counts as review round one. At most one explicit second Codex request is permitted.
- Issue #566 closes when the reviewed correction merges. Issue #567 remains open through actual tag, publication, independent public-file and registry reconciliation, and current-record delivery under later owner authorization.
- No absence of a filed post-release defect establishes universal title compatibility or independent security acceptance.
