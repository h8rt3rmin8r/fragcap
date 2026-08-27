# Data Model: Steam Non-Game Filter

## Steam App Type

An optional local Steam appinfo value attached to an installed title.

Fields:

- `app_type`: Optional case-insensitive string observed from Steam appinfo.

Validation and behavior:

- `Music`, `Tool`, `Application`, `Config`, and `Video` are non-capturable for target discovery.
- `Game`, `Demo`, and absent values remain eligible.
- Comparison is case-insensitive.
- The value is not normalized or stored by this slice; it is consumed during discovery.

## Discovery Account

The existing conservation record returned by a discovery source.

Relevant fields:

- `considered`: increments for every installed Steam title and malformed manifest already accounted for by the source.
- `produced`: increments only for emitted candidates.
- `considered_not_a_game`: increments for every Steam app type excluded by this slice.
- `parse_failed`: remains the bucket for app ids that cannot be parsed after the app type check.

Validation and behavior:

- `considered` equals produced plus all non-produced outcomes.
- Excluded app types reuse `considered_not_a_game`; no field is added.

## Candidate Target

The existing candidate emitted by the shared discovery seam.

Behavior:

- No candidate is emitted for excluded app types.
- Candidate shape is unchanged for preserved app types.
- Registration, listing, and export paths are unchanged because they consume the same candidate shape as before.
