# Feature Specification: Readable Steam Title Listing

**Feature Branch**: `codex/136-steam-list-layout`

**Created**: 2026-09-09

**Status**: Draft

**Input**: User description: "S136 fixes issue #374 by replacing the raw tab-separated human `steam list` output with a readable, width-aware listing while preserving JSON, diagnostics, ordering, and read-only behavior."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Scan installed titles and target state (Priority: P1)

As an operator, I can scan installed Steam titles and keep each app id, title name, registration state, and target identity visually associated even when the values have different lengths.

**Why this priority**: The current tab-separated output loses visual association between fields, which makes the command's primary human workflow unreliable.

**Independent Test**: Render a listing containing short and substantially different field lengths at a normal terminal width and confirm that it has a heading, contains no tab characters, and keeps all four fields aligned by display column.

**Acceptance Scenarios**:

1. **Given** installed titles with different app-id, name, state, and target widths whose complete table fits the available width, **When** the operator runs `fragcap steam list`, **Then** one headed four-column listing aligns every row by visible display column without tabs.
2. **Given** positioned, registered-but-unpositioned, and unregistered titles, **When** the listing is rendered, **Then** the three states remain textually distinct and every target value remains attached to the correct title.

---

### User Story 2 - Read long and localized titles (Priority: P2)

As an operator, I can read every long or localized title and target value without hidden truncation when a four-column table would not remain legible.

**Why this priority**: Real Steam inventories contain wide characters and long names, and silently shortening those values would replace one correlation defect with another.

**Independent Test**: Render a listing containing long names, combining characters, and wide characters at both normal and narrow widths and confirm that it switches as one listing to labeled vertical records, preserves every value, and contains no tab characters.

**Acceptance Scenarios**:

1. **Given** any row that would make the complete four-column table exceed the selected display width, **When** the listing is rendered, **Then** the whole listing uses labeled vertical records so values remain associated without truncation.
2. **Given** localized names containing combining or wide characters, **When** layout fit is calculated, **Then** visible display cells determine the layout and every original non-control character is preserved.
3. **Given** a human value containing a tab, carriage return, or line feed, **When** it is rendered, **Then** the control is represented visibly and cannot become a delimiter or split an unlabeled record.

---

### User Story 3 - Preserve automation and identity behavior (Priority: P3)

As an automation author or returning operator, I receive the same structured records, diagnostics, ordering, and target identities as before this presentation fix.

**Why this priority**: Human readability must not change the stable machine interface or the stored state used by later capture selection.

**Independent Test**: Compare structured output and diagnostics before and after the human renderer change, and exercise a listing against a known snapshot to confirm that ordering and snapshot contents remain unchanged.

**Acceptance Scenarios**:

1. **Given** the same installed titles and local target store, **When** the operator requests JSON, **Then** record fields, presence rules, values, ordering, diagnostics, and exit behavior are unchanged.
2. **Given** a stored target listing snapshot, **When** either human layout is rendered, **Then** the command performs no write and the snapshot remains byte-for-byte logically unchanged.
3. **Given** title names that compare equally without case, **When** rows are ordered, **Then** numeric app id remains the deterministic tiebreak.

### Edge Cases

- Zero installed titles still produces the existing human empty-state sentence and no structured records.
- A missing or unreadable local target store still produces the existing warning and unregistered fallback without creating a store.
- One long name or target switches the complete human listing to vertical records rather than mixing layouts or truncating one row.
- A reported terminal narrower than 40 display columns is treated as 40 columns, a terminal wider than 80 is treated as 80, and a non-terminal or unavailable measurement uses 80.
- Empty target values for unregistered titles remain explicitly associated with the `TARGET` label in vertical layout and remain an empty final column in aligned layout.

## Clarifications

### Session 2026-09-09

- Q: What happens when one complete four-column row cannot fit the selected width? -> A: Render the entire listing as labeled vertical records.
- Q: How is the width selected when output is narrow, redirected, or cannot be measured? -> A: Clamp interactive width to 40 through 80 display columns and otherwise use 80.
- Q: How are embedded layout control characters handled in human values? -> A: Render tab, carriage return, and line feed as visible escapes while structured values remain unchanged.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Human `steam list` output MUST contain no tab characters and MUST begin with a heading that identifies the listing or its four fields.
- **FR-002**: When the maximum complete row fits the selected display width, human output MUST render `APP ID`, `NAME`, `STATE`, and `TARGET` as one aligned table with two spaces between columns.
- **FR-003**: Table alignment and fit decisions MUST use visible display-cell widths so combining and wide characters do not displace later columns.
- **FR-004**: When the complete table would exceed the selected width, human output MUST render the entire listing as labeled vertical records in the existing row order.
- **FR-005**: Neither human layout may truncate, omit, reorder, or ambiguously detach any app id, name, state, or target value.
- **FR-006**: Human rendering MUST represent embedded tab, carriage-return, and line-feed controls visibly so none becomes layout syntax; all other title and target characters MUST remain unchanged.
- **FR-007**: Positioned targets MUST render as `<handle> (#<position>)`, registered-but-unpositioned targets MUST render as `<handle> (no position)`, and unregistered targets MUST render with state `unregistered` and an empty target value.
- **FR-008**: Interactive width selection MUST clamp the reported width to 40 through 80 display columns; redirected output, unavailable width, and unsupported-host measurement MUST use 80.
- **FR-009**: Rows MUST retain case-insensitive title-name ordering with numeric app id as the deterministic tiebreak.
- **FR-010**: The command MUST remain read-only, MUST NOT create a missing local store, and MUST NOT rewrite the targets listing snapshot.
- **FR-011**: JSON standard output, structured field presence rules, field values, diagnostics, empty-state behavior, and exit codes MUST remain unchanged.
- **FR-012**: Human empty-state and store-unavailable behavior MUST remain unchanged apart from the layout of title rows.
- **FR-013**: The S067 human-output contract, current implementation, focused tests, master specification, and CLI reference MUST describe the same final behavior.

### Key Entities

- **Installed title row**: One enumerated Steam title plus its app id, display name, and resolved local target identity.
- **Target identity**: Exactly one of positioned, registered-but-unpositioned, or unregistered, with the existing state and target text rules.
- **Human listing layout**: One layout decision for the complete result, either a four-column table or labeled vertical records.
- **Selected display width**: The bounded display-cell width used only to choose the human listing layout.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Every human listing fixture for short, long, localized, positioned, unpositioned, and unregistered titles contains zero tab characters and retains 100 percent of its input values.
- **SC-002**: At widths where the complete table fits, every data row begins each field at the same visible display column as its header.
- **SC-003**: At the 40-column minimum and for any over-width row at the 80-column default, 100 percent of rows use labeled vertical records and no value is truncated.
- **SC-004**: Existing JSON records and diagnostics remain exactly compatible by field, value, presence rule, ordering, and exit outcome.
- **SC-005**: Repeated human and JSON listing operations leave the local target store and listing snapshot unchanged.

## Assumptions

- Human output is a presentation interface; JSON is the stable machine-readable interface and no undocumented TSV compatibility is required.
- The established S135 display-cell accounting is the repository authority for terminal alignment.
- A single layout per invocation is easier to scan than a mixture of aligned and vertical rows.
- This slice changes presentation only and introduces no new target state, persistence, dependency, or Steam discovery behavior.
