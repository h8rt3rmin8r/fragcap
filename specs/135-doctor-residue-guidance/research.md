# Research: Doctor Residue Guidance

## D1. Preserve machine identity and separate human presentation

**Decision**: Keep each residue check's existing `name` (`native resource <session>/<resource>`) for JSON and verdict compatibility. Attach an optional human presentation whose stable label is `native residue`, plus a typed native-resource context for additive JSON fields.

**Rationale**: Changing the common name would weaken existing automation identity and make repeated residue checks ambiguous. A separate human label fixes column drift without forcing a breaking machine rename.

**Alternatives considered**: Replacing `Check.name` globally was rejected because it collapses unique machine identities. Encoding human prose into the common detail alone was rejected because the dynamic name would still break layout. Creating a second residue-only renderer was rejected because it would fork section ordering, status, and verdict behavior.

## D2. Derive wording from exact residue facts, not text parsing

**Decision**: Map the typed `state`, `health`, ownership authority, and recovery eligibility from `ResourceFinding` to health-specific diagnosis sentences. Preserve the inventory's original detail as secondary evidence where it adds facts, but never parse that prose to determine behavior.

**Rationale**: S124 already owns classification. Typed matching makes the abandoned-owner regression exact and keeps active, unknown, unsupported, and failed cases truthful.

A terminal resource can coexist with a generation-proven active session because resource health is assigned before owner activity. Healthy wording therefore reports that ownership is not determined by the terminal resource state; it does not infer that an active owner is absent. An abandoned owner is recognized by the inventory's stable `resource_id` (`session-owner`), while its `kind` remains the broader `owner` value.

**Alternatives considered**: Reformatting the current flattened key-value string was rejected because it remains implementation-facing. Parsing `detail` was rejected because prose is not an authority. Reclassifying findings in the CLI was rejected because it creates a second recovery policy.

## D3. Make structured native context additive and non-secret

**Decision**: Native residue check records gain `native_resource` with exactly `session_id`, `resource_id`, `kind`, `state`, `health`, `ownership_authority`, and `recovery_eligible`. Existing common fields and separate verdict records remain unchanged. Bundle paths and action internals are excluded.

**Rationale**: These seven facts let automation understand the finding without scraping human text and are already present in memory. Omitting bundle paths avoids adding a new local-path disclosure.

**Alternatives considered**: Flattening fields into every check record was rejected because they apply only to native residue. Emitting the complete `ResourceFinding` was rejected because its bundle and implementation detail exceed the contract. Replacing common fields was rejected as breaking.

## D4. Select one of two layouts from a bounded output width

**Decision**: Human Doctor rendering uses the existing aligned layout when it leaves useful fixed-column detail space and a compact stacked layout below that threshold. The supported width is clamped to 40 through 80 display columns. Non-terminal output remains deterministic at 80. Tests call an injected-width renderer directly.

**Rationale**: Fixed columns are scannable at ordinary widths but fail at narrow ones. A compact form keeps status visibly attached to the finding and provides predictable continuation indentation without truncating values.

**Alternatives considered**: One continuously shrinking column layout was rejected because the detail column becomes unusable. Terminal-width behavior in the pure report model was rejected because it would make tests environment-dependent. Widths below 40 were rejected as outside the clarified support boundary.

## D5. Share display-cell accounting across CLI surfaces

**Decision**: Move the existing target-discovery `display_width`, `display_cell_width`, and padding helpers into a crate-private `display` module, then use them for Doctor wrapping and the target table.

**Rationale**: The repository already has reviewed handling for combining marks, selectors, CJK, fullwidth forms, and emoji. Sharing it avoids subtly different width rules and adds no dependency.

During extraction, S135 deliberately corrects one prior helper result: U+2764 followed by its emoji variation selector previously counted as one display cell because the selector counted zero while the heart base was absent from the wide set. The shared helper counts that emoji form as two cells. This proportional deviation is required for the new color and Unicode alignment contract and is protected by the extracted helper test.

**Alternatives considered**: Byte or scalar counting was rejected because it visibly misaligns non-ASCII text. Copying the helpers was rejected as divergent logic. Adding a Unicode-width package was rejected because the existing bounded implementation covers the product's documented cases without a new lockfile package.

## D6. Keep recovery behavior completely outside presentation

**Decision**: Continue to derive `Action::Cleanup` solely from existing `recoverable` findings and execute through the shared recovery planner. Human remediation says `Run fragcap doctor --fix to review and confirm cleanup of this exact record.` No presentation field participates in action selection.

**Rationale**: Issue #373 is a comprehension defect. Cleanup authority, confirmation, partial-failure behavior, and active-resource preservation already have a security-reviewed owner.

**Alternatives considered**: Offering cleanup based on health wording was rejected because ambiguous and unsupported states need exact authority. Adding a residue-specific fixer was rejected because it would duplicate S109/S124 recovery logic.
