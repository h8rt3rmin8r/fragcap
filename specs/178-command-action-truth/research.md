# Research: Truthful CLI Action Boundaries

**Date**: 2026-10-21

## Collection eligibility

**Decision**: Gate the human Apply recommendation on existing preview fields `complete`, `eligible_sessions`, and `unresolved_sessions`; retain the proposal ID and render each unresolved row's existing `detail`.

**Rationale**: `session_gc` already records exact authority, eligibility, and path errors. The defect is a presentation path that recommends Apply whenever a proposal ID exists, including missing and unowned paths. Reusing those facts avoids a second eligibility algorithm.

**Alternatives considered**: Suppress proposal IDs on failed previews (loses diagnostic identity); change collection authorization or deletion logic (outside this issue and risks established ownership protections); treat every zero-work proposal as an error (would obscure valid read-only inspection).

## Bundle path guidance

**Decision**: Check the exact supplied cleanup/export path for absence before generating a `--yes` retry or invoking export; report a path-specific read-only inspection or correction step.

**Rationale**: A missing path is already knowable without an effect. Manifest validity and protected content remain checked by the facade on the actual operation.

**Alternatives considered**: Parse manifests in CLI (duplicates facade authority); use a blanket `doctor --fix` suggestion (may not repair a mistyped path).

## Export selector

**Decision**: Mirror show/remove's clap mutual-exclusion group for export, but leave that group optional. Keep an internal guard for direct-call tests and action-specific no-match advice.

**Rationale**: The parser can reject conflicting user arguments before a store is opened or JSON emitted. The command guard maintains the same truth if constructed in code.

**Alternatives considered**: Let `--id` win (current wrong-target behavior); compare selectors after resolution (would accept invalid syntax when they happen to resolve to the same row).

## Catalog preflight and counts

**Decision**: Validate tier/source compatibility, reject explicitly selected launch, deduplicate requested tiers, and parse a named local document before `Store::open`. The offline document is a bare JSON array for its one declared tier. Use the parsed source in the existing seeder.

**Rationale**: Store open is the first persistent effect. Current `catalog_summary` and `engine_summary` read `--from` after that effect. Deduplication before dispatch makes the filled count a count of distinct actual fills.

**Alternatives considered**: Open a store then roll back on bad input (SQLite file and sidecars can remain); count each invocation as a filled tier (overstates work); remove the launch tier from the public enum (could break existing parsing without a better migration path).

## Discovery contract

**Decision**: Keep first-run local-store and volume-eligibility writes, but state them in short/full help and the master specification. Clarify that discover does not register candidate targets, while scan can register or update them.

**Rationale**: The volume eligibility write is a documented integrity precondition of cross-volume discovery. Reclassifying the whole command as read-only hid that effect; removing the write would change discovery behavior outside this wording issue.

**Alternatives considered**: Strict read-only discovery through a separate transient volume store (adds a second authority and changes coverage); keep only the detailed `--local-db` description (leaves the first-use short claim false).
