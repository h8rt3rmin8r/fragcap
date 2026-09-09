# Data Model: Target Readiness Groups

## Readiness Group

- **Kind**: `ready` or `needs setup`, derived from the existing capture-readiness authority.
- **Heading**: `Ready to capture` for ready rows or `Needs setup` for setup-needed rows.
- **Rows**: A non-empty contiguous slice of final rendered targets, sorted by handle.
- **Start row**: The one-based global number assigned to the first row in the slice.
- **Invariant**: Ready precedes setup-needed, an empty group is absent, and every displayed target belongs to exactly one group.

## Rendered Target Row

- **Global row number**: Continuous one-based position in the complete grouped listing.
- **Stable identifier**: Existing immutable target identity written into the listing snapshot.
- **Handle**: Existing deterministic within-group sort key and displayed selector.
- **Capture readiness**: Existing derived row label, unchanged by grouping.
- **Engine evidence**: Existing complete display cell with fidelity marker and scan state.
- **Sensitivity evidence**: Existing complete display cell with fidelity marker, scan state, and install-presence note.
- **Invariant**: No displayed field is truncated, wrapped, hidden, renamed, or reclassified.

## Listing Snapshot

- **Rows**: Ordered stable identifier and handle pairs.
- **Source order**: Exact final rendered order, ready group followed by setup-needed group.
- **Replacement rule**: Each listing replaces the prior snapshot; an empty listing writes an empty snapshot.
- **Invariant**: Resolving global row N yields the same target displayed at row N.

## Next Command Candidate

- **Candidate group**: Ready whenever at least one ready row exists, otherwise setup-needed.
- **Preference**: First row in the candidate group whose install root is present or unrecorded.
- **Fallback**: First row in the candidate group when every recorded install root is missing.
- **Invariant**: A setup-needed row is never recommended while a ready row exists.

## State Transitions

This slice adds no stored state or transition. Existing target changes may alter readiness before a later listing, and that later listing derives a new final order and replaces the snapshot atomically through the existing store operation.
