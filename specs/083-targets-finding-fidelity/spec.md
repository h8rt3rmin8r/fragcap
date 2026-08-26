# Feature Specification: Targets Finding Fidelity

**Feature Branch**: `codex/083-targets-finding-fidelity`

**Created**: 2026-08-26

**Status**: Draft

**Input**: User description: "Spec out S083 and then run it end-to-end like usual"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Distinguish Guesses In The Listing (Priority: P1)

An operator reading `fragcap targets` can tell at a glance when a detected engine, anti-cheat, or DRM product is unverified instead of reading every product name as an equally certain fact.

**Why this priority**: The current listing can make heuristic evidence read as verified evidence, which violates P-9 and leads the operator to trust a wrong technology claim.

**Independent Test**: Can be fully tested by listing stored targets that carry the same product at verified and heuristic-unverified fidelity, then confirming the rendered cells differ while the product names remain visible.

**Acceptance Scenarios**:

1. **Given** two target entries with the same engine product and different finding fidelity, **When** the operator lists targets, **Then** the verified-or-stronger product renders unmarked and the heuristic-unverified product renders with a visible uncertainty marker.
2. **Given** a target entry with anti-cheat or DRM evidence at heuristic-unverified fidelity, **When** the operator lists targets, **Then** the SENSITIVITIES cell marks that product as uncertain without moving it to another column.
3. **Given** a target entry with several findings for the same product at different fidelity tiers, **When** the operator lists targets, **Then** the product renders once using the strongest fidelity carried by those findings.

---

### User Story 2 - Preserve Machine Fidelity Agreement (Priority: P2)

A script that reads `fragcap targets export` can recover the same fidelity distinction the human table renders, so human and machine surfaces do not disagree about the trust tier of a technology fact.

**Why this priority**: The issue notes that table and export must agree. The raw evidence already stores fidelity, and this slice must keep that contract guarded while changing the human summary.

**Independent Test**: Can be fully tested by exporting a target with verified and heuristic-unverified findings and confirming each finding retains its `fidelity` token through export and import.

**Acceptance Scenarios**:

1. **Given** a target with technology evidence carrying fidelity, **When** the operator exports targets, **Then** every exported finding still carries its original `fidelity` value.
2. **Given** an exported target file with finding fidelity, **When** it is imported and exported again, **Then** the finding fidelity values are unchanged.
3. **Given** a finding with missing or malformed fidelity in the internal evidence payload, **When** the listing is rendered, **Then** the product is treated as uncertain rather than silently promoted to verified.

### Edge Cases

- Repeated findings for one product and one category can carry different fidelity values; the rendered product must keep the strongest one.
- A finding with an unknown category must still be omitted from both technology columns rather than guessed into one.
- Coverage markers (`-`, `incomplete`, and `not scanned`) are not products and must not receive the uncertainty marker.
- The uncertainty marker must not make the non-handle table budget exceed the existing 80-column design constraint by more than the marker itself.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The target listing MUST render verified technology findings and heuristic-unverified technology findings as distinguishable values.
- **FR-002**: The target listing MUST use the same category partition as before: engine findings in ENGINE, anti-cheat and DRM findings in SENSITIVITIES, and unknown categories in neither.
- **FR-003**: The target listing MUST render a repeated product once per column even when multiple findings name it.
- **FR-004**: When repeated findings for the same product carry different fidelity tiers, the listing MUST render that product using the strongest fidelity among those findings.
- **FR-005**: A product whose strongest finding fidelity is verified or stronger MUST render without an uncertainty marker.
- **FR-006**: A product whose strongest finding fidelity is below verified, or whose finding fidelity is missing or malformed, MUST render with an uncertainty marker.
- **FR-007**: Coverage markers MUST keep their existing text and MUST NOT be marked as uncertain.
- **FR-008**: `targets export` MUST continue to carry each finding's raw `fidelity` token in the evidence payload.
- **FR-009**: `targets import` MUST preserve each finding's raw `fidelity` token instead of normalizing, dropping, or recomputing it.
- **FR-010**: The change MUST NOT add a new storage migration, runtime dependency, capture behavior, proxy behavior, or target process access.

### Key Entities

- **Technology Finding**: A stored evidence object with category, product, evidence, and fidelity fields.
- **Technology Summary Product**: A deduplicated product label rendered in one of the target listing's technology columns.
- **Finding Fidelity**: The trust tier attached to one technology finding. For this slice, `verified` or stronger is unmarked and every lower, missing, or malformed value is marked uncertain in the human table.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A target table containing one verified-or-stronger and one heuristic-unverified finding for the same product renders two visibly different cells in one command output.
- **SC-002**: A target table containing heuristic-unverified engine, anti-cheat, and DRM findings marks all three without changing their category columns.
- **SC-003**: Exporting, importing, and re-exporting a target with finding fidelity produces evidence values with the same fidelity tokens.
- **SC-004**: Existing coverage marker cases still render as exactly `-`, `incomplete`, and `not scanned`.
- **SC-005**: The full repository CI gate passes after the implementation.

## Assumptions

- The uncertainty marker is the suffix `?`, because it is compact, visible in plain text, works in both padded and free-running columns, and requires no color support.
- `verified` and stronger technology finding tiers are unmarked for this slice. `observed`, `heuristic-unverified`, missing, and malformed values are not silently promoted.
- The raw target-entry export already carries per-finding fidelity in the `evidence` array; this slice guards that behavior rather than inventing a second machine field.
