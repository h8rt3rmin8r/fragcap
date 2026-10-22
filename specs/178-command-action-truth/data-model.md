# Data Model: CLI Action Decisions

**Date**: 2026-10-21

S178 adds no persistent table, artifact version, or schema migration. These are transient command decisions over existing values.

## Action eligibility

- **Inputs**: Command verb, explicit selector or tier/source, named path, and existing store or collection inspection.
- **States**: `invalid-input`, `not-actionable`, `actionable-preview`, `applied`, or `failed-after-effect`.
- **Rule**: Invalid input is classified before opening a destination store, emitting target JSON, or recommending a destructive command. A collection's exact facade authority still decides whether Apply can mutate.

## Collection preview

- **Existing fields**: `mode`, `proposal_id`, `complete`, `eligible_sessions`, `retained_sessions`, `unresolved_sessions`, `results[].reason`, optional `results[].detail`, and `limitations`.
- **Actionable recommendation**: Preview, complete, positive eligible count, and zero unresolved count. `--include-retained` and `--purge-empty` are explicit scope flags reflected in the proposal.
- **Transition**: Preview establishes diagnostic identity; a later exact authorization triggers a fresh scan and only the unchanged eligible contents may be removed.

## Catalog seed request and result

- **Request**: Ordered tier list, at most one document or feature-gated network source, and destination path.
- **Preflight**: Explicit unsupported tier and incompatible source are rejected; repeated supported tiers collapse to one first-occurrence entry; a named document is read and parsed before store open.
- **Result**: Each attempted tier has `filled` or an explicit `skipped` reason. Total filled count is the number of distinct tiers actually filled.

## Target selector

- **Forms**: Zero selectors for export-all; one positional handle/name/row; or one stable `--id`.
- **Invalid combination**: Positional plus stable ID, rejected before target bytes are emitted.
- **No-match**: Existing handle/name clean miss and numeric/ID usage-error semantics remain; next-step wording is specific to show, remove, or export.

## Discovery effects

- **Discover**: May create/update the local store for volume eligibility; does not register candidate targets.
- **Scan**: May create/update the local store and register candidates.
- **Bare targets/list**: May discover and register newly found eligible titles before listing.
