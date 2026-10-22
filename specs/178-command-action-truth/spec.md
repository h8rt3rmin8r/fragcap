# Feature Specification: Truthful CLI Action Boundaries

**Feature Branch**: `codex/s178-command-action-truth`

**Created**: 2026-10-21

**Status**: Draft

**Input**: Owner-approved S178 grouping of GitHub issues #489, #491, #492, and #493.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Review bundle deletion honestly (Priority: P1)

An operator previews bundle collection and can tell which contents an authorized Apply would delete, which retained contents require an explicit wider selection, and why a missing or unowned path cannot be collected. A preview with no eligible work gives a read-only next step instead of an Apply command. Related cleanup and export commands identify an invalid bundle path before recommending a destructive retry or returning an unhelpful operating-system error.

**Why this priority**: Collection deletes evidence, so a misleading Apply suggestion has a material consequence.

**Independent Test**: Use disposable missing, empty unowned, retained, and eligible bundle roots; compare human and structured previews, offered commands, and exact contents after authorized collection.

**Acceptance Scenarios**:

1. **Given** an eligible exact bundle, **when** the user previews collection, **then** help and preview explain that Apply deletes eligible contents and identify the selected scope.
2. **Given** a missing path, unowned empty directory, incomplete scan, unresolved authority, or zero eligible content, **when** collection is previewed, **then** the reason is visible in human and structured output and no Apply command is recommended.
3. **Given** retained or custom evidence, **when** collection is previewed without explicit retained selection, **then** that evidence is not represented as eligible for ordinary deletion and the wider `--include-retained` effect is explained.
4. **Given** a missing bundle path, **when** cleanup or export is requested, **then** the error names the path and offers a valid read-only next step without suggesting `--yes`.

---

### User Story 2 - Export the target actually selected (Priority: P1)

A user names a target or passes its stable identifier to export it. If both selector forms are supplied, export refuses before writing target bytes rather than silently choosing one. Missing-selector guidance names a relevant inspection or retry action for show, remove, and export.

**Why this priority**: A silent selector precedence can export a different registered target from the one the user named.

**Independent Test**: Register two distinct targets in an isolated store and compare single-selector, no-selector, conflicting-selector, and missing-selector invocations.

**Acceptance Scenarios**:

1. **Given** two distinct targets, **when** export receives one target name and the other target's `--id`, **then** argument validation exits 2, names the conflict, and emits no target JSON.
2. **Given** one valid selector or no selector, **when** export runs, **then** the selected target or export-all behavior remains intact.
3. **Given** an unknown row or stable identifier, **when** show, remove, or export refuses it, **then** the next action matches that verb and does not claim that listing targets captures one.

---

### User Story 3 - Seed only what was done (Priority: P1)

A user selecting catalog seed tiers sees which supported sources can fill them, the exact accepted input shape, and a result counting distinct tiers actually filled. An invalid source or explicitly unsupported tier is rejected before a new destination store is created. A default multi-tier run may report skipped tiers, but it never labels skipped work as seeded.

**Why this priority**: The existing result and guidance can claim progress while doing no work and can create a store after an input failure.

**Independent Test**: Run seed in isolated directories with an absent document, repeated tiers, an unsupported launch tier, valid bare JSON arrays, and default multi-tier selection; inspect exit, output, and filesystem state.

**Acceptance Scenarios**:

1. **Given** an absent `--from` file or an incompatible explicit tier, **when** seed runs against a fresh destination, **then** it refuses with an actionable reason and creates no database.
2. **Given** a repeated supported tier, **when** seed runs, **then** it fills that tier once and counts it once.
3. **Given** a valid bare array for a document-backed tier, **when** seed runs, **then** it reports only the tiers actually filled.
4. **Given** help or a skipped-tier message, **when** a next command is suggested, **then** every suggested flag exists in the shipped parser and the input shape is distinguished from catalog export envelopes.

---

### User Story 4 - Know when discovery writes locally (Priority: P2)

A user reading targets help can distinguish target registration from local-store initialization and eligibility seeding. Discover may create or update the local store for volume eligibility but does not register discovered candidate targets; scan can register or update targets.

**Why this priority**: Calling an operation read-only while it creates a database is misleading even when candidate registration remains disabled.

**Independent Test**: Use a fresh isolated store, run discover and scan, and compare their short and full help with their observable store and target-row effects.

**Acceptance Scenarios**:

1. **Given** a fresh local store path, **when** discover runs, **then** help accurately predicts possible store creation and eligibility seeding, and no discovered candidate is registered by that command.
2. **Given** an eligible scan finding, **when** scan runs, **then** its help and summary say it can register or update a local target and the isolated store reflects that effect.
3. **Given** a bare `targets` invocation, **when** its summary appears, **then** the user can distinguish discovery from registration without a blanket read-only claim.

### Edge Cases

- The preview's proposal identifier may remain visible as evidence, but it is never accompanied by an Apply recommendation unless the proposal is complete, authorized by exact scope, and has eligible content.
- An explicitly supplied unsupported tier does not turn a partial multi-tier success into an ambiguous zero-work success.
- A failed source validation must not create a destination store or alter an existing one.
- Export conflict refusal does not print a partial JSON document, even if output is redirected.
- Human and structured reasons need equivalent facts, but S178 does not redesign the global `--json` and `--silent` contract in issue #488.
- All destructive tests use disposable synthetic roots; no owner bundle is collected.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Bundle collect short and full help MUST state that applying a reviewed proposal deletes eligible bundle contents and that `--include-retained` expands selection to retained or custom evidence.
- **FR-002**: Collection preview MUST recommend Apply only when the proposal is complete, has at least one eligible session, and has no unresolved authority. It MUST state the selected scope and refusal reason without suppressing concrete path or ownership detail.
- **FR-003**: Human and structured collection preview MUST convey equivalent eligibility, unresolved, and path-failure facts while preserving private evidence boundaries.
- **FR-004**: Bundle cleanup and export MUST validate path existence and status before offering a destructive retry or surfacing a generic path error; their next steps MUST match the diagnosed state.
- **FR-005**: Export MUST reject simultaneous positional and `--id` selectors at argument validation with exit 2 and zero target JSON bytes.
- **FR-006**: Export MUST preserve one-selector resolution and no-selector export-all behavior.
- **FR-007**: Show, remove, and export missing-selector messages MUST offer action-specific inspection or retry guidance.
- **FR-008**: Catalog seed MUST validate explicit tier/source compatibility and named source readability before creating a new destination store.
- **FR-009**: Catalog seed MUST count each distinct tier actually filled once; skipped and unsupported tiers MUST never increment that count.
- **FR-010**: Explicitly requested unsupported tiers MUST return a non-success result that names the unsupported choice; default multi-tier selection MAY skip unsupported tiers with an explicit reason.
- **FR-011**: Catalog seed help and runtime guidance MUST name only available flags and MUST distinguish accepted bare JSON arrays from catalog export envelopes for document-backed tiers.
- **FR-012**: Discover help MUST state that it registers no candidate targets while it may create or update the local store for volume eligibility; scan and bare targets summaries MUST disclose target registration or updates where those occur.
- **FR-013**: Controlled tests MUST compare command output, exit status, and local filesystem/store effects for the first-run and refusal cases above.
- **FR-014**: Existing exact collection authorization, retained-selection, path ownership, target identity, and source validation boundaries MUST remain intact.

### Key Entities

- **Action eligibility**: The command-specific facts that determine whether an operation can proceed before it creates a store, exports data, or deletes contents.
- **Collection preview**: A bounded report of selected, eligible, retained, and unresolved bundle contents, with an exact proposal identifier and an optional Apply recommendation.
- **Seed result**: Distinct requested tiers, actual fills, skipped reasons, and source validation outcome.
- **Target selector**: Exactly one positional target name/handle/row or stable identifier, or none for export-all.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Every controlled incomplete, zero-eligible, missing, and unowned collection preview shows no Apply recommendation, with equivalent human and structured reasons; a complete eligible preview offers the exact authorized action.
- **SC-002**: Two-target conflict export exits 2 with no JSON target bytes; single-selector and export-all regressions pass.
- **SC-003**: Absent source and explicit unsupported seed tier leave a fresh destination nonexistent; duplicate supported requests count exactly one filled tier.
- **SC-004**: First-run discover and scan tests demonstrate the store and target-row effects described in short and full help.
- **SC-005**: The repository's required local gates and exact-head hosted checks pass before merge handoff.

## Assumptions

- Existing first-run volume eligibility seeding is part of intended discovery behavior; S178 corrects the broad read-only wording rather than suppressing that required store write.
- Existing collection authorization and deletion mechanics are retained; this slice corrects eligibility guidance and invalid-path preflight.
- Issue #488's global JSON/silent result design and issue #155's IGDB integration remain outside S178.

## Clarifications

### Session 2026-10-21

- **Q: Does discover become strictly read-only?** **A:** No. Preserve required first-run volume eligibility seeding and state precisely that candidate targets are not registered by discover. This avoids changing established discovery coverage to solve a wording defect.
- **Q: What happens to an explicitly selected launch seed tier?** **A:** Refuse the invocation before opening the destination store, including a mixed explicit tier list, because no launch seeder exists. An omitted tier list remains useful by seeding supported bundled content and explaining absent sources; its filled count reports only actual distinct fills.
- **Q: Can an incomplete collection preview still expose a proposal identifier?** **A:** Yes, as a diagnostic identity. The human Apply recommendation appears only for a complete proposal with eligible content and no unresolved authority; the existing exact authorization and application checks remain in force.
