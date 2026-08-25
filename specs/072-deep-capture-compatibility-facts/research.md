# Phase 0 Research: Deep Capture compatibility facts

## Decisions

### R-1: Store facts beside target entries

**Decision**: Store compatibility facts in `fragcap-targets`, keyed to `targets(id)`.

**Rationale**: The compatibility record describes a known target. A second resolver or standalone compatibility database would create a parallel target identity path and eventually disagree with `targets`.

**Alternatives considered**:

- Separate Deep Capture database: rejected because target identity and lifecycle would fork.
- JSON blob on `targets`: rejected because per-observation provenance, stale state, and multiple launch cases need independent rows.

### R-2: Closed vocabularies with explicit unknown

**Decision**: Use enums and SQLite CHECK constraints for keys and known value domains.

**Rationale**: These facts will become operator guidance. Misspelled or improvised tokens are defects, while `unknown` is a legitimate result that should be queryable.

**Alternatives considered**:

- Free-form strings only: rejected because consumers could not distinguish unsupported values from typographical mistakes.
- Omit unknown rows: rejected because absence cannot distinguish "not measured" from "measured and unknown."

### R-3: Preserve proxy backend provenance structurally

**Decision**: Store proxy backend name, backend version, and proxy mode as columns.

**Rationale**: Observations can differ by proxy implementation and mode. Putting that data in notes would make a future analyzer parse prose or silently mix incompatible evidence.

**Alternatives considered**:

- Optional note only: rejected by review because it would lose machine-readable provenance.
- Closed backend enum: deferred because native backend choices are still under discovery and this slice must not freeze those names prematurely.

### R-4: Keep launch case and final-owner handoff separate

**Decision**: Store launch case as one mutually exclusive field, and store final-owner handoff plus final-owner executable separately.

**Rationale**: A cold direct-executable launch can also hand off socket ownership. Encoding handoff as a launch case would force one true observation to erase another.

**Alternatives considered**:

- `final-owner-differs` as a launch case: rejected by review as lossy.
- A separate fact row only: acceptable, but a structured field on every observation better preserves the context for any fact collected during that run.

### R-5: No CLI or export surface in this slice

**Decision**: Add model, schema, store APIs, tests, and architecture documentation only.

**Rationale**: Issue #217 is the storage foundation. Collection, display, session bundles, HAR/pcapng correlation, and community export each need their own UX and privacy review.

**Alternatives considered**:

- Add a `fragcap targets compatibility` command now: rejected as premature because no producer exists yet and display rules are not settled.
