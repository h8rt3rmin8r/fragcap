# Research: Target Readiness Groups

## Decision 1: Establish One Final Order Before Rendering

**Decision**: Sort the filtered target entries by readiness rank first and handle second, then derive the boundary between ready and setup-needed entries from that single vector.

**Rationale**: The rendered rows, continuous numbers, listing snapshot, and footer all consume the same owned sequence. This removes the risk that two independently filtered collections disagree or that the snapshot preserves the old flat handle order.

**Alternatives considered**: Render two independently collected vectors and concatenate a separate snapshot. Rejected because it duplicates ordering authority and requires extra allocation or target clones. Keep global handle order and skip between readiness classes while rendering. Rejected because numbering, widths, and snapshot order become needlessly stateful.

## Decision 2: Keep Existing Readiness Cells Inside Fixed Groups

**Decision**: Use the literal headings `Ready to capture:` and `Needs setup:` while retaining the CAPTURE column values `ready` and `needs a target` in every row.

**Rationale**: The headings establish immediate action hierarchy, while the existing cell remains part of the documented table contract and avoids a special reduced schema for single-state groups. Keeping the cell also makes every row self-describing if copied independently.

**Alternatives considered**: Remove the CAPTURE column because the heading repeats it. Rejected because it changes the established human table contract beyond #376 and complicates comparisons with prior output. Rename the row values. Rejected because the issue asks for grouping, not a readiness taxonomy change.

## Decision 3: Measure Each Group Independently With Global Row Offsets

**Decision**: Give the table renderer a global starting row number and calculate number, target, and engine widths only from the current group and its actual displayed row range.

**Rationale**: A setup-needed row with a wide handle or evidence value cannot distort the ready table, while numbering remains continuous across the complete listing. The last column remains free-running and all values preserve the current no-truncation rule.

**Alternatives considered**: Reuse widths from the complete listing. Rejected because it fails the issue's independent-width requirement. Restart row numbers per group. Rejected because numeric selectors are global and snapshot-backed.

## Decision 4: Restrict Footer Fallback Within Readiness Priority

**Decision**: If the ready group is non-empty, prefer its first present or unrecorded install and otherwise its first row. Only when the ready group is empty may the same install-presence preference run over setup-needed rows.

**Rationale**: Issue #376 explicitly forbids recommending setup work while any ready target exists. Install presence remains useful inside each readiness class but can no longer invert the readiness priority.

**Alternatives considered**: Preserve the existing global install-presence fallback, which can recommend a present setup-needed row over a missing ready row. Rejected because it contradicts the acceptance criteria. Emit no footer when every ready install is missing. Rejected because the existing populated-listing contract always provides a next command and the issue does not remove it.

## Decision 5: Preserve Storage and Machine Interfaces

**Decision**: Change no target entry, readiness derivation, export path, stable identifier, discovery path, or database schema. Only the existing listing snapshot order changes to mirror human presentation.

**Rationale**: Human grouping is a presentation concern. The snapshot is the sole storage effect that must change because its purpose is to preserve the exact order the user saw.

**Alternatives considered**: Persist a readiness-group field or grouped export. Rejected because readiness is derived at listing time and machine-readable identity is explicitly out of scope.
