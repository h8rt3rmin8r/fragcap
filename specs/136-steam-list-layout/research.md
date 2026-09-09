# Research: Readable Steam Title Listing

## Decision 1: Choose one layout for the complete listing

**Decision**: Calculate the maximum complete table width across the header and all rows. Use the aligned table only when that width fits the selected output width; otherwise render every row as one labeled vertical record.

**Rationale**: A whole-list decision keeps the same field in the same visual location throughout a result and avoids mixing row grammars. The vertical form preserves exact values when a long title or target would make four columns illegible.

**Alternatives considered**: Truncating long fields violates P-9 and issue #374. Wrapping individual table cells creates ambiguous multi-line row boundaries across four independently sized columns. Choosing layout per row destroys scan consistency.

## Decision 2: Reuse the S135 display authority

**Decision**: Use the existing shared `display_width` and `pad_display` helpers for table fit and padding. Move Doctor's stdout-width selection into the same crate-private module so both human surfaces use the 40 through 80 policy.

**Rationale**: Rust character count is not terminal-cell width for combining marks, East Asian wide characters, full-width forms, or emoji. S135 already established and tested the repository's display-cell model and width policy.

**Alternatives considered**: Byte length and scalar count misalign localized content. Adding another Unicode-width dependency is unnecessary because the workspace already owns the required bounded implementation. Duplicating terminal-width selection would create policy drift.

## Decision 3: Represent layout controls visibly in human values

**Decision**: Before measuring or rendering a human field, replace tab, carriage return, and line feed with the visible sequences `\\t`, `\\r`, and `\\n`. Preserve every other character. Do not apply this representation to JSON.

**Rationale**: The human output contract cannot guarantee zero tabs or stable record boundaries if data controls remain active layout syntax. Visible escape sequences preserve which control was observed while JSON retains the exact machine value.

**Alternatives considered**: Dropping or replacing controls with spaces loses identity. Allowing them through violates the no-tab contract and can detach fields. Escaping every non-ASCII character would harm readability and is unnecessary.

## Decision 4: Preserve the existing behavioral seam

**Decision**: Keep discovery, identity joins, sorting, warnings, exits, and JSON unchanged. Test the renderer through pure width-injected unit cases and retain the current command-level offline checks.

**Rationale**: Issue #374 is a human presentation defect. A narrow seam makes exact compatibility demonstrable and avoids accidental writes or platform-dependent test setup.

**Alternatives considered**: Refactoring discovery and rendering together expands risk without improving the requested outcome. Snapshot-testing live Steam output would depend on private machine state and violate the repository's offline-test property.
