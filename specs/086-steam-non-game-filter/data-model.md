# Data Model: Steam Non-Game Filter

## Steam App Type

An optional local Steam appinfo value attached to an installed title.

Fields:

- `app_type`: Optional case-insensitive string observed from Steam appinfo.
- `app_id`: The Steam app id string recorded in the manifest.
- `install_dir`: The resolved install directory recorded by the Steam walk.

Validation and behavior:

- `Music`, `Tool`, `Application`, `Config`, and `Video` are non-capturable for target discovery.
- `Game`, `Demo`, and absent values remain eligible.
- Comparison is case-insensitive.
- The value is not normalized or stored by this slice; it is consumed during discovery.
- For excluded app types, the app id and install directory may be shared in memory with lower-authority discovery and listing surfaces so those surfaces do not reintroduce the same platform-filtered app.

## Discovery Account

The existing conservation record returned by a discovery source.

Relevant fields:

- `considered`: increments for every installed Steam title and malformed manifest already accounted for by the source.
- `produced`: increments only for emitted candidates.
- `considered_not_a_game`: increments for every Steam app type excluded by this slice and every exact known-root child suppressed by current Steam non-game metadata.
- `parse_failed`: remains the bucket for app ids that cannot be parsed after the app type check.

Validation and behavior:

- `considered` equals produced plus all non-produced outcomes.
- Excluded app types reuse `considered_not_a_game`; no field is added.

## Candidate Target

The existing candidate emitted by the shared discovery seam.

Behavior:

- No candidate is emitted for excluded app types.
- Candidate shape is unchanged for preserved app types.
- Registration and export paths are unchanged because they consume the same candidate shape as before.
- The hero listing hides platform-created stored rows matching current Steam non-game metadata. The row remains in the store, and user-authored rows are not hidden by this filter.
