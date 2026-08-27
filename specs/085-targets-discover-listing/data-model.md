# Data Model: Targets Discover Listing

## Discovery Store Block

- **catalog path**: Path to the catalog store read by discovery.
- **local path**: Path to the local store read for volume eligibility and related local state.
- **Validation**: Both values are resolved before discovery runs and must render as labelled lines.

## Discovery Candidate Row

- **source**: Human-readable source name from the discovery source.
- **identity**: `steam:<app_id>` for Steam candidates or the path identity for directory candidates.
- **fidelity**: Fidelity token stamped by the source.
- **name**: Candidate display name.
- **Validation**: The human row excludes classification, contains no tabs, and no field is truncated.

## Discovery Evidence Line

- **category**: Technology evidence category.
- **product**: Detected product name.
- **fidelity**: Fidelity token attached to that evidence.
- **Relationship**: Belongs to exactly one discovery candidate row and renders directly after that row.
- **Validation**: Missing evidence produces no invented placeholder.

## Discovery Account Block

- **always visible**: `considered`, `produced`.
- **outcome buckets**: `parse failed`, `declined`, `not a game`, `container descended`, `container descent truncated`, `volume skipped`, `access error`.
- **Validation**: Non-zero buckets render as separate labelled lines. Zero buckets are grouped or omitted only when the required totals remain visible.

## State Transitions

No durable state changes. `targets discover` remains read-only apart from existing local volume eligibility behavior. `targets scan` continues to register candidates after printing the shared discovery listing.
