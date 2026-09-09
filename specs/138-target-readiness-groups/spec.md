# Feature Specification: Target Readiness Groups

**Feature Branch**: `codex/s138-target-readiness-groups`

**Created**: 2026-09-09

**Status**: Complete

**Input**: Work slice S138 implementing issue #376 after S136 established display-cell-aware Steam listings and the existing hero target listing established snapshot-backed row selectors.

## User Scenarios & Testing

### User Story 1 - See Capturable Targets First (Priority: P1)

As an operator opening the hero target listing, I need targets that can be captured immediately separated from targets that still need setup so the valid next action is obvious without hiding any stored target.

**Why this priority**: The hero listing is the primary onboarding surface, and a mixed flat table currently makes unusable rows compete visually with immediate capture choices.

**Independent Test**: Render a mixed set whose handles sort across readiness boundaries and verify two visibly separated groups, ready first, with deterministic handle order inside each group and no lost row or evidence cell.

**Acceptance Scenarios**:

1. **Given** ready and setup-needed targets, **When** the human listing is rendered, **Then** `Ready to capture` appears first, `Needs setup` appears second, and every target appears exactly once.
2. **Given** handles whose global alphabetical order crosses readiness groups, **When** the listing is rendered, **Then** each group is sorted by handle and ready rows remain before setup-needed rows.
3. **Given** a wide value in one group, **When** both groups are rendered, **Then** only that group's column widths reflect the value and no value in either group is truncated or wrapped.

---

### User Story 2 - Keep Numeric Selection Truthful (Priority: P2)

As an operator following `fragcap capture N`, I need row numbers, the persisted listing snapshot, and the next-command recommendation to describe the exact grouped order I saw.

**Why this priority**: Grouping becomes unsafe if presentation order and selector authority diverge, because the command could capture a different target than the numbered row the operator selected.

**Independent Test**: Render a mixed listing, inspect its final row order and snapshot, and resolve every displayed number to the same stable target while verifying the footer chooses the first usable ready row.

**Acceptance Scenarios**:

1. **Given** a mixed listing, **When** row numbers and the listing snapshot are produced, **Then** both use the exact ready-then-setup rendered order.
2. **Given** at least one ready target, **When** the next-command footer is selected, **Then** no setup-needed target is recommended, ready targets with present or unrecorded installs are preferred, and the first ready row is the final fallback within that group.
3. **Given** no ready target, **When** the footer is selected, **Then** setup-needed targets with present or unrecorded installs are preferred before the first displayed setup-needed row.

---

### User Story 3 - Preserve Single-State and Machine Interfaces (Priority: P3)

As an operator or script using target management, I need empty and single-state listings to stay concise and machine-readable export and stable target identity to remain unchanged.

**Why this priority**: The layout improvement must not add empty headings, alter target storage, or break automation that consumes exported target entries.

**Independent Test**: Exercise empty, all-ready, all-setup-needed, and mixed stores, then compare target exports and stable identifiers before and after human rendering.

**Acceptance Scenarios**:

1. **Given** an empty store, **When** the listing is rendered, **Then** the existing empty guidance appears with no readiness heading, table, or next-command footer.
2. **Given** only one readiness class, **When** the listing is rendered, **Then** only that non-empty group's heading and table appear.
3. **Given** any store, **When** target export and identity are inspected, **Then** grouping changes neither representation nor stable identifier.

### Edge Cases

- A ready target whose install root is missing remains in the ready group with its existing evidence and is preferred over every setup-needed target only after usable ready rows have been exhausted.
- A setup-needed target whose install root is present receives no footer recommendation while any ready target exists.
- Groups with one row still number against the complete final listing, not from one inside each group.
- A wide handle, engine, or sensitivity value affects only its own group's widths and remains complete even beyond the terminal width.
- Machine-wide anti-cheat findings remain a separate section after both groups and before the footer.
- The bare command remains the explicit target listing plus only its existing help footer.

## Requirements

### Functional Requirements

- **FR-001**: Human hero target output MUST partition every displayed target by the existing capture-readiness classification without changing that classification.
- **FR-002**: The non-empty ready partition MUST carry the exact heading `Ready to capture` and MUST appear before the setup-needed partition.
- **FR-003**: The non-empty setup-needed partition MUST carry the exact heading `Needs setup`.
- **FR-004**: An empty partition and its heading MUST be omitted completely.
- **FR-005**: Rows MUST be sorted by handle within each partition using the existing deterministic handle comparison.
- **FR-006**: Row numbers MUST form one continuous one-based sequence over the final ready-then-setup order and MUST NOT restart at a group boundary.
- **FR-007**: The listing snapshot MUST contain the exact stable identifiers and handles in final rendered row order so every numeric selector resolves to what the operator saw.
- **FR-008**: Each non-empty group MUST compute its table widths solely from that group's rows and headings.
- **FR-009**: Grouping MUST NOT hide, truncate, wrap, rename, or silently reclassify any target, readiness cell, engine evidence, sensitivity evidence, uncertainty marker, scan state, or missing-install note.
- **FR-010**: When any ready target exists, the next-command footer MUST select the first ready row whose install is present or unrecorded, otherwise the first ready row, and MUST NOT select a setup-needed row.
- **FR-011**: When no ready target exists, the footer MUST select the first setup-needed row whose install is present or unrecorded, otherwise the first displayed setup-needed row.
- **FR-012**: Empty output MUST retain its existing population guidance and MUST emit no group heading, table, or next-command footer.
- **FR-013**: Machine-wide findings and the bare-command help footer MUST retain their existing content, separation, and order relative to the grouped listing.
- **FR-014**: Focused or golden tests MUST cover empty, all-ready, all-setup-needed, and mixed listings, including independent group widths, continuous numbering, snapshot resolution, and footer selection.
- **FR-015**: Machine-readable target export, target-entry storage, stable target identity, discovery, registration, and capture-readiness derivation MUST remain unchanged.
- **FR-016**: User and architecture documentation MUST explain readiness grouping, within-group ordering, continuous row numbering, snapshot-backed numeric selection, and the ready-first footer rule.
- **FR-017**: S138 MUST add no dependency, schema migration, target-store rewrite, discovery cleanup, new readiness state, or final release action.

### Key Entities

- **Readiness Group**: One non-empty presentation partition identified by its fixed heading and existing capture-readiness value.
- **Rendered Row Order**: The complete ready-then-setup sequence with deterministic handle order inside each partition.
- **Listing Snapshot**: The persisted stable identifier and handle pairs in rendered row order that authorize later numeric selection.
- **Next Command Candidate**: The row selected from the ready group when one exists, otherwise from the setup-needed group, with usable install presence preferred inside the chosen group.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Mixed listings display 100 percent of targets exactly once, with 100 percent of ready rows before setup-needed rows and 100 percent of rows handle-sorted inside their group.
- **SC-002**: Empty and single-state listings display zero empty group headings, while mixed listings display exactly two readiness headings in the required order.
- **SC-003**: Every displayed row number resolves through the saved snapshot to the same stable target shown on that row across all four required listing shapes.
- **SC-004**: A wide cell in either group changes zero column positions in the other group and loses zero displayed characters.
- **SC-005**: In every listing containing a ready target, the next-command footer recommends a ready row in 100 percent of cases.
- **SC-006**: Target export bytes and stable target identifiers are unchanged by rendering the grouped human listing.
- **SC-007**: Focused tests and the complete repository gate pass with no new dependency or storage migration.

## Assumptions

- `CaptureReadiness::Ready` and `CaptureReadiness::NeedsTarget` remain the complete existing classification and are authoritative for grouping.
- Install presence changes preference only within the selected readiness group and never promotes a setup-needed row above a ready row.
- The existing CAPTURE cell labels remain useful detail inside each table even though the group heading already supplies the broad status.
- Group headings are plain text followed by a colon, matching the existing `Machine:` section style.
- This slice presents current target truth and does not remediate false-positive discovery or stale stored rows, which remain owned by issue #375 and related cleanup work.
