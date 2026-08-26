# Research: Targets Finding Fidelity

## Decision: Mark below-verified products with `?`

**Rationale**: The issue permits a trailing marker, distinct color, or suffix. A `?` suffix is compact, works without color, survives redirected output, and increases each affected product by one character while preserving the table's no-truncation rule. It makes uncertainty visible without adding a new column or changing category partitioning. `authored` remains unmarked because the project orders it above `verified`.

**Alternatives considered**:

- **Color only**: Rejected because redirected output, no-color mode, tests, and machine-readable review would still collapse fidelity.
- **Verbose suffix such as `(heuristic-unverified)`**: Rejected because it makes the padded ENGINE column much wider and degrades the hero table for a distinction that can be explained once in the specification.
- **Separate FIDELITY column**: Rejected because fidelity is per finding, not per target row or per technology column.

## Decision: Use strongest fidelity when duplicate product findings disagree

**Rationale**: The current table deduplicates product names. Rendering the same product twice to show fidelity would regress that behavior. Picking the strongest fidelity keeps the deduplicated label honest: if any finding verifies or authors the product, the product is trusted enough to render unmarked; otherwise it remains marked uncertain.

**Alternatives considered**:

- **First-seen fidelity**: Rejected because row order could make the displayed trust tier depend on evidence order rather than content.
- **Always mark when any duplicate is uncertain**: Rejected because it would understate a product that also has verified evidence.

## Decision: Preserve raw evidence fidelity in export/import

**Rationale**: The export already carries each finding object whole, including `fidelity`. Import preserves the evidence JSON verbatim. The machine surface therefore already has the data needed to agree with the listing. S083 should guard this behavior, not introduce a derived display field into the export.

**Alternatives considered**:

- **Add derived technology summary fields to export**: Rejected because it duplicates values that a machine reader can derive from evidence and could drift from the raw evidence.
- **Normalize malformed finding fidelity during import**: Rejected because import should not rewrite raw evidence that may have been produced by a future version; the listing can treat malformed or missing fidelity as uncertain without altering stored data.

## Decision: Update the master specification

**Rationale**: The marker changes a user-visible CLI contract in section 17.7. Constitution P-11 requires the master specification to describe what ships. The revision history should name issue #211.

**Alternatives considered**:

- **Leave the master spec unchanged because the existing P-4 language implies distinct rendering**: Rejected because future agents need the exact marker contract, not only the principle.
